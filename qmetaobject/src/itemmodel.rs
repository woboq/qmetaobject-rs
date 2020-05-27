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

use crate::*;

/// This trait allow to override a Qt QAbstractItemModel
pub trait QAbstractItemModel: QObject {
    /// Required for the implementation detail of the QObject custom derive
    fn get_object_description() -> &'static QObjectDescription
    where
        Self: Sized,
    {
        unsafe {
            &*cpp!([]-> *const QObjectDescription as "RustObjectDescription const*" {
                return rustObjectDescription<Rust_QAbstractItemModel>();
            })
        }
    }

    /// Refer to the Qt documentation of QAbstractItemModel::index
    fn index(&self, row: i32, column: i32, parent: QModelIndex) -> QModelIndex;

    /// Refer to the Qt documentation of QAbstractItemModel::parent
    fn parent(&self, index: QModelIndex) -> QModelIndex;

    /// Refer to the Qt documentation of QAbstractItemModel::rowCount
    fn row_count(&self, parent: QModelIndex) -> i32;

    /// Refer to the Qt documentation of QAbstractItemModel::columnCount
    fn column_count(&self, parent: QModelIndex) -> i32;

    /// Refer to the Qt documentation of QAbstractItemModel::data
    fn data(&self, index: QModelIndex, role: i32) -> QVariant;

    /// Refer to the Qt documentation of QAbstractItemModel::setData
    fn set_data(&mut self, _index: QModelIndex, _value: &QVariant, _role: i32) -> bool {
        false
    }

    /// Refer to the Qt documentation of QAbstractItemModel::roleNames
    fn role_names(&self) -> HashMap<i32, QByteArray> {
        HashMap::new()
    }
}

impl dyn QAbstractItemModel {
    /// Refer to the Qt documentation of QAbstractListModel::beginInsertRows
    pub fn begin_insert_rows(&self, parent: QModelIndex, first: i32, last: i32) {
        let obj = self.get_cpp_object();
        cpp!(unsafe [
            obj as "Rust_QAbstractItemModel *",
            parent as "QModelIndex",
            first as "int",
            last as "int"
        ] {
            if(obj) obj->beginInsertRows(parent, first, last);
        })
    }

    /// Refer to the Qt documentation of QAbstractListModel::endInsertRows
    pub fn end_insert_rows(&self) {
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "Rust_QAbstractItemModel *"]{
            if(obj) obj->endInsertRows();
        })
    }

    /// Refer to the Qt documentation of QAbstractListModel::beginRemoveRows
    pub fn begin_remove_rows(&self, parent: QModelIndex, first: i32, last: i32) {
        let obj = self.get_cpp_object();
        cpp!(unsafe [
            obj as "Rust_QAbstractItemModel *",
            parent as "QModelIndex",
            first as "int",
            last as "int"
        ] {
            if(obj) obj->beginRemoveRows(parent, first, last);
        })
    }

    /// Refer to the Qt documentation of QAbstractListModel::endRemoveRows
    pub fn end_remove_rows(&self) {
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "Rust_QAbstractItemModel *"] {
            if(obj) obj->endRemoveRows();
        })
    }

    /// Refer to the Qt documentation of QAbstractListModel::beginResetModel
    pub fn begin_reset_model(&self) {
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "Rust_QAbstractItemModel *"] {
            if(obj) obj->beginResetModel();
        })
    }

    /// Refer to the Qt documentation of QAbstractListModel::endResetModel
    pub fn end_reset_model(&self) {
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "Rust_QAbstractItemModel *"] {
            if(obj) obj->endResetModel();
        })
    }

    /// Refer to the Qt documentation of QAbstractListModel::layoutAboutToBeChanged
    ///
    /// update_model_indexes need to be called between layout_about_to_be_changed and layout_changed
    pub fn layout_about_to_be_changed(&self) {
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "Rust_QAbstractItemModel *"] {
            if (obj) obj->layoutAboutToBeChanged();
        })
    }

    /// Refer to the Qt documentation of QAbstractListModel::layoutAboutToBeChanged
    ///
    /// update_model_indexes need to be called between layout_about_to_be_changed and layout_changed
    pub fn update_model_indexes<F>(&self, mut f: F)
    where
        F: FnMut(QModelIndex) -> QModelIndex,
    {
        let f: &mut dyn FnMut(QModelIndex) -> QModelIndex = &mut f;
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "Rust_QAbstractItemModel *", f as "TraitObject"] {
            if (!obj) return;
            const auto list1 = obj->persistentIndexList();
            auto list2 = list1;
            for (QModelIndex &idx : list2) {
                rust!(update_model_indexes [
                    f: &mut dyn FnMut(QModelIndex) -> QModelIndex as "TraitObject",
                    idx : &mut QModelIndex as "QModelIndex &"
                ] {
                    *idx = f(*idx);
                });
            }
            obj->changePersistentIndexList(list1, list2);
        })
    }

    /// Refer to the Qt documentation of QAbstractListModel::layoutChanged
    ///
    /// update_model_indexes need to be called between layout_about_to_be_changed and layout_changed
    pub fn layout_changed(&self) {
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "Rust_QAbstractItemModel *"] {
            if (obj) obj->layoutChanged();
        })
    }

    /// Refer to the Qt documentation of QAbstractListModel::dataChanged
    pub fn data_changed(&self, top_left: QModelIndex, bottom_right: QModelIndex) {
        let obj = self.get_cpp_object();
        cpp!(unsafe [
            obj as "Rust_QAbstractItemModel *",
            top_left as "QModelIndex",
            bottom_right as "QModelIndex"
        ] {
            if(obj) obj->dataChanged(top_left, bottom_right);
        })
    }

    /// Refer to the Qt documentation of QAbstractItemModel::createIndex
    pub fn create_index(&self, row: i32, column: i32, id: usize) -> QModelIndex {
        let obj = self.get_cpp_object();
        cpp!(unsafe [
            obj as "Rust_QAbstractItemModel *",
            row as "int",
            column as "int",
            id as "uintptr_t"
        ] -> QModelIndex as "QModelIndex" {
            return obj ? obj->createIndex(row, column, id) : QModelIndex();
        })
    }
}

cpp! {{
    #include <qmetaobject_rust.hpp>
    #include <QtCore/QAbstractItemModel>
}}

cpp! {{
    struct Rust_QAbstractItemModel : RustObject<QAbstractItemModel> {

        using QAbstractItemModel::beginInsertRows;
        using QAbstractItemModel::endInsertRows;
        using QAbstractItemModel::beginRemoveRows;
        using QAbstractItemModel::endRemoveRows;
        using QAbstractItemModel::beginResetModel;
        using QAbstractItemModel::endResetModel;
        using QAbstractItemModel::createIndex;
        using QAbstractItemModel::changePersistentIndexList;
        using QAbstractItemModel::persistentIndexList;

        QModelIndex index(int row, int column, const QModelIndex &parent = QModelIndex()) const override {
            return rust!(Rust_QAbstractItemModel_index [
                rust_object: QObjectPinned<dyn QAbstractItemModel> as "TraitObject",
                row: i32 as "int",
                column: i32 as "int",
                parent: QModelIndex as "QModelIndex"
            ] -> QModelIndex as "QModelIndex" {
                rust_object.borrow().index(row, column, parent)
            });
        }

        QModelIndex parent(const QModelIndex &index) const override {
            return rust!(Rust_QAbstractItemModel_parent [
                rust_object: QObjectPinned<dyn QAbstractItemModel> as "TraitObject",
                index : QModelIndex as "QModelIndex"
            ] -> QModelIndex as "QModelIndex" {
                rust_object.borrow().parent(index)
            });
        }

        int rowCount(const QModelIndex &parent = QModelIndex()) const override {
            return rust!(Rust_QAbstractItemModel_rowCount [
                rust_object: QObjectPinned<dyn QAbstractItemModel> as "TraitObject",
                parent: QModelIndex as "QModelIndex"
            ] -> i32 as "int" {
                rust_object.borrow().row_count(parent)
            });
        }

        int columnCount(const QModelIndex &parent = QModelIndex()) const override {
            return rust!(Rust_QAbstractItemModel_columnCount [
                rust_object: QObjectPinned<dyn QAbstractItemModel> as "TraitObject",
                parent : QModelIndex as "QModelIndex"
            ] -> i32 as "int" {
                rust_object.borrow().column_count(parent)
            });
        }

        QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override {
            return rust!(Rust_QAbstractItemModel_data [
                rust_object: QObjectPinned<dyn QAbstractItemModel> as "TraitObject",
                index: QModelIndex as "QModelIndex",
                role: i32 as "int"
            ] -> QVariant as "QVariant" {
                rust_object.borrow().data(index, role)
            });
        }

        bool setData(const QModelIndex &index, const QVariant &value, int role = Qt::EditRole) override {
            return rust!(Rust_QAbstractItemModel_setData [
                rust_object: QObjectPinned<dyn QAbstractItemModel> as "TraitObject",
                index: QModelIndex as "QModelIndex",
                value: QVariant as "QVariant",
                role: i32 as "int"
            ] -> bool as "bool" {
                rust_object.borrow_mut().set_data(index, &value, role)
            });
        }

        QHash<int, QByteArray> roleNames() const override {
            QHash<int, QByteArray> base = QAbstractItemModel::roleNames();
            rust!(Rust_QAbstractItemModel_roleNames [
                rust_object: QObjectPinned<dyn QAbstractItemModel> as "TraitObject",
                base: *mut c_void as "QHash<int, QByteArray> &"
            ] {
                for (key, val) in rust_object.borrow().role_names().iter() {
                    add_to_hash(base, *key, val.clone());
                }
            });
            return base;
        }
    };
}}
