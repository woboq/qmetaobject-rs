/* Copyright (C) 2018 Olivier Goffart <ogoffart@woboq.com>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
//! Various bindings and wrappers for Qt classes, enums, member/static methods and functions.
extern crate std;
use std::convert::From;
use std::fmt::Display;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};
use std::os::raw::c_char;
use std::str::Utf8Error;

#[cfg(feature = "chrono_qdatetime")]
use chrono::prelude::*;

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
    /// Wrapper around [`QDate`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qdate.html
    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    pub unsafe struct QDate as "QDate"
);
impl QDate {
    /// Wrapper around [`QDate(int y, int m, int d)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qdate.html#QDate-2
    pub fn from_y_m_d(y: i32, m: i32, d: i32) -> Self {
        cpp!(unsafe [y as "int", m as "int", d as "int"] -> QDate as "QDate" {
            return QDate(y, m, d);
        })
    }

    /// Wrapper around [`QDate::getDate(int *year, int *month, int *day)`][method] method.
    ///
    /// # Wrapper-specific
    ///
    /// Returns the year, month and day components as a tuple, instead of mutable references.
    ///
    /// [method]: https://doc.qt.io/qt-5/qdate.html#getDate
    pub fn get_y_m_d(&self) -> (i32, i32, i32) {
        let mut res = (0, 0, 0);
        let (ref mut y, ref mut m, ref mut d) = res;
        cpp!(unsafe [self as "const QDate*", y as "int*", m as "int*", d as "int*"] {
            return self->getDate(y, m, d);
        });
        res
    }

    /// Wrapper around [`QDate::isValid()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qdate.html#isValid
    pub fn is_valid(&self) -> bool {
        cpp!(unsafe [self as "const QDate*"] -> bool as "bool" {
            return self->isValid();
        })
    }
}
#[cfg(feature = "chrono_qdatetime")]
impl From<NaiveDate> for QDate {
    fn from(a: NaiveDate) -> QDate {
        QDate::from_y_m_d(a.year() as i32, a.month() as i32, a.day() as i32)
    }
}
#[cfg(feature = "chrono_qdatetime")]
impl Into<NaiveDate> for QDate {
    fn into(self) -> NaiveDate {
        let (y, m, d) = self.get_y_m_d();
        NaiveDate::from_ymd(y, m as u32, d as u32)
    }
}

#[test]
fn test_qdate() {
    let date = QDate::from_y_m_d(2019, 10, 22);
    assert_eq!((2019, 10, 22), date.get_y_m_d());
}

#[test]
fn test_qdate_is_valid() {
    let valid_qdate = QDate::from_y_m_d(2019, 10, 26);
    assert!(valid_qdate.is_valid());

    let invalid_qdate = QDate::from_y_m_d(-1, -1, -1);
    assert!(!invalid_qdate.is_valid());
}

#[cfg(feature = "chrono_qdatetime")]
#[test]
fn test_qdate_chrono() {
    let chrono_date = NaiveDate::from_ymd(2019, 10, 22);
    let qdate: QDate = chrono_date.into();
    let actual_chrono_date: NaiveDate = qdate.into();

    // Ensure that conversion works for both the Into trait and get_y_m_d() function
    assert_eq!((2019, 10, 22), qdate.get_y_m_d());
    assert_eq!(chrono_date, actual_chrono_date);
}

cpp_class!(
    /// Wrapper around [`QTime`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qtime.html
    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    pub unsafe struct QTime as "QTime"
);
impl QTime {
    /// Wrapper around [`QTime(int h, int m, int s = 0, int ms = 0)`][ctor] constructor.
    ///
    /// # Wrapper-specific
    ///
    /// Default arguments converted to `Option`s.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qtime.html#QTime-2
    pub fn from_h_m_s_ms(h: i32, m: i32, s: Option<i32>, ms: Option<i32>) -> Self {
        let s = s.unwrap_or(0);
        let ms = ms.unwrap_or(0);

        cpp!(unsafe [h as "int", m as "int", s as "int", ms as "int"] -> QTime as "QTime" {
            return QTime(h, m, s, ms);
        })
    }

    /// Wrapper around [`QTime::hour()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qtime.html#hour
    pub fn get_hour(&self) -> i32 {
        cpp!(unsafe [self as "const QTime*"] -> i32 as "int" {
            return self->hour();
        })
    }

    /// Wrapper around [`QTime::minute()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qtime.html#minute
    pub fn get_minute(&self) -> i32 {
        cpp!(unsafe [self as "const QTime*"] -> i32 as "int" {
            return self->minute();
        })
    }

    /// Wrapper around [`QTime::second()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qtime.html#second
    pub fn get_second(&self) -> i32 {
        cpp!(unsafe [self as "const QTime*"] -> i32 as "int" {
            return self->second();
        })
    }

    /// Wrapper around [`QTime::msec()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qtime.html#msec
    pub fn get_msec(&self) -> i32 {
        cpp!(unsafe [self as "const QTime*"] -> i32 as "int" {
            return self->msec();
        })
    }

    /// Convenience function for obtaining the hour, minute, second and millisecond components.
    pub fn get_h_m_s_ms(&self) -> (i32, i32, i32, i32) {
        (self.get_hour(), self.get_minute(), self.get_second(), self.get_msec())
    }

    /// Wrapper around [`QTime::isValid()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qtime.html#isValid
    pub fn is_valid(&self) -> bool {
        cpp!(unsafe [self as "const QTime*"] -> bool as "bool" {
            return self->isValid();
        })
    }
}

#[cfg(feature = "chrono_qdatetime")]
impl From<NaiveTime> for QTime {
    fn from(a: NaiveTime) -> QTime {
        QTime::from_h_m_s_ms(
            a.hour() as i32,
            a.minute() as i32,
            Some(a.second() as i32),
            Some(a.nanosecond() as i32 / 1_000_000),
        )
    }
}

#[cfg(feature = "chrono_qdatetime")]
impl Into<NaiveTime> for QTime {
    fn into(self) -> NaiveTime {
        let (h, m, s, ms) = self.get_h_m_s_ms();
        NaiveTime::from_hms_milli(h as u32, m as u32, s as u32, ms as u32)
    }
}

#[test]
fn test_qtime() {
    let qtime = QTime::from_h_m_s_ms(10, 30, Some(40), Some(300));
    assert_eq!((10, 30, 40, 300), qtime.get_h_m_s_ms());
}

#[cfg(feature = "chrono_qdatetime")]
#[test]
fn test_qtime_chrono() {
    let chrono_time = NaiveTime::from_hms(10, 30, 50);
    let qtime: QTime = chrono_time.into();
    let actual_chrono_time: NaiveTime = qtime.into();

    // Ensure that conversion works for both the Into trait and get_h_m_s_ms() function
    assert_eq!((10, 30, 50, 0), qtime.get_h_m_s_ms());
    assert_eq!(chrono_time, actual_chrono_time);
}

#[test]
fn test_qtime_is_valid() {
    let valid_qtime = QTime::from_h_m_s_ms(10, 30, Some(40), Some(300));
    assert!(valid_qtime.is_valid());

    let invalid_qtime = QTime::from_h_m_s_ms(10, 30, Some(40), Some(9999));
    assert!(!invalid_qtime.is_valid());
}

cpp_class!(
    /// Wrapper around [`QDateTime`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qdatetime.html
    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    pub unsafe struct QDateTime as "QDateTime"
);
impl QDateTime {
    /// Wrapper around [`QDateTime(const QDateTime &other)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qdatetime.html#QDateTime-1
    pub fn from_date(date: QDate) -> Self {
        cpp!(unsafe [date as "QDate"] -> QDateTime as "QDateTime" {
            return QDateTime(date);
        })
    }

    /// Wrapper around [`QDateTime(const QDate &date, const QTime &time, Qt::TimeSpec spec = Qt::LocalTime)`][ctor] constructor.
    ///
    /// # Wrapper-specific
    ///
    /// `spec` is left as it is, thus it is always `Qt::LocalTime`.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qdatetime.html#QDateTime-2
    pub fn from_date_time_local_timezone(date: QDate, time: QTime) -> Self {
        cpp!(unsafe [date as "QDate", time as "QTime"] -> QDateTime as "QDateTime" {
            return QDateTime(date, time);
        })
    }

    /// Wrapper around [`QDateTime::date()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qdatetime.html#date
    pub fn get_date(&self) -> QDate {
        cpp!(unsafe [self as "const QDateTime*"] -> QDate as "QDate" {
            return self->date();
        })
    }

    /// Wrapper around [`QDateTime::time()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qdatetime.html#time
    pub fn get_time(&self) -> QTime {
        cpp!(unsafe [self as "const QDateTime*"] -> QTime as "QTime" {
            return self->time();
        })
    }

    /// Convenience function for obtaining both date and time components.
    pub fn get_date_time(&self) -> (QDate, QTime) {
        (self.get_date(), self.get_time())
    }

    /// Wrapper around [`QDateTime::isValid()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qdatetime.html#isValid
    pub fn is_valid(&self) -> bool {
        cpp!(unsafe [self as "const QDateTime*"] -> bool as "bool" {
            return self->isValid();
        })
    }
}

#[test]
fn test_qdatetime_from_date() {
    let qdate = QDate::from_y_m_d(2019, 10, 22);
    let qdatetime = QDateTime::from_date(qdate);
    let actual_qdate = qdatetime.get_date();

    assert_eq!((2019, 10, 22), actual_qdate.get_y_m_d());
}

#[test]
fn test_qdatetime_from_date_time_local_timezone() {
    let qdate = QDate::from_y_m_d(2019, 10, 22);
    let qtime = QTime::from_h_m_s_ms(10, 30, Some(40), Some(300));
    let qdatetime = QDateTime::from_date_time_local_timezone(qdate, qtime);
    let (actual_qdate, actual_qtime) = qdatetime.get_date_time();

    assert_eq!((2019, 10, 22), actual_qdate.get_y_m_d());
    assert_eq!((10, 30, 40, 300), actual_qtime.get_h_m_s_ms());

    assert_eq!(10, actual_qtime.get_hour());
    assert_eq!(30, actual_qtime.get_minute());
    assert_eq!(40, actual_qtime.get_second());
    assert_eq!(300, actual_qtime.get_msec());
}

#[test]
fn test_qdatetime_is_valid() {
    let valid_qdate = QDate::from_y_m_d(2019, 10, 26);
    let invalid_qdate = QDate::from_y_m_d(-1, -1, -1);

    let valid_qtime = QTime::from_h_m_s_ms(10, 30, Some(40), Some(300));
    let invalid_qtime = QTime::from_h_m_s_ms(10, 30, Some(40), Some(9999));

    let valid_qdatetime_from_date = QDateTime::from_date(valid_qdate);
    assert!(valid_qdatetime_from_date.is_valid());

    let valid_qdatetime_from_valid_date_valid_time =
        QDateTime::from_date_time_local_timezone(valid_qdate, valid_qtime);
    assert!(valid_qdatetime_from_valid_date_valid_time.is_valid());

    // Refer to the documentation for QDateTime's constructors using QDate, QTime.
    // If the date is valid, but the time is not, the time will be set to midnight
    let valid_qdatetime_from_valid_date_invalid_time =
        QDateTime::from_date_time_local_timezone(valid_qdate, invalid_qtime);
    assert!(valid_qdatetime_from_valid_date_invalid_time.is_valid());

    let invalid_qdatetime_from_invalid_date_valid_time =
        QDateTime::from_date_time_local_timezone(invalid_qdate, valid_qtime);
    assert!(!invalid_qdatetime_from_invalid_date_valid_time.is_valid());

    let invalid_qdatetime_from_invalid_date_invalid_time =
        QDateTime::from_date_time_local_timezone(invalid_qdate, invalid_qtime);
    assert!(!invalid_qdatetime_from_invalid_date_invalid_time.is_valid());
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
    /// Wrapper around [`QVariant`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qvariant.html
    #[derive(PartialEq)]
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

    // FIXME: do more wrappers
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

cpp_class!(
    /// Wrapper around [`QVariantList`][type] typedef.
    ///
    /// [type]: https://doc.qt.io/qt-5/qvariant.html#QVariantList-typedef
    pub unsafe struct QVariantList as "QVariantList"
);
impl QVariantList {
    /// Wrapper around [`append(const T &)`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qlist.html#append
    pub fn push(&mut self, value: QVariant) {
        cpp!(unsafe [self as "QVariantList*", value as "QVariant"] {
            self->append(std::move(value));
        })
    }

    /// Wrapper around [`insert(int, const QVariant &)`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qlist.html#insert
    pub fn insert(&mut self, index: usize, element: QVariant) {
        cpp!(unsafe [self as "QVariantList*", index as "size_t", element as "QVariant"] {
            self->insert(index, std::move(element));
        })
    }

    /// Wrapper around [`takeAt(int)`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qlist.html#takeAt
    pub fn remove(&mut self, index: usize) -> QVariant {
        cpp!(unsafe [self as "QVariantList*", index as "size_t"] -> QVariant as "QVariant" {
            return self->takeAt(index);
        })
    }

    /// Wrapper around [`size()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qlist.html#size
    pub fn len(&self) -> usize {
        cpp!(unsafe [self as "QVariantList*"] -> usize as "size_t" {
            return self->size();
        })
    }

    /// Wrapper around [`isEmpty()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qlist.html#isEmpty
    pub fn is_empty(&self) -> bool {
        cpp!(unsafe [self as "QVariantList*"] -> bool as "bool" {
            return self->isEmpty();
        })
    }
}
impl Index<usize> for QVariantList {
    type Output = QVariant;

    /// Wrapper around [`at(int)`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qlist.html#at
    fn index(&self, index: usize) -> &QVariant {
        assert!(index < self.len());
        unsafe {
            &*cpp!([self as "QVariantList*", index as "size_t"] -> *const QVariant as "const QVariant*" {
                return &self->at(index);
            })
        }
    }
}
impl IndexMut<usize> for QVariantList {
    /// Wrapper around [`operator[](int)`][method] operator method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qlist.html#operator-5b-5d
    fn index_mut(&mut self, index: usize) -> &mut QVariant {
        assert!(index < self.len());
        unsafe {
            &mut *cpp!([self as "QVariantList*", index as "size_t"] -> *mut QVariant as "QVariant*" {
                return &(*self)[index];
            })
        }
    }
}

/// Internal class used to iterate over a [`QVariantList`][]
///
/// [`QVariantList`]: ./struct.QVariantList.html
pub struct QVariantListIterator<'a> {
    list: &'a QVariantList,
    index: usize,
    size: usize,
}

impl<'a> Iterator for QVariantListIterator<'a> {
    type Item = &'a QVariant;
    fn next(&mut self) -> Option<&'a QVariant> {
        if self.index == self.size {
            None
        } else {
            self.index += 1;
            Some(&self.list[self.index - 1])
        }
    }
}

impl<'a> IntoIterator for &'a QVariantList {
    type Item = &'a QVariant;
    type IntoIter = QVariantListIterator<'a>;

    fn into_iter(self) -> QVariantListIterator<'a> {
        QVariantListIterator::<'a> { list: self, index: 0, size: self.len() }
    }
}

impl<T> FromIterator<T> for QVariantList
where
    T: Into<QVariant>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> QVariantList {
        let mut l = QVariantList::default();
        for i in iter {
            l.push(i.into());
        }
        l
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_qvariantlist() {
        let mut q = QVariantList::default();
        q.push(42.into());
        q.push(QString::from("Hello").into());
        q.push(QByteArray::from("Hello").into());
        assert_eq!(q[0].to_qbytearray().to_string(), "42");
        assert_eq!(q[1].to_qbytearray().to_string(), "Hello");
        assert_eq!(q[2].to_qbytearray().to_string(), "Hello");
        let x: Vec<QByteArray> = q.into_iter().map(|x| x.to_qbytearray()).collect();
        assert_eq!(x[0].to_string(), "42");
        assert_eq!(x[1].to_string(), "Hello");
        assert_eq!(x[2].to_string(), "Hello");
    }

    #[test]
    fn test_qvariantlist_from_iter() {
        let v = vec![1u32, 2u32, 3u32];
        let qvl: QVariantList = v.iter().collect();
        assert_eq!(qvl.len(), 3);
        assert_eq!(qvl[1].to_qbytearray().to_string(), "2");
    }

    #[test]
    fn test_qstring_and_qbytearray() {
        let qba1: QByteArray = (b"hello" as &[u8]).into();
        let qba2: QByteArray = "hello".into();
        let s: String = "hello".into();
        let qba3: QByteArray = s.clone().into();

        assert_eq!(qba1.to_string(), "hello");
        assert_eq!(qba2.to_string(), "hello");
        assert_eq!(qba3.to_string(), "hello");

        let qs1: QString = "hello".into();
        let qs2: QString = s.into();
        let qba4: QByteArray = qs1.clone().into();

        assert_eq!(qs1.to_string(), "hello");
        assert_eq!(qs2.to_string(), "hello");
        assert_eq!(qba4.to_string(), "hello");
    }

    #[test]
    fn test_qvariant_datetime() {
        use crate::QMetaType;

        let dt = QDateTime::from_date_time_local_timezone(
            QDate::from_y_m_d(2019, 10, 23),
            QTime::from_h_m_s_ms(10, 30, Some(40), Some(100)),
        );
        let v = QVariant::from(dt);
        let qstring = QString::from_qvariant(v.clone()).unwrap();
        let mut s = qstring.to_string();
        if s.ends_with(".100") {
            // Old version of qt did not include the milliseconds, so remove it
            s.truncate(s.len() - 4);
        }
        assert_eq!(s, "2019-10-23T10:30:40");
        let qdate = QDate::from_qvariant(v.clone()).unwrap();
        assert!(qdate == QDate::from_y_m_d(2019, 10, 23));
        assert!(qdate != QDate::from_y_m_d(2019, 10, 24));

        let qtime = QTime::from_qvariant(v.clone()).unwrap();
        assert!(qtime == QTime::from_h_m_s_ms(10, 30, Some(40), Some(100)));
        assert!(qtime != QTime::from_h_m_s_ms(10, 30, Some(40), None));
    }
}

cpp_class!(
    /// Wrapper around [`QModelIndex`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qmodelindex.html
    #[derive(PartialEq, Eq)]
    pub unsafe struct QModelIndex as "QModelIndex"
);
impl QModelIndex {
    /// Wrapper around [`internalId()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qmodelindex.html#internalId
    pub fn id(&self) -> usize {
        cpp!(unsafe [self as "const QModelIndex*"] -> usize as "uintptr_t" { return self->internalId(); })
    }

    /// Wrapper around [`column()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qmodelindex.html#column
    pub fn column(&self) -> i32 {
        cpp!(unsafe [self as "const QModelIndex*"] -> i32 as "int" { return self->column(); })
    }

    /// Wrapper around [`row()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qmodelindex.html#row
    pub fn row(&self) -> i32 {
        cpp!(unsafe [self as "const QModelIndex*"] -> i32 as "int" { return self->row(); })
    }

    /// Wrapper around [`isValid()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qmodelindex.html#isValid
    pub fn is_valid(&self) -> bool {
        cpp!(unsafe [self as "const QModelIndex*"] -> bool as "bool" { return self->isValid(); })
    }
}

/// Bindings for [`qreal`][type] typedef.
///
/// [type]: https://doc.qt.io/qt-5/qtglobal.html#qreal-typedef
#[allow(non_camel_case_types)]
#[cfg(qreal_is_float)]
pub type qreal = f32;

#[allow(non_camel_case_types)]
#[cfg(not(qreal_is_float))]
pub type qreal = f64;

/// Bindings for [`QRectF`][class] class.
///
/// [class]: https://doc.qt.io/qt-5/qrectf.html
#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct QRectF {
    pub x: qreal,
    pub y: qreal,
    pub width: qreal,
    pub height: qreal,
}
impl QRectF {
    /// Wrapper around [`contains(const QPointF &)`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qrectf.html#contains
    pub fn contains(&self, pos: QPointF) -> bool {
        cpp!(unsafe [self as "const QRectF*", pos as "QPointF"] -> bool as "bool" {
            return self->contains(pos);
        })
    }

    // XXX: shouldn't it be a wrapper for cpp call?
    pub fn top_left(&self) -> QPointF {
        QPointF { x: self.x, y: self.y }
    }
}

/// Bindings for [`QPointF`][class] class.
///
/// [class]: https://doc.qt.io/qt-5/qpointf.html
#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct QPointF {
    pub x: qreal,
    pub y: qreal,
}
impl std::ops::Add for QPointF {
    type Output = QPointF;
    /// Wrapper around [`operator+(const QPointF &, const QPointF &)`][func] function.
    ///
    /// [func]: https://doc.qt.io/qt-5/qpointf.html#operator-2b
    fn add(self, other: QPointF) -> QPointF {
        QPointF { x: self.x + other.x, y: self.y + other.y }
    }
}
impl std::ops::AddAssign for QPointF {
    /// Wrapper around [`operator+=(const QPointF &`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qpointf.html#operator-2b-eq
    fn add_assign(&mut self, other: QPointF) {
        *self = QPointF { x: self.x + other.x, y: self.y + other.y };
    }
}

#[test]
fn test_qpointf_qrectf() {
    let rect = QRectF { x: 200., y: 150., width: 60., height: 75. };
    let pt = QPointF { x: 12., y: 5.5 };
    assert!(!rect.contains(pt));
    assert!(rect.contains(pt + rect.top_left()));
}

cpp_class!(
    /// Wrapper around [`QColor`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qcolor.html
    #[derive(Default, Clone, Copy, PartialEq)]
    pub unsafe struct QColor as "QColor"
);
impl QColor {
    /// Wrapper around [`QColor(QLatin1String)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qcolor.html#QColor-8
    pub fn from_name(name: &str) -> Self {
        let len = name.len();
        let ptr = name.as_ptr();
        cpp!(unsafe [len as "size_t", ptr as "char*"] -> QColor as "QColor" {
            return QColor(QLatin1String(ptr, len));
        })
    }

    /// Wrapper around [`fromRgbF(qreal r, qreal g, qreal b, qreal a = 1.0)`][ctor] constructor.
    ///
    /// # Wrapper-specific
    ///
    /// Alpha is left at default `1.0`. To set it to something other that 1.0, use [`from_rgba_f`][].
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qcolor.html#fromRgbF
    /// [`from_rgba_f`]: #method.from_rgba_f
    pub fn from_rgb_f(r: qreal, g: qreal, b: qreal) -> Self {
        cpp!(unsafe [r as "qreal", g as "qreal", b as "qreal"] -> QColor as "QColor" {
            return QColor::fromRgbF(r, g, b);
        })
    }

    /// Wrapper around [`fromRgbF(qreal r, qreal g, qreal b, qreal a = 1.0)`][ctor] constructor.
    ///
    /// # Wrapper-specific
    ///
    /// Same as [`from_rgb_f`][], but accept an alpha value
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qcolor.html#fromRgbF
    /// [`from_rgb_f`]: #method.from_rgb_f
    pub fn from_rgba_f(r: qreal, g: qreal, b: qreal, a: qreal) -> Self {
        cpp!(unsafe [r as "qreal", g as "qreal", b as "qreal", a as "qreal"] -> QColor as "QColor" {
            return QColor::fromRgbF(r, g, b, a);
        })
    }
    /// Wrapper around [`getRgbF(qreal *r, qreal *g, qreal *b, qreal *a = nullptr)`][method] method.
    ///
    /// # Wrapper-specific
    ///
    /// Returns red, green, blue and alpha components as a tuple, instead of mutable references.
    ///
    /// [method]: https://doc.qt.io/qt-5/qcolor.html#getRgbF
    pub fn get_rgba(&self) -> (qreal, qreal, qreal, qreal) {
        let res = (0., 0., 0., 0.);
        let (ref r, ref g, ref b, ref a) = res;
        cpp!(unsafe [self as "const QColor*", r as "qreal*", g as "qreal*", b as "qreal*", a as "qreal*"] {
            return self->getRgbF(r, g, b, a);
        });
        res
    }
}

#[test]
fn test_qcolor() {
    let blue1 = QColor::from_name("blue");
    let blue2 = QColor::from_rgb_f(0., 0., 1.);
    assert_eq!(blue1.get_rgba().0, 0.);
    assert_eq!(blue1.get_rgba().2, 1.);
    assert!(blue1 == blue2);

    let red1 = QColor::from_name("red");
    let red2 = QColor::from_rgb_f(1., 0., 0.);
    assert_eq!(red1.get_rgba().0, 1.);
    assert_eq!(red1.get_rgba().2, 0.);
    assert!(red1 == red2);
    assert!(blue1 != red1);
}

/// Bindings for [`QSize`][class] class.
///
/// [class]: https://doc.qt.io/qt-5/qsize.html
#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct QSize {
    pub width: u32,
    pub height: u32,
}

/// Bindings for [`QImage::Format`][class] enum class.
///
/// [class]: https://doc.qt.io/qt-5/qimage.html#Format-enum
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum ImageFormat {
    Invalid = 0,
    Mono = 1,
    MonoLSB = 2,
    Indexed8 = 3,
    RGB32 = 4,
    ARGB32 = 5,
    ARGB32_Premultiplied = 6,
    RGB16 = 7,
    ARGB8565_Premultiplied = 8,
    RGB666 = 9,
    ARGB6666_Premultiplied = 10,
    RGB555 = 11,
    ARGB8555_Premultiplied = 12,
    RGB888 = 13,
    RGB444 = 14,
    ARGB4444_Premultiplied = 15,
    RGBX8888 = 16,
    RGBA8888 = 17,
    RGBA8888_Premultiplied = 18,
    BGR30 = 19,
    A2BGR30_Premultiplied = 20,
    RGB30 = 21,
    A2RGB30_Premultiplied = 22,
    Alpha8 = 23,
    Grayscale8 = 24,
    Grayscale16 = 28,
    RGBX64 = 25,
    RGBA64 = 26,
    RGBA64_Premultiplied = 27,
    BGR888 = 29,
}
cpp_class!(
    /// Wrapper around [`QImage`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qimage.html
    pub unsafe struct QImage as "QImage"
);
impl QImage {
    /// Wrapper around [`QImage(const QString &fileName, const char *format = nullptr)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qimage.html#QImage-8
    pub fn load_from_file(filename: QString) -> Self {
        cpp!(unsafe [filename as "QString"] -> QImage as "QImage" {
            return QImage(filename);
        })
    }

    /// Wrapper around [`QImage(const QSize &, QImage::Format)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qimage.html#QImage-1
    pub fn new(size: QSize, format: ImageFormat) -> Self {
        cpp!(unsafe [size as "QSize", format as "QImage::Format" ] -> QImage as "QImage" {
            return QImage(size, format);
        })
    }

    /// Wrapper around [`size()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qimage.html#size
    pub fn size(&self) -> QSize {
        cpp!(unsafe [self as "const QImage*"] -> QSize as "QSize" { return self->size(); })
    }

    /// Wrapper around [`format()`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qimage.html#format
    pub fn format(&self) -> ImageFormat {
        cpp!(unsafe [self as "const QImage*"] -> ImageFormat as "QImage::Format" { return self->format(); })
    }

    /// Wrapper around [`fill(const QColor &)`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qimage.html#fill-1
    pub fn fill(&mut self, color: QColor) {
        cpp!(unsafe [self as "QImage*", color as "QColor"] { self->fill(color); })
    }

    /// Wrapper around [`setPixelColor(const QPoint &, const QColor &)`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qimage.html#setPixelColor
    pub fn set_pixel_color(&mut self, x: u32, y: u32, color: QColor) {
        cpp!(unsafe [self as "QImage*", x as "int", y as "int", color as "QColor"] {
            self->setPixelColor(x, y, color);
        })
    }

    /// Wrapper around [`pixelColor(const QPoint &)`][method] method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qimage.html#pixelColor
    pub fn get_pixel_color(&mut self, x: u32, y: u32) -> QColor {
        cpp!(unsafe [self as "QImage*", x as "int", y as "int"] -> QColor as "QColor" {
            return self->pixelColor(x, y);
        })
    }
}
