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
use super::scenegraph::*;
use super::*;

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
            , engine(new QQmlApplicationEngine()) { }
    };
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
        use std::ffi::CString;

        let mut arguments: Vec<*mut c_char> = std::env::args()
            .map(|arg| CString::new(arg.into_bytes()).expect("argument contains invalid c-string!"))
            .map(|arg| arg.into_raw())
            .collect();
        let argc: i32 = arguments.len() as i32 - 1;
        let argv: *mut *mut c_char = arguments.as_mut_ptr();

        let result = unsafe {
            cpp!([argc as "int", argv as "char**"] -> QmlEngine as "QmlEngineHolder" {
                static int _argc  = argc;
                static char **_argv = nullptr;
                if (_argv == nullptr) {
                    // copy the arguments
                    _argv = new char*[argc + 1];
                    // argv should be null terminated
                    _argv[argc] = nullptr;
                    for (int i=0; i<argc; ++i) {
                        _argv[i] = new char[strlen(argv[i]) + 1];
                        strcpy(_argv[i], argv[i]);
                    }
                }
                return QmlEngineHolder(_argc, _argv);
            })
        };

        for arg in arguments {
            unsafe {
                CString::from_raw(arg);
            }
        }

        result
    }

    /// Loads a file as a qml file (See QQmlApplicationEngine::load(const QString & filePath))
    pub fn load_file(&mut self, path: QString) {
        unsafe {
            cpp!([self as "QmlEngineHolder*", path as "QString"] {
                self->engine->load(path);
            })
        }
    }

    //     pub fn load_url(&mut self, uri: &str) {
    //     }

    /// Loads qml data (See QQmlApplicationEngine::loadData)
    pub fn load_data(&mut self, data: QByteArray) {
        unsafe {
            cpp!([self as "QmlEngineHolder*", data as "QByteArray"] {
                self->engine->loadData(data);
            })
        }
    }

    /// Loads qml data with `url` as base url component (See QQmlApplicationEngine::loadData)
    pub fn load_data_as(&mut self, data: QByteArray, url: QUrl) {
        unsafe {
            cpp!([self as "QmlEngineHolder*", data as "QByteArray", url as "QUrl"] {
                self->engine->loadData(data, url);
            })
        }
    }

    /// Launches the application
    pub fn exec(&self) {
        unsafe { cpp!([self as "QmlEngineHolder*"] { self->app->exec(); }) }
    }
    /// Closes the application
    pub fn quit(&self) {
        unsafe { cpp!([self as "QmlEngineHolder*"] { self->app->quit(); }) }
    }

    /// Sets a property for this QML context (calls QQmlEngine::rootContext()->setContextProperty)
    pub fn set_property(&mut self, name: QString, value: QVariant) {
        unsafe {
            cpp!([self as "QmlEngineHolder*", name as "QString", value as "QVariant"] {
                self->engine->rootContext()->setContextProperty(name, value);
            })
        }
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
        cpp!(unsafe [self as "QmlEngineHolder*", name as "QString", obj_ptr as "QObject*"] {
            self->engine->rootContext()->setContextProperty(name, obj_ptr);
        })
    }

    pub fn invoke_method(&mut self, name: QByteArray, args: &[QVariant]) -> QVariant {
        let args_size = args.len();
        let args_ptr = args.as_ptr();
        unsafe {
            cpp!([self as "QmlEngineHolder*", name as "QByteArray", args_size as "size_t", args_ptr as "QVariant*"]
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
            })
        }
    }

    /// Give a QObject to the engine by wraping it in a QJSValue
    ///
    /// This will create the C++ object.
    /// Panic if the C++ object was already created.
    pub fn new_qobject<T: QObject>(&mut self, obj: T) -> QJSValue {
        let obj_ptr = into_leaked_cpp_ptr(obj);
        unsafe {
            cpp!([self as "QmlEngineHolder*", obj_ptr as "QObject*"] -> QJSValue as "QJSValue" {
                return self->engine->newQObject(obj_ptr);
            })
        }
    }

    /// Adds an import path for this QML engine (calls QQmlEngine::addImportPath)
    pub fn add_import_path(&mut self, path: QString) {
        unsafe {
            cpp!([self as "QmlEngineHolder*", path as "QString"] {
                self->engine->addImportPath(path);
            })
        }
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
        unsafe {
            cpp!([mut engine as "QmlEngineHolder"] {
            engine.view = std::unique_ptr<QQuickView>(new QQuickView(engine.engine.get(), nullptr));
            engine.view->setResizeMode(QQuickView::SizeRootObjectToView);
        } )
        };
        QQuickView { engine }
    }

    /// Returns the wrapper to the engine
    pub fn engine(&mut self) -> &mut QmlEngine {
        &mut self.engine
    }

    /// Refer to the Qt documentation of QQuickView::show
    pub fn show(&mut self) {
        let engine = self.engine();
        unsafe {
            cpp!([engine as "QmlEngineHolder*"] {
            engine->view->show();
        } )
        };
    }

    /// Refer to the Qt documentation of QQuickView::setSource
    pub fn set_source(&mut self, url: QString) {
        let engine = self.engine();
        unsafe {
            cpp!([engine as "QmlEngineHolder*", url as "QString"] {
            engine->view->setSource(url);
        } )
        };
    }
}

impl Default for QQuickView {
    fn default() -> Self {
        Self::new()
    }
}

/// See QQmlComponent::CompilationMode
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompilationMode {
    PreferSynchronous,
    Asynchronous,
}

/// See QQmlComponent::Status
#[repr(u32)]
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

        QQmlComponentHolder(QQmlEngine *e) : component(new QQmlComponent(e)) {}
    };
}}

cpp_class!(
    /// Wrapper for QQmlComponent
    pub unsafe struct QmlComponent as "QQmlComponentHolder"
);

impl QmlComponent {
    /// Create a QmlComponent using the QmlEngine.
    pub fn new(engine: &QmlEngine) -> QmlComponent {
        unsafe {
            cpp!([engine as "QmlEngineHolder*"] -> QmlComponent as "QQmlComponentHolder" {
                return QQmlComponentHolder(engine->engine.get());
            })
        }
    }

    /// Returns a pointer to the underlying QQmlComponent. Similar to QObject::get_cpp_object()
    pub fn get_cpp_object(&self) -> *mut c_void {
        unsafe {
            cpp!([self as "QQmlComponentHolder*"] -> *mut c_void as "QQmlComponent*" {
                return self->component.get();
            })
        }
    }

    /// Performs QQmlComponent::loadUrl
    pub fn load_url(&mut self, url: QUrl, compilation_mode: CompilationMode) {
        unsafe {
            cpp!([self as "QQmlComponentHolder*", url as "QUrl", compilation_mode as "QQmlComponent::CompilationMode"]{
                self->component->loadUrl(url, compilation_mode);
            })
        }
    }

    /// Performs QQmlComponent::setData with a default url
    pub fn set_data(&mut self, data: QByteArray) {
        unsafe {
            cpp!([self as "QQmlComponentHolder*", data as "QByteArray"]{
                self->component->setData(data, QUrl());
            })
        }
    }

    /// Performs QQmlComponent::setData
    pub fn set_data_as(&mut self, data: QByteArray, url: QUrl) {
        unsafe {
            cpp!([self as "QQmlComponentHolder*", data as "QByteArray", url as "QUrl"]{
                self->component->setData(data, url);
            })
        }
    }

    /// Performs QQmlComponent::create
    pub fn create(&mut self) -> *mut c_void {
        unsafe {
            cpp!([self as "QQmlComponentHolder*"] -> *mut c_void as "QObject*" {
                return self->component->create();
            })
        }
    }

    /// Performs QQmlComponent::status
    pub fn status(&self) -> ComponentStatus {
        unsafe {
            cpp!([self as "QQmlComponentHolder*"] -> ComponentStatus as "QQmlComponent::Status" {
                return self->component->status();
            })
        }
    }

    /// See Qt documentation for QQmlComponent::statusChanged
    pub fn status_changed_signal() -> CppSignal<fn(status: ComponentStatus)> {
        unsafe {
            CppSignal::new(
                cpp!([] -> SignalCppRepresentation as "SignalCppRepresentation"  {
                    return &QQmlComponent::statusChanged;
                }),
            )
        }
    }
}

/// Register the given type as a QML type
///
/// Refer to the Qt documentation for qmlRegisterType.
pub fn qml_register_type<T: QObject + Default + Sized>(
    uri: &std::ffi::CStr,
    version_major: u32,
    version_minor: u32,
    qml_name: &std::ffi::CStr,
) {
    let uri_ptr = uri.as_ptr();
    let qml_name_ptr = qml_name.as_ptr();
    let meta_object = T::static_meta_object();

    extern "C" fn extra_destruct(c: *mut c_void) {
        unsafe { cpp!([c as "QObject*"]{ QQmlPrivate::qdeclarativeelement_destructor(c); }) }
    }

    extern "C" fn creator_fn<T: QObject + Default + Sized>(c: *mut c_void) {
        let b: Box<RefCell<T>> = Box::new(RefCell::new(T::default()));
        let ed: extern "C" fn(c: *mut c_void) = extra_destruct;
        unsafe {
            T::qml_construct(&b, c, ed);
        }
        std::boxed::Box::into_raw(b);
    };
    let creator_fn: extern "C" fn(c: *mut c_void) = creator_fn::<T>;

    let size = T::cpp_size();

    unsafe {
        cpp!([qml_name_ptr as "char*", uri_ptr as "char*", version_major as "int",
                        version_minor as "int", meta_object as "const QMetaObject *",
                        creator_fn as "CreatorFunction", size as "size_t"]{

            const char *className = qml_name_ptr;
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

            auto ptrType = QMetaType::registerNormalizedType(pointerName.constData(),
                QtMetaTypePrivate::QMetaTypeFunctionHelper<void*>::Destruct,
                QtMetaTypePrivate::QMetaTypeFunctionHelper<void*>::Construct,
                int(sizeof(void*)), QMetaType::MovableType | QMetaType::PointerToQObject,
                meta_object);

            int parserStatusCast = meta_object && meta_object->inherits(&QQuickItem::staticMetaObject)
                ? QQmlPrivate::StaticCastSelector<QQuickItem,QQmlParserStatus>::cast() : -1;

            QQmlPrivate::RegisterType type = {
                0 /*version*/, ptrType, 0, /* FIXME?*/
                int(size), creator_fn,
                QString(),
                uri_ptr, version_major, version_minor, qml_name_ptr, meta_object,
                nullptr, nullptr, // attached properties
                parserStatusCast, -1, -1,
                nullptr, nullptr,
                nullptr,
                0
            };
            QQmlPrivate::qmlregister(QQmlPrivate::TypeRegistration, &type);
        })
    }
}

cpp! {{
    typedef QObject *(*QmlRegisterSingletonTypeCallback)(size_t);
}}

lazy_static! {
    static ref REGISTERED_SINGLETON_CREATE_FN: std::sync::Mutex<Vec<Box<dyn Fn() -> *mut c_void + Send>>> = std::sync::Mutex::new(Vec::new());
}

/// Register the specified type as a singleton QML object.
///
/// A new object will be created for each new instance of `QmlEngine` by calling
/// the provided `create_fn`. 
///
/// Refer to the Qt documentation for qmlRegisterSingletonType.
/// 
/// # Panics
/// The process will be aborted when `create_fn` panics.
pub fn qml_register_singleton_type<T: QObject + Sized + Default, F: Fn() -> T + Send + 'static>(
    uri: &std::ffi::CStr,
    version_major: u32,
    version_minor: u32,
    type_name: &std::ffi::CStr,
    create_fn: F
) {
    let uri_ptr = uri.as_ptr();
    let type_name_ptr = type_name.as_ptr();

    let create_wrapper = move || {
        let obj = create_fn();
        let obj_box: Box<RefCell<T>> = Box::new(RefCell::new(obj));
        let obj_ptr = unsafe { T::cpp_construct(&obj_box) };
        Box::into_raw(obj_box);
        obj_ptr        
    };

    let create_fn_id; 
    {
        let mut create_fns = REGISTERED_SINGLETON_CREATE_FN.lock().unwrap();
        create_fn_id = create_fns.len();
        create_fns.push(Box::new(create_wrapper));
    }

    extern "C" fn callback_fn<T: QObject + Default + Sized>(
        create_fn_id: usize,
    ) -> *mut c_void {
        let create_fns = REGISTERED_SINGLETON_CREATE_FN.lock().unwrap();
        let create_fn = &create_fns[create_fn_id];
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(create_fn)) {
            Ok(obj_ptr) => obj_ptr,
            Err(panic) => {
                eprintln!("qml_register_singleton_type create_fn panicked: {:?}", panic);
                std::process::abort()
            }
        }
    };
    let callback_fn: extern "C" fn(
        create_fn_id: usize,
    ) -> *mut c_void = callback_fn::<T>;

    unsafe {
        cpp!([uri_ptr as "const char*", version_major as "int", version_minor as "int",
                type_name_ptr as "const char*", create_fn_id as "size_t", callback_fn as "QmlRegisterSingletonTypeCallback"] {
            auto callback_wrapper = [create_fn_id, callback_fn] (QQmlEngine *, QJSEngine *) -> QObject * {
                return callback_fn(create_fn_id);
            };
            qmlRegisterSingletonType<QObject, std::function<QObject *(QQmlEngine *, QJSEngine *)>>
              (uri_ptr, version_major, version_minor, type_name_ptr, std::move(callback_wrapper));
        })
    }
}

/// Register the passed object as a singleton QML object.
///
/// The object is shared between all instances of `QmlEngine`.
///
/// Refer to the Qt documentation for qmlRegisterSingletonInstance.
/// Only avaiable in Qt 5.14 or above.
#[cfg(qt_5_14)]
pub fn qml_register_singleton_instance<T: QObject + Sized + Default>(
    uri: &std::ffi::CStr,
    version_major: u32,
    version_minor: u32,
    type_name: &std::ffi::CStr,
    obj: T,
) {
    let uri_ptr = uri.as_ptr();
    let type_name_ptr = type_name.as_ptr();

    let obj_box = Box::new(RefCell::new(obj));
    let obj_ptr = unsafe { T::cpp_construct(&obj_box) };
    Box::into_raw(obj_box);

    unsafe {
        cpp!([uri_ptr as "char*", version_major as "int", version_minor as "int",
                type_name_ptr as "char*", obj_ptr as "QObject*"] {
            #ifdef QT_5_14
            qmlRegisterSingletonInstance(uri_ptr, version_major, version_minor, type_name_ptr,
                obj_ptr);
            #endif
        })
    }
}

/// Register the given enum as a QML type
///
/// Refer to the Qt documentation for qmlRegisterUncreatableMetaObject.
pub fn qml_register_enum<T: QEnum>(
    uri: &std::ffi::CStr,
    version_major: u32,
    version_minor: u32,
    qml_name: &std::ffi::CStr,
) {
    let uri_ptr = uri.as_ptr();
    let qml_name_ptr = qml_name.as_ptr();
    let meta_object = T::static_meta_object();

    unsafe {
        cpp!([qml_name_ptr as "char*", uri_ptr as "char*", version_major as "int",
                        version_minor as "int", meta_object as "const QMetaObject *"]{
            qmlRegisterUncreatableMetaObject(*meta_object, uri_ptr, version_major,
                version_minor, qml_name_ptr, "Access to enums & flags only");
        })
    }
}

/// A QObject-like trait to inherit from QQuickItem.
///
/// Work in progress
pub trait QQuickItem: QObject {
    fn get_object_description() -> &'static QObjectDescription
    where
        Self: Sized,
    {
        unsafe {
            &*cpp!([]-> *const QObjectDescription as "RustObjectDescription const*" {
            return rustObjectDescription<Rust_QQuickItem>();
        } )
        }
    }

    fn class_begin(&mut self) {}
    fn component_complete(&mut self) {}

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
        rust!(Rust_QQuickItem_classBegin[rust_object : QObjectPinned<dyn QQuickItem> as "TraitObject"] {
            rust_object.borrow_mut().class_begin();
        });
    }

    void componentComplete() override {
        QQuickItem::componentComplete();
        rust!(Rust_QQuickItem_componentComplete[rust_object : QObjectPinned<dyn QQuickItem> as "TraitObject"] {
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
            rust_object : QObjectPinned<dyn QQuickItem> as "TraitObject",
            event : QMouseEvent as "QMouseEvent*"
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
    virtual void geometryChanged(const QRectF &new_geometry,
                                 const QRectF &old_geometry) {
        rust!(Rust_QQuickItem_geometryChanged[rust_object : QObjectPinned<dyn QQuickItem> as "TraitObject",
                new_geometry : QRectF as "QRectF", old_geometry : QRectF as "QRectF"] {
            rust_object.borrow_mut().geometry_changed(new_geometry, old_geometry);
        });
        QQuickItem::geometryChanged(new_geometry, old_geometry);
    }

    QSGNode *updatePaintNode(QSGNode *node, UpdatePaintNodeData *) override {
        return rust!(Rust_QQuickItem_updatePaintNode[rust_object : QObjectPinned<dyn QQuickItem> as "TraitObject",
                    node : *mut c_void as "QSGNode*"] -> SGNode<ContainerNode> as "QSGNode*" {
            rust_object.borrow_mut().update_paint_node(unsafe { SGNode::<ContainerNode>::from_raw(node) })
        });
    }
    /*
    virtual void releaseResources();
    virtual void updatePolish();
*/

};

}}

impl<'a> dyn QQuickItem + 'a {
    pub fn bounding_rect(&self) -> QRectF {
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "Rust_QQuickItem*"] -> QRectF as "QRectF" {
            return obj ? obj->boundingRect() : QRectF();
        })
    }
    pub fn update(&self) {
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "Rust_QQuickItem*"] { if (obj) obj->update(); });
    }
}

#[repr(u32)]
pub enum QMouseEventType {
    MouseButtonPress = 2,
    MouseButtonRelease = 3,
    //MouseButtonDblClick = 4,
    MouseMove = 5,
}

/// A reference to a QMouseEvent
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct QMouseEvent<'a>(*const c_void, std::marker::PhantomData<&'a u32>);
impl<'a> QMouseEvent<'a> {
    /// Returns the type of event
    pub fn event_type(self) -> QMouseEventType {
        cpp!(unsafe [self as "QMouseEvent*"] -> QMouseEventType as "int" { return self->type(); })
    }
    /// Return the position, wrapper around Qt's QMouseEvent::localPos()
    pub fn position(self) -> QPointF {
        cpp!(unsafe [self as "QMouseEvent*"] -> QPointF as "QPointF" { return self->localPos(); })
    }
}

cpp_class!(
    /// Wrapper for QJSValue
    pub unsafe struct QJSValue as "QJSValue"
);
impl QJSValue {
    pub fn to_string(&self) -> QString {
        unsafe {
            cpp!([self as "const QJSValue*"] -> QString as "QString" { return self->toString(); })
        }
    }

    pub fn to_bool(&self) -> bool {
        unsafe { cpp!([self as "const QJSValue*"] -> bool as "bool" { return self->toBool(); }) }
    }

    pub fn to_number(&self) -> f64 {
        unsafe { cpp!([self as "const QJSValue*"] -> f64 as "double" { return self->toNumber(); }) }
    }

    pub fn to_variant(&self) -> QVariant {
        unsafe {
            cpp!([self as "const QJSValue*"] -> QVariant as "QVariant" { return self->toVariant(); })
        }
    }

    pub fn to_qobject<'a, T: QObject + 'a>(&'a self) -> Option<QObjectPinned<'a, T>> {
        let mo = T::static_meta_object();
        let obj = unsafe {
            cpp!([self as "const QJSValue*", mo as "const QMetaObject*"] -> *mut c_void as "QObject*" {
                QObject *obj = self->toQObject();
                // FIXME! inheritence?
                return obj && obj->metaObject()->inherits(mo) ? obj : nullptr;
            })
        };
        if obj.is_null() {
            return None;
        }
        Some(unsafe { T::get_from_cpp(obj) })
    }
}
impl From<QString> for QJSValue {
    fn from(a: QString) -> QJSValue {
        unsafe { cpp!([a as "QString"] -> QJSValue as "QJSValue" { return QJSValue(a); }) }
    }
}
impl From<i32> for QJSValue {
    fn from(a: i32) -> QJSValue {
        unsafe { cpp!([a as "int"] -> QJSValue as "QJSValue" { return QJSValue(a); }) }
    }
}
impl From<u32> for QJSValue {
    fn from(a: u32) -> QJSValue {
        unsafe { cpp!([a as "uint"] -> QJSValue as "QJSValue" { return QJSValue(a); }) }
    }
}
impl From<f64> for QJSValue {
    fn from(a: f64) -> QJSValue {
        unsafe { cpp!([a as "double"] -> QJSValue as "QJSValue" { return QJSValue(a); }) }
    }
}
impl From<bool> for QJSValue {
    fn from(a: bool) -> QJSValue {
        unsafe { cpp!([a as "bool"] -> QJSValue as "QJSValue" { return QJSValue(a); }) }
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
/// # extern crate qmetaobject; use qmetaobject::*;
/// #[derive(Default, QObject)]
/// struct QExampleQmlPlugin {
///     base: qt_base_class!(trait QQmlExtensionPlugin),
///     plugin: qt_plugin!("org.qt-project.Qt.QQmlExtensionInterface/1.0"),
/// }
///
/// impl QQmlExtensionPlugin for QExampleQmlPlugin {
///     fn register_types(&mut self, uri: &std::ffi::CStr) {
///         // call `qml_register_type` here
///     }
/// }
/// ```

pub trait QQmlExtensionPlugin: QObject {
    #[doc(hidden)] // implementation detail for the QObject custom derive
    fn get_object_description() -> &'static QObjectDescription
    where
        Self: Sized,
    {
        unsafe {
            &*cpp!([]-> *const QObjectDescription as "RustObjectDescription const*" {
            return rustObjectDescription<Rust_QQmlExtensionPlugin>();
        } )
        }
    }

    /// Refer to the Qt documentation of QQmlExtensionPlugin::registerTypes
    fn register_types(&mut self, uri: &std::ffi::CStr);
}

cpp! {{
#include <qmetaobject_rust.hpp>
#include <QtQml/QQmlExtensionPlugin>
struct Rust_QQmlExtensionPlugin : RustObject<QQmlExtensionPlugin> {
    void registerTypes(const char *uri) override  {
        rust!(Rust_QQmlExtensionPlugin_registerTypes[rust_object : QObjectPinned<dyn QQmlExtensionPlugin> as "TraitObject",
                                                            uri : *const std::os::raw::c_char as "const char*"] {
            rust_object.borrow_mut().register_types(unsafe { std::ffi::CStr::from_ptr(uri) });
        });
    }
};

}}
