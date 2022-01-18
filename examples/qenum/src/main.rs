use cstr::cstr;

use qmetaobject::prelude::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq, QEnum)]
#[repr(C)]
enum Options {
    Foo = 1,
    Bar = 2,
    Quaz = 3,
}

fn main() {
    qml_register_enum::<Options>(cstr!("RustCode"), 1, 0, cstr!("Options"));
    let mut engine = QmlEngine::new();
    engine.load_data(
        r#"
        import QtQuick 2.6
        import QtQuick.Window 2.0
        // Import our Rust classes
        import RustCode 1.0

        Window {
            visible: true
            Text {
                anchors.centerIn: parent
                text: `Hello! Bar is ${Options.Bar}, Foo is ${Options.Foo}.`
            }
        }
    "#
        .into(),
    );
    engine.exec();
}
