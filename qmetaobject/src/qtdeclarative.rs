use super::*;
use std::ffi::CString;

cpp!{{
    #include <QtQuick/QtQuick>
    #include <QtCore/QDebug>

    static int argc = 1;
    static char name[] = "rust";
    static char *argv[] = { name };

    struct QmlEngine {
        std::unique_ptr<QGuiApplication> app;
        std::unique_ptr<QQmlApplicationEngine> engine;

        QmlEngine() : app(new QGuiApplication(argc, argv)), engine(new QQmlApplicationEngine) { }
    };
}}

cpp_class!(pub struct QmlEngine, "QmlEngine");
impl QmlEngine {
    pub fn new() -> QmlEngine {
        Default::default()
    }

    /// Loads a file as a qml file (See QQmlApplicationEngine::load(const QString & filePath))
    pub fn load_file(&mut self, path: QString) {
        unsafe {cpp!([self as "QmlEngine*", path as "QString"] {
            self->engine->load(path);
        })}
    }

//     pub fn load_url(&mut self, uri: &str) {
//     }

    /// Loads qml data (See QQmlApplicationEngine::loadData)
    pub fn load_data(&mut self, data: QByteArray) {
        unsafe { cpp!([self as "QmlEngine*", data as "QByteArray"] {
            self->engine->loadData(data);
        })}
    }

    /// Launches the application
    pub fn exec(&mut self) {
        unsafe { cpp!([self as "QmlEngine*"] { self->app->exec(); })}
    }
    /// Closes the application
    pub fn quit(&mut self) {
        unsafe { cpp!([self as "QmlEngine*"] { self->app->quit(); })}
    }

    /// Sets a property for this QML context (calls QQmlEngine::rootContext()->setContextProperty)
    pub fn set_property(&mut self, name: QString, value: QVariant) {
        unsafe { cpp!([self as "QmlEngine*", name as "QString", value as "QVariant"] {
            self->engine->rootContext()->setContextProperty(name, value);
        })}
    }

    /// Sets a property for this QML context (calls QQmlEngine::rootContext()->setContextProperty)
    pub fn set_object_property<T : QObject + Sized>(&mut self, name: QString, obj: &mut T) {
        let obj_ptr = obj.get_cpp_object().get();
        unsafe { cpp!([self as "QmlEngine*", name as "QString", obj_ptr as "QObject*"] {
            self->engine->rootContext()->setContextProperty(name, obj_ptr);
        })}
    }

    pub fn invoke_method(&mut self, name: QByteArray, args : &[QVariant]) -> QVariant {
        let args_size = args.len();
        let args_ptr = args.as_ptr();
        unsafe{ cpp!([self as "QmlEngine*", name as "QByteArray", args_size as "size_t", args_ptr as "QVariant*"]
                -> QVariant as "QVariant" {
            auto robjs = self->engine->rootObjects();
            if (robjs.isEmpty())
                return {};
            QVariant ret;
            QGenericArgument args[9] = {};
            for (uint i = 0; i < args_size; ++i)
                args[i] = Q_ARG(QVariant, args_ptr[i]);
            QMetaObject::invokeMethod(robjs.first(), name, Q_RETURN_ARG(QVariant,ret),
                    args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8]);
            return ret;
        })}
    }
}

pub fn qml_register_type<T : QObject + Default + Sized>(uri : &str, version_major : u32,
                                                        version_minor : u32, qml_name : &str)
{
    let c_uri = CString::new(uri).unwrap();
    let uri_ptr = c_uri.as_ptr();
    let c_qml_name = CString::new(qml_name).unwrap();
    let qml_name_ptr = c_qml_name.as_ptr();
    let meta_object = T::static_meta_object();

    unsafe { cpp!([qml_name_ptr as "char*", meta_object as "const QMetaObject *"]{

       /* const char *className = qml_name_ptr;
        // BEGIN: From QML_GETTYPENAMES
        const int nameLen = int(strlen(className));
        QVarLengthArray<char,48> pointerName(nameLen+2);
        memcpy(pointerName.data(), className, size_t(nameLen));
        pointerName[nameLen] = '*';
        pointerName[nameLen+1] = '\0';
        /*const int listLen = int(strlen("QQmlListProperty<"));
        QVarLengthArray<char,64> listName(listLen + nameLen + 2);
        memcpy(listName.data(), "QQmlListProperty<", size_t(listLen));
        memcpy(listName.data()+listLen, className, size_t(nameLen));
        listName[listLen+nameLen] = '>';
        listName[listLen+nameLen+1] = '\0';*/
        //END

        auto ptrType = QMetaType::registerType(pointerName,
            QtMetaTypePrivate::QMetaTypeFunctionHelper<void*>::Destruct,
            QtMetaTypePrivate::QMetaTypeFunctionHelper<void*>::Construct,
            int(sizeof(void*)), QMetaType::MovableType | QMetaType::PointerToQObject,
            meta_object);

        QQmlPrivate::RegisterType type = {
            0 /*version*/, ptrType, 0, /* FIXME?*/

        sizeof(T), QQmlPrivate::createInto<T>,
        QString(),

        uri, versionMajor, versionMinor, qmlName, &T::staticMetaObject,

        QQmlPrivate::attachedPropertiesFunc<T>(),
        QQmlPrivate::attachedPropertiesMetaObject<T>(),

        QQmlPrivate::StaticCastSelector<T,QQmlParserStatus>::cast(),
        QQmlPrivate::StaticCastSelector<T,QQmlPropertyValueSource>::cast(),
        QQmlPrivate::StaticCastSelector<T,QQmlPropertyValueInterceptor>::cast(),

        nullptr, nullptr,

        nullptr,
        0
    };*/
    })}
}
