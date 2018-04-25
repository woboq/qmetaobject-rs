#[macro_use]
extern crate qmetaobject;
use qmetaobject::{QObject, QVariant, QAbstractListModel, QModelIndex};

#[macro_use]
extern crate cpp;

cpp!{{
    #include <QtQuick/QtQuick>
    #include <QtWidgets/QtWidgets>

    #include <qmetaobject_rust.hpp>
}}

use qmetaobject::QByteArray;

#[derive(QObject,Default)]
struct MyStruct {

    base : qt_base_class!(trait QObject),
    //base : *mut std::os::raw::c_void,

    yy : qt_property!(u32; NOTIFY yyChanged),

    qq : qt_property!(String),


    xx: qt_method!( fn xx(&self) -> f32 {
        println!("MyStruct.xx Called" );
        return 42.5;
    } ),

    add: qt_method!( fn add(&self, a: u32, b: f64) -> f32 {
        println!("MyStruct.xx Called" );
        return (a + self.yy) as f32 + b as f32;
    } ),

    yyChanged: qt_signal!(),

    another_signa: qt_signal!(foo: u32),

    toString_: qt_method!(fn toString_(&self) -> QByteArray {
        "I'm the object".into()
    } ),


}
/*impl MyStruct {
    fn xx(&self) -> i32 {
        println!("MyStruct.xx Called" );
        return 42;
    }
}*/


#[derive(QObject,Default)]
struct MyModel {

    base : qt_base_class!(trait QAbstractListModel),

    values : Vec<String>

}
impl QAbstractListModel for MyModel {
    fn row_count(&self) -> i32 {
        self.values.len() as i32
    }
    fn data(&self, index: QModelIndex, role:i32) -> QVariant {
        let idx = index.row();
        if role == 0xff0012 && idx >= 0 && (idx as usize) < self.values.len() {
            QVariant::from(QByteArray::from(&*(self.values[idx as usize])))
        } else {
            QVariant::default()
        }
    }
    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        [(0xff0012, QByteArray::from("the_text"))].iter().cloned().collect()
    }
}



fn main() {

    let mut xx = MyStruct::default();
    xx.yy = 85;
    xx.qq = "Hello".to_owned();
    let ptr = unsafe { xx.cpp_construct() };

    let mut mm = MyModel::default();
    mm.values = vec!["hello, ".to_owned(), "world".to_owned()];
    let ptr2 = unsafe { mm.cpp_construct() };

    unsafe { cpp!{[ptr as "QObject*", ptr2 as "QObject*"] {

        int argc = 1;
        char name[] = "hello";
        char *argv[] = { name };
        QApplication app(argc, argv);
        QQmlApplicationEngine engine;

        engine.rootContext()->setContextProperty("_foo", ptr);
        engine.rootContext()->setContextProperty("_model", ptr2);
//        QLabel w("dds");
//        w.show();
        engine.loadData(R"(

import QtQuick 2.0
import QtQuick.Window 2.0

Window {
    visible: true
    width: 520; height: 680
    Rectangle {
        id: page
        color: 'lightgray'
        anchors.fill: parent

        Text {
            id: helloText
            text: 'Hello world! \n' + _foo.xx() + '\n' + _foo.yy +  '\n' + _foo.qq
                    + '\n' + _foo.add(2.2 , 3.3)
            anchors.horizontalCenter: page.horizontalCenter
            font.pointSize: 24; font.bold: true
        }
        MouseArea {
            anchors.fill: helloText
            onClicked: {
                _foo.yy += 5;
                console.log(_foo.toString_());
            }
        }
        ListView {
            width: parent.width;
            anchors.top: helloText.bottom
            anchors.bottom: parent.bottom
            model: _model
            delegate: Rectangle {
                //color: 'blue';
                border.width: 3
                border.color: 'green'
                width: parent.width
                height: 123;
                Text {
                    text: the_text
                }
            }
        }

    }
}

        )");
        app.exec();

    }}}
}
