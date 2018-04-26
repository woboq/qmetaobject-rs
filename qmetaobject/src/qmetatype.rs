use super::*;

fn register_metatype_common<T : Sized + Clone + Default>(
    name : *const std::os::raw::c_char, gadget_metaobject : *const QMetaObject) -> i32 {
    let size = std::mem::size_of::<T>() as u32;

    extern fn deleter_fn<T>(_v: Box<T>) { };
    let deleter_fn : extern fn(_v: Box<T>) = deleter_fn;

    extern fn creator_fn<T : Default + Clone>(c : *const T) -> Box<T> {
        if c.is_null() { Box::new( Default::default() ) }
        else { Box::new(unsafe { (*c).clone() }) }
    };
    let creator_fn : extern fn(c : *const T) -> Box<T> = creator_fn;

    extern fn destructor_fn<T>(ptr : *mut T) { unsafe { std::ptr::read(ptr); } };
    let destructor_fn : extern fn(ptr : *mut T) = destructor_fn;

    extern fn constructor_fn<T : Default + Clone>(dst : *mut T, c : *const T) -> *mut T {
        unsafe {
            let n = if c.is_null() {  Default::default() }
                    else { (*c).clone() };
            std::ptr::write(dst, n);
        }
        dst
    };
    let constructor_fn : extern fn(ptr : *mut T, c : *const T) -> *mut T = constructor_fn;

    unsafe {
        cpp!([name as "const char*", size as "int", deleter_fn as "QMetaType::Deleter",
              creator_fn as "QMetaType::Creator", destructor_fn as "QMetaType::Destructor",
              constructor_fn as "QMetaType::Constructor", gadget_metaobject as "const QMetaObject*"] -> i32 as "int" {
            QMetaType::TypeFlags extraFlag(gadget_metaobject ? QMetaType::IsGadget : 0);
            return QMetaType::registerType(name ? name : gadget_metaobject->className(), deleter_fn, creator_fn, destructor_fn,
                constructor_fn, size,
                QMetaType::NeedsConstruction | QMetaType::NeedsDestruction | QMetaType::MovableType | extraFlag,
                gadget_metaobject);
        })
    }
}

pub trait QMetaType : Clone + Default {
    //const NAME : &'static str;
    fn register(name : &str) -> i32 {
        //if name.is_empty() { Self::NAME } else { name }
        let name = std::ffi::CString::new(name).unwrap();
        register_metatype_common::<Self>(name.as_ptr(), std::ptr::null())
    }
}

impl<T : QGadget> QMetaType for T where T: Clone + Default {
    fn register(_name : &str) -> i32 {
        //assert!(_name == T::static_meta_object().className());
        register_metatype_common::<T>(std::ptr::null(), T::static_meta_object())
    }
}

impl QMetaType for String {
    fn register(name : &str) -> i32 {
        assert!(name == "String");
        let c_name = b"String\0".as_ptr() as *const std::os::raw::c_char;
        let type_id = register_metatype_common::<String>(c_name, std::ptr::null());
        extern fn converter_fn1(_ : *const c_void, s: &String, ptr : *mut QByteArray) {
            unsafe { std::ptr::write(ptr, QByteArray::from(&*s as &str)); }
        };
        let converter_fn1: extern fn(_ : *const c_void, s: &String, ptr : *mut QByteArray) = converter_fn1;
        extern fn converter_fn2(_ : *const c_void, s: &QByteArray, ptr : *mut String) {
            unsafe { std::ptr::write(ptr, s.to_string()); }
        };
        let converter_fn2: extern fn(_ : *const c_void, s: &QByteArray, ptr : *mut String) = converter_fn2;
        extern fn converter_fn3(_ : *const c_void, s: &String, ptr : *mut QString) {
            let s : &str = &*s;
            unsafe { std::ptr::write(ptr, QString::from(&*s as &str)); }
        };
        let converter_fn3: extern fn(_ : *const c_void, s: &String, ptr : *mut QString) = converter_fn3;
        extern fn converter_fn4(_ : *const c_void, s: &QString, ptr : *mut String) {
            unsafe { std::ptr::write(ptr, s.to_string()); }
        };
        let converter_fn4: extern fn(_ : *const c_void, s: &QString, ptr : *mut String) = converter_fn4;


        unsafe { cpp!([type_id as "int",
                    converter_fn1 as "QtPrivate::AbstractConverterFunction::Converter",
                    converter_fn2 as "QtPrivate::AbstractConverterFunction::Converter",
                    converter_fn3 as "QtPrivate::AbstractConverterFunction::Converter",
                    converter_fn4 as "QtPrivate::AbstractConverterFunction::Converter"] {
            //FIXME, the ConverterFunctor are gonna be leaking
            auto c = new QtPrivate::ConverterFunctor<TraitObject, TraitObject, TraitObject>(converter_fn1);
            if (!c->registerConverter(type_id, QMetaType::QByteArray))
                delete c;
            c = new QtPrivate::ConverterFunctor<TraitObject, TraitObject, TraitObject>(converter_fn2);
            if (!c->registerConverter(QMetaType::QByteArray, type_id))
                delete c;
            c = new QtPrivate::ConverterFunctor<TraitObject, TraitObject, TraitObject>(converter_fn3);
            if (!c->registerConverter(type_id, QMetaType::QString))
                delete c;
            c = new QtPrivate::ConverterFunctor<TraitObject, TraitObject, TraitObject>(converter_fn4);
            if (!c->registerConverter(QMetaType::QString, type_id))
                delete c;
        }) };
        type_id
    }
}

