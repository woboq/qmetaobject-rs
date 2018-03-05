//FIXME!  this should not be in this crate, but is there because i don't want to have cpp! in the
// test crate

use super::*;

cpp!{{
    #include <QtQuick/QtQuick>
    #include <QtCore/QDebug>
}}

pub fn do_test<T: QObject + Sized>(mut obj: T, qml: &str) -> bool {

    let qml_text = "import QtQuick 2.0\n".to_owned() + qml;
    let qml_ba = QByteArray::from_str(&qml_text);
    let obj_ptr = obj.get_cpp_object().ptr;
    unsafe { cpp!([qml_ba as "QByteArray", obj_ptr as "QObject*"] -> bool as "bool" {

        static int argc = 1;
        static char name[] = "hello";
        static char *argv[] = { name };
        QGuiApplication app(argc, argv);

        QQmlApplicationEngine engine;
        engine.rootContext()->setContextProperty("_obj", obj_ptr);
        engine.loadData(qml_ba);
        auto robjs = engine.rootObjects();
        if (robjs.isEmpty())
            return false;
        QVariant b;
        if (!QMetaObject::invokeMethod(robjs.first(), "doTest", Q_RETURN_ARG(QVariant,b)))
            qWarning() << "calling 'doTest' failed";
        return b.toBool();
    })}
}

