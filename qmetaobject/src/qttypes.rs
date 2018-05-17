extern crate std;
use std::os::raw::c_char;
use std::convert::From;
use std::fmt::Display;
use std::ops::{Index,IndexMut};
use std::iter::FromIterator;

cpp_class!(pub struct QByteArray as "QByteArray");
impl QByteArray {
    pub fn to_slice(&self) -> &[u8] {
        unsafe {
            let mut size : usize = 0;
            let c_ptr = cpp!([self as "const QByteArray*", mut size as "size_t"] -> *const u8 as "const char*" {
                size = self->size();
                return self->constData();
            });
            std::slice::from_raw_parts(c_ptr, size)
        }
    }
    pub fn to_str(&self) -> &str {
        std::str::from_utf8(self.to_slice()).unwrap()
    }

}
impl<'a> From<&'a str> for QByteArray {
    fn from(s : &'a str) -> QByteArray {
        let len = s.len();
        let ptr = s.as_ptr();
        unsafe { cpp!([len as "size_t", ptr as "char*"] -> QByteArray as "QByteArray"
        { return QByteArray(ptr, len); })}
    }
}

impl From<String> for QByteArray {
    fn from(s : String) -> QByteArray { QByteArray::from(&*s) }
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
impl std::fmt::Debug for QByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
impl PartialEq for QByteArray {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cpp!([self as "QByteArray*", other as "QByteArray*"] -> bool as "bool" {
            return *self == *other;
        })}
    }
}

cpp_class!(pub struct QString as "QString");
impl QString {
    pub fn to_slice(&self) -> &[u16] {
        unsafe {
            let mut size : usize = 0;
            let c_ptr = cpp!([self as "const QString*", mut size as "size_t"] -> *const u16 as "const QChar*" {
                size = self->size();
                return self->constData();
            });
            std::slice::from_raw_parts(c_ptr, size)
        }
    }
}
impl<'a> From<&'a str> for QString {
    fn from(s : &'a str) -> QString {
        let len = s.len();
        let ptr = s.as_ptr();
        unsafe { cpp!([len as "size_t", ptr as "char*"] -> QString as "QString"
        { return QString::fromUtf8(ptr, len); })}
    }
}
impl From<String> for QString {
    fn from(s : String) -> QString { QString::from(&*s) }
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
impl PartialEq for QString {
    fn eq(&self, other: &Self) -> bool {
        unsafe { cpp!([self as "QString*", other as "QString*"] -> bool as "bool" {
            return *self == *other;
        })}
    }
}

cpp_class!(pub struct QVariant as "QVariant");
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
    fn from(a : QString) -> QVariant {
        unsafe {cpp!([a as "QString"] -> QVariant as "QVariant" { return QVariant(a); })}
    }
}
impl From<QByteArray> for QVariant {
    fn from(a : QByteArray) -> QVariant {
        unsafe {cpp!([a as "QByteArray"] -> QVariant as "QVariant" { return QVariant(a); })}
    }
}
impl From<QVariantList> for QVariant {
    fn from(a : QVariantList) -> QVariant {
        unsafe {cpp!([a as "QVariantList"] -> QVariant as "QVariant" { return QVariant(a); })}
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
impl From<bool> for QVariant {
    fn from(a : bool) -> QVariant {
        unsafe {cpp!([a as "bool"] -> QVariant as "QVariant" { return QVariant(a); })}
    }
}
impl<'a, T> From<&'a T> for QVariant where T : Into<QVariant> + Clone {
    fn from(a : &'a T) -> QVariant {
        return (*a).clone().into();
    }
}

cpp_class!(pub struct QVariantList as "QVariantList");
impl QVariantList {
    pub fn push(&mut self, value: QVariant) {
        unsafe {cpp!([self as "QVariantList*", value as "QVariant"]
            { self->append(value); }
        )}
    }
    pub fn insert(&mut self, index: usize, element: QVariant) {
        unsafe {cpp!([self as "QVariantList*", index as "size_t", element as "QVariant"]
            { self->insert(index, element); }
        )}
    }
    pub fn remove(&mut self, index: usize) -> QVariant {
        unsafe {cpp!([self as "QVariantList*", index as "size_t"] -> QVariant as "QVariant"
            { return self->takeAt(index); }
        )}
    }
    pub fn len(&self) -> usize {
        unsafe {cpp!([self as "QVariantList*"] -> usize as "size_t"
            { return self->size(); }
        )}
    }
}


impl Index<usize> for QVariantList {
    type Output = QVariant;
    fn index(&self, index: usize) -> &QVariant {
        assert!(index < self.len());
        unsafe {cpp!([self as "QVariantList*", index as "size_t"] -> &QVariant as "const QVariant*"
            { return &self->at(index); }
        )}
    }
}
impl IndexMut<usize> for QVariantList {
    fn index_mut(&mut self, index: usize) -> &mut QVariant {
        assert!(index < self.len());
        unsafe {cpp!([self as "QVariantList*", index as "size_t"] -> &mut QVariant as "QVariant*"
            { return &(*self)[index]; }
        )}
    }
}

pub struct QVariantListIterator<'a> {
    list: &'a QVariantList,
    index: usize,
    size: usize
}

impl<'a> Iterator for QVariantListIterator<'a> {
    type Item = &'a QVariant;
    fn next(&mut self) -> Option<&'a QVariant> {
        if self.index == self.size {
            None
        } else {
            self.index+=1;
            Some(&self.list[self.index-1])
        }
    }
}

impl<'a> IntoIterator for &'a QVariantList {
    type Item = &'a QVariant;
    type IntoIter = QVariantListIterator<'a>;

    fn into_iter(self) -> QVariantListIterator<'a> {
        QVariantListIterator::<'a> { list:self, index: 0, size: self.len() }
    }
}

impl<T> FromIterator<T> for QVariantList where T : Into<QVariant>  {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> QVariantList {
        let mut l = QVariantList::default();
        for i in iter {
            l.push(i.into());
        }
        return l;
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
        let x : Vec<QByteArray> = q.into_iter().map(|x| x.to_qbytearray()).collect();
        assert_eq!(x[0].to_string(), "42");
        assert_eq!(x[1].to_string(), "Hello");
        assert_eq!(x[2].to_string(), "Hello");

    }

    #[test]
    fn test_qvariantlist_from_iter() {
        let v = vec![1u32,2u32,3u32];
        let qvl : QVariantList = v.iter().collect();
        assert_eq!(qvl.len(), 3);
        assert_eq!(qvl[1].to_qbytearray().to_string(), "2");

    }
}


cpp_class!(pub struct QModelIndex as "QModelIndex");
impl QModelIndex {
    pub fn row(&self) -> i32 {
        unsafe {
            cpp!([self as "const QModelIndex*"] -> i32 as "int" { return self->row(); })
        }
    }
}



