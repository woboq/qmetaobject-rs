#![recursion_limit="10240"]

#[macro_use]
extern crate cpp;

#[allow(unused_imports)]
extern crate qmetaobject_impl;
#[doc(hidden)]
pub use qmetaobject_impl::*;

#[macro_use]
#[allow(unused_imports)]
extern crate lazy_static;
pub use lazy_static::*;


use std::os::raw::c_void;

cpp!{{
    #include <qmetaobject_rust.hpp>
}}

pub trait QObject {
    fn meta_object(&self)->*const QMetaObject;
}

pub fn base_meta_object()->*const QMetaObject {
    unsafe {
        cpp!{[] -> *const QMetaObject as "const void*" { return &QObject::staticMetaObject; } }
    }
}

#[no_mangle]
pub extern "C" fn RustObject_metaObject(p: *mut QObject) -> *const QMetaObject {
    return unsafe { (*p).meta_object() };
}


#[repr(C)]
pub struct QMetaObject {
    pub superdata : *const QMetaObject,
    pub string_data: *const u8,
    pub data: *const i32,
    pub static_metacall: extern fn(o: *mut c_void, c: u32, idx: u32, a: *const *mut c_void),
    pub r: *const c_void,
    pub e: *const c_void,
}
unsafe impl Sync for QMetaObject {}




#[macro_export]
macro_rules! qt_property {
    ($t:ty) => { std::marker::PhantomData<$t> };
}




#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
