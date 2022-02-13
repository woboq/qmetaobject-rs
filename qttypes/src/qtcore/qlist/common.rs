use std::{marker::PhantomData, ops::Index};

/// Internal class used to iterate over a [`QList`][]
///
/// [`QList`]: https://doc.qt.io/qt-5/qlist.html
pub struct QListIterator<'a, T, I>
where
    T: Index<usize, Output = I>,
{
    list: &'a T,
    index: usize,
    size: usize,
    item: PhantomData<&'a I>,
}

impl<'a, T, I> QListIterator<'a, T, I>
where
    T: Index<usize, Output = I>,
{
    pub fn new(list: &'a T, index: usize, size: usize) -> Self {
        Self { list, index, size, item: PhantomData }
    }
}

impl<'a, T, I> Iterator for QListIterator<'a, T, I>
where
    T: Index<usize, Output = I>,
{
    type Item = &'a I;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.size {
            None
        } else {
            self.index += 1;
            Some(&self.list[self.index - 1])
        }
    }
}
