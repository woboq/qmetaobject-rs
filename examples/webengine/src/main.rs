extern crate qmetaobject;
use qmetaobject::*;

qrc!(my_resource,
    "webengine" {
        "main.qml",
        "index.html",
    },
);

fn main() {
    webengine::initialize();
    my_resource();
    let mut engine = QmlEngine::new();
    engine.load_file("qrc:/webengine/main.qml".into());
    engine.exec();
}