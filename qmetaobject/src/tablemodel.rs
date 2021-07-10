use std::collections::HashMap;

use cpp::cpp;

use super::*;

pub trait QAbstractTableModel: QObject {
    fn get_object_description() -> &'static QObjectDescriptor
    where
        Self: Sized,
    {
        unsafe {
            &*cpp!([]-> *const QObjectDescriptor as "RustQObjectDescriptor const*" {
                return RustQObjectDescriptor::instance<Rust_QAbstractTableModel>();
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractTableModel::rowCount
    fn row_count(&self) -> i32;
    /// Refer to the Qt documentation of QAbstractTableModel::columnCount
    fn column_count(&self) -> i32;
    /// Refer to the Qt documentation of QAbstractTableModel::data
    fn data(&self, index: QModelIndex, role: i32) -> QVariant;
    /// Refer to the Qt documentation of QAbstractTableModel::setData
    fn set_data(&mut self, _index: QModelIndex, _value: &QVariant, _role: i32) -> bool {
        false
    }
    /// Refer to the Qt documentation of QAbstractTableModel::roleNames
    fn role_names(&self) -> HashMap<i32, QByteArray> {
        HashMap::new()
    }

    /// Refer to the Qt documentation of QAbstractItemModel::beginInsertRows
    fn begin_insert_rows(&mut self, first: i32, last: i32) {
        let p = QModelIndex::default();
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractTableModel*", p as "QModelIndex", first as "int", last as "int"]{
                if(obj) obj->beginInsertRows(p, first, last);
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractItemModel::endInsertRows
    fn end_insert_rows(&mut self) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractTableModel*"]{
                if(obj) obj->endInsertRows();
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractItemModel::beginInsertColumns
    fn begin_insert_columns(&mut self, first: i32, last: i32) {
        let p = QModelIndex::default();
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractTableModel*", p as "QModelIndex", first as "int", last as "int"]{
                if(obj) obj->beginInsertColumns(p, first, last);
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractItemModel::endInsertColumns
    fn end_insert_columns(&mut self) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractTableModel*"]{
                if(obj) obj->endInsertColumns();
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractItemModel::beginRemoveRows
    fn begin_remove_rows(&mut self, first: i32, last: i32) {
        let p = QModelIndex::default();
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractTableModel*", p as "QModelIndex", first as "int", last as "int"]{
                if(obj) obj->beginRemoveRows(p, first, last);
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractItemModel::endRemoveRows
    fn end_remove_rows(&mut self) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractTableModel*"]{
                if(obj) obj->endRemoveRows();
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractItemModel::beginRemoveColumns
    fn begin_remove_columns(&mut self, first: i32, last: i32) {
        let p = QModelIndex::default();
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractTableModel*", p as "QModelIndex", first as "int", last as "int"]{
                if(obj) obj->beginRemoveColumns(p, first, last);
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractItemModel::endRemoveColumns
    fn end_remove_columns(&mut self) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractTableModel*"]{
                if(obj) obj->endRemoveColumns();
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractItemModel::beginResetModel
    fn begin_reset_model(&mut self) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractTableModel*"]{
                if(obj) obj->beginResetModel();
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractItemModel::endResetModel
    fn end_reset_model(&mut self) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractTableModel*"]{
                if(obj) obj->endResetModel();
            })
        }
    }
    /// Refer to the Qt documentation of QAbstractItemModel::dataChanged
    fn data_changed(&mut self, top_left: QModelIndex, bottom_right: QModelIndex) {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractTableModel*", top_left as "QModelIndex", bottom_right as "QModelIndex"]{
                if(obj) obj->dataChanged(top_left, bottom_right);
            })
        }
    }
    /// Returns a QModelIndex for the given row and column
    fn index(&self, row: i32, col: i32) -> QModelIndex {
        let obj = self.get_cpp_object();
        unsafe {
            cpp!([obj as "Rust_QAbstractTableModel*", row as "int", col as "int"] -> QModelIndex as "QModelIndex" {
                return obj ? obj->index(row, col) : QModelIndex();
            })
        }
    }
}

cpp! {{
    #include <qmetaobject_rust.hpp>
    #include <QtCore/QAbstractTableModel>

    struct Rust_QAbstractTableModel : RustObject<QAbstractTableModel> {

        using QAbstractTableModel::beginInsertRows;
        using QAbstractTableModel::endInsertRows;
        using QAbstractTableModel::beginInsertColumns;
        using QAbstractTableModel::endInsertColumns;
        using QAbstractTableModel::beginRemoveRows;
        using QAbstractTableModel::endRemoveRows;
        using QAbstractTableModel::beginRemoveColumns;
        using QAbstractTableModel::endRemoveColumns;
        using QAbstractTableModel::beginResetModel;
        using QAbstractTableModel::endResetModel;

        int rowCount(const QModelIndex & = QModelIndex()) const override {
            return rust!(Rust_QAbstractTableModel_rowCount[rust_object : QObjectPinned<dyn QAbstractTableModel> as "TraitObject"]
                    -> i32 as "int" {
                rust_object.borrow().row_count()
            });
        }

        int columnCount(const QModelIndex & = QModelIndex()) const override {
            return rust!(Rust_QAbstractTableModel_columnCount[rust_object : QObjectPinned<dyn QAbstractTableModel> as "TraitObject"]
                    -> i32 as "int" {
                rust_object.borrow().column_count()
            });
        }

        QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override {
            return rust!(Rust_QAbstractTableModel_data[rust_object : QObjectPinned<dyn QAbstractTableModel> as "TraitObject",
                    index : QModelIndex as "QModelIndex", role : i32 as "int"] -> QVariant as "QVariant" {
                rust_object.borrow().data(index, role)
            });
        }

        bool setData(const QModelIndex &index, const QVariant &value, int role = Qt::EditRole) override {
            return rust!(Rust_QAbstractTableModel_setData[rust_object : QObjectPinned<dyn QAbstractTableModel> as "TraitObject",
                    index : QModelIndex as "QModelIndex", value : QVariant as "QVariant", role : i32 as "int"]
                    -> bool as "bool" {
                rust_object.borrow_mut().set_data(index, &value, role)
            });
        }

        //Qt::ItemFlags flags(const QModelIndex &index) const override;

        //QVariant headerData(int section, Qt::Orientation orientation, int role = Qt::DisplayRole) const override;

        QHash<int, QByteArray> roleNames() const override {
            QHash<int, QByteArray> base = QAbstractTableModel::roleNames();
            rust!(Rust_QAbstractTableModel_roleNames[rust_object : QObjectPinned<dyn QAbstractTableModel> as "TraitObject",
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
