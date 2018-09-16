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

#![recursion_limit="10240"]

#[macro_use]
extern crate cpp;

#[allow(unused_imports)]
#[macro_use]
extern crate qmetaobject_impl;
#[doc(hidden)]
pub use qmetaobject_impl::*;

/* In order to be able to use the lazy_static macro from the QObject custom derive, we re-export
   it under a new name qmetaobject_lazy_static */
#[macro_use] extern crate lazy_static;
#[allow(unused_imports)]
#[doc(hidden)]
pub use lazy_static::*;
#[doc(hidden)]
#[macro_export] macro_rules! qmetaobject_lazy_static { ($($t:tt)*) => { lazy_static!($($t)*) } }

//#[macro_use]
//extern crate bitflags;

use std::os::raw::c_void;
use std::cell::RefCell;

pub mod qttypes;
pub use qttypes::*;

cpp!{{
    #include <qmetaobject_rust.hpp>
}}

#[doc(hidden)]
pub struct QObjectCppWrapper {
    ptr: *mut c_void
}
impl Drop for QObjectCppWrapper {
    fn drop(&mut self) {
        let ptr = self.ptr;
        unsafe { cpp!([ptr as "QObject*"] {
            // The event 513 is catched by RustObject and deletes the object.
            QEvent e(QEvent::Type(513));
            if (ptr)
                ptr->event(&e);
        }) };
    }
}
impl Default for QObjectCppWrapper {
    fn default() -> QObjectCppWrapper { QObjectCppWrapper{ ptr: std::ptr::null_mut() } }
}
impl QObjectCppWrapper {
    pub fn get(&self) -> *mut c_void { self.ptr }
    pub fn set(&mut self, val : *mut c_void) { self.ptr = val; }
}

#[doc(hidden)]
#[repr(C)]
pub struct QObjectDescription {
    pub size : usize,
    pub meta_object: *const QMetaObject,
    pub create : unsafe extern fn(trait_object_ptr : *const c_void) -> *mut c_void,
    pub qml_construct: unsafe extern fn(mem : *mut c_void, trait_object_ptr : *const c_void,
                                        extra_destruct : extern fn(*mut c_void)),
}

/// Trait that is implemented by the QObject custom derive macro
///
/// Do not implement this trait yourself, use `#[derive(QObject)]`.
///
/// The method of this trait fits into two categories: the ones that are re-implemented by
/// the custom derive, and the ones that are used by this macro and need to be implemented
/// by other QObject-like trait which you use in the qt_base_class! macro.
pub trait QObject {
    // Functions re-implemented by the custom derive:

    /// Returns a pointer to a meta object
    fn meta_object(&self)->*const QMetaObject;
    /// Returns a pointer to a meta object
    fn static_meta_object()->*const QMetaObject where Self:Sized;
    /// return a C++ pointer to the QObject*  (can be null if not yet initialized)
    fn get_cpp_object(&self)-> *mut c_void;
    /// Construct the C++ Object.
    ///
    /// Note, once this function is called, the object must not be moved in memory.
    unsafe fn cpp_construct(&mut self) -> *mut c_void;
    /// Construct the C++ Object, suitable for callbacks to construct QML objects.
    unsafe fn qml_construct(&mut self, mem : *mut c_void, extra_destruct : extern fn(*mut c_void));
    /// Return the size of the C++ object
    fn cpp_size() -> usize where Self:Sized;
    /// Return a reference to an object, given a pointer to a C++ object
    unsafe fn get_from_cpp<'a>(p: *const c_void) -> *const Self where Self:Sized;

    // Part of the trait structure that sub trait must have.
    // Copy/paste this code replacing QObject with the type.

    /// Returns a QObjectDescription for this type
    fn get_object_description() -> &'static QObjectDescription where Self:Sized {
        unsafe { cpp!([]-> &'static QObjectDescription as "RustObjectDescription const*" {
            return rustObjectDescription<RustObject<QObject>>();
        } ) }
    }
    /// Implementation for get_from_cpp
    unsafe fn get_rust_object<'a>(p: &'a mut c_void)->&'a mut Self  where Self:Sized {
        // This function is not using get_object_description because we want t be extra fast.
        // Actually, this could be done without indireciton to C++ if we could extract the offset
        // of rust_object.a at (rust) compile time.
        let ptr = cpp!{[p as "RustObject<QObject>*"] -> *mut c_void as "void*" {
            return p->rust_object.a;
        }};
        std::mem::transmute::<*mut c_void, &'a mut Self>(ptr)
    }
}
impl QObject {
    /// Creates a C++ object and construct a QVariant containing a pointer to it.
    ///
    /// The cpp_construct function must already have been called.
    ///
    /// FIXME: should probably not be used
    pub unsafe fn as_qvariant(&self) -> QVariant {
        let self_ = self.get_cpp_object();
        cpp!{[self_ as "QObject*"] -> QVariant as "QVariant"  {
            return QVariant::fromValue(self_);
        }}
    }


    pub fn destroyed_signal() -> CppSignal<fn()> {
        unsafe { CppSignal::new(cpp!([] -> SignalCppRepresentation as "SignalCppRepresentation"  {
            return &QObject::destroyed;
        }))}
    }

    /// FIXME. take self by special reference?  panic if cpp_object does not exist?
    pub fn set_object_name(&self, name: QString) {
        let self_ = self.get_cpp_object();
        unsafe {cpp!([self_ as "QObject*", name as "QString"] {
            if (self_) self_->setObjectName(std::move(name));
        })}
    }
    pub fn object_name_changed_signal() -> CppSignal<fn(QString)> {
        unsafe {CppSignal::new(cpp!([] -> SignalCppRepresentation as "SignalCppRepresentation"  {
            return &QObject::objectNameChanged;
        }))}
    }
}

cpp_class!(unsafe struct QPointerImpl as "QPointer<QObject>");

/// A Wrapper around a QPointer
// (we only need a *const T to support the !Sized case. (Maybe there is a better way)
pub struct QPointer<T : QObject + ?Sized>(QPointerImpl, *const T);
impl<T: QObject + ?Sized> QPointer<T> {
    pub fn cpp_ptr(&self) -> *const c_void {
        let x = &self.0;
        cpp!(unsafe [x as "QPointer<QObject>*"] -> *const c_void as "QObject*" {
            return x->data();
        })
    }

    pub fn as_ptr(&self) -> *const T where T:Sized {
        unsafe { T::get_from_cpp(self.cpp_ptr()) }
    }

    /// Returns a reference to the opbject, or None if it was deleted
    pub fn as_ref(&self) -> Option<&T> {
        let x = self.cpp_ptr();
        if x.is_null() { None } else { unsafe { Some(&*self.1) } }
    }
}
impl<'a, T: QObject + ?Sized> From<&'a T> for QPointer<T> {
    fn from(obj : &'a T) -> Self {
        let cpp_obj = obj.get_cpp_object();
        QPointer(cpp!(unsafe [cpp_obj as "QObject *"] -> QPointerImpl  as "QPointer<QObject>" {
            return cpp_obj;
        }), obj as *const T)
    }
}

impl<T: QObject> Default for QPointer<T> {
    fn default() -> Self {
        QPointer(Default::default(), std::ptr::null())
    }
}

impl<T: QObject + ?Sized> Clone for QPointer<T> {
    fn clone(&self) -> Self {
        QPointer(self.0.clone(), self.1)
    }
}

/*
// Represent a pointer owned by rust
//
// (Same as PinBox, but also construct the object)
pub struct QObjectBox<T : QObject> {
    inner: Box<T>,
}
impl QObjectBox<T> {
    pub fn new(data :T) -> Self {
        let mut inner = Box::new(data);
        unsafe { inner.cpp_construct() }; // Now, data is pinned, we can call cpp_construct
        QObjectBox{ inner }
    }

    pub fn into_leaked_cpp_ptr(self) -> *mut c_void {
        let obj_ptr = self.inner.get_cpp_object();
        std::boxed::Box::into_raw(b);
        obj_ptr
    }

    // add PinBox API
}

// Do we need this?
pub struct QObjectRc<T : QObject> {
    inner: Rc<T>,
}

// Wrapper around QWeakPointer<QObject>
pub struct QObjectWatcher {}

pub struct QObjectRef<'a, T : QObject> {
    inner: &'a T,
}
*/

/// Create the C++ object and return a C++ pointer to a QObject.
///
/// The ownership is given to CPP, the resulting QObject* ptr need to be used somewhere
/// that takes ownership
///
/// Panics if the C++ object was already created.
pub fn into_leaked_cpp_ptr<T: QObject>(obj : T) -> *mut c_void {
    let mut b : Box<T> = Box::new(obj);
    let obj_ptr = unsafe { b.cpp_construct() };
    std::boxed::Box::into_raw(b);
    obj_ptr
}

/// Trait that is implemented by the QGadget custom derive macro
///
/// Do not implement this trait yourself, use `#[derive(QGadget)]`.
pub trait QGadget  {
    /// Returns a pointer to a meta object
    fn meta_object(&self)->*const QMetaObject;
    /// Returns a pointer to a meta object
    fn static_meta_object()->*const QMetaObject where Self:Sized;
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn RustObject_metaObject(p: *mut QObject) -> *const QMetaObject {
    return unsafe { (*p).meta_object() };
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn RustObject_destruct(p: *mut QObject) {
    // We are destroyed from the C++ code, which means that the object was owned by C++ and we
    // can destroy the rust object as well
    let _b = unsafe { Box::from_raw(p) };
}

/// This function is called from the implementation of the signal.
#[doc(hidden)]
pub unsafe fn invoke_signal(object : *mut c_void, meta : *const QMetaObject, id : u32, a: &[*mut c_void] ) {
    let a = a.as_ptr();
    cpp!([object as "QObject*", meta as "const QMetaObject*", id as "int", a as "void**"] {
        QMetaObject::activate(object, meta, id, a);
    })
}

/// Same as a C++ QMetaObject.
#[doc(hidden)]
#[repr(C)]
pub struct QMetaObject {
    pub superdata : *const QMetaObject,
    pub string_data: *const u8,
    pub data: *const u32,
    pub static_metacall: extern fn(o: *mut c_void, c: u32, idx: u32, a: *const *mut c_void),
    pub r: *const c_void,
    pub e: *const c_void,
}
unsafe impl Sync for QMetaObject {}
unsafe impl Send for QMetaObject {}

/// This macro must be used once as a type in a struct that derives from QObject.
/// It is anotate from which QObject like trait it is supposed to derive.
/// the field which it annotate will be an internal property holding a pointer
/// to the actual C++ object
///
/// The trait needs to be like the QObject trait, see the documentation of the QObject trait.
///
/// ```
/// # #[macro_use] extern crate qmetaobject; use qmetaobject::QObject;
/// #[derive(QObject)]
/// struct Foo {
///    base : qt_base_class!(trait QObject),
/// }
/// ```
///
/// Note: in the future, the plan is to extent so you could derive from other struct by doing
/// `base : qt_base_class(struct Foo)`. But this is not yet implemented
#[macro_export]
macro_rules! qt_base_class {
    ($($t:tt)*) => { $crate::QObjectCppWrapper };
}

/// This macro can be used as a type of a field and can then turn this field in a Qt property.
/// The first parameter is the type of this property. Then we can have the meta keywords similar
/// to these found in Q_PROPERTY.
///
/// Can be used within a struct that derives from QObject or QGadget
///
/// `NOTIFY` followed by the name of a signal that need to be declared separately.
/// `WRITE` followed by the name of a setter. `READ` follow by the name of a getter. Note that
/// these are not mandatory and if no setter or no getter exist, it will set the field.
/// `CONST` is also supported.
///
/// `ALIAS` followed by an identifier allow to give a different name than the actual field name.
///
/// ```
/// # #[macro_use] extern crate qmetaobject; use qmetaobject::QObject;
/// #[derive(QObject)]
/// struct Foo {
///    base : qt_base_class!(trait QObject),
///    foo : qt_property!(u32; NOTIFY foo_changed WRITE set_foo) ,
///    foo_changed: qt_signal!()
/// }
/// impl Foo {
///    fn set_foo(&mut self, val : u32) { self.foo = val; }
/// }
/// ```
#[macro_export]
macro_rules! qt_property {
    ($t:ty $(; $($rest:tt)*)*) => { $t };
}

/// This macro can be used to declare a method which will become a meta method.
///
/// Inside you can either declare the method signature, or write the full method.
///
/// Can be used within a struct that derives from QObject or QGadget
///
/// ```
/// # #[macro_use] extern crate qmetaobject; use qmetaobject::QObject;
/// #[derive(QObject)]
/// struct Foo {
///    base : qt_base_class!(trait QObject),
///    defined_method : qt_method!(fn defined_method(&self, foo : u32) -> u32 {
///       println!("contents goes here.");
///       return 42;
///    }),
///    outofline_method : qt_method!(fn(&self, foo : u32)-> u32),
/// }
/// impl Foo {
///    fn outofline_method(&mut self, foo : u32) -> u32 {
///       println!("Or here.");
///       return 69;
///    }
/// }
/// ```
#[macro_export]
macro_rules! qt_method {
    ($($t:tt)*) => { std::marker::PhantomData<()> };
}

/// Declares a signal
///
/// Inside you can either declare the method signature, or write the full method.
///
/// To be used within a struct that derives from QObject
///
/// ```
/// # #[macro_use] extern crate qmetaobject; use qmetaobject::QObject;
/// #[derive(QObject)]
/// struct Foo {
///    base : qt_base_class!(trait QObject),
///    my_signal : qt_signal!(xx: u32, yy: String),
/// }
/// fn some_code(foo : &mut Foo) {
///    foo.my_signal(42, "42".into()); // emits the signal
/// }
/// ```
#[macro_export]
macro_rules! qt_signal {
    ($($name:ident : $ty:ty),*) => { $crate::RustSignal<fn($($ty),*)> };
    //() => { $crate::RustSignal0 };
    //($a0:ident : $t0:ty) => { $crate::RustSignal1<$t0> };
    //($a0:ident : $t0:ty, $a1:ident : $t1:ty) => { $crate::RustSignal2<$t0,$t1> };
}

/// Equivalent to the Q_PLUGIN_METADATA macro.
///
/// To be used within a struct that derives from QObject, and it should contain a string which is
/// the IID
///
/// ```
/// # #[macro_use] extern crate qmetaobject; use qmetaobject::qtdeclarative::QQmlExtensionPlugin;
/// #[derive(Default, QObject)]
/// struct MyPlugin {
///     base: qt_base_class!(trait QQmlExtensionPlugin),
///     plugin: qt_plugin!("org.qt-project.Qt.QQmlExtensionInterface/1.0")
/// }
/// # impl QQmlExtensionPlugin for MyPlugin { fn register_types(&mut self, uri : &std::ffi::CStr) { } }
/// ```
#[macro_export]
macro_rules! qt_plugin {
    ($($t:tt)*) => { std::marker::PhantomData<()> };
}

cpp!{{
    struct FnBoxWrapper {
        TraitObject fnbox;
        ~FnBoxWrapper() {
            if (fnbox) {
                rust!(FnBoxWrapper_destructor [fnbox : *mut FnMut() as "TraitObject"] {
                    unsafe { let _ = Box::from_raw(fnbox); }
                });
            }
        }
        FnBoxWrapper &operator=(const FnBoxWrapper&) = delete;
#if false && QT_VERSION >= QT_VERSION_CHECK(5, 10, 0)
        FnBoxWrapper(const FnBoxWrapper&) = delete;
#else
        // Prior to Qt 5.10 we can't have move-only wrapper. Just do the auto_ptr kind of hack.
        FnBoxWrapper(const FnBoxWrapper &o) : fnbox(o.fnbox) {  const_cast<FnBoxWrapper &>(o).fnbox = {}; }
#endif
        FnBoxWrapper(FnBoxWrapper &&o) : fnbox(o.fnbox) {  o.fnbox = {}; }
        FnBoxWrapper &operator=(FnBoxWrapper &&o) { std::swap(o.fnbox, fnbox); return *this; }
        void operator()() {
            rust!(FnBoxWrapper_operator [fnbox : *mut FnMut() as "TraitObject"] {
                unsafe { (*fnbox)(); }
            });
        }
    };
}}

/// Call the callback once, after a given duration.
pub fn single_shot<F>(interval : std::time::Duration, func : F) where F: FnMut() + 'static {
    let func : Box<FnMut()> = Box::new(func);
    let mut func_raw = Box::into_raw(func);
    let interval_ms : u32 = interval.as_secs() as u32 * 1000 + interval.subsec_nanos() * 1e-6 as u32;
    unsafe{ cpp!([interval_ms as "int", mut func_raw as "FnBoxWrapper"] {
        QTimer::singleShot(std::chrono::milliseconds(interval_ms), std::move(func_raw));
    })};
}

macro_rules! identity{ ($x:ty) => { $x } } // workaround old version of syn
/// Returns a callback that can be called in any thread. Calling the callback will then call the
///
/// given closure in the current Qt thread.
/// If the current thread does no longer have an event loop when the callback is sent, the
/// callback will not be recieved.
///
/// ```
/// # extern crate qmetaobject; use qmetaobject::queued_callback;
/// // in this example, we do not pass '()' as an argument.
/// let callback = queued_callback(|()| println!("hello from main thread"));
/// std::thread::spawn(move || {callback(());}).join();
/// ```
pub fn queued_callback<T : Send, F : FnMut(T) + 'static>(func: F) -> identity!(impl Fn(T) + Send)
{
    let current_thread = cpp!(unsafe [] -> QPointerImpl as "QPointer<QThread>" {
        return QThread::currentThread();
    });

    // In this case, it is safe to send the function to another thread, as we will only call it
    // from this thread.
    // We put it in a RefCell so we can call it mutably.
    struct UnsafeSendFn<T>(RefCell<T>);
    unsafe impl<T> Send for UnsafeSendFn<T> {}
    unsafe impl<T> Sync for UnsafeSendFn<T> {}
    // put func in an arc because we need to keep it alive as long as the internal Box<FnMut> is
    // alive. (And we can't just move it there because the returned closure can be called serveral
    // times.
    let func = std::sync::Arc::new(UnsafeSendFn(RefCell::new(func)));

    move |x| {
        let mut x = Some(x); // Workaround the fact we can't have a Box<FnOnce>
        let func = func.clone();
        let func : Box<FnMut()> = Box::new(move || {
            // the borrow_mut could panic if the function was called recursively. This could happen
            // if the event-loop re-enter.
            let f = &mut (*(func.0).borrow_mut());
            x.take().map(move |x| { f(x); });
        });
        let mut func_raw = Box::into_raw(func);
        unsafe{ cpp!([mut func_raw as "FnBoxWrapper", current_thread as "QPointer<QThread>"] {
            if (!current_thread) return;
            if (!qApp || current_thread != qApp->thread()) {
                QObject *reciever = new QObject();
                reciever->moveToThread(current_thread);
                QMetaObject::invokeMethod(reciever, std::move(func_raw), Qt::QueuedConnection); // does not allow move-only
                reciever->deleteLater();
            } else {
                QMetaObject::invokeMethod(qApp, std::move(func_raw), Qt::QueuedConnection);
            }
        })};
    }
}

/* Small helper function for Rust_QAbstractItemModel::roleNames */
fn add_to_hash(hash: *mut c_void, key: i32, value: QByteArray) {
    unsafe {
        cpp!([hash as "QHash<int, QByteArray>*", key as "int", value as "QByteArray"]{
        (*hash)[key] = std::move(value);
    })
    }
}

/// Refer to the documentation of Qt::UserRole
pub const USER_ROLE: i32 = 0x0100;

pub mod itemmodel;
pub use itemmodel::*;
pub mod listmodel;
pub use listmodel::*;
pub mod qtdeclarative;
pub use qtdeclarative::*;
pub mod qmetatype;
pub use qmetatype::*;
#[macro_use]
pub mod qrc;
pub mod connections;
pub use connections::RustSignal;
use connections::{CppSignal, SignalCppRepresentation};
pub mod scenegraph;
