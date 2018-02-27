extern crate std;
use std::os::raw::c_char;

cpp_class!(pub struct QByteArray, "QByteArray");
impl QByteArray {
    pub fn from_str(s : &str) -> QByteArray {
        let len = s.len();
        let ptr = s.as_ptr();
        unsafe { cpp!([len as "size_t", ptr as "char*"] -> QByteArray as "QByteArray"
        { return QByteArray(ptr, len); })}
    }

    pub fn to_string(&self) -> String {
        unsafe {
            let c_ptr = cpp!([self as "const QByteArray*"] -> *const c_char as "const char*"
                { return self->constData(); });
            std::ffi::CStr::from_ptr(c_ptr).to_string_lossy().into_owned()
        }
    }
}
impl Default for QByteArray {
    fn default() -> QByteArray {
        unsafe {cpp!([] -> QByteArray as "QByteArray" { return QByteArray(); })}
    }
}

cpp_class!(pub struct QString, "QString");
impl QString {
    pub fn from_str(s : &str) -> QString {
        let len = s.len();
        let ptr = s.as_ptr();
        unsafe { cpp!([len as "size_t", ptr as "char*"] -> QString as "QString"
        { return QString::fromUtf8(ptr, len); })}
    }

    pub fn to_string(&self) -> String {
        unsafe {
            let ba = cpp!([self as "const QString*"] -> QByteArray as "QByteArray"
                { return self->toUtf8(); });
            ba.to_string()
        }
    }
}
impl Default for QString {
    fn default() -> QString {
        unsafe {cpp!([] -> QString as "QString" { return QString(); })}
    }
}


cpp_class!(pub struct QVariant, "QVariant");
impl QVariant {
}
impl Default for QVariant {
    fn default() -> QVariant {
        unsafe {cpp!([] -> QVariant as "QVariant" { return QVariant(); })}
    }
}


cpp_class!(pub struct QModelIndex, "QModelIndex");
impl QModelIndex {
}
impl Default for QModelIndex {
    fn default() -> QModelIndex {
        unsafe {cpp!([] -> QModelIndex as "QModelIndex" { return QModelIndex(); })}
    }
}
