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
use cpp::cpp;

use super::*;

cpp! {{
#if QT_VERSION < QT_VERSION_CHECK(6, 0, 0)

    // Hack to access QMetaType::registerConverterFunction which is private, but ConverterFunctor
    // is a friend
    namespace QtPrivate {
    template<>
    struct ConverterFunctor<TraitObject, TraitObject, TraitObject> : public AbstractConverterFunction
    {
        using AbstractConverterFunction::AbstractConverterFunction;
        bool registerConverter(int from, int to) {
            return QMetaType::registerConverterFunction(this, from, to);
        }
    };
    }

    using RustQMetaType = QMetaType;
    using RustMetaTypeConverterFn = QtPrivate::AbstractConverterFunction::Converter;

    static void rust_register_qmetatype_conversion(int from, int to, RustMetaTypeConverterFn converter_fn) {
        // NOTE: the ConverterFunctor are gonna be leaking (in Qt, they are supposed to be allocated in static storage
        auto c = new QtPrivate::ConverterFunctor<TraitObject, TraitObject, TraitObject>(converter_fn);
        if (!c->registerConverter(from, to))
            delete c;
    }

#else

    namespace QtPrivate {
    template<>
    struct IsMetaTypePair<TraitObject, true>
    {
        inline static bool registerConverter(QMetaType::ConverterFunction f, QMetaType from, QMetaType to) {
            return QMetaType::registerConverterFunction(f, from, to);
        }
    };
    }

    struct RustQMetaType : QtPrivate::QMetaTypeInterface {
        // some typedef that are gone in Qt6
        typedef void (*Deleter)(void *);
        typedef void (*Creator)(const QtPrivate::QMetaTypeInterface *, void *, const void *); // copy
        typedef void (*Destructor)(const QtPrivate::QMetaTypeInterface *, void *);
        typedef void (*Constructor)(const QtPrivate::QMetaTypeInterface *, void *);

        const QMetaObject *metaObject;
        QByteArray name;

        RustQMetaType(
            const QMetaObject *metaObject,
            QByteArray name,
            ushort align,
            uint size,
            uint flags,
            Constructor constructor_fn,
            Creator creator_or_copy_fn,
            Destructor destructor_fn,
            QtPrivate::QMetaTypeInterface::EqualsFn equals_fn = nullptr
        ) : QtPrivate::QMetaTypeInterface {
                /*.revision=*/ 0,
                /*.alignment=*/ align,
                /*.size=*/ size,
                /*.flags=*/ flags,
                /*.typeId=*/ 0,
                /*.metaObjectFn=*/ [](const QtPrivate::QMetaTypeInterface *iface) {
                    return static_cast<const RustQMetaType *>(iface)->metaObject;
                },
                /*.name=*/ name.constData(),
                /*.defaultCtr=*/ constructor_fn,
                /*.copyCtr=*/ creator_or_copy_fn,
                /*.moveCtr=*/ nullptr,
                /*.dtor=*/ destructor_fn,
                /*.equals=*/ equals_fn,
                /*.lessThan=*/ nullptr,
                /*.debugStream=*/ nullptr,
                /*.dataStreamOut=*/ nullptr,
                /*.dataStreamIn=*/ nullptr,
                /*.legacyRegisterOp=*/ nullptr,
            },
            metaObject(metaObject),
            name(std::move(name))
        {}
    };

    typedef bool (*RustMetaTypeConverterFn)(const void *src, void *dst);

    static void rust_register_qmetatype_conversion(int from, int to, RustMetaTypeConverterFn converter_fn) {
        QtPrivate::IsMetaTypePair<TraitObject, true>::registerConverter(converter_fn, QMetaType(from), QMetaType(to));
    }

#endif
}}

fn register_metatype_common<T: QMetaType>(
    name: *const c_char,
    gadget_metaobject: *const QMetaObject,
) -> i32 {
    use std::any::TypeId;
    use std::collections::{HashMap, HashSet};
    use std::sync::Mutex;

    lazy_static! {
        static ref HASHMAP: Mutex<HashMap<TypeId, (i32, HashSet<CString>)>> =
            Mutex::new(HashMap::new());
    };

    let mut h = HASHMAP.lock().unwrap_or_else(|e| e.into_inner());
    let e = h.entry(TypeId::of::<T>()).or_insert_with(|| {
        let size = std::mem::size_of::<T>() as u32;
        let align = std::mem::align_of::<T>() as u16;

        #[cfg(not(qt_6_0))]
        extern "C" fn deleter_fn<T>(v: *mut T) {
            let _ = unsafe { Box::from_raw(v) };
        }
        #[cfg(not(qt_6_0))]
        let deleter_fn: extern "C" fn(v: *mut T) = deleter_fn;

        #[cfg(qt_6_0)]
        let deleter_fn: *const c_void = ::core::ptr::null();

        #[cfg(not(qt_6_0))]
        extern "C" fn creator_fn<T: Default + Clone>(c: *const T) -> *const T {
            if c.is_null() {
                Box::into_raw(Box::new(Default::default()))
            } else {
                Box::into_raw(Box::new(unsafe { (*c).clone() }))
            }
        }
        #[cfg(not(qt_6_0))]
        let creator_or_copy_fn: extern "C" fn(c: *const T) -> *const T = creator_fn;

        extern "C" fn destructor_fn<T>(#[cfg(qt_6_0)] _ : *const c_void, ptr: *mut T) {
            unsafe {
                std::ptr::read(ptr);
            }
        }
        let destructor_fn: extern "C" fn(#[cfg(qt_6_0)] _ : *const c_void, ptr: *mut T) = destructor_fn;

        #[cfg(not(qt_6_0))]
        extern "C" fn constructor_fn<T: Default + Clone>(dst: *mut T, c: *const T) -> *mut T {
            unsafe {
                let n = if c.is_null() {
                    Default::default()
                } else {
                    (*c).clone()
                };
                std::ptr::write(dst, n);
            }
            dst
        }
        #[cfg(not(qt_6_0))]
        let constructor_fn: extern "C" fn(ptr: *mut T, c: *const T) -> *mut T = constructor_fn;

        #[cfg(qt_6_0)]
        unsafe extern "C" fn copy_constructor_fn<T: Clone>(_: *const c_void, dst: *mut T, c: *const T) {
            std::ptr::write(dst, (*c).clone());
        }
        #[cfg(qt_6_0)]
        let creator_or_copy_fn: unsafe extern "C" fn(_: *const c_void, ptr: *mut T, c: *const T)  = copy_constructor_fn;

        #[cfg(qt_6_0)]
        unsafe extern "C" fn default_constructor_fn<T: Default>(_: *const c_void, dst: *mut T) {
            std::ptr::write(dst, T::default());
        }
        #[cfg(qt_6_0)]
        let constructor_fn: unsafe extern "C" fn(_: *const c_void, dst: *mut T) = default_constructor_fn;

        let name = CString::new(format!("{:?}", TypeId::of::<T>())).unwrap();
        let name = name.as_ptr();
        let type_id = cpp!(unsafe [
            name as "const char *",
            size as "uint",
            align as "ushort",
            deleter_fn as "RustQMetaType::Deleter",
            creator_or_copy_fn as "RustQMetaType::Creator",
            destructor_fn as "RustQMetaType::Destructor",
            constructor_fn as "RustQMetaType::Constructor",
            gadget_metaobject as "const QMetaObject *"
        ] -> i32 as "int" {
            QMetaType::TypeFlags extraFlag(gadget_metaobject ? QMetaType::IsGadget : 0);
            auto flags = QMetaType::NeedsConstruction | QMetaType::NeedsDestruction | QMetaType::MovableType | extraFlag;

        #if QT_VERSION < QT_VERSION_CHECK(6, 0, 0)

            Q_UNUSED(align);
            return QMetaType::registerType(
                gadget_metaobject ? gadget_metaobject->className() : name,
                deleter_fn,
                creator_or_copy_fn,
                destructor_fn,
                constructor_fn,
                size,
                flags,
                gadget_metaobject
            );

        #else

            QByteArray name_ba(gadget_metaobject ? gadget_metaobject->className() : name);
            Q_UNUSED(deleter_fn)
            // FIXME: the rust code generate Qt5 compatible function and we wrap them in the Qt6 ones, it would be better
            // to just use the Qt6 signature directly
            // We should also consider building this structure at compile time!
            auto mt = new RustQMetaType(gadget_metaobject, name_ba, align, size, flags, constructor_fn, creator_or_copy_fn, destructor_fn);
            return QMetaType(mt).id();

        #endif
        });


        /*
        #[cfg(qt_6_0)]
        let type_id = {
            #[repr(C)]
            #[allow(style)]
            struct QMetaTypeInterface {
                revision: u16,
                alignment: u16,
                size: u32,
                flags: u32,
                typeId: ::core::sync::atomic::AtomicI32,
                metaObjectFn: extern "C" fn(*const QMetaTypeInterface)-> *const QMetaObject,
                name: *const c_char,
                defaultCtr:  extern "C" fn(*const QMetaTypeInterface, *mut c_void),
                copyCtr:  extern "C" fn(*const QMetaTypeInterface, *mut c_void, *const c_void),
                moveCtr:  extern "C" fn(*const QMetaTypeInterface, *mut c_void, *mut c_void),
                dtor: extern "C" fn(*const QMetaTypeInterface, *mut c_void),
                equals: usize,
                lessThan: usize,
                debugStream: usize,
                dataStreamOut: usize,
                dataStreamIn: usize,
                legacyRegisterOp: usize,

                /// Added for the rust one
                meta_object: *const QMetaObject,
            };
        };*/

        if T::CONVERSION_TO_STRING.is_some() {
            extern "C" fn converter_fn<T : QMetaType>(#[cfg(not(qt_6_0))] _ : *const c_void, src: &T, dst : *mut QString) -> bool {
                unsafe { std::ptr::write(dst, (T::CONVERSION_TO_STRING.unwrap())(src)) };
                true
            }
            let converter_fn: extern "C" fn(#[cfg(not(qt_6_0))] *const c_void, &T, *mut QString) -> bool = converter_fn;
            cpp!(unsafe [type_id as "int", converter_fn as "RustMetaTypeConverterFn"] {
                rust_register_qmetatype_conversion(type_id, QMetaType::QString, converter_fn);
            });
        };

        if T::CONVERSION_FROM_STRING.is_some() {
            extern "C" fn converter_fn<T : QMetaType>(#[cfg(not(qt_6_0))] _ : *const c_void, src : &QString, dst : *mut T) -> bool {
                unsafe { std::ptr::write(dst, (T::CONVERSION_FROM_STRING.unwrap())(src)) };
                true
            }
            let converter_fn: extern "C" fn(#[cfg(not(qt_6_0))] *const c_void, &QString, *mut T) -> bool = converter_fn;
            cpp!(unsafe [type_id as "int", converter_fn as "RustMetaTypeConverterFn"] {
                rust_register_qmetatype_conversion(QMetaType::QString, type_id, converter_fn);
            });
        };
        (type_id, HashSet::new())
    });
    let id = e.0;
    if !name.is_null() && !e.1.contains(unsafe { CStr::from_ptr(name) }) {
        let x = cpp!(unsafe [name as "const char *", id as "int"] -> i32 as "int" {
            if (int exist = QMetaType::type(name)) {
                if (exist != id) {
                    qWarning("Attempt to register %s as a typedef of %s, while it was already registered as %s",
                        name, QMetaType::typeName(id), QMetaType::typeName(exist));
                }
                return exist;
            }
        #if QT_VERSION < QT_VERSION_CHECK(6, 0, 0)
            return QMetaType::registerTypedef(name, id);
        #else
            QMetaType::registerNormalizedTypedef(QMetaObject::normalizedType(name), QMetaType(id));
            return id;
        #endif
        });
        assert_eq!(x, id, "Attempt to register the same type with different name");
        e.1.insert(unsafe { CStr::from_ptr(name) }.to_owned());
    }
    id
}

fn register_metatype_qobject<T: QObject>() -> i32 {
    let metaobject = T::static_meta_object();
    cpp!(unsafe [metaobject as "const QMetaObject *"] -> i32 as "int" {
        QByteArray name_ba(metaobject->className());
        name_ba += "*";
    #if QT_VERSION < QT_VERSION_CHECK(6, 0, 0)
        return QMetaType::registerType(
            name_ba.constData(),
            [](void *p) { delete static_cast<void **>(p); },
            [](const void *p) -> void * {
                using T = void *;
                return new T{ p ? *static_cast<const T *>(p) : nullptr};
            },
            QtMetaTypePrivate::QMetaTypeFunctionHelper<void *>::Destruct,
            QtMetaTypePrivate::QMetaTypeFunctionHelper<void *>::Construct,
            sizeof(void *),
            QMetaType::MovableType | QMetaType::PointerToQObject,
            metaobject
        );
    #else
        // TODO: We should also consider building this structure at compile time!
        auto mt = new RustQMetaType(metaobject, name_ba,
            alignof(void *), sizeof(void *), QMetaType::RelocatableType | QMetaType::PointerToQObject,
                [](const QtPrivate::QMetaTypeInterface *, void *dst)
                    { *static_cast<void**>(dst) = nullptr; },
                [](const QtPrivate::QMetaTypeInterface *, void *dst, const void *src)
                    { *static_cast<void**>(dst) = *static_cast<void*const*>(src); },
                nullptr,
                [](const QtPrivate::QMetaTypeInterface *, const void *a, const void *b)
                    { return *static_cast<void*const*>(a) == *static_cast<void*const*>(b); });
        return QMetaType(mt).id();
    #endif
    })
}

/// Implement this trait for type that should be known to the QMetaObject system
///
/// Once implemented for a type, it can be used as a type of a qt_property! or
/// as a parameter of a qt_method!
///
/// ```
/// use qmetaobject::QMetaType;
///
/// #[derive(Default, Clone)]
/// struct MyStruct(u32, String);
///
/// impl QMetaType for MyStruct {}
/// ```
pub trait QMetaType: Clone + Default + 'static {
    /// Registers the type.
    ///
    /// See the Qt documentation of qRegisterMetaType()
    ///
    /// The default implementation should work for most types
    fn register(name: Option<&CStr>) -> i32 {
        register_metatype_common::<Self>(
            name.map_or(std::ptr::null(), |x| x.as_ptr()),
            std::ptr::null(),
        )
    }

    fn id() -> i32 {
        Self::register(None)
    }

    /// Returns a QVariant containing a copy of this object
    fn to_qvariant(&self) -> QVariant {
        let id: i32 = Self::id();
        cpp!(unsafe [self as "const void *", id as "int"] -> QVariant as "QVariant" {
        #if QT_VERSION < QT_VERSION_CHECK(6,0,0)
            return QVariant(id, self);
        #else
            return QVariant(QMetaType(id), self);
        #endif
        })
    }

    /// Attempt to convert from a QVariant to this type.
    fn from_qvariant(mut variant: QVariant) -> Option<Self> {
        let id: i32 = Self::id();
        let var_ptr = &mut variant as *mut QVariant;
        let ptr = cpp!(unsafe [
            var_ptr as "QVariant *",
            id as "int"
        ] -> *const c_void as "const void *" {
            return var_ptr->canConvert(id) && var_ptr->convert(id)
                ? var_ptr->constData()
                : nullptr;
        });
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &*(ptr as *const Self) }.clone())
        }
    }

    /// If this is set to a function, it enable the conversion to and from QString
    const CONVERSION_TO_STRING: Option<fn(&Self) -> QString> = None;
    const CONVERSION_FROM_STRING: Option<fn(&QString) -> Self> = None;
}

#[doc(hidden)]
#[cfg(qt_6_0)]
/// Return the internal pointer to `QtPrivate::QMetaTypeInterface`
pub fn qmetatype_interface_ptr<T: PropertyType>(name: &CStr) -> *const c_void {
    let id = T::register_type(name);
    cpp!(unsafe [id as "int"] -> *const c_void as "const void*" {
    #if QT_VERSION >= QT_VERSION_CHECK(6,0,0)
        return QMetaType(id).iface();
    #else
        Q_UNUSED(id);
        return nullptr;
    #endif
    })
}

/// QGadget are automatically QMetaType
impl<T: QGadget> QMetaType for T
where
    T: Clone + Default + 'static,
{
    fn register(name: Option<&std::ffi::CStr>) -> i32 {
        register_metatype_common::<T>(
            name.map_or(std::ptr::null(), |x| x.as_ptr()),
            T::static_meta_object(),
        )
    }
}

impl QMetaType for String {
    const CONVERSION_TO_STRING: Option<fn(&Self) -> QString> = Some(|s| QString::from(&*s as &str));
    const CONVERSION_FROM_STRING: Option<fn(&QString) -> Self> = Some(|s| s.to_string());
}

macro_rules! qdeclare_builtin_metatype {
    ($name:ty => $value:expr) => {
        impl QMetaType for $name {
            fn register(_name: Option<&CStr>) -> i32 {
                $value
            }
        }
    };
}

// See https://doc.qt.io/qt-5/qmetatype.html#Type-enum

qdeclare_builtin_metatype! {()   => 43}
qdeclare_builtin_metatype! {bool => 1}
qdeclare_builtin_metatype! {i32  => 2}
qdeclare_builtin_metatype! {u32  => 3}
qdeclare_builtin_metatype! {i64  => 4}
qdeclare_builtin_metatype! {u64  => 5}
qdeclare_builtin_metatype! {f64  => 6}
qdeclare_builtin_metatype! {i16  => 33}
qdeclare_builtin_metatype! {i8   => 34}
qdeclare_builtin_metatype! {u16  => 36}
qdeclare_builtin_metatype! {u8   => 37}
qdeclare_builtin_metatype! {f32  => 38}
//qdeclare_builtin_metatype!{"*c_void" => 31}
qdeclare_builtin_metatype! {QVariantMap => 8}
qdeclare_builtin_metatype! {QVariantList  => 9}
qdeclare_builtin_metatype! {QString => 10}
qdeclare_builtin_metatype! {QByteArray => 12}
qdeclare_builtin_metatype! {QDate => 14}
qdeclare_builtin_metatype! {QTime => 15}
qdeclare_builtin_metatype! {QDateTime => 16}
qdeclare_builtin_metatype! {QUrl => 17}
qdeclare_builtin_metatype! {QRectF => 20}
qdeclare_builtin_metatype! {QSize => 21}
qdeclare_builtin_metatype! {QSizeF => 22}
qdeclare_builtin_metatype! {QPoint => 25}
qdeclare_builtin_metatype! {QPointF => 26}
impl QMetaType for QVariant {
    fn register(_name: Option<&CStr>) -> i32 {
        41
    }
    fn to_qvariant(&self) -> QVariant {
        self.clone()
    }
    fn from_qvariant(variant: QVariant) -> Option<Self> {
        Some(variant)
    }
}
qdeclare_builtin_metatype! {QModelIndex => 42}
qdeclare_builtin_metatype! {QJsonValue => 45}
qdeclare_builtin_metatype! {QJsonObject => 46}
qdeclare_builtin_metatype! {QJsonArray => 47}
qdeclare_builtin_metatype! {QPixmap => if cfg!(qt_6_0) { 0x1001 } else { 65 }}
qdeclare_builtin_metatype! {QColor => if cfg!(qt_6_0) { 0x1003 } else { 67 }}
qdeclare_builtin_metatype! {QImage => if cfg!(qt_6_0) { 0x1006 } else { 70 }}
qdeclare_builtin_metatype! {QStringList => 11}

#[cfg(target_pointer_width = "32")]
qdeclare_builtin_metatype! {isize  => 2} // That's QMetaType::Int
#[cfg(target_pointer_width = "32")]
qdeclare_builtin_metatype! {usize  => 3} // That's QMetaType::UInt
#[cfg(target_pointer_width = "64")]
qdeclare_builtin_metatype! {isize  => 4} // That's QMetaType::LongLong
#[cfg(target_pointer_width = "64")]
qdeclare_builtin_metatype! {usize  => 5} // That's QMetaType::ULongLong

/// Internal trait used to pass or read the type in a Q_PROPERTY
///
/// Don't implement this trait, implement the QMetaType trait.
pub trait PropertyType {
    fn register_type(name: &CStr) -> i32;
    // Note: this is &mut self because of the lazy initialization of the QObject* for the QObject impl
    unsafe fn pass_to_qt(&mut self, a: *mut c_void);
    unsafe fn read_from_qt(a: *const c_void) -> Self;
}

impl<T: QMetaType> PropertyType for T
where
    T: QMetaType,
{
    fn register_type(name: &CStr) -> i32 {
        <T as QMetaType>::register(Some(name))
    }

    unsafe fn pass_to_qt(&mut self, a: *mut c_void) {
        let r = a as *mut Self;
        if !r.is_null() {
            *r = self.clone();
        }
    }

    unsafe fn read_from_qt(a: *const c_void) -> Self {
        let r = a as *const Self;
        (*r).clone()
    }
}

impl<T> PropertyType for RefCell<T>
where
    T: QObject,
{
    fn register_type(_name: &CStr) -> i32 {
        register_metatype_qobject::<T>()
    }

    unsafe fn pass_to_qt(&mut self, a: *mut c_void) {
        let pinned = QObjectPinned::new(self);
        let r = a as *mut *const c_void;
        *r = pinned.get_or_create_cpp_object()
    }

    unsafe fn read_from_qt(_a: *const c_void) -> Self {
        panic!("Cannot write into an Object property");
    }
}

impl<T> PropertyType for QPointer<T>
where
    T: QObject,
{
    fn register_type(_name: &CStr) -> i32 {
        register_metatype_qobject::<T>()
    }

    unsafe fn pass_to_qt(&mut self, a: *mut c_void) {
        let pinned = self.as_pinned();
        let r = a as *mut *const c_void;
        match pinned {
            Some(pinned) => *r = pinned.get_or_create_cpp_object(),
            None => *r = std::ptr::null(),
        }
    }

    unsafe fn read_from_qt(a: *const c_void) -> Self {
        let r = a as *const *mut c_void;
        if a.is_null() || (*r).is_null() {
            Self::default()
        } else {
            let obj = T::get_from_cpp(*r);
            obj.borrow().into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qmetatype() {
        #[derive(Default, Clone, Debug, Eq, PartialEq)]
        struct MyInt {
            x: u32,
        }
        impl QMetaType for MyInt {}

        assert_eq!(MyInt::register(Some(&CString::new("MyInt").unwrap())), MyInt::id());
        let m42 = MyInt { x: 42 };
        let m43 = MyInt { x: 43 };

        assert_eq!(Some(m42.clone()), MyInt::from_qvariant(m42.clone().to_qvariant()));
        assert_eq!(Some(m43.clone()), MyInt::from_qvariant(m43.clone().to_qvariant()));

        assert_eq!(None, u32::from_qvariant(m43.to_qvariant()));
        assert_eq!(None, MyInt::from_qvariant(45u32.to_qvariant()));
        assert_eq!(Some(45), u32::from_qvariant(45u32.to_qvariant()));
    }

    #[test]
    #[should_panic(expected = "Attempt to register the same type with different name")]
    fn test_qmetatype_register_wrong_type1() {
        #[derive(Default, Clone, Debug, Eq, PartialEq)]
        struct MyType {}
        impl QMetaType for MyType {}
        // registering with the name of an existing type should panic
        MyType::register(Some(&CString::new("QString").unwrap()));
    }

    #[test]
    #[should_panic(expected = "Attempt to register the same type with different name")]
    fn test_qmetatype_register_wrong_type2() {
        #[derive(Default, Clone, Debug, Eq, PartialEq)]
        struct MyType {}
        impl QMetaType for MyType {}
        String::register(Some(&CString::new("String").unwrap()));
        // registering with the name of an existing type should panic
        MyType::register(Some(&CString::new("String").unwrap()));
    }

    #[test]
    fn test_qvariant_datetime() {
        let dt = QDateTime::from_date_time_local_timezone(
            QDate::from_y_m_d(2019, 10, 23),
            QTime::from_h_m_s_ms(10, 30, Some(40), Some(100)),
        );
        let v = QVariant::from(dt);
        let qstring = QString::from_qvariant(v.clone()).unwrap();
        let mut s = qstring.to_string();
        if s.ends_with(".100") {
            // Old version of qt did not include the milliseconds, so remove it
            s.truncate(s.len() - 4);
        }
        assert_eq!(s, "2019-10-23T10:30:40");
        let qdate = QDate::from_qvariant(v.clone()).unwrap();
        assert!(qdate == QDate::from_y_m_d(2019, 10, 23));
        assert!(qdate != QDate::from_y_m_d(2019, 10, 24));

        let qtime = QTime::from_qvariant(v.clone()).unwrap();
        assert!(qtime == QTime::from_h_m_s_ms(10, 30, Some(40), Some(100)));
        assert!(qtime != QTime::from_h_m_s_ms(10, 30, Some(40), None));
    }

    #[test]
    fn test_qvariant_qpoint_qrect() {
        // test that conversion through a variant lead the the right data
        assert_eq!(
            QPoint::from_qvariant(QPointF { x: 23.1, y: 54.2 }.to_qvariant()),
            Some(QPoint { x: 23, y: 54 })
        );
        let qrectf = QRectF { x: 4.1, y: 9.1, height: 7.3, width: 9.0 };
        assert_eq!(QRectF::from_qvariant(qrectf.to_qvariant()), Some(qrectf));
        assert_eq!(
            QSize::from_qvariant(QSizeF { width: 123.1, height: 254.2 }.to_qvariant()),
            Some(QSize { width: 123, height: 254 })
        );
    }

    #[test]
    fn test_qvariant_qstringlist() {
        let list = QStringList::from(["abc", "def"]);
        let t = QVariant::from(list.clone());

        assert!(t == list.to_qvariant());
        assert!(QStringList::from_qvariant(t).unwrap() == list);
    }
}
