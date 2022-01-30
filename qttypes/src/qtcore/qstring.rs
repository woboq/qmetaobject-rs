use crate::internal_prelude::*;
use crate::qtcore::{QByteArray, QUrl};

use std::fmt::Display;

cpp! {{
    #include <QtCore/QString>
    #include <QtCore/QUrl>
}}

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
