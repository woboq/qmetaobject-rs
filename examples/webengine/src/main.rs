use qmetaobject::prelude::*;
#[cfg(not(no_qt))]
use qmetaobject::webengine;

qrc!(my_resource,
    "webengine" {
        "main.qml",
        "index.html",
    },
);

fn main() {
    #[cfg(not(no_qt))]
    webengine::initialize();
    my_resource();
    let mut engine = QmlEngine::new();
    engine.load_file("qrc:/webengine/main.qml".into());
    #[cfg(not(no_qt))]
    engine.exec();
}
