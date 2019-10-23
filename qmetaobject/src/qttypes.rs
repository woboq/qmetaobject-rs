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
    /// Wrapper around Qt's QByteArray
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
    /// Constructs a QByteArray from a slice. (Copy the slice.)
    fn from(s: &'a [u8]) -> QByteArray {
        let len = s.len();
        let ptr = s.as_ptr();
        unsafe {
            cpp!([len as "size_t", ptr as "char*"] -> QByteArray as "QByteArray"
        { return QByteArray(ptr, len); })
        }
    }
}
impl<'a> From<&'a str> for QByteArray {
    /// Constructs a QByteArray from a &str. (Copy the string.)
    fn from(s: &'a str) -> QByteArray {
        s.as_bytes().into()
    }
}

impl From<String> for QByteArray {
    /// Constructs a QByteArray from a String. (Copy the string.)
    fn from(s: String) -> QByteArray {
        QByteArray::from(&*s)
    }
}
impl From<QString> for QByteArray {
    /// Converts a QString to a QByteArray
    fn from(s: QString) -> QByteArray {
        unsafe {
            cpp!([s as "QString"] -> QByteArray as "QByteArray"
            { return std::move(s).toUtf8(); })
        }
    }
}
impl Display for QByteArray {
    /// Prints the contents of the QByteArray if it contains UTF-8,  nothing otherwise
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe {
            let c_ptr = cpp!([self as "const QByteArray*"] -> *const c_char as "const char*" {
                return self->constData();
            });
            f.write_str(
                std::ffi::CStr::from_ptr(c_ptr)
                    .to_str()
                    .map_err(|_| Default::default())?,
            )
        }
    }
}
impl std::fmt::Debug for QByteArray {
    /// Prints the contents of the QByteArray if it contains UTF-8,  nothing otherwise
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

cpp_class!(
    /// Wrapper around Qt's QDate class
    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    pub unsafe struct QDate as "QDate"
);

impl QDate {
    /// Constructs a QDate from the year, month and date.
    /// Refer to the Qt documentation for the QDate constructor.
    pub fn from_y_m_d(y: i32, m: i32, d: i32) -> Self {
        cpp!(unsafe [y as "int", m as "int", d as "int"] -> QDate as "QDate" {
            return QDate(y, m, d);
        })
    }

    /// Returns the year, month and day components.
    /// Refer to the Qt documentation of QDate::getDate.
    pub fn get_y_m_d(&self) -> (i32, i32, i32) {
        let res = (0, 0, 0);
        let (ref y, ref m, ref d) = res;
        cpp!(unsafe [self as "const QDate*", y as "int*", m as "int*", d as "int*"] {
            return self->getDate(y, m, d);
        });
        res
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
    /// Wrapper around Qt's QTime class
    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    pub unsafe struct QTime as "QTime"
);

impl QTime {
    /// Constructs a QTime from hours and minutes, and optionally seconds and milliseconds.
    /// Refer to the Qt documentation for the QTime constructor.
    pub fn from_h_m_s_ms(h: i32, m: i32, s: Option<i32>, ms: Option<i32>) -> Self {
        let s = s.unwrap_or(0);
        let ms = ms.unwrap_or(0);

        cpp!(unsafe [h as "int", m as "int", s as "int", ms as "int"] -> QTime as "QTime" {
            return QTime(h, m, s, ms);
        })
    }

    /// Refer to the Qt documentation for QTime::hour.
    pub fn get_hour(&self) -> i32 {
        cpp!(unsafe [self as "const QTime*"] -> i32 as "int" {
            return self->hour();
        })
    }

    /// Refer to the Qt documentation for QTime::minute.
    pub fn get_minute(&self) -> i32 {
        cpp!(unsafe [self as "const QTime*"] -> i32 as "int" {
            return self->minute();
        })
    }

    /// Refer to the Qt documentation for QTime::second.
    pub fn get_second(&self) -> i32 {
        cpp!(unsafe [self as "const QTime*"] -> i32 as "int" {
            return self->second();
        })
    }

    /// Refer to the Qt documentation for QTime::msec.
    pub fn get_msec(&self) -> i32 {
        cpp!(unsafe [self as "const QTime*"] -> i32 as "int" {
            return self->msec();
        })
    }

    /// Convenience function for obtaining the hour, minute, second and millisecond components.
    pub fn get_h_m_s_ms(&self) -> (i32, i32, i32, i32) {
        (self.get_hour(), self.get_minute(), self.get_second(), self.get_msec())
    }
}

#[cfg(feature = "chrono_qdatetime")]
impl From<NaiveTime> for QTime {
    fn from(a: NaiveTime) -> QTime {
        QTime::from_h_m_s_ms(a.hour() as i32, a.minute() as i32, Some(a.second() as i32), Some(a.nanosecond() as i32 / 1_000_000))
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

cpp_class!(
    /// Wrapper around Qt's QDateTime class
    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    pub unsafe struct QDateTime as "QDateTime"
);

impl QDateTime {
    /// Constructs a QDateTime from a QDate.
    /// Refer to the documentation for QDateTime's constructor using QDate.
    pub fn from_date(date: QDate) -> Self {
        cpp!(unsafe [date as "QDate"] -> QDateTime as "QDateTime" {
            return QDateTime(date);
        })
    }

    /// Constructs a QDateTime from a QDate and a QTime, using the current system timezone.
    ///
    /// Equivalent to the C++ code `QDateTime(date, time)`.
    /// Refer to the documentation for QDateTime's constructor using QDate, QTime and Qt::TimeSpec.
    pub fn from_date_time_local_timezone(date: QDate, time: QTime) -> Self {
        cpp!(unsafe [date as "QDate", time as "QTime"] -> QDateTime as "QDateTime" {
            return QDateTime(date, time);
        })
    }

    /// Gets the date component from a QDateTime.
    /// Refer to the documentation for QDateTime::date.
    pub fn get_date(&self) -> QDate {
        cpp!(unsafe [self as "const QDateTime*"] -> QDate as "QDate" {
            return self->date();
        })
    }

    /// Gets the time component from a QDateTime.
    /// Refer to the documentation for QDateTime::time.
    pub fn get_time(&self) -> QTime {
        cpp!(unsafe [self as "const QDateTime*"] -> QTime as "QTime" {
            return self->time();
        })
    }

    /// Convenience function for obtaining both date and time components.
    pub fn get_date_time(&self) -> (QDate, QTime) {
        (self.get_date(), self.get_time())
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

cpp_class!(
/// Wrapper around Qt's QUrl class
    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    pub unsafe struct QUrl as "QUrl"
);
impl From<QString> for QUrl {
    fn from(qstring: QString) -> QUrl {
        unsafe { cpp!([qstring as "QString"] -> QUrl as "QUrl" {
            return QUrl(qstring);
        })}
    }
}

cpp_class!(
/// Wrapper around Qt's QString class
#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub unsafe struct QString as "QString");
impl QString {
    /// Return a slice containing the UTF-16 data
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
    fn from(qurl: QUrl) -> QString {
        unsafe { cpp!([qurl as "QUrl"] -> QString as "QString" {
            return qurl.toString();
        })}
    }
}
impl<'a> From<&'a str> for QString {
    /// Copy the data from a &str
    fn from(s: &'a str) -> QString {
        let len = s.len();
        let ptr = s.as_ptr();
        unsafe { cpp!([len as "size_t", ptr as "char*"] -> QString as "QString"
        { return QString::fromUtf8(ptr, len); })}
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
/// Wrapper around a QVariant
#[derive(PartialEq)] pub unsafe struct QVariant as "QVariant");
impl QVariant {
    pub fn to_qbytearray(&self) -> QByteArray {
        // FIXME
        unsafe {
            cpp!([self as "const QVariant*"] -> QByteArray as "QByteArray" { return self->toByteArray(); })
        }
    }

    pub fn to_bool(&self) -> bool {
        unsafe { cpp!([self as "const QVariant*"] -> bool as "bool" { return self->toBool(); }) }
    }
}
impl From<QString> for QVariant {
    fn from(a: QString) -> QVariant {
        unsafe { cpp!([a as "QString"] -> QVariant as "QVariant" { return QVariant(a); }) }
    }
}
impl From<QByteArray> for QVariant {
    fn from(a: QByteArray) -> QVariant {
        unsafe { cpp!([a as "QByteArray"] -> QVariant as "QVariant" { return QVariant(a); }) }
    }
}
impl From<QDate> for QVariant {
    fn from(a: QDate) -> QVariant {
        unsafe { cpp!([a as "QDate"] -> QVariant as "QVariant" { return QVariant(a); }) }
    }
}
impl From<QTime> for QVariant {
    fn from(a: QTime) -> QVariant {
        unsafe { cpp!([a as "QTime"] -> QVariant as "QVariant" { return QVariant(a); }) }
    }
}
impl From<QDateTime> for QVariant {
    fn from(a: QDateTime) -> QVariant {
        unsafe { cpp!([a as "QDateTime"] -> QVariant as "QVariant" { return QVariant(a); }) }
    }
}
impl From<QVariantList> for QVariant {
    fn from(a: QVariantList) -> QVariant {
        unsafe { cpp!([a as "QVariantList"] -> QVariant as "QVariant" { return QVariant(a); }) }
    }
}
impl From<i32> for QVariant {
    fn from(a: i32) -> QVariant {
        unsafe { cpp!([a as "int"] -> QVariant as "QVariant" { return QVariant(a); }) }
    }
}
impl From<u32> for QVariant {
    fn from(a: u32) -> QVariant {
        unsafe { cpp!([a as "uint"] -> QVariant as "QVariant" { return QVariant(a); }) }
    }
}
impl From<f32> for QVariant {
    fn from(a: f32) -> QVariant {
        unsafe { cpp!([a as "float"] -> QVariant as "QVariant" { return QVariant(a); }) }
    }
}
impl From<f64> for QVariant {
    fn from(a: f64) -> QVariant {
        unsafe { cpp!([a as "double"] -> QVariant as "QVariant" { return QVariant(a); }) }
    }
}
impl From<bool> for QVariant {
    fn from(a: bool) -> QVariant {
        unsafe { cpp!([a as "bool"] -> QVariant as "QVariant" { return QVariant(a); }) }
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
/// Wrapper around QVariantList
pub unsafe struct QVariantList as "QVariantList");
impl QVariantList {
    pub fn push(&mut self, value: QVariant) {
        cpp!(unsafe [self as "QVariantList*", value as "QVariant"]
            { self->append(std::move(value)); }
        )
    }
    pub fn insert(&mut self, index: usize, element: QVariant) {
        cpp!(unsafe [self as "QVariantList*", index as "size_t", element as "QVariant"]
            { self->insert(index, std::move(element)); }
        )
    }
    pub fn remove(&mut self, index: usize) -> QVariant {
        cpp!(unsafe [self as "QVariantList*", index as "size_t"] -> QVariant as "QVariant"
            { return self->takeAt(index); }
        )
    }
    pub fn len(&self) -> usize {
        unsafe {cpp!([self as "QVariantList*"] -> usize as "size_t"
            { return self->size(); }
        )}
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Index<usize> for QVariantList {
    type Output = QVariant;
    fn index(&self, index: usize) -> &QVariant {
        assert!(index < self.len());
        unsafe { &*cpp!([self as "QVariantList*", index as "size_t"] -> *const QVariant as "const QVariant*"
            { return &self->at(index); }
        )}
    }
}
impl IndexMut<usize> for QVariantList {
    fn index_mut(&mut self, index: usize) -> &mut QVariant {
        assert!(index < self.len());
        unsafe { &mut *cpp!([self as "QVariantList*", index as "size_t"] -> *mut QVariant as "QVariant*"
            { return &(*self)[index]; }
        )}
    }
}

/// Iternal class used to iterate over a QVariantList
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
        QVariantListIterator::<'a> {
            list: self,
            index: 0,
            size: self.len(),
        }
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
    fn test_qstring_and_qbytearrazy() {
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
}

cpp_class!(
/// Wrapper around Qt's QModelIndex
#[derive(PartialEq, Eq)] pub unsafe struct QModelIndex as "QModelIndex");
impl QModelIndex {
    /// Return the QModelIndex::internalId
    pub fn id(&self) -> usize {
        unsafe {
            cpp!([self as "const QModelIndex*"] -> usize as "uintptr_t" { return self->internalId(); })
        }
    }
    pub fn column(&self) -> i32 {
        unsafe { cpp!([self as "const QModelIndex*"] -> i32 as "int" { return self->column(); }) }
    }
    pub fn row(&self) -> i32 {
        unsafe { cpp!([self as "const QModelIndex*"] -> i32 as "int" { return self->row(); }) }
    }
    pub fn is_valid(&self) -> bool {
        unsafe { cpp!([self as "const QModelIndex*"] -> bool as "bool" { return self->isValid(); }) }
    }
}

#[allow(non_camel_case_types)]
type qreal = f64;

/// Wrapper around QRectF
#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct QRectF {
    pub x: qreal,
    pub y: qreal,
    pub width: qreal,
    pub height: qreal,
}

impl QRectF {
    pub fn contains(&self, pos: QPointF) -> bool {
        cpp!(unsafe [self as "const QRectF*", pos as "QPointF"] -> bool as "bool" {
            return self->contains(pos);
        })
    }
    pub fn top_left(&self) -> QPointF {
        QPointF {
            x: self.x,
            y: self.y,
        }
    }
}

/// Wrapper around QPointF
#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct QPointF {
    pub x: qreal,
    pub y: qreal,
}
impl std::ops::Add for QPointF {
    type Output = QPointF;
    fn add(self, other: QPointF) -> QPointF {
        QPointF {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl std::ops::AddAssign for QPointF {
    fn add_assign(&mut self, other: QPointF) {
        *self = QPointF {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

#[test]
fn test_qpointf_qrectf() {
    let rect = QRectF {
        x: 200.,
        y: 150.,
        width: 60.,
        height: 75.,
    };
    let pt = QPointF { x: 12., y: 5.5 };
    assert!(!rect.contains(pt));
    assert!(rect.contains(pt + rect.top_left()));
}

cpp_class!(
/// Wrapper around QColor
#[derive(Default, Clone, Copy, PartialEq)] pub unsafe struct QColor as "QColor");
impl QColor {
    /// Construct a QColor from a string. Refer to the Qt documentation of QColor::setNamedColor
    pub fn from_name(name: &str) -> Self {
        let len = name.len();
        let ptr = name.as_ptr();
        cpp!(unsafe [len as "size_t", ptr as "char*"] -> QColor as "QColor" {
            return QColor(QLatin1String(ptr, len));
        })
    }
    /// Refer to the Qt documentation of QColor::fromRgbF
    pub fn from_rgb_f(r: qreal, g: qreal, b: qreal) -> Self {
        cpp!(unsafe [r as "qreal", g as "qreal", b as "qreal"] -> QColor as "QColor" {
            return QColor::fromRgbF(r, g, b);
        })
    }
    /// Same as from_rgb_f, but accept an alpha value.
    pub fn from_rgba_f(r: qreal, g: qreal, b: qreal, a: qreal) -> Self {
        cpp!(unsafe [r as "qreal", g as "qreal", b as "qreal", a as "qreal"] -> QColor as "QColor" {
            return QColor::fromRgbF(r, g, b, a);
        })
    }

    /// Returns the individual component as floating point.
    /// Refer to the Qt documentation of QColor::getRgbF.
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

/// Wrapper around QSize
#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct QSize {
    pub width: u32,
    pub height: u32,
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum ImageFormat {
    Invalid,
    Mono,
    MonoLSB,
    Indexed8,
    RGB32,
    ARGB32,
    ARGB32_Premultiplied,
    RGB16,
    ARGB8565_Premultiplied,
    RGB666,
    ARGB6666_Premultiplied,
    RGB555,
    ARGB8555_Premultiplied,
    RGB888,
    RGB444,
    ARGB4444_Premultiplied,
    RGBX8888,
    RGBA8888,
    RGBA8888_Premultiplied,
    BGR30,
    A2BGR30_Premultiplied,
    RGB30,
    A2RGB30_Premultiplied,
    Alpha8,
    Grayscale8,
}
cpp_class!(
/// Wrapper around QImage
pub unsafe struct QImage as "QImage");
impl QImage {
    pub fn load_from_file(filename: QString) -> Self {
        cpp!(unsafe [filename as "QString"] -> QImage as "QImage" {
            return QImage(filename);
        })
    }
    pub fn new(size: QSize, format: ImageFormat) -> Self {
        cpp!(unsafe [size as "QSize", format as "QImage::Format" ] -> QImage as "QImage" {
            return QImage(size, format);
        })
    }
    pub fn size(&self) -> QSize {
        cpp!(unsafe [self as "const QImage*"] -> QSize as "QSize" { return self->size(); })
    }
    pub fn format(&self) -> ImageFormat {
        cpp!(unsafe [self as "const QImage*"] -> ImageFormat as "QImage::Format" { return self->format(); })
    }
    pub fn fill(&mut self, color: QColor) {
        cpp!(unsafe [self as "QImage*", color as "QColor"] { self->fill(color); })
    }
    pub fn set_pixel_color(&mut self, x: u32, y: u32, color: QColor) {
        cpp!(unsafe [self as "QImage*", x as "int", y as "int", color as "QColor"]
            { self->setPixelColor(x, y, color); })
    }
    pub fn get_pixel_color(&mut self, x: u32, y: u32) -> QColor {
        cpp!(unsafe [self as "QImage*", x as "int", y as "int"] -> QColor as "QColor"
            { return self->pixelColor(x, y); })
    }
}
