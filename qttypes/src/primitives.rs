use std::fmt::Display;

use std::os::raw::c_char;
use std::str::Utf8Error;

#[cfg(not(no_qt))]
use cpp::{cpp, cpp_class};

cpp! {{
    #include <QtCore/QString>
    #include <QtCore/QByteArray>
    #include <QtCore/QUrl>
}}

/// Bindings for [`qreal`][type] typedef.
///
/// [type]: https://doc.qt.io/qt-5/qtglobal.html#qreal-typedef
#[allow(non_camel_case_types)]
#[cfg(qreal_is_float)]
pub type qreal = f32;

#[allow(non_camel_case_types)]
#[cfg(not(qreal_is_float))]
pub type qreal = f64;

cpp_class!(
    /// Wrapper around [`QString`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qstring.html
    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    pub unsafe struct QString as "QString"
);

impl QString {
    /// Return a slice containing the UTF-16 data.
    pub fn to_slice(&self) -> &[u16] {
        unsafe {
            let mut size: usize = 0;
            let c_ptr = cpp!([self as "const QString*", mut size as "size_t"] -> *const u16 as "const QChar*" {
                size = self->size();
                return self->constData();
            });
            std::slice::from_raw_parts(c_ptr, size)
        }
    }
}

impl From<QUrl> for QString {
    /// Wrapper around [`QUrl::toString(QUrl::FormattingOptions=...)`][method] method.
    ///
    /// # Wrapper-specific
    ///
    /// Formatting options are left at defaults.
    ///
    /// [method]: https://doc.qt.io/qt-5/qurl.html#toString
    fn from(qurl: QUrl) -> QString {
        cpp!(unsafe [qurl as "QUrl"] -> QString as "QString" {
            return qurl.toString();
        })
    }
}

impl<'a> From<&'a str> for QString {
    /// Copy the data from a `&str`.
    fn from(s: &'a str) -> QString {
        let len = s.len();
        let ptr = s.as_ptr();
        cpp!(unsafe [len as "size_t", ptr as "char*"] -> QString as "QString" {
            return QString::fromUtf8(ptr, len);
        })
    }
}

impl From<String> for QString {
    fn from(s: String) -> QString {
        QString::from(&*s)
    }
}

impl Into<String> for QString {
    fn into(self) -> String {
        String::from_utf16_lossy(self.to_slice())
    }
}

impl Display for QString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        QByteArray::from(self.clone()).fmt(f)
    }
}

impl std::fmt::Debug for QString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

cpp_class!(
    /// Wrapper around [`QByteArray`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qbytearray.html
    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    pub unsafe struct QByteArray as "QByteArray"
);
impl QByteArray {
    pub fn to_slice(&self) -> &[u8] {
        unsafe {
            let mut size: usize = 0;
            let c_ptr = cpp!([self as "const QByteArray*", mut size as "size_t"] -> *const u8 as "const char*" {
                size = self->size();
                return self->constData();
            });
            std::slice::from_raw_parts(c_ptr, size)
        }
    }
    pub fn to_str(&self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(self.to_slice())
    }
}
impl<'a> From<&'a [u8]> for QByteArray {
    /// Constructs a `QByteArray` from a slice. (Copy the slice.)
    fn from(s: &'a [u8]) -> QByteArray {
        let len = s.len();
        let ptr = s.as_ptr();
        cpp!(unsafe [len as "size_t", ptr as "char*"] -> QByteArray as "QByteArray" {
            return QByteArray(ptr, len);
        })
    }
}
impl<'a> From<&'a str> for QByteArray {
    /// Constructs a `QByteArray` from a `&str`. (Copy the string.)
    fn from(s: &'a str) -> QByteArray {
        s.as_bytes().into()
    }
}
impl From<String> for QByteArray {
    /// Constructs a `QByteArray` from a `String`. (Copy the string.)
    fn from(s: String) -> QByteArray {
        QByteArray::from(&*s)
    }
}
impl From<QString> for QByteArray {
    /// Converts a `QString` to a `QByteArray`
    fn from(s: QString) -> QByteArray {
        cpp!(unsafe [s as "QString"] -> QByteArray as "QByteArray" {
            return std::move(s).toUtf8();
        })
    }
}
impl Display for QByteArray {
    /// Prints the contents of the `QByteArray` if it contains UTF-8, do nothing otherwise.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {
            let c_ptr = cpp!([self as "const QByteArray*"] -> *const c_char as "const char*" {
                return self->constData();
            });
            f.write_str(std::ffi::CStr::from_ptr(c_ptr).to_str().map_err(|_| Default::default())?)
        }
    }
}
impl std::fmt::Debug for QByteArray {
    /// Prints the contents of the `QByteArray` if it contains UTF-8,  nothing otherwise
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

cpp_class!(
    /// Wrapper around [`QUrl`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qurl.html
    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    pub unsafe struct QUrl as "QUrl"
);
impl QUrl {
    /// Wrapper around [`QUrl::fromUserInput(const QString &userInput)`][method] static method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qurl.html#fromUserInput
    pub fn from_user_input(user_input: QString) -> QUrl {
        cpp!(unsafe [user_input as "QString"] -> QUrl as "QUrl" {
            return QUrl::fromUserInput(user_input);
        })
    }
}
impl From<QString> for QUrl {
    fn from(qstring: QString) -> QUrl {
        cpp!(unsafe [qstring as "QString"] -> QUrl as "QUrl" {
            return QUrl(qstring);
        })
    }
}
