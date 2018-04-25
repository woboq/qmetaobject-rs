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

pub mod qttypes;
pub use qttypes::*;

cpp!{{
    #include <qmetaobject_rust.hpp>
}}

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

#[repr(C)]
pub struct QObjectDescription {
    pub size : usize,
    pub meta_object: *const QMetaObject,
    pub create : unsafe extern fn(trait_object_ptr : *const c_void) -> *mut c_void,
    pub qml_construct: unsafe extern fn(mem : *mut c_void, trait_object_ptr : *const c_void,
                                        extra_destruct : extern fn(*mut c_void)),
}

pub trait QObject {
    // these are reimplemented by the QObject procedural macro
    fn meta_object(&self)->*const QMetaObject;
    fn static_meta_object()->*const QMetaObject where Self:Sized;
    // return a pointer to the QObject*  (can be null if not yet initialized)
    // Ideally this should be covariant
    fn get_cpp_object(&self)-> *mut c_void;
    unsafe fn cpp_construct(&mut self) -> *mut c_void;
    unsafe fn qml_construct(&mut self, mem : *mut c_void, extra_destruct : extern fn(*mut c_void));
    fn cpp_size() -> usize where Self:Sized;




    // These are not, they are part of the trait structure that sub trait must have
    // Copy/paste this code replacing QObject with the type
    fn get_object_description() -> &'static QObjectDescription where Self:Sized {
        unsafe { cpp!([]-> &'static QObjectDescription as "RustObjectDescription const*" {
            return rustObjectDescription<RustObject<QObject>>();
        } ) }
    }
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
    pub fn as_qvariant(&self) -> QVariant {
        unsafe {
            let self_ = self.get_cpp_object();
            cpp!{[self_ as "QObject*"] -> QVariant as "QVariant"  {
                return QVariant::fromValue(self_);
            }}
        }
    }
}

pub trait QGadget  {
    fn meta_object(&self)->*const QMetaObject;
    fn static_meta_object()->*const QMetaObject where Self:Sized;

    fn to_qvariant(&self) -> QVariant where Self: Clone + Default {
        let id : i32 = register_gadget_metatype::<Self>(); // FIXME: we should not register it always
        //let id = self.metatype();
        unsafe { cpp!([self as "const void*", id as "int"] -> QVariant as "QVariant"  {
            return QVariant(id, self);
        } ) }
    }
}

#[no_mangle]
pub extern "C" fn RustObject_metaObject(p: *mut QObject) -> *const QMetaObject {
    return unsafe { (*p).meta_object() };
}

#[no_mangle]
pub extern "C" fn RustObject_destruct(p: *mut QObject) {
    // We are destroyed from the C++ code, which means that the object was owned by C++ and we
    // can destroy the rust object as well
    let _b = unsafe { Box::from_raw(p) };
}

pub unsafe fn invoke_signal(object : *mut c_void, meta : *const QMetaObject, id : u32, a: &[*mut c_void] ) {
    let a = a.as_ptr();
    cpp!([object as "QObject*", meta as "const QMetaObject*", id as "int", a as "void**"] {
        QMetaObject::activate(object, meta, id, a);
    })
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

pub mod listmodel;
pub use listmodel::*;
pub mod qtdeclarative;
pub use qtdeclarative::*;
pub mod qmetatype;
pub use qmetatype::*;

