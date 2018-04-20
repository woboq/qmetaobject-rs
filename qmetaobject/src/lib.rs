#![recursion_limit="10240"]

#[macro_use]
extern crate cpp;

#[allow(unused_imports)]
#[macro_use]
extern crate qmetaobject_impl;
#[doc(hidden)]
pub use qmetaobject_impl::*;

#[macro_use]
#[allow(unused_imports)]
extern crate lazy_static;
pub use lazy_static::*;

use std::os::raw::c_void;
use std::cell::Cell;

pub mod qttypes;
pub use qttypes::*;

cpp!{{
    #include <qmetaobject_rust.hpp>
}}

pub struct QObjectCppWrapper {
    ptr: Cell<*mut c_void>
}
impl Drop for QObjectCppWrapper {
    fn drop(&mut self) {
        let ptr = self.ptr.get();
        unsafe { cpp!([ptr as "QObject*"] {
            // The event 513 is catched by RustObject and deletes the object.
            QEvent e(QEvent::Type(513));
            if (ptr)
                ptr->event(&e);
        }) };
    }
}
impl Default for QObjectCppWrapper {
    fn default() -> QObjectCppWrapper { QObjectCppWrapper{ ptr: Cell::new(std::ptr::null_mut()) } }
}
impl QObjectCppWrapper {
    pub fn get(&self) -> *mut c_void { self.ptr.get() }
    pub fn set(&self, val : *mut c_void) { self.ptr.set(val); }
}

pub trait QObject {

    // these are reimplemented by the QObject procedural macro
    fn meta_object(&self)->*const QMetaObject;
    fn static_meta_object()->*const QMetaObject where Self:Sized;
    fn get_cpp_object<'a>(&'a self)->&'a QObjectCppWrapper;


    // These are not, they are part of the trait structure that sub trait must have
    // Copy/paste this code replacing QObject with the type
    fn base_meta_object()->*const QMetaObject where Self:Sized {
        unsafe {
            cpp!{[] -> *const QMetaObject as "const void*" { return &QObject::staticMetaObject; } }
        }
    }
    unsafe fn get_rust_object<'a>(p: &'a mut c_void)->&'a mut Self  where Self:Sized {
        let ptr = cpp!{[p as "RustObject<QObject>*"] -> *mut c_void as "void*" {
            return p->rust_object.a;
        }};
        std::mem::transmute::<*mut c_void, &'a mut Self>(ptr)
    }
    fn construct_cpp_object(self_ : *const QObject) -> *mut c_void where Self:Sized {
        unsafe {
            cpp!{[self_ as "TraitObject"] -> *mut c_void as "void*"  {
                auto q = new RustObject<QObject>();
                q->rust_object = self_;
                return q;
            }}
        }
    }
}
impl QObject {
    pub fn as_qvariant(&self) -> QVariant {
        unsafe {
            let self_ = self.get_cpp_object().get();
            cpp!{[self_ as "QObject*"] -> QVariant as "QVariant"  {
                return QVariant::fromValue(self_);
            }}
        }
    }
}

#[no_mangle]
pub extern "C" fn RustObject_metaObject(p: *mut QObject) -> *const QMetaObject {
    return unsafe { (*p).meta_object() };
}

#[no_mangle]
pub extern "C" fn RustObject_destruct(p: *mut QObject) {
    let b = unsafe { Box::from_raw(p) };
    b.get_cpp_object().set(std::ptr::null_mut());
}

pub fn invoke_signal(object : *mut c_void, meta : *const QMetaObject, id : u32, a: &[*mut c_void] ) {
    let a = a.as_ptr();
    unsafe { cpp!([object as "QObject*", meta as "const QMetaObject*", id as "int", a as "void**"] {
        QMetaObject::activate(object, meta, id, a);
    })}
}

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

#[macro_export]
macro_rules! qt_property {
    ($t:ty $(; $($rest:tt)*)*) => { $t };
}

#[macro_export]
macro_rules! qt_base_class {
    ($($t:tt)*) => { $crate::QObjectCppWrapper };
}


#[macro_export]
macro_rules! qt_method {
    ($($t:tt)*) => { std::marker::PhantomData<()> };
}

#[macro_export]
macro_rules! qt_signal {
    ($($t:tt)*) => { std::marker::PhantomData<()> };
}

//#[cfg(test)]
pub mod test;

pub mod listmodel;
pub use listmodel::*;
pub mod qtdeclarative;
pub use qtdeclarative::*;
pub mod qmetatype;
pub use qmetatype::*;

