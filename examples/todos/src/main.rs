use cstr::cstr;

use qmetaobject::prelude::*;

mod implementation;

qrc!(my_resource,
    "todos/qml" {
        "main.qml",
    },
);

fn main() {
    my_resource();
    qml_register_type::<implementation::Todos>(cstr!("RustCode"), 1, 0, cstr!("Todos"));
    let mut engine = QmlEngine::new();
    engine.load_file("qrc:/todos/qml/main.qml".into());
    engine.exec();
}
