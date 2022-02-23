use std::{fmt, iter::FromIterator, ops::Index};

use crate::internal_prelude::*;

use crate::QString;

use super::common::QListIterator;

cpp_class!(
    /// Wrapper around [`QStringList`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qstringlist.html
    #[derive(Default, Clone, PartialEq, Eq)]
    pub unsafe struct QStringList as "QStringList"
);
impl QStringList {
    pub fn new() -> QStringList {
        cpp!(unsafe [] -> QStringList as "QStringList" {
            return QStringList();
        })
    }

    pub fn insert(&mut self, index: usize, value: QString) {
        cpp!(unsafe [self as "QStringList*", index as "size_t", value as "QString"] {
            self->insert(index, value);
        });
    }

    pub fn push(&mut self, value: QString) {
        cpp!(unsafe [self as "QStringList*", value as "QString"] {
           self->append(value);
        });
    }

    pub fn clear(&mut self) {
        cpp!(unsafe [self as "QStringList*"] {
            self->clear();
        });
    }

    pub fn remove(&mut self, index: usize) {
        cpp!(unsafe [self as "QStringList*", index as "size_t"] {
            self->removeAt(index);
        })
    }

    pub fn len(&self) -> usize {
        cpp!(unsafe [self as "QStringList*"] -> usize as "size_t" { return self->size(); })
    }
}

impl Index<usize> for QStringList {
    type Output = QString;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            &*cpp!([self as "QStringList*", index as "size_t"] -> *const QString as "const QString*" {
                return &(*self)[index];
            })
        }
    }
}

impl fmt::Debug for QStringList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.into_iter()).finish()
    }
}

impl<T, const N: usize> From<[T; N]> for QStringList
where
    QString: From<T>,
{
    fn from(s: [T; N]) -> Self {
        let mut list = QStringList::new();
        for i in s {
            list.push(QString::from(i));
        }
        list
    }
}

impl<T> From<Vec<T>> for QStringList
where
    QString: From<T>,
{
    fn from(s: Vec<T>) -> Self {
        s.into_iter().map(QString::from).collect()
    }
}

impl<T> From<&[T]> for QStringList
where
    T: Clone,
    QString: From<T>,
{
    fn from(s: &[T]) -> Self {
        s.iter().cloned().map(QString::from).collect()
    }
}

impl<T> From<QStringList> for Vec<T>
where
    T: Clone,
    QString: Into<T>,
{
    fn from(arr: QStringList) -> Self {
        arr.into_iter().cloned().map(|x| x.into()).collect()
    }
}

impl<'a> IntoIterator for &'a QStringList {
    type Item = &'a QString;
    type IntoIter = QListIterator<'a, QStringList, QString>;

    fn into_iter(self) -> Self::IntoIter {
        QListIterator::new(self, 0, self.len())
    }
}

impl<T> FromIterator<T> for QStringList
where
    T: Into<QString>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut l = QStringList::default();
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
    fn test_qstringlist() {
        let mut qstringlist = QStringList::new();
        qstringlist.push("One".into());
        qstringlist.push("Two".into());

        assert_eq!(qstringlist.len(), 2);
        assert_eq!(qstringlist[0], QString::from("One"));

        qstringlist.remove(0);
        assert_eq!(qstringlist[0], QString::from("Two"));

        qstringlist.insert(0, "Three".into());
        assert_eq!(qstringlist[0], QString::from("Three"));

        qstringlist.clear();
        assert_eq!(qstringlist.len(), 0);
    }

    #[test]
    fn test_qstringlist_from_iter() {
        let v = vec!["abc", "efg", "hij"];
        let qvl: QStringList = v.clone().into_iter().collect();
        assert_eq!(qvl.len(), 3);
        assert_eq!(qvl[1].to_string(), v[1].to_string());
    }

    #[test]
    fn test_vec_from_qstringlist() {
        let qstringlist = generate_qstring_list();
        let temp: Vec<String> = qstringlist.clone().into();
        assert_eq!(temp, vec!["Three".to_string(), "Two".to_string()]);
        let temp: Vec<QString> = qstringlist.clone().into();
        assert_eq!(temp, vec![QString::from("Three"), QString::from("Two")]);
    }

    #[test]
    fn test_qstringlist_from_slice_ref() {
        let qstringlist = generate_qstring_list();
        let t = ["Three", "Two"];
        assert_eq!(qstringlist, QStringList::from(t));
        let t = ["Three".to_string(), "Two".to_string()];
        assert_eq!(qstringlist, QStringList::from(t));
        let t = [QString::from("Three"), QString::from("Two")];
        assert_eq!(qstringlist, QStringList::from(t));
    }

    #[test]
    fn test_qstringlist_from_slice() {
        let qstringlist = generate_qstring_list();

        assert_eq!(qstringlist, QStringList::from(["Three", "Two"]));
        assert_eq!(qstringlist, QStringList::from(["Three".to_string(), "Two".to_string()]));
        assert_eq!(qstringlist, QStringList::from([QString::from("Three"), QString::from("Two")]));
    }

    #[test]
    fn test_qstringlist_from_vec() {
        let qstringlist = generate_qstring_list();
        assert_eq!(qstringlist, QStringList::from(vec!["Three", "Two"]));
        assert_eq!(qstringlist, QStringList::from(vec!["Three".to_string(), "Two".to_string()]));
        assert_eq!(
            qstringlist,
            QStringList::from(vec![QString::from("Three"), QString::from("Two")])
        );
    }

    fn generate_qstring_list() -> QStringList {
        let mut qstringlist = QStringList::new();
        qstringlist.push("Three".into());
        qstringlist.push("Two".into());
        return qstringlist;
    }
}
