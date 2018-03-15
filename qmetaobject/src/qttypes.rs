extern crate std;
use std::os::raw::c_char;
use std::convert::From;
use std::fmt::Display;

cpp_class!(pub struct QByteArray, "QByteArray");
impl<'a> From<&'a str> for QByteArray {
    fn from(s : &'a str) -> QByteArray {
        let len = s.len();
        let ptr = s.as_ptr();
        unsafe { cpp!([len as "size_t", ptr as "char*"] -> QByteArray as "QByteArray"
        { return QByteArray(ptr, len); })}
    }
}
impl From<QString> for QByteArray {
    fn from(s : QString) -> QByteArray {
        unsafe {
            cpp!([s as "QString"] -> QByteArray as "QByteArray"
            { return std::move(s).toUtf8(); })
        }
    }
}
impl Display for QByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {
            let c_ptr = cpp!([self as "const QByteArray*"] -> *const c_char as "const char*" {
                return self->constData();
            });
            f.write_str(std::ffi::CStr::from_ptr(c_ptr).to_str().map_err(|_| Default::default())?)
        }
    }
}


cpp_class!(pub struct QString, "QString");
impl<'a> From<&'a str> for QString {
    fn from(s : &'a str) -> QString {
        let len = s.len();
        let ptr = s.as_ptr();
        unsafe { cpp!([len as "size_t", ptr as "char*"] -> QString as "QString"
        { return QString::fromUtf8(ptr, len); })}
    }
}
impl Display for QString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        QByteArray::from(self.clone()).fmt(f)
    }
}


cpp_class!(pub struct QVariant, "QVariant");
impl QVariant {
    pub fn to_qbytearray(&self) -> QByteArray {
        // FIXME
        unsafe {
            cpp!([self as "const QVariant*"] -> QByteArray as "QByteArray" { return self->toByteArray(); })
        }
    }

    pub fn from_qbytearray(a : QByteArray) -> QVariant {
        unsafe {cpp!([a as "QByteArray"] -> QVariant as "QVariant" { return QVariant(a); })}
    }
}
impl From<QString> for QVariant {
    fn from(a : QString) -> QVariant {
        unsafe {cpp!([a as "QString"] -> QVariant as "QVariant" { return QVariant(a); })}
    }
}
impl From<i32> for QVariant {
    fn from(a : i32) -> QVariant {
        unsafe {cpp!([a as "int"] -> QVariant as "QVariant" { return QVariant(a); })}
    }
}
impl From<u32> for QVariant {
    fn from(a : u32) -> QVariant {
        unsafe {cpp!([a as "uint"] -> QVariant as "QVariant" { return QVariant(a); })}
    }
}

cpp_class!(pub struct QModelIndex, "QModelIndex");
impl QModelIndex {
    pub fn row(&self) -> i32 {
        unsafe {
            cpp!([self as "const QModelIndex*"] -> i32 as "int" { return self->row(); })
        }
    }
}


