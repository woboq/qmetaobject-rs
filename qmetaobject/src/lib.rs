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

cpp!{{
    #include <qmetaobject_rust.hpp>
}}

pub struct QObjectCppWrapper {
    pub ptr: *mut c_void
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

pub trait QObject {

    // these are reimplemented by the QObject procedural macro
    fn meta_object(&self)->*const QMetaObject;
    fn static_meta_object()->*const QMetaObject where Self:Sized;
    fn get_cpp_object<'a>(&'a mut self)->&'a mut QObjectCppWrapper;


    // These are not, they are part of the trait structure that sub trait must have
    // Copy/paste this code replacing QObject with the type
    fn base_meta_object()->*const QMetaObject where Self:Sized {
        unsafe {
            cpp!{[] -> *const QMetaObject as "const void*" { return &QObject::staticMetaObject; } }
        }
    }
    unsafe fn get_rust_object<'a>(p: &'a mut c_void)->&'a mut Self  where Self:Sized {
        //std::mem::transmute::<*mut std::os::raw::c_void, &mut #name>(
        //                  p.offset(8/*virtual_table*/ + 8 /* d_ptr */)) }; // FIXME
        let ptr = cpp!{[p as "RustObject<QObject>*"] -> *mut c_void as "void*" {
            return p->data.a;
        }};
        std::mem::transmute::<*mut c_void, &'a mut Self>(ptr)
    }
    fn construct_cpp_object(&mut self) where Self:Sized {
        let p = unsafe {
            let p : *mut QObject = self;
            cpp!{[p as "TraitObject"] -> *mut c_void as "void*"  {
                auto q = new RustObject<QObject>();
                q->data = p;
                return q;
            }}
        };
        let cpp_object = self.get_cpp_object();
        assert!(cpp_object.ptr.is_null(), "The cpp object was already created");
        cpp_object.ptr = p;
    }
}

#[no_mangle]
pub extern "C" fn RustObject_metaObject(p: *mut QObject) -> *const QMetaObject {
    return unsafe { (*p).meta_object() };
}

#[no_mangle]
pub extern "C" fn RustObject_destruct(p: *mut QObject) {
    let mut b = unsafe { Box::from_raw(p) };
    b.get_cpp_object().ptr = std::ptr::null_mut();
}

pub fn invoke_signal(object : *mut c_void, meta : *const QMetaObject, id : u32, a: &[*mut c_void] ) {
    let a = a.as_ptr();
    unsafe { cpp!([object as "QObject*", meta as "const QMetaObject*", id as "int", a as "void**"] {
        QMetaObject::activate(object, meta, id, a);
    })}
}

pub fn register_metatype<T : Sized + Clone + Default>(name : &str) -> i32 {
    let size = std::mem::size_of::<T>() as u32;

    extern fn deleter_fn<T>(_v: Box<T>) {
    };
    let deleter_fn : extern fn(_v: Box<T>) = deleter_fn;

    extern fn creator_fn<T : Default + Clone>(c : *const T) -> Box<T> {
        if c.is_null() { Box::new( Default::default() ) }
        else { Box::new(unsafe { (*c).clone() }) }
    };
    let creator_fn : extern fn(c : *const T) -> Box<T> = creator_fn;

    extern fn destructor_fn<T>(ptr : *mut T) {
        unsafe { std::ptr::read(ptr); }
    };
    let destructor_fn : extern fn(ptr : *mut T) = destructor_fn;

    extern fn constructor_fn<T : Default + Clone>(dst : *mut T, c : *const T) -> *mut T {
        unsafe {
            let n = if c.is_null() {  Default::default() }
                    else { (*c).clone() };
            std::ptr::write(dst, n);
        }
        dst
    };
    let constructor_fn : extern fn(ptr : *mut T, c : *const T) -> *mut T = constructor_fn;

    let name = std::ffi::CString::new(name).unwrap();
    unsafe {
        let name = name.as_ptr();
        cpp!([name as "const char*", size as "int", deleter_fn as "QMetaType::Deleter",
                   creator_fn as "QMetaType::Creator", destructor_fn as "QMetaType::Destructor",
                   constructor_fn as "QMetaType::Constructor"] -> i32 as "int" {
            return QMetaType::registerType(name, deleter_fn, creator_fn, destructor_fn,
                constructor_fn, size,
                QMetaType::NeedsConstruction | QMetaType::NeedsDestruction | QMetaType::MovableType,
                nullptr);
        })
    }
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

#[macro_export]
macro_rules! qt_property {
    ($t:ty) => { $t };
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

