/* Copyright (C) 2018 Olivier Goffart <ogoffart@woboq.com>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
//! Signals, slots and connections between them.
//!
//! First of all, make sure you read about [Signals & Slots][] on Qt documentation website,
//! then the series of articles about [How Qt Signals and Slots Work][woboq-blog-1] on [woboq][]
//! blog.
//!
//! This module implements several concepts related to Qt's signals and slot
//! mechanism such as wrappers for Qt types and extensions specific to Rust.
//!
//! Key points:
//!  - emitting signal is equivalent to just calling a method marked as such;
//!  - signal method takes ownership over its arguments, and borrows them to connected slots;
//!  - argument number zero (`a[0]`) usually used to communicate returned value from the
//!    signal/slot back to the caller, but is currently ignored by this implementation;
//!
//! Subclasses of `QObject` defined in Rust code declare their signals using [`qt_signal!`][]
//! pseudo-macro. It generates both struct member and a method with the same name. Those members
//! will be generated as `RustSignal<fn(...)>` with appropriate generics, and calling
//! corresponding methods actually emits the signal which invokes all connected slots.
//!
//! `RustSignal`s are useless on their own, and in fact are only needed to take space in the
//! struct, so that we could take a reference to it. In order to use them, e.g. connect to a slot,
//! they must be converted into the corresponding `Signal` via [`to_cpp_representation`][]
//! method.
//!
//! `Signal` is a type-safe wrapper for the `SignalInner`, which in turn is what
//! Qt recognizes as a signal ID. For signals defined in C++ they are "a pointer to a member
//! function", but for signals defined in Rust we are currently using `RustSignal` fields' offsets
//! for that. `SignalInner` objects can be obtained both from objects defined in Rust
//! (by calling [`to_cpp_representation`][] on their signal members), as well as from
//! objects defined in C++ and wrapped in Rust (by using `cpp!` macro).
//!
//! Note: Currently neither `Signal` nor any other type hold the information about type of
//! object a signal belongs to, so care must be taken to ensure `Signal` is used to connect
//! the same type of objects it was derived from. It is __surprising behavior__ to use signals
//! representations with non-related types. In the best case, specified signal would not be found
//! which would result in a warning; in the worst case, signal IDs may clash (e.g. same offset)
//! resulting in a warning about incompatible types or even connecting to a wrong function.
//!
//! Signals connect to [`Slot`][]s. `Slot` can be any rust closure with compatible argument count
//! and types. This trait is implemented for up to ten arguments. In terms of Qt, there also
//! exist a return value of a slot, but it is ignored (assumed void) by current implementation.
//!
//! Finally, function [`connect`][] is used to connect `Signal`s (obtained by any means either
//! from `RustSignal`s defined on rust `QObject`s, or from Rust wrappers for C++ classes) to slots
//! (usual Rust closures with compatible argument count and types).
//!
//! # Implementation details
//!
//! Many traits like `Default`, `Copy` etc are not derived but instead manually
//! implemented in this module because `#[derive(...)]` generated code like this:
//! ```ignore
//! impl<Args: Copy> Copy for ... { ... }
//! ```
//! but we don't want to require the `Args: Copy` constraint.
//!
//! To learn more about internals, take a look at documentation in `qmetaobject_rust.hpp` source
//! file.
//!
//! [Signals & Slots]: https://doc.qt.io/qt-5/signalsandslots.html
//! [woboq]: https://woboq.com/blog/
//! [woboq-blog-1]: https://woboq.com/blog/how-qt-signals-slots-work.html
//! [`qt_signal!`]: ../macro.qt_signal.html
//! [`to_cpp_representation`]: ./struct.RustSignal.html#method.to_cpp_representation
//! [`Slot`]: ./trait.Slot.html
//! [`connect`]: ./fn.connect.html
#![deny(missing_docs)]
use std::os::raw::c_void;

use cpp::{cpp, cpp_class};

use super::*;

/// Inner functor type of a `QRustClosureSlotObject` class.
///
/// Corresponds to `typedef ... FuncType` in Qt slot internals.
type RustFuncType = dyn FnMut(*const *const c_void);

cpp! {{
    #include <QtCore/QObject>
    #include "qmetaobject_rust.hpp"

    // Access private function of QObject. Pretend to define QObjectPrivate.
    // This rely on QObjectPrivate being a friend of QObject.
    class QObjectPrivate {
    public:
        static QMetaObject::Connection rust_connectImpl(
            const QObject *sender,
            void **signal,
            const QObject *receiver,
            void **slotPtr,
            QtPrivate::QSlotObjectBase *slot,
            Qt::ConnectionType type,
            const int *types,
            const QMetaObject *senderMetaObject
        ) {
            return QObject::connectImpl(sender, signal, receiver, slotPtr, slot,
                                        type, types, senderMetaObject);
        }
    };

    // Qt defines base 'interface' class for  abstract slots.  There are two
    // implementors in the Qt library itself:  actually object's slot method
    // QSlotObject, and a functor QFunctorSlotObject. Hereby we define third
    // type of slots: Rust closure.  Unlike previous two,  this one is not a
    // template,  because all generics stuff is  already handled on the Rust
    // side,  while C++ only has access to a trait object  which basically a
    // closure FnMut.  This class largely mirrors QFunctorSlotObject.
    class QRustClosureSlotObject : public QtPrivate::QSlotObjectBase
    {
    public:
        /// Wrapper for `*mut dyn RustFuncType`,
        /// which is a closure responsible for calling `Slot::apply`,
        /// which in turn calls the actual handler with unpacked arguments.
        using Func = TraitObject;

    private:
        Func function;

        static void impl(int which, QSlotObjectBase *this_, QObject *r, void **a, bool *ret) {
            // Only used when comparing slots, which is unsupported here.
            Q_UNUSED(ret);
            // QObject pointer to receiver `r` will always match sender, which
            // isn't too useful for slots. See connect() & rust_connectImpl().
            Q_UNUSED(r);

            switch (which) {

            case Destroy:
                delete static_cast<QRustClosureSlotObject *>(this_);
                break;

            case Call: {
                auto slot = static_cast<QRustClosureSlotObject *>(this_)->function;
                rust!(QRustClosureSlotObject_call [
                    slot: *mut RustFuncType as "Func",
                    a: *const *const c_void as "void **"
                ] {
                    // SAFETY: `slot` is guaranteed to be an instance of FnMut, because it is only
                    // ever created from connect().
                    let slot: &mut RustFuncType = unsafe { &mut *slot };
                    slot(a);
                });
                break;
            }

            // Equality traits are not implemented for Rust closures. This is
            // where `ret` flag is supposed to be used, but it is already
            // initialized to `false` by Qt.
            case Compare:
                break;

            // Dummy enum variant representing the total number of enum members
            case NumOperations:
                break;
            }
        }

    public:
        Q_DISABLE_COPY(QRustClosureSlotObject);
        explicit QRustClosureSlotObject(Func f) : QSlotObjectBase(&impl), function(f) {}

        ~QRustClosureSlotObject() {
            rust!(QRustClosureSlotObject_destruct [
                function: *mut RustFuncType as "Func"
            ] {
                let _ = unsafe { Box::from_raw(function) };
            });
        }
    };
}}

cpp_class!(
    /// Wrapper for [QMetaObject::Connection] class.
    ///
    /// # Qt documentation
    ///
    /// Represents a handle to a signal-slot (or signal-functor) connection.
    ///
    /// It can be used to check if the connection is valid and to disconnect it
    /// using QObject::disconnect(). For a signal-functor connection without a
    /// context object, it is the only way to selectively disconnect that
    /// connection.
    ///
    /// As Connection is just a handle, the underlying signal-slot connection
    /// is unaffected when Connection is destroyed or reassigned.
    ///
    /// [QMetaObject::Connection]: https://doc.qt.io/qt-5/qmetaobject-connection.html
    pub unsafe struct ConnectionHandle as "QMetaObject::Connection"
);

impl ConnectionHandle {
    /// Wrapper for [`bool QObject::disconnect(const QMetaObject::Connection &connection)`][qt] static member.
    ///
    /// # Qt documentation
    ///
    /// Disconnect a connection.
    ///
    /// If the connection is invalid or has already been disconnected, do
    /// nothing and return false.
    ///
    /// [qt]: https://doc.qt.io/qt-5/qobject.html#disconnect-4
    pub fn disconnect(&mut self) {
        cpp!(unsafe [self as "const QMetaObject::Connection *"] {
            QObject::disconnect(*self);
        })
    }

    /// Wrapper for [`bool QMetaObject::Connection::operator bool() const`][qt] operator.
    ///
    /// Returns `true` if the connection is valid.
    ///
    /// [qt]: https://doc.qt.io/qt-5/qmetaobject-connection.html#operator-bool
    pub fn is_valid(&self) -> bool {
        cpp!(unsafe [self as "const QMetaObject::Connection *"] -> bool as "bool" {
            return *self; // implicit conversion
        })
    }
}

cpp_class!(
    /// Internal class that can be used to construct C++ signal.  Should only be used as an implementation
    /// details when writing bindings to Qt signals to construct a [`Signal<...>`][Signal].
    ///
    /// It has the same size as any pointer to a member function like `void (QObject::*)()`,
    /// and can be constructed from signals defined in both C++ classes and Rust structs.
    ///
    /// To learn more about internals, take a look at documentation in `qmetaobject_rust.hpp` source
    /// file.
    ///
    /// [Signal]: ./struct.Signal.html
    pub unsafe struct SignalInner as "SignalInner"
);

impl SignalInner {
    /// Construct signal representation from offset of the signal relative to
    /// the base address of the object.
    ///
    /// # Panics
    ///
    /// Tl; dr: signal struct `self` must belong to the object `obj`.
    ///
    /// This method panics if the signal offset lies outside of object's memory
    /// space, i.e. if the offset is less than 0 or greater or equal to
    /// object's size. Object's size must be known at compile time.
    pub fn from_offset<O: QObject + Sized>(offset: isize) -> Self {
        assert!(
            offset >= 0 && offset < std::mem::size_of::<O>() as isize,
            "Signal is not part of the Object: offset {} is outside of type `{}` object's memory",
            offset,
            std::any::type_name::<O>()
        );
        cpp!(unsafe [offset as "ptrdiff_t"] -> SignalInner as "SignalInner" {
            return SignalInner(offset);
        })
    }
}

/// High-level typed wrapper for a pointer to a C++/Qt signal.
///
/// While low-level `SignalInner` operated on pointers to 'erased'
/// member functions types, this struct adds type-safe behavior on top of that.
///
/// `Args` is a type that matches the argument of the signal.
///
/// For example, a C++ signal with signature `void (MyType::*)(int, QString)` will be represented
/// by the `Signal<fn(int, QString)>` type.
pub struct Signal<Args> {
    inner: SignalInner,
    phantom: std::marker::PhantomData<Args>,
}

impl<Args> Signal<Args> {
    /// Wraps low-level type-erased signal representation in a high-level types wrapper.
    ///
    /// # Safety
    ///
    /// Caller must ensure that number, types and order of arguments strictly
    /// matches between signal represented by `inner` and `Args`. Passing
    /// incorrect information may result in **Undefined Behavior.**
    ///
    /// # Example
    ///
    /// ```
    /// use cpp::cpp;
    /// use qmetaobject::*;
    ///
    /// fn object_name_changed() -> Signal<fn(QString)> {
    ///     unsafe {
    ///         Signal::new(cpp!([] -> SignalInner as "SignalInner"  {
    ///             return &QObject::objectNameChanged;
    ///         }))
    ///     }
    /// }
    /// # fn main() {
    /// #     let _ = object_name_changed();
    /// # }
    /// ```
    pub unsafe fn new(inner: SignalInner) -> Self {
        Signal { inner, phantom: Default::default() }
    }
}

// see module-level docs
impl<Args> Clone for Signal<Args> {
    fn clone(&self) -> Self {
        *self
    }
}

// see module-level docs
impl<Args> Copy for Signal<Args> {}

/// Types of signals constructed with the `qt_signal!` macro.
///
/// This type is empty, only its address within the corresponding object matters.
///
/// `Args` represents the type of the arguments, same as in `Signal`.
pub struct RustSignal<Args> {
    phantom: std::marker::PhantomData<Args>,
    _u: bool, // Actually put a field so it has a size;
}

// see module-level docs
impl<Args> Default for RustSignal<Args> {
    fn default() -> Self {
        RustSignal { phantom: Default::default(), _u: Default::default() }
    }
}

impl<Args> RustSignal<Args> {
    /// Construct a corresponding `Signal` from this `RustSignal` struct member.
    ///
    /// The container object must be passed, because `RustSignal` does not have a reference to it.
    /// It does not bind the object though, the object reference is only needed to calculate the
    /// internal offset which is same for all instances of its type.
    ///
    /// # Panics
    ///
    /// Tl; dr: signal struct `self` must belong to the object `obj`.
    ///
    /// This method panics if the signal offset lies outside of object's memory
    /// space, i.e. if the offset is less than 0 or greater or equal to
    /// object's size. Object's size must be known at compile time.
    // TODO: Add owning type to generics and get rid of `obj` argument.
    // TODO: Rename to signal() or something.
    pub fn to_cpp_representation<O: QObject + Sized>(&self, obj: &O) -> Signal<Args> {
        let base_ptr = obj as *const _ as isize;
        let signal_ptr = self as *const _ as isize;
        let offset = signal_ptr - base_ptr;
        let inner = SignalInner::from_offset::<O>(offset);
        Signal { inner, phantom: Default::default() }
    }
}

/// Trait for slots that can be connected to Signal<Args>
///
/// You should not implement this trait. It is already implemented for such `FnMut` that has the
/// same count and types of arguments.
pub trait Slot<Args> {
    /// `a` is an array containing the pointers to return value and arguments:
    ///  - `a[0]` is a pointer to the return value,
    ///  - `a[1]` is a pointer to the first argument,
    ///  - `a[2]` is a pointer to the second argument,
    ///  - and so on...
    unsafe fn apply(&mut self, a: *const *const c_void);
}

/// Convert a signal's array of arguments into tuple.
///
/// This helper trait is implemented for all `fn(...)` types with up to 10
/// arguments which are used as the `Args` generic parameter of signals or
/// slots. Since it does not transfer ownership, nor deals with lifetimes,
/// it is only implemented for types where all signal arguments (tuple members)
/// are `Clone`.
pub trait SignalArgArrayToTuple {
    /// Tuple type of all arguments of a function.
    ///
    /// Number, types and order of members corresponds to arguments of a function.
    type Tuple;

    /// Clone arguments array into tuple. Signal's return value is ignored.
    ///
    /// See [`Slot::apply`][] docs for more.
    ///
    /// [`Slot::apply`]: ./struct.Slot.html#method.apply
    unsafe fn args_array_to_tuple(a: *const *const c_void) -> Self::Tuple;
}

/// Return number of given token trees as an integer expression.
macro_rules! count_repetitions {
    () => { 0 };
    ( $head:tt $( $rest:tt )* ) => { count_repetitions!( $( $rest )* ) + 1 };
}

macro_rules! declare_slot_traits {
    // Note that terminal case (without repetition) generates code for no-arg slots
    (@continue) => {/* terminal case */};
    (@continue $A:ident : $N:literal $($tail:tt)*) => {
        declare_slot_traits![ $($tail)* ];
    };
    // In most cases here, comma in inside the repetitions like $($A,)* because
    // they are dealing with tuples. Basically it takes case of the base case:
    // tuple of single element like (A0,). Similarly to Python, it would not be
    // recognized as a tuple unless there is a trailing comma.
    //
    // Due to the way recursion works in declarative macros,
    // $A is (argc-$N)'th the argument type;
    // $N is the argument $A's position starting from 1. This corresponds to
    //    its position in args array, which reserves [0] item for the return
    //    value.
    ( $( $A:ident : $N:tt )* ) => {
        // Slot wraps a function of all arguments in the repetition of $A.
        // It enables any FnMut with respective count and types of argument to act as a Slot.
        impl<T, $( $A, )* > Slot<fn( $( $A, )* )> for T
            where T : FnMut( $( &$A, )* )
        {
            // `argc` and `a` are unused in the terminal case with 0 repetitions
            #[allow(unused_variables)]
            unsafe fn apply(&mut self, a: *const *const c_void) {
                // See docs for `Slot::apply` first.
                //
                // argc is N + 1 where N is count of arguments of a slot,
                // and 1 is for the return value. Hence, argc is at least 1.
                //
                // For example, when invoked with [H:3 I:2 J:1],
                // the argc is 4 and the biggest $N which is the highest
                // arguments' number is only 3.
                //
                // Then, the repetition of $N is terms of argc is
                //   = (N, N - 1, ..., 1)
                //   = (argc - 1, argc - 2, ..., (argc - N)[= 1])
                //   = (argc - i) for i in [1..=N]
                // so the repetition of (argc - $N) will become
                //     (argc - (argc - 1), argc - (argc - 2), ..., argc - (argc - N))
                //   = (1, 2, ..., N)
                let argc = count_repetitions!( $( $A )* ) + 1;
                self($({
                    let arg_ptr = *a.offset(argc - $N);
                    &*( arg_ptr as *const $A )
                },)*);
            }
        }

        impl< $( $A: Clone, )* > SignalArgArrayToTuple for fn( $( $A, )* ) {
            type Tuple = ( $( $A, )* );

            #[allow(unused_variables)]
            unsafe fn args_array_to_tuple(a: *const *const c_void) -> Self::Tuple {
                let argc = count_repetitions!( $( $A )* ) + 1;
                ($({
                    // Same logic as in Slot::apply above
                    let arg_ptr = *a.offset(argc - $N);
                    let arg_ref = &*( arg_ptr as *const $A );
                    arg_ref.clone()
                },)*)
            }
        }

        declare_slot_traits![ @continue $( $A: $N )* ];
    }
}

// Declare up to 10 arguments (not counting the return value)
declare_slot_traits![A:10 B:9 C:8 D:7 E:6 F:5 G:4 H:3 I:2 J:1];

// FIXME:
// - should not need to be unsafe: we should not take a *const c_void, but a wrapper to a QObject or something similar
/// Connect signal from sender object to a slot.
///
/// Similar to [`QMetaObject::Connection QObject::connect(const QObject *sender, PointerToMemberFunction signal, Functor functor)`][qt],
/// but not a direct wrapper.
///
/// Arguments:
///
///  - Sender is a raw pointer to an instance of `QObject` subclass.
///  - Signal is one of the signals of the sender. See [`Signal`][] for more.
///  - Slot can be any rust clojure `FnMut` with compatible argument count and types (functor-like
/// slot).
///
/// [`Signal`]: ./struct.Signal.html
/// [qt]: https://doc.qt.io/qt-5/qobject.html#connect-4
pub unsafe fn connect<Args, F: Slot<Args>>(
    sender: *const c_void,
    signal: Signal<Args>,
    mut slot: F,
) -> ConnectionHandle {
    let mut cpp_signal = signal.inner;
    // wrap the slot functor and convert closure into a raw trait object (aka fat pointer)
    let slot_closure = move |a: *const *const c_void| slot.apply(a);
    let slot_closure_boxed: Box<dyn FnMut(*const *const c_void)> = Box::new(slot_closure);
    let slot_closure_raw: *mut dyn FnMut(*const *const c_void) = Box::into_raw(slot_closure_boxed);

    cpp!(unsafe [
        sender as "const QObject *",
        mut cpp_signal as "SignalInner",
        slot_closure_raw as "TraitObject"
    ] -> ConnectionHandle as "QMetaObject::Connection" {
        return QObjectPrivate::rust_connectImpl(
            sender,
            cpp_signal.asRawSignal(),
            sender,
            /*slot*/nullptr, // a pointer only used when using Qt::UniqueConnection
            new QRustClosureSlotObject(slot_closure_raw),
            Qt::DirectConnection,
            /*types*/nullptr,
            sender->metaObject()
        );
    })
}
