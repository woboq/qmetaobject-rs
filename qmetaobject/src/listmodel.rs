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

use std::collections::HashMap;
use std::iter::FromIterator;
use std::ops::Index;

use cpp::cpp;

use super::*;

/// This trait allow to override a Qt QAbstractListModel
pub trait QAbstractListModel: QObject {
    /// Required for the implementation detail of the QObject custom derive
    fn get_object_description() -> &'static QObjectDescriptor
    where
        Self: Sized,
    {
        unsafe {
            &*cpp!([]-> *const QObjectDescriptor as "RustQObjectDescriptor const*" {
                return RustQObjectDescriptor::instance<Rust_QAbstractListModel>();
            })
        }
    }

    /// Refer to the Qt documentation of QAbstractListModel::rowCount
    fn row_count(&self) -> i32;
    /// Refer to the Qt documentation of QAbstractListModel::data
    fn data(&self, index: QModelIndex, role: i32) -> QVariant;
    /// Refer to the Qt documentation of QAbstractListModel::setData
    fn set_data(&mut self, _index: QModelIndex, _value: &QVariant, _role: i32) -> bool {
        false
    }
    /// Refer to the Qt documentation of QAbstractListModel::roleNames
    fn role_names(&self) -> HashMap<i32, QByteArray> {
        HashMap::new()
    }

    /// Refer to the Qt documentation of QAbstractListModel::beginInsertRows
    fn begin_insert_rows(&mut self, first: i32, last: i32) {
        let p = QModelIndex::default();
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractListModel*", p as "QModelIndex", first as "int", last as "int"]{
                if(obj) obj->beginInsertRows(p, first, last);
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractListModel::endInsertRows
    fn end_insert_rows(&mut self) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractListModel*"]{
                if(obj) obj->endInsertRows();
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractListModel::beginRemoveRows
    fn begin_remove_rows(&mut self, first: i32, last: i32) {
        let p = QModelIndex::default();
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractListModel*", p as "QModelIndex", first as "int", last as "int"]{
                if(obj) obj->beginRemoveRows(p, first, last);
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractListModel::endRemoveRows
    fn end_remove_rows(&mut self) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractListModel*"]{
                if(obj) obj->endRemoveRows();
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractListModel::beginResetModel
    fn begin_reset_model(&mut self) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractListModel*"]{
                if(obj) obj->beginResetModel();
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractListModel::endResetModel
    fn end_reset_model(&mut self) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractListModel*"]{
                if(obj) obj->endResetModel();
            })
        }
    }

    /// Refer to the Qt documentation of QAbstractListModel::beginMoveRows
    fn begin_move_rows(
        &mut self,
        source_parent: QModelIndex,
        source_first: i32,
        source_last: i32,
        destination_parent: QModelIndex,
        destination_child: i32,
    ) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractListModel*", source_parent as "QModelIndex", source_first as "int", source_last as "int", destination_parent as "QModelIndex", destination_child as "int"]{
                if(obj) obj->beginMoveRows(source_parent, source_first, source_last, destination_parent, destination_child);
            })
        }
    }

    /// Refer to the Qt documentation of QAbstractListModel::endMoveRows
    fn end_move_rows(&mut self) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractListModel*"]{
                if(obj) obj->endMoveRows();
            })
        }
    }

    /// Refer to the Qt documentation of QAbstractListModel::dataChanged
    fn data_changed(&mut self, top_left: QModelIndex, bottom_right: QModelIndex) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractListModel*", top_left as "QModelIndex", bottom_right as "QModelIndex"]{
                if(obj) obj->dataChanged(top_left, bottom_right);
            })
        }
    }

    /// Returns a QModelIndex for the given row (in the first column)
    fn row_index(&self, i: i32) -> QModelIndex {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractListModel*", i as "int"] -> QModelIndex as "QModelIndex" {
                return obj ? obj->index(i) : QModelIndex();
            })
        }
    }
}

cpp! {{
    #include <qmetaobject_rust.hpp>
    #include <QtCore/QAbstractListModel>

    struct Rust_QAbstractListModel : RustObject<QAbstractListModel> {

        using QAbstractListModel::beginInsertRows;
        using QAbstractListModel::endInsertRows;
        using QAbstractListModel::beginRemoveRows;
        using QAbstractListModel::endRemoveRows;
        using QAbstractListModel::beginResetModel;
        using QAbstractListModel::endResetModel;
        using QAbstractListModel::beginMoveRows;
        using QAbstractListModel::endMoveRows;

        int rowCount(const QModelIndex & = QModelIndex()) const override {
            return rust!(Rust_QAbstractListModel_rowCount[rust_object : QObjectPinned<dyn QAbstractListModel> as "TraitObject"]
                    -> i32 as "int" {
                rust_object.borrow().row_count()
            });
        }

        //int columnCount(const QModelIndex &parent = QModelIndex()) const override;

        QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override {
            return rust!(Rust_QAbstractListModel_data[rust_object : QObjectPinned<dyn QAbstractListModel> as "TraitObject",
                    index : QModelIndex as "QModelIndex", role : i32 as "int"] -> QVariant as "QVariant" {
                rust_object.borrow().data(index, role)
            });
        }

        bool setData(const QModelIndex &index, const QVariant &value, int role = Qt::EditRole) override {
            return rust!(Rust_QAbstractListModel_setData[rust_object : QObjectPinned<dyn QAbstractListModel> as "TraitObject",
                    index : QModelIndex as "QModelIndex", value : QVariant as "QVariant", role : i32 as "int"]
                    -> bool as "bool" {
                rust_object.borrow_mut().set_data(index, &value, role)
            });
        }

        //Qt::ItemFlags flags(const QModelIndex &index) const override;

        //QVariant headerData(int section, Qt::Orientation orientation, int role = Qt::DisplayRole) const override;

        QHash<int, QByteArray> roleNames() const override {
            QHash<int, QByteArray> base = QAbstractListModel::roleNames();
            rust!(Rust_QAbstractListModel_roleNames[rust_object : QObjectPinned<dyn QAbstractListModel> as "TraitObject",
                    base: *mut c_void as "QHash<int, QByteArray>&"] {
                for (key, val) in rust_object.borrow().role_names().iter() {
                    add_to_hash(base, *key, val.clone());
                }
            });
            return base;
        }

        //QModelIndex index(int row, int column, const QModelIndex &parent) const override;

        //QModelIndex parent(const QModelIndex &child) const override;
    };
}}

/// A trait used in SimpleListModel.
/// Can be derived with `#[derive(SimpleListModel)]`, in which case all the member of the struct
/// get exposed. The public member needs to implement the QMetaType trait
pub trait SimpleListItem {
    /// Get the item in for the given role.
    /// Note that the role is, in a way, an index in the names() array.
    fn get(&self, role: i32) -> QVariant;
    /// Array of the role names.
    fn names() -> Vec<QByteArray>;
}

/// A simple QAbstractListModel which just wrap a vector of items.
#[derive(QObject, Default)]
// This is a bit weird because the rules are different as we are in the qmetaobject crate
#[QMetaObjectCrate = "super"]
pub struct SimpleListModel<T: SimpleListItem + 'static> {
    //    base : qt_base_class!(trait QAbstractListModel),
    #[qt_base_class = "QAbstractListModel"]
    base: QObjectCppWrapper,
    values: Vec<T>,
}

impl<T> QAbstractListModel for SimpleListModel<T>
where
    T: SimpleListItem,
{
    fn row_count(&self) -> i32 {
        self.values.len() as i32
    }
    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let idx = index.row();
        if idx >= 0 && (idx as usize) < self.values.len() {
            self.values[idx as usize].get(role - USER_ROLE).clone()
        } else {
            QVariant::default()
        }
    }
    fn role_names(&self) -> HashMap<i32, QByteArray> {
        T::names().iter().enumerate().map(|(i, x)| (i as i32 + USER_ROLE, x.clone())).collect()
    }
}
impl<T: SimpleListItem> SimpleListModel<T> {
    pub fn insert(&mut self, index: usize, element: T) {
        (self as &mut dyn QAbstractListModel).begin_insert_rows(index as i32, index as i32);
        self.values.insert(index, element);
        (self as &mut dyn QAbstractListModel).end_insert_rows();
    }
    pub fn push(&mut self, value: T) {
        let idx = self.values.len();
        self.insert(idx, value);
    }
    pub fn remove(&mut self, index: usize) {
        (self as &mut dyn QAbstractListModel).begin_remove_rows(index as i32, index as i32);
        self.values.remove(index);
        (self as &mut dyn QAbstractListModel).end_remove_rows();
    }
    pub fn change_line(&mut self, index: usize, value: T) {
        self.values[index] = value;
        let idx = (self as &mut dyn QAbstractListModel).row_index(index as i32);
        (self as &mut dyn QAbstractListModel).data_changed(idx, idx);
    }
    pub fn reset_data(&mut self, data: Vec<T>) {
        (self as &mut dyn QAbstractListModel).begin_reset_model();
        self.values = data;
        (self as &mut dyn QAbstractListModel).end_reset_model();
    }
    /// Returns an iterator over the items in the model
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.values.iter()
    }
}

impl<T> FromIterator<T> for SimpleListModel<T>
where
    T: SimpleListItem + Default,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> SimpleListModel<T> {
        let mut m = SimpleListModel::default();
        m.values = Vec::from_iter(iter.into_iter());
        m
    }
}
impl<'a, T> FromIterator<&'a T> for SimpleListModel<T>
where
    T: SimpleListItem + Default + Clone,
{
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> SimpleListModel<T> {
        let mut m = SimpleListModel::<T>::default();
        m.values = Vec::from_iter(iter.into_iter().cloned());
        m
    }
}

impl<T> Index<usize> for SimpleListModel<T>
where
    T: SimpleListItem,
{
    type Output = T;

    fn index(&self, index: usize) -> &T {
        &self.values[index]
    }
}
