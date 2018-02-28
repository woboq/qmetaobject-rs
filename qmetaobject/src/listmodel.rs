use super::*;

pub trait QAbstractListModel : QObject {

    // These are not, they are part of the trait structure that sub trait must have
    // Copy/paste this code replacing QObject with the type
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
    fn construct_cpp_object_xx(&mut self) where Self:Sized {
        let p = unsafe {
            let p : *mut QAbstractListModel = self;
            cpp!{[p as "TraitObject"] -> *mut c_void as "void*"  {
                auto q = new Rust_QAbstractListModel();
                q->rust_object = p;
                return q;
            }}
        };
        let cpp_object = self.get_cpp_object();
        assert!(cpp_object.ptr.is_null(), "The cpp object was already created");
        cpp_object.ptr = p;
    }


    fn row_count(&self) -> i32;
    fn data(&self, index: QModelIndex, role:i32) -> QVariant;
    fn set_data(&mut self, _index: QModelIndex, _value: QVariant, _role: i32) -> bool { false }
}


cpp!{{
#include <qmetaobject_rust.hpp>
struct Rust_QAbstractListModel : RustObject<QAbstractListModel> {


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

    /*QHash<int, QByteArray> roleNames() const override {
        return rust!(Rust_QAbstractListModel_roleNames[rust_object : &QAbstractListModel as "TraitObject"]
                -> bool as "bool" {
            rust_object.roleName(index, role)
        });

    }*/

    //QModelIndex index(int row, int column, const QModelIndex &parent) const override;

    //QModelIndex parent(const QModelIndex &child) const override;


};
}}

