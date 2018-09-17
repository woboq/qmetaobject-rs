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

use super::*;
use std::collections::HashMap;

/// This trait allow to override a Qt QAbstractItemModel
pub trait QAbstractItemModel: QObject {
    /// Required for the implementation detail of the QObject custom derive
    fn get_object_description() -> &'static QObjectDescription
    where
        Self: Sized,
    {
        unsafe {
            cpp!([]-> &'static QObjectDescription as "RustObjectDescription const*" {
            return rustObjectDescription<Rust_QAbstractItemModel>();
        } )
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

impl QAbstractItemModel {
    /// Refer to the Qt documentation of QAbstractItemModel::createIndex
    pub fn create_index(&self, row: i32, column: i32, id: usize) -> QModelIndex {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractItemModel*", row as "int", column as "int", id as "uintptr_t"] -> QModelIndex as "QModelIndex" {
            return obj ? obj->createIndex(row, column, id) : QModelIndex();
        })
        }
    }
}

cpp!{{
#include <qmetaobject_rust.hpp>
#include <QtCore/QAbstractItemModel>
}}

cpp!{{
struct Rust_QAbstractItemModel : RustObject<QAbstractItemModel> {

    using QAbstractItemModel::createIndex;

    const QMetaObject *metaObject() const override {
        return rust!(Rust_QAbstractItemModel_metaobject[rust_object : QObjectPinned<QAbstractItemModel> as "TraitObject"]
                -> *const QMetaObject as "const QMetaObject*" {
            rust_object.borrow().meta_object()
        });
    }

    QModelIndex index(int row, int column, const QModelIndex &parent = QModelIndex()) const override {
        return rust!(Rust_QAbstractItemModel_index[rust_object : QObjectPinned<QAbstractItemModel> as "TraitObject",
                row : i32 as "int", column : i32 as "int", parent : QModelIndex as "QModelIndex"] -> QModelIndex as "QModelIndex" {
            rust_object.borrow().index(row, column, parent)
        });
    }

    QModelIndex parent(const QModelIndex &index) const override {
        return rust!(Rust_QAbstractItemModel_parent[rust_object : QObjectPinned<QAbstractItemModel> as "TraitObject",
                index : QModelIndex as "QModelIndex"] -> QModelIndex as "QModelIndex" {
            rust_object.borrow().parent(index)
        });
    }

    int rowCount(const QModelIndex &parent = QModelIndex()) const override {
        return rust!(Rust_QAbstractItemModel_rowCount[rust_object : QObjectPinned<QAbstractItemModel> as "TraitObject",
                  parent : QModelIndex as "QModelIndex"]
                -> i32 as "int" {
            rust_object.borrow().row_count(parent)
        });
    }

    int columnCount(const QModelIndex &parent = QModelIndex()) const override {
        return rust!(Rust_QAbstractItemModel_columnCount[rust_object : QObjectPinned<QAbstractItemModel> as "TraitObject",
                     parent : QModelIndex as "QModelIndex"]
                -> i32 as "int" {
            rust_object.borrow().column_count(parent)
        });
    }

    QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override {
        return rust!(Rust_QAbstractItemModel_data[rust_object : QObjectPinned<QAbstractItemModel> as "TraitObject",
                index : QModelIndex as "QModelIndex", role : i32 as "int"] -> QVariant as "QVariant" {
            rust_object.borrow().data(index, role)
        });
    }

    bool setData(const QModelIndex &index, const QVariant &value, int role = Qt::EditRole) override {
        return rust!(Rust_QAbstractItemModel_setData[rust_object : QObjectPinned<QAbstractItemModel> as "TraitObject",
                index : QModelIndex as "QModelIndex", value : QVariant as "QVariant", role : i32 as "int"]
                -> bool as "bool" {
            rust_object.borrow_mut().set_data(index, &value, role)
        });
    }

    QHash<int, QByteArray> roleNames() const override {
        QHash<int, QByteArray> base = QAbstractItemModel::roleNames();
        rust!(Rust_QAbstractItemModel_roleNames[rust_object : QObjectPinned<QAbstractItemModel> as "TraitObject",
                base: *mut c_void as "QHash<int, QByteArray>&"] {
            for (key, val) in rust_object.borrow().role_names().iter() {
                add_to_hash(base, key.clone(), val.clone());
            }
        });
        return base;
    }
};
}}
