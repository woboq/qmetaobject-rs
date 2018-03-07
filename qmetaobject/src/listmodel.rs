use super::*;
use std::collections::HashMap;
use std::iter::FromIterator;

pub trait QAbstractListModel : QObject {
    fn base_meta_object()->*const QMetaObject where Self:Sized {
        unsafe { cpp!{[] -> *const QMetaObject as "const QMetaObject*" {
            return &QAbstractListModel::staticMetaObject;
        }}}
    }
    unsafe fn get_rust_object<'a>(p: &'a mut c_void)->&'a mut Self  where Self:Sized {
        let ptr = cpp!{[p as "RustObject<QAbstractListModel>*"] -> *mut c_void as "void*" {
            return p->rust_object.a;
        }};
        std::mem::transmute::<*mut c_void, &'a mut Self>(ptr)
    }
     fn construct_cpp_object(self_ : *mut QAbstractListModel) -> *mut c_void where Self:Sized {
        unsafe {
            cpp!{[self_ as "TraitObject"] -> *mut c_void as "void*"  {
                auto q = new Rust_QAbstractListModel();
                q->rust_object = self_;
                return q;
            }}
        }
    }

    fn row_count(&self) -> i32;
    fn data(&self, index: QModelIndex, role:i32) -> QVariant;
    fn set_data(&mut self, _index: QModelIndex, _value: QVariant, _role: i32) -> bool { false }
    fn role_names(&self) -> HashMap<i32, QByteArray> { HashMap::new() }
}

impl QAbstractListModel {
    pub fn begin_insert_rows(&mut self, first : i32, last: i32) {
        let p = QModelIndex::default();
        let obj = self.get_cpp_object().ptr;
        unsafe { cpp!([obj as "Rust_QAbstractListModel*", p as "QModelIndex", first as "int", last as "int"]{
            obj->beginInsertRows(p, first, last);
        })}
    }
    pub fn end_insert_rows(&mut self) {
        let obj = self.get_cpp_object().ptr;
        unsafe { cpp!([obj as "Rust_QAbstractListModel*"]{
            obj->endInsertRows();
        })}
    }
    pub fn begin_remove_rows(&mut self, first : i32, last: i32) {
        let p = QModelIndex::default();
        let obj = self.get_cpp_object().ptr;
        unsafe { cpp!([obj as "Rust_QAbstractListModel*", p as "QModelIndex", first as "int", last as "int"]{
            obj->beginRemoveRows(p, first, last);
        })}
    }
    pub fn end_remove_rows(&mut self) {
        let obj = self.get_cpp_object().ptr;
        unsafe { cpp!([obj as "Rust_QAbstractListModel*"]{
            obj->endRemoveRows();
        })}
    }
    pub fn begin_reset_model(&mut self) {
        let obj = self.get_cpp_object().ptr;
        unsafe { cpp!([obj as "Rust_QAbstractListModel*"]{
            obj->beginResetModel();
        })}
    }
    pub fn end_reset_model(&mut self) {
        let obj = self.get_cpp_object().ptr;
        unsafe { cpp!([obj as "Rust_QAbstractListModel*"]{
            obj->endResetModel();
        })}
    }
}

/* Small helper funciton for Rust_QAbstractListModel::roleNames */
fn add_to_hash(hash: *mut c_void, key: i32, value: QByteArray) {
    unsafe { cpp!([hash as "QHash<int, QByteArray>*", key as "int", value as "QByteArray"]{
        (*hash)[key] = std::move(value);
    })}
}

cpp!{{
#include <qmetaobject_rust.hpp>
struct Rust_QAbstractListModel : RustObject<QAbstractListModel> {

    using QAbstractListModel::beginInsertRows;
    using QAbstractListModel::endInsertRows;
    using QAbstractListModel::beginRemoveRows;
    using QAbstractListModel::endRemoveRows;
    using QAbstractListModel::beginResetModel;
    using QAbstractListModel::endResetModel;


    const QMetaObject *metaObject() const override {
        return rust!(Rust_QAbstractListModel_metaobject[rust_object : &QAbstractListModel as "TraitObject"]
                -> *const QMetaObject as "const QMetaObject*" {
            rust_object.meta_object()
        });
    }

    int rowCount(const QModelIndex & = QModelIndex()) const override {
        return rust!(Rust_QAbstractListModel_rowCount[rust_object : &QAbstractListModel as "TraitObject"]
                -> i32 as "int" {
            rust_object.row_count()
        });
    }

    /// @see QAbstractItemModel::columnCount
    //int columnCount(const QModelIndex &parent = QModelIndex()) const override;

    QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override {
        return rust!(Rust_QAbstractListModel_data[rust_object : &QAbstractListModel as "TraitObject",
                index : QModelIndex as "QModelIndex", role : i32 as "int"] -> QVariant as "QVariant" {
            rust_object.data(index, role)
        });
    }

    bool setData(const QModelIndex &index, const QVariant &value, int role = Qt::EditRole) override {
        return rust!(Rust_QAbstractListModel_setData[rust_object : &mut QAbstractListModel as "TraitObject",
                index : QModelIndex as "QModelIndex", value : QVariant as "QVariant", role : i32 as "int"]
                -> bool as "bool" {
            rust_object.set_data(index, value, role)
        });
    }

    //Qt::ItemFlags flags(const QModelIndex &index) const override;

    //QVariant headerData(int section, Qt::Orientation orientation, int role = Qt::DisplayRole) const override;

    QHash<int, QByteArray> roleNames() const override {
        QHash<int, QByteArray> base = QAbstractListModel::roleNames();
        rust!(Rust_QAbstractListModel_roleNames[rust_object : &QAbstractListModel as "TraitObject",
                base: *mut c_void as "QHash<int, QByteArray>&"] {
            for (key, val) in rust_object.role_names().iter() {
                add_to_hash(base, key.clone(), val.clone());
            }
        });
        return base;
    }

    //QModelIndex index(int row, int column, const QModelIndex &parent) const override;

    //QModelIndex parent(const QModelIndex &child) const override;


};
}}

pub const USER_ROLE : i32 = 0x0100;

pub trait SimpleListItem {
    fn get(&self, idx : i32) -> QVariant;
    fn names() -> Vec<QByteArray>;
}

// This is a bit weird because the rules are different as we are in the qmetaobject crate

#[derive(QObject, Default)]
#[QMetaObjectCrate="super"]
pub struct SimpleListModel<T : SimpleListItem + 'static> {
//    base : qt_base_class!(trait QAbstractListModel),
    #[qt_base_class="QAbstractListModel"]
    base: QObjectCppWrapper,
    values: Vec<T>
}

impl<T> QAbstractListModel for SimpleListModel<T> where T: SimpleListItem {
    fn row_count(&self) -> i32 {
        self.values.len() as i32
    }
    fn data(&self, index: QModelIndex, role:i32) -> QVariant {
        let idx = index.row();
        if idx >= 0 && (idx as usize) < self.values.len() {
            self.values[idx as usize].get(role - USER_ROLE).clone()
        } else {
            QVariant::default()
        }
    }
    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        T::names().iter().enumerate().map(|(i,x)| (i as i32+USER_ROLE, x.clone())).collect()
    }
}
impl<T : SimpleListItem> SimpleListModel<T> {
    pub fn insert(&mut self, index: usize, element: T) {
        (self as &mut QAbstractListModel).begin_insert_rows(index as i32, index as i32);
        self.values.insert(index, element);
        (self as &mut QAbstractListModel).end_insert_rows();
    }
    pub fn push(&mut self, value : T) {
        let idx = self.values.len();
        self.insert(idx, value);
    }
    pub fn remove(&mut self, index: usize) {
        (self as &mut QAbstractListModel).begin_remove_rows(index as i32, index as i32);
        self.values.remove(index);
        (self as &mut QAbstractListModel).end_insert_rows();
    }
}

impl<T : SimpleListItem> FromIterator<T> for SimpleListModel<T> where T: Default  {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> SimpleListModel<T> {
        let mut m = SimpleListModel::default();
        m.values = Vec::from_iter(iter.into_iter());
        m
    }
}
