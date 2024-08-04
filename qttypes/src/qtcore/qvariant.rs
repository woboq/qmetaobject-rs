use std::fmt;

use crate::{
    cpp, cpp_class, QByteArray, QDate, QDateTime, QString, QStringList, QTime, QUrl, QVariantList,
    QVariantMap,
};

cpp_class!(
    /// Wrapper around [`QVariant`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qvariant.html
    #[derive(PartialEq, Eq)]
    pub unsafe struct QVariant as "QVariant"
);
impl QVariant {
    /// Wrapper around [`toByteArray()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qvariant.html#toByteArray
    pub fn to_qbytearray(&self) -> QByteArray {
        cpp!(unsafe [self as "const QVariant*"] -> QByteArray as "QByteArray" {
            return self->toByteArray();
        })
    }

    /// Wrapper around [`toBool()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qvariant.html#toBool
    pub fn to_bool(&self) -> bool {
        cpp!(unsafe [self as "const QVariant*"] -> bool as "bool" {
            return self->toBool();
        })
    }

    /// Wrapper around [`userType()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qvariant.html#userType
    pub fn user_type(&self) -> i32 {
        cpp!(unsafe [self as "const QVariant*"] -> i32 as "int" {
            return self->userType();
        })
    }

    /// Wrapper around [`isValid()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qvariant.html#isValid
    pub fn is_valid(&self) -> bool {
        cpp!(unsafe [self as "const QVariant*"] -> bool as "bool" {
            return self->isValid();
        })
    }

    /// Wrapper around [`isNull()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qvariant.html#isNull
    pub fn is_null(&self) -> bool {
        cpp!(unsafe [self as "const QVariant*"] -> bool as "bool" {
            return self->isNull();
        })
    }

    /// Wrapper around [`toMap()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qvariant.html#toMap
    pub fn to_qvariantmap(&self) -> QVariantMap {
        cpp!(unsafe [self as "const QVariant*"] -> QVariantMap as "QVariantMap" {
            return self->toMap();
        })
    }

    /// Wrapper around [`toString()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qvariant.html#toString
    pub fn to_qstring(&self) -> QString {
        cpp!(unsafe [self as "const QVariant*"] -> QString as "QString" {
            return self->toString();
        })
    }

    /// Wrapper around [`toInt()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qvariant.html#toInt
    pub fn to_int(&self) -> u32 {
        cpp!(unsafe [self as "const QVariant*"] -> u32 as "int" {
            return self->toInt();
        })
    }

    /// Wrapper around ['typeName()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qvariant.html#typeName
    pub fn type_name(&self) -> QString {
        cpp!(unsafe [self as "const QVariant*"] -> QString as "QString" {
            return self->typeName();
        })
    }

    // FIXME: do more wrappers
}

impl fmt::Debug for QVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.to_qstring().to_string();
        let qtype = self.type_name().to_string();
        if data.len() == 0 {
            write!(f, "QVariant({})", qtype.as_str())
        } else {
            write!(f, "QVariant({}: \"{}\")", qtype.as_str(), data.as_str())
        }
    }
}

impl From<QString> for QVariant {
    /// Wrapper around [`QVariant(const QString &)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-14
    fn from(a: QString) -> QVariant {
        cpp!(unsafe [a as "QString"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<QVariantMap> for QVariant {
    /// Wrapper around [`QVariant(const QMap<QString, QVariant> &)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-22
    fn from(a: QVariantMap) -> QVariant {
        cpp!(unsafe [a as "QVariantMap"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<QByteArray> for QVariant {
    /// Wrapper around [`QVariant(const QByteArray &)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-12
    fn from(a: QByteArray) -> QVariant {
        cpp!(unsafe [a as "QByteArray"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<QDate> for QVariant {
    /// Wrapper around [`QVariant(const QDate &)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-18
    fn from(a: QDate) -> QVariant {
        cpp!(unsafe [a as "QDate"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<QTime> for QVariant {
    /// Wrapper around [`QVariant(const QTime &)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-19
    fn from(a: QTime) -> QVariant {
        cpp!(unsafe [a as "QTime"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<QDateTime> for QVariant {
    /// Wrapper around [`QVariant(const QDateTime &)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-20
    fn from(a: QDateTime) -> QVariant {
        cpp!(unsafe [a as "QDateTime"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<QUrl> for QVariant {
    /// Wrapper around [`QVariant(const QUrl &)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-35
    fn from(a: QUrl) -> QVariant {
        cpp!(unsafe [a as "QUrl"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<QVariantList> for QVariant {
    /// Wrapper around [`QVariant(const QVariantList &)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-21
    fn from(a: QVariantList) -> QVariant {
        cpp!(unsafe [a as "QVariantList"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}

impl From<QStringList> for QVariant {
    /// Wrapper around [`QVariant(const QStringList &)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-16
    fn from(a: QStringList) -> Self {
        cpp!(unsafe [a as "QStringList"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}

impl From<i32> for QVariant {
    /// Wrapper around [`QVariant(int)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-4
    fn from(a: i32) -> QVariant {
        cpp!(unsafe [a as "int"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<u32> for QVariant {
    /// Wrapper around [`QVariant(uint)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-5
    fn from(a: u32) -> QVariant {
        cpp!(unsafe [a as "uint"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<i64> for QVariant {
    /// Wrapper around [`QVariant(int)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-4
    fn from(a: i64) -> QVariant {
        cpp!(unsafe [a as "qlonglong"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<u64> for QVariant {
    /// Wrapper around [`QVariant(uint)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-5
    fn from(a: u64) -> QVariant {
        cpp!(unsafe [a as "qulonglong"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<f32> for QVariant {
    /// Wrapper around [`QVariant(float)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-10
    fn from(a: f32) -> QVariant {
        cpp!(unsafe [a as "float"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<f64> for QVariant {
    /// Wrapper around [`QVariant(double)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-9
    fn from(a: f64) -> QVariant {
        cpp!(unsafe [a as "double"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl From<bool> for QVariant {
    /// Wrapper around [`QVariant(bool)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qvariant.html#QVariant-8
    fn from(a: bool) -> QVariant {
        cpp!(unsafe [a as "bool"] -> QVariant as "QVariant" {
            return QVariant(a);
        })
    }
}
impl<'a, T> From<&'a T> for QVariant
where
    T: Into<QVariant> + Clone,
{
    fn from(a: &'a T) -> QVariant {
        (*a).clone().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qvariant_debug_qstring() {
        let qv: QVariant = QString::from("Hello, QVariant!").into();
        assert_eq!(qv.to_qstring().to_string(), "Hello, QVariant!");
        assert_eq!(format!("{:?}", qv), "QVariant(QString: \"Hello, QVariant!\")");
    }

    #[test]
    fn qvariant_debug_bool() {
        let qv = QVariant::from(false);
        assert_eq!(qv.to_qstring().to_string(), String::from("false"));
        assert_eq!(format!("{:?}", qv), "QVariant(bool: \"false\")");
    }

    #[test]
    fn qvariant_debug_int() {
        let qv = QVariant::from(313);
        assert_eq!(qv.to_int(), 313);
        assert_eq!(format!("{:?}", qv), "QVariant(int: \"313\")");
    }
}
