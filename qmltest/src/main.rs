#![recursion_limit="10240"]


#[macro_use]
extern crate cpp;

// extern crate libc;
use std::os::raw::c_void;
use std::mem;

// #![feature(trace_macros)]
// trace_macros!(true);


#[macro_use]
extern crate proc_macro_hack;
#[macro_use]
extern crate qmetaobject_impl;

proc_macro_expr_decl! {
    /// Add one to an expression.
    add_one! => add_one_impl
}

proc_macro_expr_decl! {
    qobject! => qobject_impl
}


cpp!{{
    #include <QtQuick/QtQuick>
    #include <QtWidgets/QtWidgets>

    struct RustObject;
    extern "C" QMetaObject *RustObject_metaObject(const RustObject *);

    struct RustObject : QObject {
        void *data;
        const QMetaObject *metaObject() const override {
            return RustObject_metaObject(this);
        }
        int qt_metacall(QMetaObject::Call _c, int _id, void **_a) override {
            _id = QObject::qt_metacall(_c, _id, _a);
            if (_id < 0)
                return _id;
            const QMetaObject *mo = metaObject();
            if (_c == QMetaObject::InvokeMetaMethod || _c == QMetaObject::RegisterMethodArgumentMetaType) {
                int methodCount = mo->methodCount();
                if (_id < methodCount)
                    mo->d.static_metacall(this, _c, _id, _a);
                _id -= methodCount;
            } else if ((_c >= QMetaObject::ReadProperty && _c <= QMetaObject::QueryPropertyUser)
                || _c == QMetaObject::RegisterPropertyMetaType) {
                int propertyCount = mo->propertyCount();
                if (_id < propertyCount)
                    mo->d.static_metacall(this, _c, _id, _a);
                _id -= propertyCount;
            }
            return _id;
        }
    };
}}

#[no_mangle]
pub extern "C" fn RustObject_metaObject(_p: *mut c_void) -> *const QMetaObject {
    let s = MyStruct{};
    return s.meta_object();
}


#[repr(C)]
pub struct QMetaObject {
    superdata : *const QMetaObject,
    string_data: *const u8,
    data: *const i32,
    static_metacall: extern fn(o: *mut c_void, c: u32, idx: u32, a: *const *mut c_void),
    r: *const c_void,
    e: *const c_void,
}


trait QObject {
    fn meta_object(&self)->*const QMetaObject;
//    fn callFunction(&mut self, idx: usize, args : &[*mut c_void]);
//    fn create() -> Box<Self>:

    fn base_meta_object()->*const QMetaObject {
        unsafe {
            cpp!{[] -> *const QMetaObject as "const void*" { return &QObject::staticMetaObject; } }
        }
    }
}
trait QAIM : QObject {
    //fn create() { println!("Create OBJECT"); }
}


macro_rules! qt_method {
    ($t:ident) => { t :u32 };
}
#[derive(QObject)]
struct MyStruct {

  //  qt_method!(xx),


//     qt_method!(xx)

}
impl MyStruct {
    fn xx(&self) -> i32 {
        println!("MyStruct.xx Called" );
        return add_one!(42);
    }
}

impl QAIM for MyStruct {
}


fn main() {

   // let xx = MyStruct::create();

    unsafe { cpp!{[] {

        int argc = 1;
        char name[] = "hello";
        char *argv[] = { name };
        QApplication app(argc, argv);
        QQmlApplicationEngine engine;
        RustObject x;
        engine.rootContext()->setContextProperty("_foo", &x);
//        QLabel w("dds");
//        w.show();
        engine.loadData(R"(

import QtQuick 2.0
import QtQuick.Window 2.0

Window {
    visible: true
    width: 320; height: 480
    Rectangle {
        id: page
        color: 'lightgray'
        anchors.fill: parent

        Text {
            id: helloText
            text: 'Hello world!' + _foo.xx()
            y: 30
            anchors.horizontalCenter: page.horizontalCenter
            font.pointSize: 24; font.bold: true
        }
    }
}

        )");
        app.exec();

    }}}
}
