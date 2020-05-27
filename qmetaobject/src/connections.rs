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
use super::*;
use std;
use std::os::raw::c_void;

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

    class RustSlotOject : public QtPrivate::QSlotObjectBase
    {
        TraitObject slot;
        static void impl(int which, QSlotObjectBase *this_, QObject *r, void **a, bool *ret) {
            switch (which) {
            case Destroy:
                delete static_cast<RustSlotOject*>(this_);
                break;
            case Call: {
                auto slot = static_cast<RustSlotOject*>(this_)->slot;
                rust!(RustSlotObject_call[slot : *mut dyn FnMut(*const *const c_void) as "TraitObject",
                                          a : *const *const c_void as "void**"] {
                       unsafe { (*slot)(a); }
                });
                break;
            }
            case Compare: // not implemented
            case NumOperations:
                Q_UNUSED(ret); Q_UNUSED(r);
            }
        }
    public:
        explicit RustSlotOject(TraitObject slot) : QSlotObjectBase(&impl), slot(slot) {}
        ~RustSlotOject() {
            rust!(RustSlotOject_destruct [slot : *mut dyn FnMut(*const *const c_void) as "TraitObject"] {
                let _ = unsafe { Box::from_raw(slot) };
            });
        }
        Q_DISABLE_COPY(RustSlotOject);
    };


}}

cpp_class!(
    /// Wrapper around Qt's QMetaObject::Connection
    ///
    /// Can be used to detect if a connection is valid, and to disconnect a connection.
    pub unsafe struct ConnectionHandle as "QMetaObject::Connection"
);

impl ConnectionHandle {
    /// Disconnect this connection.
    ///
    /// After this function is called, all ConnectionHandle referring to this connection will be invalided.
    /// Calling disconnect on an invalided connection does nothing.
    pub fn disconnect(&mut self) {
        cpp!(unsafe [self as "const QMetaObject::Connection *"] {
            QObject::disconnect(*self);
        })
    }

    /// Returns true if the connection is valid.
    pub fn is_valid(&self) -> bool {
        cpp!(unsafe [self as "const QMetaObject::Connection *"] -> bool as "bool" {
            return *self; // implicit conversion
        })
    }
}

cpp_class!(
    /// Internal class that can be used to construct C++ signal.  Should only be used as an implementation
    /// details when writing bindings to Qt signals to construct a `CppSignal<...>`.
    ///
    /// It has the same size as a `void (QObject::*)()` and can be constructed from signals.
    pub unsafe struct SignalCppRepresentation as "SignalCppRepresentation"
);

/// Represents a pointer to a C++ signal. Args is a type that matches the argument of the signal.
///
/// For example, a C++ signal with signature `void (MyType::*)(int, QString)` will be represented
/// by the struct `CppSignal<fn(int, QString)>`
pub struct CppSignal<Args> {
    inner: SignalCppRepresentation,
    phantom: std::marker::PhantomData<Args>,
}

impl<Args> CppSignal<Args> {
    pub unsafe fn new(inner: SignalCppRepresentation) -> Self {
        CppSignal { inner, phantom: Default::default() }
    }
}

impl<Args> Clone for CppSignal<Args> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Args> Copy for CppSignal<Args> {}

/// Types of signals constructed with the qt_signal! macro.
///
/// This type is empty, only its address within the corresponding object matters
///
/// Args represents the type of the arguments, similar to the CppSignal ones
pub struct RustSignal<Args> {
    phantom: std::marker::PhantomData<Args>,
    _u: bool, // Actually put a field so it has a size;
}

impl<Args> Default for RustSignal<Args> {
    fn default() -> Self {
        RustSignal {
            phantom: Default::default(),
            _u: Default::default()
        }
    }
}

impl<Args> RustSignal<Args> {
    /// return a CppSignal corresponding to this signal.
    ///
    /// The container object must be passed.
    pub fn to_cpp_representation<O: QObject + Sized>(&self, obj: &O) -> CppSignal<Args> {
        let o = obj as *const O as isize;
        let r = self as *const RustSignal<Args> as isize;
        let diff = r - o;
        assert!(
            diff >= 0 && diff < std::mem::size_of::<O>() as isize,
            "Signal not part of the Object"
        );
        let inner = unsafe {
            cpp!([diff as "qintptr"] -> SignalCppRepresentation as "SignalCppRepresentation" {
                SignalCppRepresentation u;
                u.rust_signal = diff;
                return u;
            })
        };
        CppSignal { inner, phantom: Default::default() }
    }
}

/// Trait for slots that can be connected to Signal<Args>
///
/// You should not implement this trait. It is already implemented for FnMut which has the
/// same arguments.
pub trait Slot<Args> {
    unsafe fn apply(&mut self, a: *const *const c_void);
}

/// Helper trait implemented for the fn(...) type used as the `Args` of signals or slots
///
/// Allows to convert an argument array to a tuple
pub trait SignalArgArrayToTuple {
    type Tuple;
    unsafe fn args_array_to_tuple(a: *const *const c_void) -> Self::Tuple;
}

macro_rules! declare_SlotTraits {
    (@continue $A:ident : $N:tt $($tail:tt)*) => { declare_SlotTraits![$($tail)*]; };
    (@continue) => {};
    ($($A:ident : $N:tt)*) => {
        impl<T $(, $A )*> Slot<fn($($A),*)> for T
            where T : FnMut($(&$A),*)
        {
            #[allow(unused_variables)]
            unsafe fn apply(&mut self, a : *const *const c_void) {
                #[allow(unused_mut)]
                let mut count = 0;
                $(count+=($N,1).1;)*
                self(
                    // a is an array containing the argument, with a[0] being a pointer to the
                    // return value, and a[1] being a pointer to the first argument.
                    // $N is (count-1, count-2, ..., 0), so (count - $N) will be (1,2,...,count)
                    $(&(*(*(a.offset(count - $N)) as *const $A))),*
                );
            }
        }

        impl<$($A : Clone),*> SignalArgArrayToTuple for fn($($A),*) {
            type Tuple = ($($A,)*);
            #[allow(unused_variables)]
            unsafe fn args_array_to_tuple(a : *const *const c_void) -> Self::Tuple {
                #[allow(unused_mut)]
                let mut count = 0;
                $(count+=($N,1).1;)*
                (
                    // Same logic as in Slot::apply
                    $((*(*(a.offset(count - $N)) as *const $A)).clone(),)*

                )
            }
        }

        declare_SlotTraits![@continue $($A: $N)*];
    }
}
declare_SlotTraits![A9:9 A8:8 A7:7 A6:6 A5:5 A4:4 A3:3 A2:2 A1:1 A0:0];

// FIXME:
// - should not need to be unsafe: we should not take a *const c_void, but a wrapper to a QObject or something similar
pub unsafe fn connect<Args, F: Slot<Args>>(
    sender: *const c_void,
    signal: CppSignal<Args>,
    mut slot: F,
) -> ConnectionHandle {
    let mut cpp_signal = signal.inner;
    let apply_closure = move |a: *const *const c_void| slot.apply(a);
    let b: Box<dyn FnMut(*const *const c_void)> = Box::new(apply_closure);
    let slot_raw = Box::into_raw(b);
    cpp!(unsafe [
        sender as "const QObject *",
        mut cpp_signal as "SignalCppRepresentation",
        slot_raw as "TraitObject"
    ] -> ConnectionHandle as "QMetaObject::Connection" {
        return QObjectPrivate::rust_connectImpl(
            sender,
            reinterpret_cast<void **>(&cpp_signal),
            sender,
            nullptr,
            new RustSlotOject(slot_raw),
            Qt::DirectConnection,
            nullptr,
            sender->metaObject()
        );
    })
}
