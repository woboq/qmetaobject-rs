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

    /// Wrapper around [`bool QString::isEmpty() const`][method] method
    ///
    /// [method]: https://doc.qt.io/qt-5/qstring.html#isEmpty
    /// ```
    /// use qttypes::QString;
    ///
    /// assert!(QString::default().is_empty());
    /// assert!(QString::from("").is_empty());
    /// assert!(!QString::from("abc").is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        cpp!(unsafe [self as "const QString*"] -> bool as "bool" {
            return self->isEmpty();
        })
    }

    /// Wrapper around [`bool QString::isNull() const`][method] method
    ///
    /// [method]: https://doc.qt.io/qt-5/qstring.html#isNull
    /// ```
    /// use qttypes::QString;
    ///
    /// assert!(QString::default().is_null());
    /// assert!(!QString::from("").is_null());
    /// assert!(!QString::from("abc").is_null());
    /// ```
    pub fn is_null(&self) -> bool {
        cpp!(unsafe [self as "const QString*"] -> bool as "bool" {
            return self->isNull();
        })
    }

    /// Returns the number of characters in this string.
    pub fn len(&self) -> usize {
        cpp!(unsafe [self as "const QString*"] -> usize as "size_t" {
            return self->length();
        })
    }

    /// Wrapper around [`bool QString::isUpper() const`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qstring.html#isUpper
    #[cfg(qt_5_12)]
    pub fn is_upper(&self) -> bool {
        cpp!(unsafe [self as "const QString*"] -> bool as "bool" {
            #if QT_VERSION >= QT_VERSION_CHECK(5,12,0)
            return self->isUpper();
            #else
            return false;
            #endif
        })
    }

    /// Wrapper around [`void QString::shrink_to_fit()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qstring.html#shrink_to_fit
    pub fn shrink_to_fit(&mut self) {
        cpp!(unsafe [self as "QString*"] {
            self->squeeze();
        })
    }

    /// Wrapper around [`QString QString::toUpper() const`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qstring.html#toUpper
    pub fn to_upper(&self) -> QString {
        cpp!(unsafe [self as "const QString*"] -> QString as "QString" {
            return self->toUpper();
        })
    }

    /// Wrapper around [`QString QString::toLower() const`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qstring.html#toLower
    pub fn to_lower(&self) -> QString {
        cpp!(unsafe [self as "const QString*"] -> QString as "QString" {
            return self->toLower();
        })
    }

    /// Wrapper around [`QString QString::trimmed() const`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qstring.html#trimmed
    pub fn trimmed(&self) -> QString {
        cpp!(unsafe [self as "const QString*"] -> QString as "QString" {
            return self->trimmed();
        })
    }

    /// Wrapper around [`QString QString::toCascadeFold() const`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qstring.html#toCascadeFold
    pub fn to_case_folded(&self) -> QString {
        cpp!(unsafe [self as "const QString*"] -> QString as "QString" {
            return self->toCaseFolded();
        })
    }

    /// Wrapper around [`QString QString::simplified() const`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qstring.html#simplified
    pub fn simplified(&self) -> QString {
        cpp!(unsafe [self as "const QString*"] -> QString as "QString" {
            return self->simplified();
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let upper = QString::from("ABC");
        let lower = QString::from("abc");

        assert_eq!(lower.len(), 3);

        #[cfg(qt_5_12)]
        assert!(upper.is_upper());
        #[cfg(qt_5_12)]
        assert!(!lower.is_upper());

        assert_eq!(lower.to_upper(), upper);
        assert_eq!(upper.to_lower(), lower);
        assert_eq!(
            QString::from("  lots\t of\nwhitespace\r\n ").simplified(),
            QString::from("lots of whitespace")
        );
        assert_eq!(upper.to_lower(), upper.to_case_folded());

        assert_eq!(QString::from(" ABC Hello\n").trimmed(), QString::from("ABC Hello"));
    }
}
