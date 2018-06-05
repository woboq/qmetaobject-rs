use super::*;

cpp!{{
    #include <QtQuick/QtQuick>
    #include <QtCore/QDebug>

    static int argc = 1;
    static char name[] = "rust";
    static char *argv[] = { name };

    struct QmlEngineHolder {
        std::unique_ptr<QGuiApplication> app;
        std::unique_ptr<QQmlApplicationEngine> engine;
        std::unique_ptr<QQuickView> view;

        QmlEngineHolder() : app(new QGuiApplication(argc, argv)), engine(new QQmlApplicationEngine) { }
    };
}}

/// Wrap a Qt Application and a QmlEngine
cpp_class!(pub struct QmlEngine as "QmlEngineHolder");
impl QmlEngine {
    /// create a new QmlEngine
    pub fn new() -> QmlEngine {
        Default::default()
    }

    /// Loads a file as a qml file (See QQmlApplicationEngine::load(const QString & filePath))
    pub fn load_file(&mut self, path: QString) {
        unsafe {cpp!([self as "QmlEngineHolder*", path as "QString"] {
            self->engine->load(path);
        })}
    }

//     pub fn load_url(&mut self, uri: &str) {
//     }

    /// Loads qml data (See QQmlApplicationEngine::loadData)
    pub fn load_data(&mut self, data: QByteArray) {
        unsafe { cpp!([self as "QmlEngineHolder*", data as "QByteArray"] {
            self->engine->loadData(data);
        })}
    }

    /// Launches the application
    pub fn exec(&mut self) {
        unsafe { cpp!([self as "QmlEngineHolder*"] { self->app->exec(); })}
    }
    /// Closes the application
    pub fn quit(&mut self) {
        unsafe { cpp!([self as "QmlEngineHolder*"] { self->app->quit(); })}
    }

    /// Sets a property for this QML context (calls QQmlEngine::rootContext()->setContextProperty)
    pub fn set_property(&mut self, name: QString, value: QVariant) {
        unsafe { cpp!([self as "QmlEngineHolder*", name as "QString", value as "QVariant"] {
            self->engine->rootContext()->setContextProperty(name, value);
        })}
    }

    /// Sets a property for this QML context (calls QQmlEngine::rootContext()->setContextProperty)
    pub fn set_object_property<T : QObject + Sized>(&mut self, name: QString, obj: &mut T) {
        let obj_ptr = unsafe { obj.cpp_construct() }; // FIXME! unsafe
        unsafe { cpp!([self as "QmlEngineHolder*", name as "QString", obj_ptr as "QObject*"] {
            self->engine->rootContext()->setContextProperty(name, obj_ptr);
        })}
    }

    pub fn invoke_method(&mut self, name: QByteArray, args : &[QVariant]) -> QVariant {
        let args_size = args.len();
        let args_ptr = args.as_ptr();
        unsafe{ cpp!([self as "QmlEngineHolder*", name as "QByteArray", args_size as "size_t", args_ptr as "QVariant*"]
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

    /// Give a QObject to the engine by wraping it in a QJSValue
    ///
    /// This will create the C++ object.
    /// Panic if the C++ object was already created.
    pub fn new_qobject<T : QObject>(&mut self, obj : T) -> QJSValue {
        let obj_ptr = into_leaked_cpp_ptr(obj);
        unsafe { cpp!([self as "QmlEngineHolder*", obj_ptr as "QObject*"] -> QJSValue as "QJSValue" {
            return self->engine->newQObject(obj_ptr);
        })}
    }
}

/// Wrap a QQuickView
pub struct QQuickView {
    engine : QmlEngine
}
impl QQuickView {
    /// Creates a new QQuickView, it's engine and an application
    pub fn new() -> QQuickView {
        let mut engine = QmlEngine::new();
        unsafe{ cpp!([mut engine as "QmlEngineHolder"] {
            engine.view = std::unique_ptr<QQuickView>(new QQuickView(engine.engine.get(), nullptr));
            engine.view->setResizeMode(QQuickView::SizeRootObjectToView);
        } ) };
        QQuickView { engine: engine }
    }

    /// Returns the wrapper to the engine
    pub fn engine(&mut self) -> &mut QmlEngine { &mut self.engine }

    /// Refer to the Qt documentation of QQuickView::show
    pub fn show(&mut self) {
        let engine = self.engine();
        unsafe{ cpp!([engine as "QmlEngineHolder*"] {
            engine->view->show();
        } ) };
    }

    /// Refer to the Qt documentation of QQuickView::setSource
    pub fn set_source(&mut self, url: QString) {
        let engine = self.engine();
        unsafe{ cpp!([engine as "QmlEngineHolder*", url as "QString"] {
            engine->view->setSource(url);
        } ) };
    }
}

/// Register the given type as a QML type
///
/// Refer to the Qt documentation for qmlRegisterType.
pub fn qml_register_type<T : QObject + Default + Sized>(uri : &std::ffi::CStr, version_major : u32,
                                                        version_minor : u32, qml_name : &std::ffi::CStr)
{
    let uri_ptr = uri.as_ptr();
    let qml_name_ptr = qml_name.as_ptr();
    let meta_object = T::static_meta_object();

    extern fn extra_destruct(c : *mut c_void) {
        unsafe { cpp!([c as "QObject*"]{ QQmlPrivate::qdeclarativeelement_destructor(c); })}
    }

    extern fn creator_fn<T : QObject + Default + Sized>(c : *mut c_void)  {
        let mut b : Box<T> = Box::new(T::default());
        let ed : extern fn(c : *mut c_void) = extra_destruct;
        unsafe { b.qml_construct(c, ed); }
        std::boxed::Box::into_raw(b);
    };
    let creator_fn : extern fn(c : *mut c_void) = creator_fn::<T>;

    let size = T::cpp_size();

    unsafe { cpp!([qml_name_ptr as "char*", uri_ptr as "char*", version_major as "int",
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

        QQmlPrivate::RegisterType type = {
            0 /*version*/, ptrType, 0, /* FIXME?*/
            int(size), creator_fn,
            QString(),
            uri_ptr, version_major, version_minor, qml_name_ptr, meta_object,
            nullptr, nullptr, // attached properties
            -1, -1, -1,
            nullptr, nullptr,
            nullptr,
            0
        };
        QQmlPrivate::qmlregister(QQmlPrivate::TypeRegistration, &type);
    })}
}

/// A QObject-like trait to inherit from QQuickItem.
///
/// Work in progress
pub trait QQuickItem : QObject {
    fn get_object_description() -> &'static QObjectDescription where Self:Sized {
        unsafe { cpp!([]-> &'static QObjectDescription as "RustObjectDescription const*" {
            return rustObjectDescription<Rust_QQuickItem>();
        } ) }
    }
    unsafe fn get_rust_object<'a>(p: &'a mut c_void)->&'a mut Self  where Self:Sized {
        let ptr = cpp!{[p as "Rust_QQuickItem*"] -> *mut c_void as "void*" {
            return p->rust_object.a;
        }};
        std::mem::transmute::<*mut c_void, &'a mut Self>(ptr)
    }

    //virtual QRectF boundingRect() const;
    //virtual QRectF clipRect() const;
}

impl QQuickItem {
    // here goes the API
    /*pub fn begin_insert_rows(&mut self, first : i32, last: i32) {
        let p = QModelIndex::default();
        let obj = self.get_cpp_object();
        unsafe { cpp!([obj as "Rust_QAbstractListModel*", p as "QModelIndex", first as "int", last as "int"]{
            if (obj) obj->beginInsertRows(p, first, last);
        })}
    }*/
}


cpp!{{
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
    virtual void itemChange(ItemChange, const ItemChangeData &);
    void classBegin() override;
    void componentComplete() override;
    virtual void keyPressEvent(QKeyEvent *event);
    virtual void keyReleaseEvent(QKeyEvent *event);
    virtual void inputMethodEvent(QInputMethodEvent *);
    virtual void focusInEvent(QFocusEvent *);
    virtual void focusOutEvent(QFocusEvent *);
    virtual void mousePressEvent(QMouseEvent *event);
    virtual void mouseMoveEvent(QMouseEvent *event);
    virtual void mouseReleaseEvent(QMouseEvent *event);
    virtual void mouseDoubleClickEvent(QMouseEvent *event);
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
    virtual void windowDeactivateEvent();
    virtual void geometryChanged(const QRectF &newGeometry,
                                 const QRectF &oldGeometry);

    virtual QSGNode *updatePaintNode(QSGNode *, UpdatePaintNodeData *);
    virtual void releaseResources();
    virtual void updatePolish();
*/

    const QMetaObject *metaObject() const override {
        return rust!(Rust_QQuickItem_metaobject[rust_object : &QQuickItem as "TraitObject"]
                -> *const QMetaObject as "const QMetaObject*" {
            rust_object.meta_object()
        });
    }
};

}}

/// Wrapper for QJSValue
cpp_class!(pub struct QJSValue as "QJSValue");
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
        unsafe { cpp!([self as "const QJSValue*"] -> QVariant as "QVariant" { return self->toVariant(); }) }
    }

    // FIXME: &mut could be usefull, but then there can be several access to this object as mutable
    pub fn to_qobject<'a, T : QObject + 'a>(&'a self) -> Option<&'a QObject> {
        let mo = T::static_meta_object();
        let obj = unsafe { cpp!([self as "const QJSValue*", mo as "const QMetaObject*"] -> *mut c_void as "QObject*" {
            QObject *obj = self->toQObject();
            // FIXME! inheritence?
            return obj && obj->metaObject()->inherits(mo) ? obj : nullptr;
        }) };
        if obj.is_null() { return None; }
        let obj : &'a mut c_void = unsafe { &mut *obj  };
        Some(unsafe { T::get_from_cpp(obj) })
    }
}
impl From<QString> for QJSValue {
    fn from(a : QString) -> QJSValue {
        unsafe {cpp!([a as "QString"] -> QJSValue as "QJSValue" { return QJSValue(a); })}
    }
}
impl From<i32> for QJSValue {
    fn from(a : i32) -> QJSValue {
        unsafe {cpp!([a as "int"] -> QJSValue as "QJSValue" { return QJSValue(a); })}
    }
}
impl From<u32> for QJSValue {
    fn from(a : u32) -> QJSValue {
        unsafe {cpp!([a as "uint"] -> QJSValue as "QJSValue" { return QJSValue(a); })}
    }
}
impl From<f64> for QJSValue {
    fn from(a : f64) -> QJSValue {
        unsafe {cpp!([a as "double"] -> QJSValue as "QJSValue" { return QJSValue(a); })}
    }
}
impl From<bool> for QJSValue {
    fn from(a : bool) -> QJSValue {
        unsafe {cpp!([a as "bool"] -> QJSValue as "QJSValue" { return QJSValue(a); })}
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
        let v = vec![1u32,2u32,3u32];
        let qvl : QVariantList = v.iter().collect();
        assert_eq!(qvl.len(), 3);
        assert_eq!(qvl[1].to_qbytearray().to_string(), "2");

    }
}


/// A QObject-like trait to inherit from QQmlExtensionPlugin.
///
/// Refer to the Qt documentation of QQmlExtensionPlugin
pub trait QQmlExtensionPlugin : QObject {
    #[doc(hidden)] // implementation detail for the QObject custom derive
    fn get_object_description() -> &'static QObjectDescription where Self:Sized {
        unsafe { cpp!([]-> &'static QObjectDescription as "RustObjectDescription const*" {
            return rustObjectDescription<Rust_QQmlExtensionPlugin>();
        } ) }
    }
    #[doc(hidden)] // implementation detail for the QObject custom derive
    unsafe fn get_rust_object<'a>(p: &'a mut c_void)->&'a mut Self  where Self:Sized {
        let ptr = cpp!{[p as "Rust_QQmlExtensionPlugin*"] -> *mut c_void as "void*" {
            return p->rust_object.a;
        }};
        std::mem::transmute::<*mut c_void, &'a mut Self>(ptr)
    }

    /// Refer to the Qt documentation of QQmlExtensionPlugin::registerTypes
    fn register_types(&mut self, uri : &std::ffi::CStr);
}


cpp!{{
#include <qmetaobject_rust.hpp>
#include <QtQml/QQmlExtensionPlugin>
struct Rust_QQmlExtensionPlugin : RustObject<QQmlExtensionPlugin> {
    const QMetaObject *metaObject() const override {
        return rust!(Rust_QQmlExtensionPlugin_metaobject[rust_object : &QQmlExtensionPlugin as "TraitObject"]
                -> *const QMetaObject as "const QMetaObject*" {
            rust_object.meta_object()
        });
    }

    void registerTypes(const char *uri) override  {
        rust!(Rust_QQmlExtensionPlugin_registerTypes[rust_object : &mut QQmlExtensionPlugin as "TraitObject",
                                                            uri : *const std::os::raw::c_char as "const char*"] {
            rust_object.register_types(unsafe { std::ffi::CStr::from_ptr(uri) });
        });
    }
};

}}
