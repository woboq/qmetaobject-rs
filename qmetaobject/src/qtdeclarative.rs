/* Copyright (C) 2018 Olivier Goffart <ogoffart@woboq.com>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
use cpp::{cpp, cpp_class};

use crate::scenegraph::*;
use crate::*;

/// Qt is not thread safe, and the engine can only be created once and in one thread.
/// So this is a guard that will be used to panic if the engine is created twice
static HAS_ENGINE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

cpp! {{
    #include <memory>
    #include <QtQuick/QtQuick>
    #include <QtCore/QDebug>
    #include <QtWidgets/QApplication>
    #include <QtQml/QQmlComponent>

    struct SingleApplicationGuard {
        SingleApplicationGuard() {
            rust!(Rust_QmlEngineHolder_ctor[] {
                HAS_ENGINE.compare_exchange(false, true, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst)
                    .expect("There can only be one QmlEngine in the process");
            });
        }
        ~SingleApplicationGuard() {
            rust!(Rust_QmlEngineHolder_dtor[] {
                HAS_ENGINE.compare_exchange(true, false, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst)
                    .unwrap();
            });
        }
    };

    struct QmlEngineHolder : SingleApplicationGuard {
        std::unique_ptr<QApplication> app;
        std::unique_ptr<QQmlApplicationEngine> engine;
        std::unique_ptr<QQuickView> view;

        QmlEngineHolder(int &argc, char **argv)
            : app(new QApplication(argc, argv))
            , engine(new QQmlApplicationEngine())
        {}
    };

// Equivalent with QMetaObject::inherits(), for 5.6 and lower.
    bool qmeta_inherits(const QMetaObject *child, const QMetaObject *check) {
#if QT_VERSION <= QT_VERSION_CHECK(5, 7, 0)
        do {
            if (child == check) {
                return true;
            }
        } while ((child = child->superClass()));
        return false;
#else
        return child->inherits(check);
#endif
    }

#if QT_VERSION <= QT_VERSION_CHECK(6, 0, 0)
    using CreatorFunction = void (*)(void *);
#else
    using CreatorFunction = void (*)(void *, void*);
#endif
}}

cpp_class!(
    /// Wrap a Qt Application and a QmlEngine
    ///
    /// Note that since there can only be one Application in the process, creating two
    /// QmlEngine at the same time is not allowed. Doing that will panic.
    pub unsafe struct QmlEngine as "QmlEngineHolder"
);
impl QmlEngine {
    /// Create a new QmlEngine
    pub fn new() -> QmlEngine {
        let mut arguments: Vec<*mut c_char> = std::env::args()
            .map(|arg| CString::new(arg.into_bytes()).expect("argument contains invalid c-string!"))
            .map(|arg| arg.into_raw())
            .collect();
        let argc: i32 = arguments.len() as i32;
        let argv: *mut *mut c_char = arguments.as_mut_ptr();

        let result = cpp!(unsafe [
            argc as "int",
            argv as "char **"
        ] -> QmlEngine as "QmlEngineHolder" {
            // Static variables when used inside function are initialized only once
            static int _argc  = argc;
            static char **_argv = nullptr;
            // this is *real* initialization, and it would also happen only once
            if (_argv == nullptr) {
                // copy the arguments
                _argv = new char *[argc + 1];
                // argv should be null terminated
                _argv[argc] = nullptr;
                for (int i = 0; i < argc; ++i) {
                    _argv[i] = new char[strlen(argv[i]) + 1];
                    strcpy(_argv[i], argv[i]);
                }
            }
            return QmlEngineHolder(_argc, _argv);
        });

        // run destructor
        for arg in arguments {
            let _ = unsafe { CString::from_raw(arg) };
        }

        result
    }

    /// Loads a file as a qml file (See QQmlApplicationEngine::load(const QString & filePath))
    pub fn load_file(&mut self, path: QString) {
        cpp!(unsafe [self as "QmlEngineHolder *", path as "QString"] {
            self->engine->load(path);
        })
    }

    /// Loads the root QML file located at url (See QQmlApplicationEngine::load(const QUrl &url))
    pub fn load_url(&mut self, url: QUrl) {
        cpp!(unsafe [self as "QmlEngineHolder *", url as "QUrl"] {
            self->engine->load(url);
        })
    }

    /// Loads qml data (See QQmlApplicationEngine::loadData)
    pub fn load_data(&mut self, data: QByteArray) {
        cpp!(unsafe [self as "QmlEngineHolder *", data as "QByteArray"] {
            self->engine->loadData(data);
        })
    }

    /// Loads qml data with `url` as base url component (See QQmlApplicationEngine::loadData)
    pub fn load_data_as(&mut self, data: QByteArray, url: QUrl) {
        cpp!(unsafe [self as "QmlEngineHolder *", data as "QByteArray", url as "QUrl"] {
            self->engine->loadData(data, url);
        })
    }

    /// Launches the application
    pub fn exec(&self) {
        cpp!(unsafe [self as "QmlEngineHolder *"] {
            self->app->exec();
        })
    }

    /// Closes the application
    pub fn quit(&self) {
        cpp!(unsafe [self as "QmlEngineHolder *"] {
            self->app->quit();
        })
    }

    /// Sets a property for this QML context (calls QQmlEngine::rootContext()->setContextProperty)
    pub fn set_property(&mut self, name: QString, value: QVariant) {
        cpp!(unsafe [self as "QmlEngineHolder *", name as "QString", value as "QVariant"] {
            self->engine->rootContext()->setContextProperty(name, value);
        })
    }

    /// Sets an object for this QML context (calls QQmlEngine::rootContext()->setContextObject)
    pub fn set_object<T: QObject + Sized>(&mut self, obj: QObjectPinned<T>) {
        let obj_ptr = obj.get_or_create_cpp_object();
        cpp!(unsafe [self as "QmlEngineHolder *", obj_ptr as "QObject *"] {
            self->engine->rootContext()->setContextObject(obj_ptr);
        })
    }

    /// Sets a property for this QML context (calls QQmlEngine::rootContext()->setContextProperty)
    ///
    // (TODO: consider making the lifetime the one of the engine, instead of static)
    pub fn set_object_property<T: QObject + Sized>(
        &mut self,
        name: QString,
        obj: QObjectPinned<T>,
    ) {
        let obj_ptr = obj.get_or_create_cpp_object();
        cpp!(unsafe [self as "QmlEngineHolder *", name as "QString", obj_ptr as "QObject *"] {
            self->engine->rootContext()->setContextProperty(name, obj_ptr);
        })
    }

    /// Calls [invokeMethod](https://doc.qt.io/qt-5/qmetaobject.html#invokeMethod) on first available root object 
    /// -- typicaly it is your Application Window.
    /// 
    /// Returns `None` if either `invokeMethod` returned `false` or there are no root object present.
    pub fn invoke_method(&mut self, name: QByteArray, args: &[QVariant]) -> Option<QVariant> {
        let args_size = args.len();
        let args_ptr = args.as_ptr();

        assert!(args_size <= 9);

        let mut result: bool = false;
        let rz_ptr = (&mut result) as *mut bool;

        let return_arg: QVariant = cpp!(unsafe [
            self as "QmlEngineHolder *",
            name as "QByteArray",
            args_size as "size_t",
            args_ptr as "QVariant *",
            rz_ptr as "bool *"
        ] -> QVariant as "QVariant"
        {
            auto robjs = self->engine->rootObjects();
            if (robjs.isEmpty()) {
                return {};
            }
            QVariant ret;
            #define INVOKE_METHOD(...) QMetaObject::invokeMethod(robjs.first(), name, Q_RETURN_ARG(QVariant, ret) __VA_ARGS__);
            switch (args_size) {
                case 0: *rz_ptr = INVOKE_METHOD(); break;
                case 1: *rz_ptr = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0])); break;
                case 2: *rz_ptr = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1])); break;
                case 3: *rz_ptr = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2])); break;
                case 4: *rz_ptr = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2]), Q_ARG(QVariant, args_ptr[3])); break;
                case 5: *rz_ptr = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2]), Q_ARG(QVariant, args_ptr[3]), Q_ARG(QVariant, args_ptr[4])); break;
                case 6: *rz_ptr = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2]), Q_ARG(QVariant, args_ptr[3]), Q_ARG(QVariant, args_ptr[4]), Q_ARG(QVariant, args_ptr[5])); break;
                case 7: *rz_ptr = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2]), Q_ARG(QVariant, args_ptr[3]), Q_ARG(QVariant, args_ptr[4]), Q_ARG(QVariant, args_ptr[5]), Q_ARG(QVariant, args_ptr[6])); break;
                case 8: *rz_ptr = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2]), Q_ARG(QVariant, args_ptr[3]), Q_ARG(QVariant, args_ptr[4]), Q_ARG(QVariant, args_ptr[5]), Q_ARG(QVariant, args_ptr[6]), Q_ARG(QVariant, args_ptr[7])); break;
                case 9: *rz_ptr = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2]), Q_ARG(QVariant, args_ptr[3]), Q_ARG(QVariant, args_ptr[4]), Q_ARG(QVariant, args_ptr[5]), Q_ARG(QVariant, args_ptr[6]), Q_ARG(QVariant, args_ptr[7]), Q_ARG(QVariant, args_ptr[8])); break;
            }
            #undef INVOKE_METHOD
            return ret;
        });

        return if result {
            Some(return_arg)
        } else {
            None
        }
    }

    /// This method is the same as [invoke_method] but does not capture or return function's return value
    /// 
    /// Returns `None` if either `invokeMethod` returned `false` or there are no root object present.
    pub fn invoke_method_noreturn(&mut self, name: QByteArray, args: &[QVariant]) -> Option<()> {
        let args_size = args.len();
        let args_ptr = args.as_ptr();

        assert!(args_size <= 9);

        let result: bool = cpp!(unsafe [
            self as "QmlEngineHolder *",
            name as "QByteArray",
            args_size as "size_t",
            args_ptr as "QVariant *"
        ] -> bool as "bool" {
            auto robjs = self->engine->rootObjects();
            if (robjs.isEmpty()) {
                return false;
            }
            
            bool rz = false;
            #define INVOKE_METHOD(...) QMetaObject::invokeMethod(robjs.first(), name __VA_ARGS__);
            switch (args_size) {
                case 0: rz = INVOKE_METHOD(); break;
                case 1: rz = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0])); break;
                case 2: rz = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1])); break;
                case 3: rz = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2])); break;
                case 4: rz = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2]), Q_ARG(QVariant, args_ptr[3])); break;
                case 5: rz = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2]), Q_ARG(QVariant, args_ptr[3]), Q_ARG(QVariant, args_ptr[4])); break;
                case 6: rz = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2]), Q_ARG(QVariant, args_ptr[3]), Q_ARG(QVariant, args_ptr[4]), Q_ARG(QVariant, args_ptr[5])); break;
                case 7: rz = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2]), Q_ARG(QVariant, args_ptr[3]), Q_ARG(QVariant, args_ptr[4]), Q_ARG(QVariant, args_ptr[5]), Q_ARG(QVariant, args_ptr[6])); break;
                case 8: rz = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2]), Q_ARG(QVariant, args_ptr[3]), Q_ARG(QVariant, args_ptr[4]), Q_ARG(QVariant, args_ptr[5]), Q_ARG(QVariant, args_ptr[6]), Q_ARG(QVariant, args_ptr[7])); break;
                case 9: rz = INVOKE_METHOD(, Q_ARG(QVariant, args_ptr[0]), Q_ARG(QVariant, args_ptr[1]), Q_ARG(QVariant, args_ptr[2]), Q_ARG(QVariant, args_ptr[3]), Q_ARG(QVariant, args_ptr[4]), Q_ARG(QVariant, args_ptr[5]), Q_ARG(QVariant, args_ptr[6]), Q_ARG(QVariant, args_ptr[7]), Q_ARG(QVariant, args_ptr[8])); break;
            }
            #undef INVOKE_METHOD

            return rz;
        });

        return if result {
            Some(())
        } else {
            None
        }        
    }

    pub fn trim_component_cache(&self) {
        cpp!(unsafe [self as "QmlEngineHolder *"] {
            self->engine->trimComponentCache();
        })
    }

    pub fn clear_component_cache(&self) {
        cpp!(unsafe [self as "QmlEngineHolder *"] {
            self->engine->clearComponentCache();
        })
    }
    
    /// Give a QObject to the engine by wrapping it in a QJSValue
    ///
    /// This will create the C++ object.
    /// Panic if the C++ object was already created.
    pub fn new_qobject<T: QObject>(&mut self, obj: T) -> QJSValue {
        let obj_ptr = into_leaked_cpp_ptr(obj);
        cpp!(unsafe [
            self as "QmlEngineHolder *",
            obj_ptr as "QObject *"
        ] -> QJSValue as "QJSValue" {
            return self->engine->newQObject(obj_ptr);
        })
    }

    /// Adds an import path for this QML engine (calls QQmlEngine::addImportPath)
    pub fn add_import_path(&mut self, path: QString) {
        cpp!(unsafe [self as "QmlEngineHolder *", path as "QString"] {
            self->engine->addImportPath(path);
        })
    }

    /// Returns a pointer to the C++ object. The pointer is of the type `QQmlEngine *` in C++.
    pub fn cpp_ptr(&self) -> *mut c_void {
        cpp!(unsafe [self as "QmlEngineHolder *"] -> *mut c_void as "QQmlEngine *" {
            return self->engine.get();
        })
    }
}

/// Bindings to a QQuickView
pub struct QQuickView {
    engine: QmlEngine,
}

impl QQuickView {
    /// Creates a new QQuickView, it's engine and an application
    pub fn new() -> QQuickView {
        let mut engine = QmlEngine::new();
        cpp!(unsafe [mut engine as "QmlEngineHolder"] {
            engine.view = std::unique_ptr<QQuickView>(new QQuickView(engine.engine.get(), nullptr));
            engine.view->setResizeMode(QQuickView::SizeRootObjectToView);
        });
        QQuickView { engine }
    }

    /// Returns the wrapper to the engine
    pub fn engine(&mut self) -> &mut QmlEngine {
        &mut self.engine
    }

    /// Refer to the Qt documentation of QQuickView::show
    pub fn show(&mut self) {
        let engine = self.engine();
        cpp!(unsafe [engine as "QmlEngineHolder *"] {
            engine->view->show();
        });
    }

    /// Refer to the Qt documentation of QQuickView::setSource
    pub fn set_source(&mut self, url: QString) {
        let engine = self.engine();
        cpp!(unsafe [engine as "QmlEngineHolder *", url as "QString"] {
            engine->view->setSource(url);
        });
    }
}

impl Default for QQuickView {
    fn default() -> Self {
        Self::new()
    }
}

/// See QQmlComponent::CompilationMode
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompilationMode {
    PreferSynchronous,
    Asynchronous,
}

/// See QQmlComponent::Status
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComponentStatus {
    Null,
    Ready,
    Loading,
    Error,
}

cpp! {{
    struct QQmlComponentHolder {
        std::unique_ptr<QQmlComponent> component;

        QQmlComponentHolder(QQmlEngine *e)
            : component(new QQmlComponent(e))
        {}
    };
}}

cpp_class!(
    /// Wrapper for QQmlComponent
    pub unsafe struct QmlComponent as "QQmlComponentHolder"
);

impl QmlComponent {
    /// Create a QmlComponent using the QmlEngine.
    pub fn new(engine: &QmlEngine) -> QmlComponent {
        cpp!(unsafe [engine as "QmlEngineHolder *"] -> QmlComponent as "QQmlComponentHolder" {
            return QQmlComponentHolder(engine->engine.get());
        })
    }

    /// Returns a pointer to the underlying QQmlComponent. Similar to QObject::get_cpp_object()
    pub fn get_cpp_object(&self) -> *mut c_void {
        cpp!(unsafe [self as "QQmlComponentHolder *"] -> *mut c_void as "QQmlComponent *" {
            return self->component.get();
        })
    }

    /// Performs QQmlComponent::loadUrl
    pub fn load_url(&mut self, url: QUrl, compilation_mode: CompilationMode) {
        cpp!(unsafe [
            self as "QQmlComponentHolder *",
            url as "QUrl",
            compilation_mode as "QQmlComponent::CompilationMode"
        ] {
            self->component->loadUrl(url, compilation_mode);
        })
    }

    /// Performs QQmlComponent::setData with a default url
    pub fn set_data(&mut self, data: QByteArray) {
        cpp!(unsafe [self as "QQmlComponentHolder *", data as "QByteArray"] {
            self->component->setData(data, QUrl());
        })
    }

    /// Performs QQmlComponent::setData
    pub fn set_data_as(&mut self, data: QByteArray, url: QUrl) {
        cpp!(unsafe [self as "QQmlComponentHolder *", data as "QByteArray", url as "QUrl"] {
            self->component->setData(data, url);
        })
    }

    /// Performs QQmlComponent::create
    pub fn create(&mut self) -> *mut c_void {
        cpp!(unsafe [self as "QQmlComponentHolder *"] -> *mut c_void as "QObject *" {
            return self->component->create();
        })
    }

    /// Performs QQmlComponent::status
    pub fn status(&self) -> ComponentStatus {
        cpp!(unsafe [
            self as "QQmlComponentHolder *"
        ] -> ComponentStatus as "QQmlComponent::Status" {
            return self->component->status();
        })
    }

    /// See Qt documentation for QQmlComponent::statusChanged
    pub fn status_changed_signal() -> Signal<fn(status: ComponentStatus)> {
        unsafe {
            Signal::new(cpp!([] -> SignalInner as "SignalInner"  {
                return &QQmlComponent::statusChanged;
            }))
        }
    }
}

/// Register the given type as a QML type
///
/// Refer to the Qt documentation for qmlRegisterType.
pub fn qml_register_type<T: QObject + Default + Sized>(
    uri: &CStr,
    version_major: u32,
    version_minor: u32,
    qml_name: &CStr,
) {
    let uri_ptr = uri.as_ptr();
    let qml_name_ptr = qml_name.as_ptr();
    let meta_object = T::static_meta_object();

    extern "C" fn extra_destruct(c: *mut c_void) {
        cpp!(unsafe [c as "QObject *"] {
            QQmlPrivate::qdeclarativeelement_destructor(c);
        })
    }

    extern "C" fn creator_fn<T: QObject + Default + Sized>(
        c: *mut c_void,
        #[cfg(qt_6_0)] _: *mut c_void,
    ) {
        let b: Box<RefCell<T>> = Box::new(RefCell::new(T::default()));
        let ed: extern "C" fn(c: *mut c_void) = extra_destruct;
        unsafe {
            T::qml_construct(&b, c, ed);
        }
        Box::leak(b);
    }
    let creator_fn: extern "C" fn(c: *mut c_void, #[cfg(qt_6_0)] _: *mut c_void) = creator_fn::<T>;

    let size = T::cpp_size();

    let type_id = <RefCell<T> as PropertyType>::register_type(Default::default());

    cpp!(unsafe [
        qml_name_ptr as "char *",
        uri_ptr as "char *",
        version_major as "int",
        version_minor as "int",
        meta_object as "const QMetaObject *",
        creator_fn as "CreatorFunction",
        size as "size_t",
        type_id as "int"
    ] {
        // BEGIN: From QML_GETTYPENAMES
        // FIXME: list type?
        /*const int listLen = int(strlen("QQmlListProperty<"));
        QVarLengthArray<char,64> listName(listLen + nameLen + 2);
        memcpy(listName.data(), "QQmlListProperty<", size_t(listLen));
        memcpy(listName.data()+listLen, className, size_t(nameLen));
        listName[listLen+nameLen] = '>';
        listName[listLen+nameLen+1] = '\0';*/
        // END

        int parserStatusCast = meta_object && qmeta_inherits(meta_object, &QQuickItem::staticMetaObject)
            ? QQmlPrivate::StaticCastSelector<QQuickItem, QQmlParserStatus>::cast()
            : -1;

        QQmlPrivate::RegisterType api = {
            /*version*/ 0,

        #if QT_VERSION < QT_VERSION_CHECK(6,0,0)
            /*typeId*/ type_id,
        #else
            /*typeId*/ QMetaType(type_id),
        #endif
            /*listId*/ {},  // FIXME: list type?
            /*objectSize*/ int(size),
            /*create*/ creator_fn,
        #if QT_VERSION >= QT_VERSION_CHECK(6,0,0)
            /* userdata */ nullptr,
        #endif
            /*noCreationReason*/ QString(),
        #if QT_VERSION >= QT_VERSION_CHECK(6,0,0)
            /* createValueType */ nullptr,
        #endif

            /*uri*/ uri_ptr,
        #if QT_VERSION < QT_VERSION_CHECK(6,0,0)
            /*versionMajor*/ version_major,
            /*versionMinor*/ version_minor,
        #else
            /*version*/ QTypeRevision::fromVersion(version_major, version_minor),
        #endif
            /*elementName*/ qml_name_ptr,
            /*metaObject*/ meta_object,

            /*attachedPropertiesFunction*/ nullptr,
            /*attachedPropertiesMetaObject*/ nullptr,

            /*parserStatusCast*/ parserStatusCast,
            /*valueSourceCast*/ -1,
            /*valueInterceptorCast*/ -1,

            /*extensionObjectCreate*/ nullptr,
            /*extensionMetaObject*/ nullptr,
            /*customParser*/ nullptr,
            /*revision*/ {}  // FIXME: support revisions?
        };
        QQmlPrivate::qmlregister(QQmlPrivate::TypeRegistration, &api);
    })
}

/// Wrapper around [`void qmlRegisterModule(const char *uri, int versionMajor, int versionMinor)`][qt] function.
///
/// [qt]: https://doc.qt.io/qt-5/qqmlengine.html#qmlRegisterModule
#[cfg(qt_5_9)]
pub fn qml_register_module(uri: &CStr, version_major: u32, version_minor: u32) {
    let uri_ptr = uri.as_ptr();

    cpp!(unsafe [
        uri_ptr as "const char *",
        version_major as "int",
        version_minor as "int"
    ] {
    #if QT_VERSION >= QT_VERSION_CHECK(5,9,0)
        qmlRegisterModule(
            uri_ptr,
            version_major,
            version_minor
        );
    #endif
    });
}

/// Alias for type of `QQmlPrivate::RegisterSingletonType::qobjectApi` callback
/// and its C++ counterpart.
type QmlRegisterSingletonTypeCallback =
    extern "C" fn(qml_engine: *mut c_void, js_engine: *mut c_void) -> *mut c_void;
cpp! {{
    using QmlRegisterSingletonTypeCallback = QObject *(*)(QQmlEngine *, QJSEngine *);
}}

/// Initialization for singleton QML objects.
pub trait QSingletonInit {
    /// Initialize the singleton QML object.
    ///
    /// Will be called on a default-constructed object after the C++ object
    /// has been created.
    ///
    /// # Panics
    /// The process will be aborted when the method panics.
    fn init(&mut self);
}

/// Register the specified type as a singleton QML object.
///
/// A new object will be default-constructed for each new instance of `QmlEngine`.
/// After construction of the corresponding C++ object the `QSingletonInit::init()` function
/// will be called.
///
/// Refer to the Qt documentation for [qmlRegisterSingletonType][qt].
///
/// # Panics
///
/// The process will be aborted when the default or init functions panic.
///
/// [qt]: https://doc.qt.io/qt-5/qqmlengine.html#qmlRegisterSingletonType-3
pub fn qml_register_singleton_type<T: QObject + QSingletonInit + Sized + Default>(
    uri: &CStr,
    version_major: u32,
    version_minor: u32,
    qml_name: &CStr,
) {
    let uri_ptr = uri.as_ptr();
    let qml_name_ptr = qml_name.as_ptr();
    let meta_object = T::static_meta_object();

    extern "C" fn callback_fn<T: QObject + Default + Sized + QSingletonInit>(
        _qml_engine: *mut c_void,
        _js_engine: *mut c_void,
    ) -> *mut c_void {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let obj_box: Box<RefCell<T>> = Box::new(RefCell::new(T::default()));
            let obj_ptr = unsafe { T::cpp_construct(&obj_box) };
            obj_box.borrow_mut().init();
            Box::leak(obj_box);
            obj_ptr
        }));
        match result {
            Ok(value) => value,
            Err(_panic) => {
                eprintln!("qml_register_singleton_type T::default or T::init panicked.");
                std::process::abort()
            }
        }
    }
    let callback_fn: QmlRegisterSingletonTypeCallback = callback_fn::<T>;

    let type_id = <RefCell<T> as PropertyType>::register_type(Default::default());

    cpp!(unsafe [
            uri_ptr as "const char *",
            version_major as "int",
            version_minor as "int",
            qml_name_ptr as "const char *",
            meta_object as "const QMetaObject *",
            callback_fn as "QmlRegisterSingletonTypeCallback",
            type_id as "int"
        ] {

            QQmlPrivate::RegisterSingletonType api = {
    #if QT_VERSION < QT_VERSION_CHECK(6,0,0)
                /*version*/ 2, // for now we are happy with pre-5.14 version 2
    #else
                /*structVersion */ 0,
    #endif

                /*uri*/ uri_ptr,
    #if QT_VERSION < QT_VERSION_CHECK(6,0,0)
                /*versionMajor*/ version_major,
                /*versionMinor*/ version_minor,
    #else
                /*version*/ QTypeRevision::fromVersion(version_major, version_minor),
    #endif
                /*typeName*/ qml_name_ptr,

                /*scriptApi*/ nullptr,
                /*qobjectApi*/ callback_fn,
                // new in version 1
                /*instanceMetaObject*/ meta_object,
                // new in version 2
    #if QT_VERSION < QT_VERSION_CHECK(6,0,0)
                /*typeId*/ type_id,
    #else
                /*typeId*/ QMetaType(type_id),
    #endif
    #if QT_VERSION >= QT_VERSION_CHECK(6,0,0)
                /* extensionObjectCreate */ nullptr,
                /* extensionMetaObject */ nullptr,
    #endif
                /*revision*/ {},
    #if QT_VERSION >= QT_VERSION_CHECK(5,14,0) && QT_VERSION < QT_VERSION_CHECK(6,0,0)
                // new in version 3
                /*generalizedQobjectApi*/ {}
    #endif
            };

            QQmlPrivate::qmlregister(QQmlPrivate::SingletonRegistration, &api);
        })
}

/// Register the passed object as a singleton QML object.
///
/// As there is currently no method to unregister a singleton object, the
/// passed object is leaked and cannot be dropped.
///
/// The object is shared between all instances of `QmlEngine`.
///
/// Refer to the Qt documentation for [qmlRegisterSingletonInstance][qt] (not documented at the time of writing).
///
/// # Availability
///
/// Only available in Qt 5.14 or above.
///
/// [qt]: https://doc.qt.io/qt-5/qtqml-cppintegration-overview.html
// XXX: replace link with real documentation, when it will be generated.
#[cfg(qt_5_14)]
pub fn qml_register_singleton_instance<T: QObject + Sized + Default>(
    uri: &CStr,
    version_major: u32,
    version_minor: u32,
    type_name: &CStr,
    obj: T,
) {
    let uri_ptr = uri.as_ptr();
    let type_name_ptr = type_name.as_ptr();

    let obj_box = Box::new(RefCell::new(obj));
    let obj_ptr = unsafe { T::cpp_construct(&obj_box) };
    Box::leak(obj_box);

    cpp!(unsafe [
            uri_ptr as "char *",
            version_major as "int",
            version_minor as "int",
            type_name_ptr as "char *",
            obj_ptr as "QObject *"
        ] {
    #if QT_VERSION >= QT_VERSION_CHECK(5,14,0)
            qmlRegisterSingletonInstance(
                uri_ptr,
                version_major,
                version_minor,
                type_name_ptr,
                obj_ptr
            );
    #endif
        })
}

/// Register the given enum as a QML type.
///
/// Refer to the Qt documentation for [qmlRegisterUncreatableMetaObject][qt].
///
/// [qt]: https://doc.qt.io/qt-5/qqmlengine.html#qmlRegisterUncreatableMetaObject
#[cfg(qt_5_8)]
pub fn qml_register_enum<T: QEnum>(
    uri: &CStr,
    version_major: u32,
    version_minor: u32,
    qml_name: &CStr,
) {
    let uri_ptr = uri.as_ptr();
    let qml_name_ptr = qml_name.as_ptr();
    let meta_object = T::static_meta_object();

    cpp!(unsafe [
            qml_name_ptr as "char *",
            uri_ptr as "char *",
            version_major as "int",
            version_minor as "int",
            meta_object as "const QMetaObject *"
        ] {
    #if QT_VERSION >= QT_VERSION_CHECK(5, 8, 0)
            qmlRegisterUncreatableMetaObject(
                *meta_object,
                uri_ptr,
                version_major,
                version_minor,
                qml_name_ptr,
                "Access to enums & flags only"
            );
    #endif
        })
}

/// A QObject-like trait to inherit from QQuickItem.
///
/// Work in progress
pub trait QQuickItem: QObject {
    fn get_object_description() -> &'static QObjectDescriptor
    where
        Self: Sized,
    {
        unsafe {
            &*cpp!([]-> *const QObjectDescriptor as "RustQObjectDescriptor const*" {
                return RustQObjectDescriptor::instance<Rust_QQuickItem>();
            })
        }
    }

    fn class_begin(&mut self) {}

    fn component_complete(&mut self) {}

    fn release_resources(&mut self) {}

    /// Handle mouse press, release, or move events. Returns true if the event was accepted.
    fn mouse_event(&mut self, _event: QMouseEvent) -> bool {
        false
    }

    fn geometry_changed(&mut self, _new_geometry: QRectF, _old_geometry: QRectF) {}

    fn update_paint_node(&mut self, node: SGNode<ContainerNode>) -> SGNode<ContainerNode> {
        node
    }
}

cpp! {{
    #include <qmetaobject_rust.hpp>
    #include <QtQuick/QQuickItem>

    #if QT_VERSION < QT_VERSION_CHECK(6, 0, 0)
        #define QT_QQUICKITEM_GEOMETRYCHANGE geometryChanged
    #else
        #define QT_QQUICKITEM_GEOMETRYCHANGE geometryChange
    #endif

    struct Rust_QQuickItem : RustObject<QQuickItem> {
    /*
        virtual QRectF boundingRect() const;
        virtual QRectF clipRect() const;
        virtual bool contains(const QPointF &point) const;
        virtual QVariant inputMethodQuery(Qt::InputMethodQuery query) const;
        virtual bool isTextureProvider() const;
        virtual QSGTextureProvider *textureProvider() const;
        virtual void itemChange(ItemChange, const ItemChangeData &);*/
        void classBegin() override {
            QQuickItem::classBegin();
            rust!(Rust_QQuickItem_classBegin[
                rust_object: QObjectPinned<dyn QQuickItem> as "TraitObject"
            ] {
                rust_object.borrow_mut().class_begin();
            });
        }

        void componentComplete() override {
            QQuickItem::componentComplete();
            rust!(Rust_QQuickItem_componentComplete[
                rust_object: QObjectPinned<dyn QQuickItem> as "TraitObject"
            ] {
                rust_object.borrow_mut().component_complete();
            });
        }

        /*virtual void keyPressEvent(QKeyEvent *event);
        virtual void keyReleaseEvent(QKeyEvent *event);
        virtual void inputMethodEvent(QInputMethodEvent *);
        virtual void focusInEvent(QFocusEvent *);
        virtual void focusOutEvent(QFocusEvent *);*/

        void mousePressEvent(QMouseEvent *event) override { handleMouseEvent(event); }
        void mouseMoveEvent(QMouseEvent *event) override { handleMouseEvent(event); }
        void mouseReleaseEvent(QMouseEvent *event) override { handleMouseEvent(event); }
        //void mouseDoubleClickEvent(QMouseEvent *event) override { handleMouseEvent(event); }

        void handleMouseEvent(QMouseEvent *event) {
           if (!rust!(Rust_QQuickItem_mousePressEvent[
                rust_object: QObjectPinned<dyn QQuickItem> as "TraitObject",
                event: QMouseEvent as "QMouseEvent *"
            ] -> bool as "bool" {
                rust_object.borrow_mut().mouse_event(event)
            })) { event->ignore(); }
        }

        /*
        virtual void mouseUngrabEvent(); // XXX todo - params?
        virtual void touchUngrabEvent();
        virtual void wheelEvent(QWheelEvent *event);
        virtual void touchEvent(QTouchEvent *event);
        virtual void hoverEnterEvent(QHoverEvent *event);
        virtual void hoverMoveEvent(QHoverEvent *event);
        virtual void hoverLeaveEvent(QHoverEvent *event);
        virtual void dragEnterEvent(QDragEnterEvent *);
        virtual void dragMoveEvent(QDragMoveEvent *);
        virtual void dragLeaveEvent(QDragLeaveEvent *);
        virtual void dropEvent(QDropEvent *);
        virtual bool childMouseEventFilter(QQuickItem *, QEvent *);
        virtual void windowDeactivateEvent();*/
        void QT_QQUICKITEM_GEOMETRYCHANGE (const QRectF &new_geometry, const QRectF &old_geometry) override{
            rust!(Rust_QQuickItem_geometryChanged[
                rust_object: QObjectPinned<dyn QQuickItem> as "TraitObject",
                new_geometry: QRectF as "QRectF",
                old_geometry: QRectF as "QRectF"
            ] {
                rust_object.borrow_mut().geometry_changed(new_geometry, old_geometry);
            });
            QQuickItem::QT_QQUICKITEM_GEOMETRYCHANGE(new_geometry, old_geometry);
        }

        QSGNode *updatePaintNode(QSGNode *node, UpdatePaintNodeData *) override {
            return rust!(Rust_QQuickItem_updatePaintNode[
                rust_object: QObjectPinned<dyn QQuickItem> as "TraitObject",
                node: *mut c_void as "QSGNode *"
            ] -> SGNode<ContainerNode> as "QSGNode *" {
                rust_object.borrow_mut().update_paint_node(unsafe {
                    SGNode::<ContainerNode>::from_raw(node)
                })
            });
        }

        void releaseResources() override {
            QQuickItem::releaseResources();
            rust!(Rust_QQuickItem_releaseResources[
                rust_object: QObjectPinned<dyn QQuickItem> as "TraitObject"
            ] {
                rust_object.borrow_mut().release_resources();
            });
        }
        /*
        virtual void updatePolish();
        */
    };
}}

impl<'a> dyn QQuickItem + 'a {
    pub fn bounding_rect(&self) -> QRectF {
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "Rust_QQuickItem *"] -> QRectF as "QRectF" {
            return obj ? obj->boundingRect() : QRectF();
        })
    }

    pub fn update(&self) {
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "Rust_QQuickItem *"] {
            if (obj) obj->update();
        });
    }
}

/// Only a specific subset of [`QEvent::Type`][qt] enum.
///
/// [qt]: https://doc.qt.io/qt-5/qevent.html#Type-enum
#[repr(C)]
#[non_exhaustive]
pub enum QMouseEventType {
    MouseButtonPress = 2,
    MouseButtonRelease = 3,
    // FIXME: WIP
    //MouseButtonDblClick = 4,
    MouseMove = 5,
}

/// A reference to a [`QMouseEvent`][qt] instance.
///
/// [qt]: https://doc.qt.io/qt-5/qmouseevent.html
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct QMouseEvent<'a>(*const c_void, std::marker::PhantomData<&'a u32>);

impl<'a> QMouseEvent<'a> {
    /// Returns the type of event
    pub fn event_type(self) -> QMouseEventType {
        cpp!(unsafe [self as "QMouseEvent *"] -> QMouseEventType as "int" {
            return self->type();
        })
    }
    /// Return the position, wrapper around Qt's QMouseEvent::localPos()
    pub fn position(self) -> QPointF {
        cpp!(unsafe [self as "QMouseEvent *"] -> QPointF as "QPointF" {
            return self->localPos();
        })
    }
}

cpp_class!(
    /// Wrapper for QJSValue
    pub unsafe struct QJSValue as "QJSValue"
);

impl QJSValue {
    pub fn is_bool(&self) -> bool {
        cpp!(unsafe [self as "const QJSValue *"] -> bool as "bool" {
            return self->isBool();
        })
    }

    pub fn is_number(&self) -> bool {
        cpp!(unsafe [self as "const QJSValue *"] -> bool as "bool" {
            return self->isNumber();
        })
    }

    pub fn is_string(&self) -> bool {
        cpp!(unsafe [self as "const QJSValue *"] -> bool as "bool" {
            return self->isString();
        })
    }

    pub fn to_string(&self) -> QString {
        cpp!(unsafe [self as "const QJSValue *"] -> QString as "QString" {
            return self->toString();
        })
    }

    pub fn to_bool(&self) -> bool {
        cpp!(unsafe [self as "const QJSValue *"] -> bool as "bool" {
            return self->toBool();
        })
    }

    pub fn to_number(&self) -> f64 {
        cpp!(unsafe [self as "const QJSValue *"] -> f64 as "double" {
            return self->toNumber();
        })
    }

    pub fn to_variant(&self) -> QVariant {
        cpp!(unsafe [self as "const QJSValue *"] -> QVariant as "QVariant" {
            return self->toVariant();
        })
    }

    pub fn to_qobject<'a, T: QObject + 'a>(&'a self) -> Option<QObjectPinned<'a, T>> {
        let mo = T::static_meta_object();
        let obj = cpp!(unsafe [
            self as "const QJSValue *",
            mo as "const QMetaObject *"
        ] -> *mut c_void as "QObject *" {
            QObject *obj = self->toQObject();
            // FIXME: inheritance?
            return obj && qmeta_inherits(obj->metaObject(), mo) ? obj : nullptr;
        });
        if obj.is_null() {
            return None;
        }
        Some(unsafe { T::get_from_cpp(obj) })
    }
}

impl From<QString> for QJSValue {
    fn from(a: QString) -> QJSValue {
        cpp!(unsafe [a as "QString"] -> QJSValue as "QJSValue" {
            return QJSValue(a);
        })
    }
}

impl From<i32> for QJSValue {
    fn from(a: i32) -> QJSValue {
        cpp!(unsafe [a as "int"] -> QJSValue as "QJSValue" {
            return QJSValue(a);
        })
    }
}

impl From<u32> for QJSValue {
    fn from(a: u32) -> QJSValue {
        cpp!(unsafe [a as "uint"] -> QJSValue as "QJSValue" {
            return QJSValue(a);
        })
    }
}

impl From<f64> for QJSValue {
    fn from(a: f64) -> QJSValue {
        cpp!(unsafe [a as "double"] -> QJSValue as "QJSValue" {
            return QJSValue(a);
        })
    }
}

impl From<bool> for QJSValue {
    fn from(a: bool) -> QJSValue {
        cpp!(unsafe [a as "bool"] -> QJSValue as "QJSValue" {
            return QJSValue(a);
        })
    }
}

impl QMetaType for QJSValue {
    fn register(_name: Option<&CStr>) -> i32 {
        cpp!(unsafe [] -> i32 as "int" { return qMetaTypeId<QJSValue>(); })
    }
}

#[cfg(test)]
mod qjsvalue_tests {
    use super::*;

    #[test]
    fn test_qjsvalue() {
        let foo = QJSValue::from(45);
        assert_eq!(foo.to_number(), 45 as f64);
        assert_eq!(foo.to_string(), "45".into());
        assert_eq!(foo.to_variant().to_qbytearray(), "45".into());
    }

    #[test]
    fn test_is_bool() {
        let bool_value = QJSValue::from(true);
        let num_value = QJSValue::from(42);

        assert!(bool_value.is_bool());
        assert!(!num_value.is_bool());
    }

    #[test]
    fn test_is_number() {
        let string_value = QJSValue::from(QString::from("Konqui"));
        let num_value = QJSValue::from(42);

        assert!(num_value.is_number());
        assert!(!string_value.is_number());
    }

    #[test]
    fn test_is_string() {
        let string_value = QJSValue::from(QString::from("Konqui"));
        let num_value = QJSValue::from(42);

        assert!(string_value.is_string());
        assert!(!num_value.is_string());
    }

    #[test]
    fn test_qvariantlist_from_iter() {
        let v = vec![1u32, 2u32, 3u32];
        let qvl: QVariantList = v.iter().collect();
        assert_eq!(qvl.len(), 3);
        assert_eq!(qvl[1].to_qbytearray().to_string(), "2");
    }
}

/// A QObject-like trait to inherit from QQmlExtensionPlugin.
///
/// Refer to the Qt documentation of QQmlExtensionPlugin
///
/// See also the 'qmlextensionplugins' example.
///
/// ```
/// use qmetaobject::*;
/// use std::ffi::CStr;
///
/// #[derive(Default, QObject)]
/// struct QExampleQmlPlugin {
///     base: qt_base_class!(trait QQmlExtensionPlugin),
///     plugin: qt_plugin!("org.qt-project.Qt.QQmlExtensionInterface/1.0"),
/// }
///
/// impl QQmlExtensionPlugin for QExampleQmlPlugin {
///     fn register_types(&mut self, uri: &CStr) {
///         // call `qml_register_type` here
///     }
/// }
/// ```
pub trait QQmlExtensionPlugin: QObject {
    #[doc(hidden)] // implementation detail for the QObject custom derive
    fn get_object_description() -> &'static QObjectDescriptor
    where
        Self: Sized,
    {
        unsafe {
            &*cpp!([]-> *const QObjectDescriptor as "RustQObjectDescriptor const *" {
                return RustQObjectDescriptor::instance<Rust_QQmlExtensionPlugin>();
            })
        }
    }

    /// Refer to the Qt documentation of QQmlExtensionPlugin::registerTypes
    fn register_types(&mut self, uri: &CStr);
}

cpp! {{
    #include <qmetaobject_rust.hpp>
    #include <QtQml/QQmlExtensionPlugin>

    struct Rust_QQmlExtensionPlugin : RustObject<QQmlExtensionPlugin> {
        void registerTypes(const char *uri) override  {
            rust!(Rust_QQmlExtensionPlugin_registerTypes[
                rust_object: QObjectPinned<dyn QQmlExtensionPlugin> as "TraitObject",
                uri: *const c_char as "const char *"
            ] {
                rust_object.borrow_mut().register_types(unsafe { CStr::from_ptr(uri) });
            });
        }
    };
}}

cpp! {{
    #include <qmetaobject_rust.hpp>
    #include <QtQuick/QQuickItem>
    #include <QtQuick/QQuickPaintedItem>
    #include <QtGui/QPainter>

    struct Rust_QQuickPaintedItem : RustObject<QQuickPaintedItem> {
        void classBegin() override {
            QQuickPaintedItem::classBegin();
            rust!(Rust_QQuickPaintedItem_classBegin[
                rust_object: QObjectPinned<dyn QQuickPaintedItem> as "TraitObject"
            ] {
                rust_object.borrow_mut().class_begin();
            });
        }

        void componentComplete() override {
            QQuickPaintedItem::componentComplete();
            rust!(Rust_QQuickPaintedItem_componentComplete[
                rust_object: QObjectPinned<dyn QQuickPaintedItem> as "TraitObject"
            ] {
                rust_object.borrow_mut().component_complete();
            });
        }

        void mousePressEvent(QMouseEvent *event) override { handleMouseEvent(event); }
        void mouseMoveEvent(QMouseEvent *event) override { handleMouseEvent(event); }
        void mouseReleaseEvent(QMouseEvent *event) override { handleMouseEvent(event); }

        void handleMouseEvent(QMouseEvent *event) {
           if (!rust!(Rust_QQuickPaintedItem_mousePressEvent[
                rust_object: QObjectPinned<dyn QQuickPaintedItem> as "TraitObject",
                event: QMouseEvent as "QMouseEvent *"
            ] -> bool as "bool" {
                rust_object.borrow_mut().mouse_event(event)
            })) { event->ignore(); }
        }

        void QT_QQUICKITEM_GEOMETRYCHANGE (const QRectF &new_geometry, const QRectF &old_geometry) override{
            rust!(Rust_QQuickPaintedItem_geometryChanged[
                rust_object: QObjectPinned<dyn QQuickPaintedItem> as "TraitObject",
                new_geometry: QRectF as "QRectF",
                old_geometry: QRectF as "QRectF"
            ] {
                rust_object.borrow_mut().geometry_changed(new_geometry, old_geometry);
            });
            QQuickPaintedItem::QT_QQUICKITEM_GEOMETRYCHANGE(new_geometry, old_geometry);
        }

        void releaseResources() override {
            QQuickPaintedItem::releaseResources();
            rust!(Rust_QQuickPaintedItem_releaseResources[
                rust_object: QObjectPinned<dyn QQuickPaintedItem> as "TraitObject"
            ] {
                rust_object.borrow_mut().release_resources();
            });
        }

        void paint(QPainter *p) override {
            rust!(Rust_QQuickPaintedItem_paint[
                rust_object: QObjectPinned<dyn QQuickPaintedItem> as "TraitObject",
                p: *mut QPainter as "QPainter*"
            ] {
                rust_object.borrow_mut().paint(&mut *p);
            });
        }
    };
}}

/// A QQuickItem-like trait to inherit from QQuickPaintedItem.
pub trait QQuickPaintedItem: QQuickItem {
    fn get_object_description() -> &'static QObjectDescriptor
    where
        Self: Sized,
    {
        unsafe {
            &*cpp!([]-> *const QObjectDescriptor as "RustQObjectDescriptor const*" {
                return RustQObjectDescriptor::instance<Rust_QQuickPaintedItem>();
            })
        }
    }

    fn paint(&mut self, _p: &mut QPainter) {}
}
