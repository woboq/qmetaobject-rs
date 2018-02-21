#[macro_use]
extern crate qmetaobject;
use qmetaobject::QObject;

#[macro_use]
extern crate cpp;

cpp!{{
    #include <QtQuick/QtQuick>
    #include <QtWidgets/QtWidgets>

    #include <qmetaobject_rust.hpp>
}}


#[derive(QObject,Default)]
struct MyStruct {

  //  qt_method!(xx),


//     qt_method!(xx)
    yy : qt_property!(u32),


    foovar : u32


}
impl MyStruct {
    fn xx(&self) -> i32 {
        println!("MyStruct.xx Called" );
        return 42;
    }
}

trait QAIM : QObject {
    //fn create() { println!("Create OBJECT"); }
}


impl QAIM for MyStruct {
}


fn main() {

    let mut xx = MyStruct::default();
    let ptr : *mut QObject = &mut xx;

    unsafe { cpp!{[ptr as "TraitObject"] {

        int argc = 1;
        char name[] = "hello";
        char *argv[] = { name };
        QApplication app(argc, argv);
        QQmlApplicationEngine engine;
        RustObject<QObject> x;
        x.data = ptr;

       // qDebug() << x.metaObject()->property(1).isReadable();

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
            text: 'Hello world!' + _foo.xx() + '\n' + _foo.yy
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
