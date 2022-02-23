use std::{
    iter::FromIterator,
    ops::{Index, IndexMut},
};

use crate::internal_prelude::*;

use super::common::QListIterator;
use crate::QVariant;

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

impl<'a> IntoIterator for &'a QVariantList {
    type Item = &'a QVariant;
    type IntoIter = QListIterator<'a, QVariantList, QVariant>;

    fn into_iter(self) -> Self::IntoIter {
        QListIterator::new(self, 0, self.len())
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
    use crate::{QByteArray, QString};

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
}
