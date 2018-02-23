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

    base : qt_base_class!(trait QObject),
    //base : *mut std::os::raw::c_void,

    yy : qt_property!(u32),

    qq : qt_property!(f32),


    xx: qt_method!( fn xx(&self) -> i32 {
        println!("MyStruct.xx Called" );
        return 42;
    } ),

    yyChanged: qt_signal!()


}
/*impl MyStruct {
    fn xx(&self) -> i32 {
        println!("MyStruct.xx Called" );
        return 42;
    }
}*/

fn main() {

    let mut xx = MyStruct::default();
    xx.yy = 85;
    xx.construct_cpp_object();
    let ptr = xx.get_cpp_object().ptr;


    unsafe { cpp!{[ptr as "QObject*"] {

        int argc = 1;
        char name[] = "hello";
        char *argv[] = { name };
        QApplication app(argc, argv);
        QQmlApplicationEngine engine;

        qDebug() << ptr->metaObject()->method(4).methodSignature();
        qDebug() << ptr->metaObject()->method(5).methodSignature();
        qDebug() << ptr->metaObject()->method(6).methodSignature();

        engine.rootContext()->setContextProperty("_foo", ptr);
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
            text: 'Hello world! \n' + _foo.xx() + '\n' + _foo.yy +  '\n' + _foo.qq
            y: 30
            anchors.horizontalCenter: page.horizontalCenter
            font.pointSize: 24; font.bold: true
        }
        MouseArea {
            anchors.fill: parent
            onClicked: {
                _foo.yy += 5;
                console.log(_foo.yy);
                _foo.yyChanged()
            }
        }
    }
}

        )");
        app.exec();

    }}}
}
