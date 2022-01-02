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
use std::cell::RefCell;
use std::ffi::CStr;
use std::rc::Rc;

use if_rust_version::if_rust_version;
use qmetaobject::*;

mod common;
use self::common::*;

#[test]
fn self_test() {
    #[derive(QObject, Default)]
    struct Basic {
        base: qt_base_class!(trait QObject),
        value: qt_property!(bool),
    }

    let mut obj = Basic::default();
    obj.value = true;
    assert!(do_test(obj, "Item { function doTest() { return _obj.value  } }"));

    let mut obj = Basic::default();
    obj.value = false;
    assert!(!do_test(obj, "Item { function doTest() { return _obj.value  } }"));
}

#[test]
fn self_test_variant() {
    let obj = QVariant::from(true);
    assert!(do_test_variant(obj, "Item { function doTest() { return _obj  } }"));

    let obj = QVariant::from(false);
    assert!(!do_test_variant(obj, "Item { function doTest() { return _obj  } }"));
}

#[derive(QObject, Default)]
struct MyObject {
    base: qt_base_class!(trait QObject),
    prop_x: qt_property!(u32; NOTIFY prop_x_changed),
    prop_x_changed: qt_signal!(),
    prop_y: qt_property!(String; NOTIFY prop_y_changed),
    prop_y_changed: qt_signal!(),
    prop_z: qt_property!(QString; NOTIFY prop_z_changed),
    prop_z_changed: qt_signal!(v: QString),

    multiply_and_add1: qt_method!(
        fn multiply_and_add1(&self, a: u32, b: u32) -> u32 {
            a * b + 1
        }
    ),

    concatenate_strings: qt_method!(
        fn concatenate_strings(&self, a: QString, b: QString, c: QByteArray) -> QString {
            let res = a.to_string() + &(b.to_string()) + &(c.to_string());
            QString::from(&res as &str)
        }
    ),

    method_out_of_line: qt_method!(fn(&self, a: QString) -> QString),

    prop_color: qt_property!(QColor),
    prop_jsvalue: qt_property!(QJSValue),
}

impl MyObject {
    fn method_out_of_line(&self, a: QString) -> QString {
        (self.prop_y.clone() + &a.to_string()).into()
    }
}

#[test]
fn property_read_write_notify() {
    let obj = MyObject::default();
    assert!(do_test(
        obj,
        "Item {
        property int yo: _obj.prop_x;
        function doTest() {
            _obj.prop_x = 123;
            return yo === 123;
        }}"
    ));

    let obj = MyObject::default();
    assert!(do_test(
        obj,
        "Item {
        property string yo: _obj.prop_y + ' ' + _obj.prop_z;
        function doTest() {
            _obj.prop_y = 'hello';
            _obj.prop_z = 'world';
            return yo === 'hello world';
        }}"
    ));
}

#[test]
fn call_method() {
    let obj = MyObject::default();
    assert!(do_test(
        obj,
        "Item {
        function doTest() {
            return _obj.multiply_and_add1(45, 76) === 45*76+1;
        }}"
    ));

    let obj = MyObject::default();
    assert!(do_test(
        obj,
        r"
        Item {
            function doTest() {
                return _obj.concatenate_strings('abc', 'def', 'hij') == 'abcdefhij';
            }
        }
        "
    ));

    let obj = MyObject::default();
    assert!(do_test(
        obj,
        r"
        Item {
            function doTest() {
                return _obj.concatenate_strings(123, 456, 789) == '123456789';
            }
        }
        "
    ));

    let obj = MyObject::default();
    assert!(do_test(
        obj,
        r"
        Item {
            function doTest() {
                _obj.prop_y = '8887';
                return _obj.method_out_of_line('hello') == '8887hello';
            }
        }
        "
    ));
}

#[derive(Default, QObject)]
struct RegisteredObj {
    base: qt_base_class!(trait QObject),
    value: qt_property!(u32),
    square: qt_method!(
        fn square(&self, v: u32) -> u32 {
            self.value * v
        }
    ),
}

#[test]
fn register_type() {
    qml_register_type::<RegisteredObj>(
        CStr::from_bytes_with_nul(b"TestRegister\0").unwrap(),
        1,
        0,
        CStr::from_bytes_with_nul(b"RegisteredObj\0").unwrap(),
    );

    let obj = MyObject::default(); // not used but needed for do_test
    assert!(do_test(
        obj,
        r"
        import TestRegister 1.0

        Item {
            RegisteredObj {
                id: test
                value: 55
            }
            function doTest() {
                return test.square(66) === 55 * 66;
            }
        }
        "
    ));
}

#[test]
#[cfg(qt_5_9)]
fn register_module() {
    qml_register_module(CStr::from_bytes_with_nul(b"TestEmptyModule\0").unwrap(), 1, 2);

    let obj = MyObject::default(); // not used but needed for do_test
    assert!(do_test(
        obj,
        r"
        import TestEmptyModule 1.2 as TM

        QtObject {
            function doTest() {
                return typeof TM === 'object';
            }
        }
        "
    ));
}

#[derive(Default, QObject)]
struct RegisterSingletonInstanceObj {
    base: qt_base_class!(trait QObject),
    value: u32,
    get_value: qt_method!(
        fn get_value(&self) -> u32 {
            self.value
        }
    ),
}

#[test]
#[cfg(qt_5_14)]
fn register_singleton_instance() {
    let mut myobj = RegisterSingletonInstanceObj::default();
    myobj.value = 123;
    qml_register_singleton_instance(
        CStr::from_bytes_with_nul(b"TestRegister\0").unwrap(),
        1,
        0,
        CStr::from_bytes_with_nul(b"RegisterSingletonInstanceObj\0").unwrap(),
        myobj,
    );

    let obj = MyObject::default(); // not used but needed for do_test
    assert!(do_test(
        obj,
        r"
        import TestRegister 1.0;

        Item {
            function doTest() {
                return RegisterSingletonInstanceObj.get_value() === 123;
            }
        }
        "
    ));
}

#[derive(QObject, Default)]
struct RegisterSingletonTypeObj {
    base: qt_base_class!(trait QObject),
    value: u32,
    get_value2: qt_method!(
        fn get_value2(&self) -> u32 {
            self.value
        }
    ),
}

impl QSingletonInit for RegisterSingletonTypeObj {
    fn init(&mut self) {
        self.value = 456;
    }
}

#[test]
fn register_singleton_type() {
    qml_register_singleton_type::<RegisterSingletonTypeObj>(
        CStr::from_bytes_with_nul(b"TestRegister\0").unwrap(),
        1,
        0,
        CStr::from_bytes_with_nul(b"RegisterSingletonTypeObj\0").unwrap(),
    );

    let obj = MyObject::default(); // not used but needed for do_test
    assert!(do_test(
        obj,
        r"
        import TestRegister 1.0;

        Item {
            function doTest() {
                return RegisterSingletonTypeObj.get_value2() === 456;
            }
        }
        "
    ));
}

#[test]
fn simple_gadget() {
    #[derive(Default, Clone, QGadget)]
    struct MySimpleGadget {
        num_value: qt_property!(u32; ALIAS numValue),
        str_value: qt_property!(String; ALIAS strValue),
        concat: qt_method!(
            fn concat(&self, separator: String) -> String {
                return format!("{}{}{}", self.str_value, separator, self.num_value);
            }
        ),
    }

    let mut my_gadget = MySimpleGadget::default();
    my_gadget.num_value = 33;
    my_gadget.str_value = "plop".into();

    assert!(do_test_variant(
        my_gadget.to_qvariant(),
        r"
        Item {
            function doTest() {
                return _obj.strValue == 'plop'
                    && _obj.numValue == 33
                    && _obj.concat(':') == 'plop:33';
            }
        }
        "
    ));
}

#[derive(QObject, Default)]
struct ObjectWithObject {
    base: qt_base_class!(trait QObject),
    prop_object: qt_property!(RefCell<MyObject>; CONST),

    subx: qt_method!(
        fn subx(&self) -> u32 {
            self.prop_object.borrow().prop_x
        }
    ),
}

#[test]
fn qobject_properties() {
    let my_obj = ObjectWithObject::default();
    my_obj.prop_object.borrow_mut().prop_x = 56;
    assert!(do_test(
        my_obj,
        r"
        Item {
            property int yo: _obj.prop_object.prop_x;
            function doTest() {
                if (yo !== 56) {
                    console.log('ERROR #1: 56 != ' +  yo);
                    return false;
                }
                _obj.prop_object.prop_x = 4545;
                if (yo !== 4545) {
                    console.log('ERROR #2: 4545 != ' +  yo);
                    return false;
                }
                return _obj.subx() === 4545;
            }
        }
        "
    ));
}

#[derive(QObject, Default)]
struct SomeObject {
    base: qt_base_class!(trait QObject),
}

#[derive(QObject, Default)]
struct ObjectWithSomeObjectPointer {
    base: qt_base_class!(trait QObject),
    prop_changed: qt_signal!(),
    prop: qt_property!(QPointer<SomeObject>; NOTIFY prop_changed),
}

#[test]
fn qpointer_properties() {
    let my_obj = ObjectWithSomeObjectPointer::default();
    qml_register_type::<SomeObject>(
        CStr::from_bytes_with_nul(b"SomeObjectLib\0").unwrap(),
        1,
        0,
        CStr::from_bytes_with_nul(b"SomeObject\0").unwrap(),
    );
    assert!(do_test(
        my_obj,
        "
        import SomeObjectLib 1.0

        Item {
            SomeObject { id: some }
            function doTest() {
                if(_obj.prop !== null) {
                    return false;
                }
                _obj.prop = some;
                if(_obj.prop !== some) {
                    return false;
                }
                _obj.prop = null;
                if(_obj.prop !== null) {
                    return false;
                }
                return true;
            }
        }
        "
    ));
}

#[test]
fn qpointer_properties_incompatible() {
    qml_register_type::<ObjectWithSomeObjectPointer>(
        CStr::from_bytes_with_nul(b"SomeObjectLib\0").unwrap(),
        1,
        0,
        CStr::from_bytes_with_nul(b"ObjectWithSome\0").unwrap(),
    );
    assert!(test_loading_logs(
        "
        import SomeObjectLib 1.0

        Item {
            Text { id: some }
            ObjectWithSome { prop: some }
        }
        ",
        "Unable to assign QQuickText to SomeObject"
    ));
}

#[test]
fn test_single_shot() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let engine = Rc::new(QmlEngine::new());
    let engine_copy = engine.clone();
    single_shot(std::time::Duration::from_millis(0), move || {
        engine_copy.quit();
    });
    engine.exec();
}

#[test]
fn test_queued_callback() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let engine = Rc::new(QmlEngine::new());
    let engine_copy = engine.clone();
    let callback = queued_callback(move |()| engine_copy.quit());
    std::thread::spawn(move || {
        callback(());
    })
    .join()
    .unwrap();
    engine.exec();
}

#[test]
fn getter() {
    #[derive(QObject, Default)]
    struct ObjectWithGetter {
        base: qt_base_class!(trait QObject),
        prop_x: qt_property!(u32; READ prop_x_getter CONST),
        prop_y: qt_property!(String; READ prop_y_getter CONST),
    }
    impl ObjectWithGetter {
        fn prop_x_getter(&self) -> u32 {
            return 85;
        }

        fn prop_y_getter(&self) -> String {
            return "foo".into();
        }
    }

    let my_obj = ObjectWithGetter::default();
    assert!(do_test(
        my_obj,
        "
        Item {
            function doTest() {
                return _obj.prop_x === 85
                    && _obj.prop_y == 'foo';
            }
        }
        "
    ));
}

#[test]
fn setter() {
    #[derive(QObject, Default)]
    struct ObjectWithGetter {
        base: qt_base_class!(trait QObject),
        prop_x: qt_property!(u32; WRITE prop_x_setter NOTIFY prop_x_notify),
        prop_x_notify: qt_signal!(),
        prop_y: qt_property!(String; NOTIFY prop_y_notify WRITE prop_y_setter),
        prop_y_notify: qt_signal!(),

        prop_x_setter: qt_method!(fn(&mut self, v: u32) -> ()),
        prop_y_setter: qt_method!(fn(&mut self, v: String)),
    }
    impl ObjectWithGetter {
        fn prop_x_setter(&mut self, v: u32) {
            self.prop_x = v;
            self.prop_x_notify();
        }

        fn prop_y_setter(&mut self, v: String) {
            self.prop_y = v;
            self.prop_y_notify();
        }
    }

    let my_obj = ObjectWithGetter::default();
    assert!(do_test(
        my_obj,
        "
        Item {
            property var test: '' + _obj.prop_x + _obj.prop_y;
            function doTest() {
                if (test !== '0') {
                    console.log('FAILURE #1', test);
                    return false;
                }
                _obj.prop_x = 96;
                if (test !== '96') {
                    console.log('FAILURE #2', test);
                    return false;
                }
                _obj.prop_y = 'hello';
                if (test !== '96hello') {
                    console.log('FAILURE #3', test);
                    return false;
                }
                _obj.prop_x_setter(88);
                _obj.prop_y_setter('world');
                if (test !== '88world') {
                    console.log('FAILURE #4', test);
                    return false;
                }
                return true;
            }
        }
        "
    ));
}

#[test]
fn connect_rust_signal() {
    #[derive(QObject, Default)]
    struct Foo {
        base: qt_base_class!(trait QObject),
        my_signal: qt_signal!(xx: u32, yy: String),
        my_signal2: qt_signal!(yy: String),
    }

    let f = RefCell::new(Foo::default());
    let obj_ptr = unsafe { QObjectPinned::new(&f).get_or_create_cpp_object() };
    let mut result = None;
    let mut result2 = None;
    let mut con = unsafe {
        connect(
            obj_ptr,
            f.borrow().my_signal.to_cpp_representation(&*f.borrow()),
            |xx: &u32, yy: &String| {
                result = Some(format!("{} -> {}", xx, yy));
            },
        )
    };
    assert!(con.is_valid());

    let con2 = unsafe {
        connect(
            obj_ptr,
            f.borrow().my_signal2.to_cpp_representation(&*f.borrow()),
            |yy: &String| {
                result2 = Some(yy.clone());
            },
        )
    };
    assert!(con2.is_valid());

    f.borrow().my_signal(12, "goo".into());
    assert_eq!(result, Some("12 -> goo".to_string()));
    f.borrow().my_signal(18, "moo".into());
    assert_eq!(result, Some("18 -> moo".to_string()));
    con.disconnect();
    f.borrow().my_signal(25, "foo".into());
    assert_eq!(result, Some("18 -> moo".to_string())); // still the same as before as we disconnected

    assert_eq!(result2, None);
    f.borrow().my_signal2("hop".into());
    assert_eq!(result2, Some("hop".into()));
    assert_eq!(result, Some("18 -> moo".to_string())); // still the same as before as we disconnected
}

#[test]
fn connect_cpp_signal() {
    #[derive(QObject, Default)]
    struct Foo {
        base: qt_base_class!(trait QObject),
    }

    let f = RefCell::new(Foo::default());
    let obj_ptr = unsafe { QObjectPinned::new(&f).get_or_create_cpp_object() };
    let mut result = None;
    let con = unsafe {
        connect(obj_ptr, <dyn QObject>::object_name_changed_signal(), |name: &QString| {
            result = Some(name.clone());
        })
    };
    assert!(con.is_valid());
    (&*f.borrow() as &dyn QObject).set_object_name("YOYO".into());
    assert_eq!(result, Some("YOYO".into()));
}

#[test]
fn with_life_time() {
    #[derive(QObject, Default)]
    struct WithLT<'a> {
        base: qt_base_class!(trait QObject),
        _something: Option<&'a u32>,
        my_signal: qt_signal!(xx: u32, yy: String),
        my_method: qt_method!(
            fn my_method(&self, _x: u32) {}
        ),
        my_property: qt_property!(u32),
    }

    #[derive(QObject, Default)]
    struct WithWhereClose<T>
    where
        T: Clone + 'static,
    {
        #[qt_base_class = "QObject"] // FIXME
        base: QObjectCppWrapper,
        _something: Option<T>,
    }
}

#[test]
fn qpointer() {
    let ptr;
    let pt2;
    {
        let obj = RefCell::new(MyObject::default());
        obj.borrow_mut().prop_x = 23;
        unsafe { QObjectPinned::new(&obj).get_or_create_cpp_object() };
        ptr = QPointer::from(&*obj.borrow());
        pt2 = ptr.clone();
        assert_eq!(ptr.as_ref().map_or(898, |x| x.prop_x), 23);
        assert_eq!(pt2.as_ref().map_or(898, |x| x.prop_x), 23);
        assert_eq!(
            ptr.as_pinned().map_or(989, |x| {
                let old = x.borrow().prop_x;
                x.borrow_mut().prop_x = 42;
                old
            }),
            23
        );
        assert_eq!(pt2.as_ref().map_or(898, |x| x.prop_x), 42);
    }
    assert!(ptr.as_ref().is_none());
    assert!(pt2.as_ref().is_none());

    let ptr;
    let pt2;
    {
        #[derive(Default)]
        struct XX(QString);
        impl SimpleListItem for XX {
            fn get(&self, _idx: i32) -> QVariant {
                self.0.clone().into()
            }
            fn names() -> Vec<QByteArray> {
                vec![QByteArray::from("a")]
            }
        }
        let mut obj = SimpleListModel::<XX>::default();
        obj.push(XX("foo".into()));
        let obj = RefCell::new(obj);
        unsafe { QObjectPinned::new(&obj).get_or_create_cpp_object() };
        let obj_ref: &dyn QAbstractListModel = &*obj.borrow();
        ptr = QPointer::<dyn QAbstractListModel>::from(obj_ref);
        pt2 = ptr.clone();
        assert_eq!(ptr.as_ref().map_or(898, |x| x.row_count()), 1);
        assert_eq!(pt2.as_ref().map_or(898, |x| x.row_count()), 1);
    }
    assert!(ptr.as_ref().is_none());
    assert!(pt2.as_ref().is_none());
}

/* Panic test are a bad idea as the exception has to cross the C++ boundaries, and Qt is not exception safe
#[derive(QObject, Default)]
struct StupidObject {
    base: qt_base_class!(trait QObject),
    prop_x: qt_property!(u32; READ prop_x_getter CONST),
    prop_y: qt_property!(u32; WRITE prop_y_setter),
    method: qt_method!(fn method(&mut self) { *self = StupidObject::default(); }),
}
impl StupidObject {
    fn prop_x_getter(&mut self) -> u32 {
        *self = StupidObject::default();
        0
    }
    fn prop_y_setter(&mut self, _: u32) {
        *self = StupidObject::default();
    }
}

#[test]
#[should_panic(expected = "Internal pointer changed")]
fn panic_when_moved_method() {
    let my_obj = StupidObject::default();
    do_test(my_obj, "Item { x: _obj.method(); }");
}
#[test]
#[should_panic(expected = "Internal pointer changed")]
fn panic_when_moved_getter() {
    let my_obj = StupidObject::default();
    do_test(my_obj, "Item { x: _obj.prop_x; }");
}
#[test]
#[should_panic(expected = "Internal pointer changed")]
fn panic_when_moved_setter() {
    let my_obj = StupidObject::default();
    do_test(my_obj, "Item { function doTest() { _obj.prop_y = 45; } }");
}
*/

#[test]
#[should_panic(expected = "There can only be one QmlEngine in the process")]
fn two_engines() {
    let _lock = lock_for_test();
    let _a = QmlEngine::new();
    let _b = QmlEngine::new();
}

#[derive(QEnum)]
#[repr(u8)]
enum MyEnum {
    None,
    First = 1,
    Four = 4,
}

#[derive(QObject, Default)]
struct MyEnumObject {
    base: qt_base_class!(trait QObject),
}

#[test]
fn enum_properties() {
    qml_register_enum::<MyEnum>(
        CStr::from_bytes_with_nul(b"MyEnumLib\0").unwrap(),
        1,
        0,
        CStr::from_bytes_with_nul(b"MyEnum\0").unwrap(),
    );
    let my_obj = MyObject::default();
    assert!(do_test(
        my_obj,
        "
        import MyEnumLib 1.0

        Item {
            function doTest() {
                if(MyEnum.None !== 0) {
                    return false;
                }
                if(MyEnum.First !== 1) {
                    return false;
                }
                if(MyEnum.Four !== 4) {
                    return false;
                }
                return true;
            }
        }
        "
    ));
}

#[test]
fn threading() {
    let _lock = lock_for_test();

    #[derive(QObject, Default)]
    struct MyAsyncObject {
        base: qt_base_class!(trait QObject),
        result: qt_property!(QString; NOTIFY result_changed),
        result_changed: qt_signal!(),
        recompute_result: qt_method!(
            fn recompute_result(&self, name: String) {
                let qptr = QPointer::from(&*self);
                let set_value = queued_callback(move |val: QString| {
                    qptr.as_pinned().map(|self_| {
                        self_.borrow_mut().result = val;
                        self_.borrow().result_changed();
                    });
                });
                std::thread::spawn(move || {
                    // do stuff asynchronously ...
                    let r = QString::from("Hello ".to_owned() + &name);
                    set_value(r);
                })
                .join()
                .unwrap();
            }
        ),
    }

    let obj = std::cell::RefCell::new(MyAsyncObject::default());
    let engine = QmlEngine::new();
    unsafe {
        connect(
            QObject::cpp_construct(&obj),
            obj.borrow().result_changed.to_cpp_representation(&*obj.borrow()),
            || engine.quit(),
        )
    };
    obj.borrow().recompute_result("World".into());
    engine.exec();
    assert_eq!(obj.borrow().result, QString::from("Hello World"));
}

#[test]
fn load_data_as() {
    #[derive(QObject, Default)]
    struct Basic {
        base: qt_base_class!(trait QObject),
        value: qt_property!(bool),
    }

    let mut obj = Basic::default();
    obj.value = true;
    let error = do_test_error_with_url(
        obj,
        "Item { function doTest() { return _intentional_error } }",
        "file:///path/file.ext",
    );
    assert!(error.contains("file:///path/file.ext"));
}

#[test]
fn test_future() {
    if_rust_version!(>= 1.39 {
        let _lock = lock_for_test();

        #[derive(QObject, Default)]
        struct ObjectWithSignal {
            base: qt_base_class!(trait QObject),
            sig: qt_signal!(),
            sig_with_args: qt_signal!(xx: u32, yy: String),
        }
        let o = RefCell::new(ObjectWithSignal::default());
        let obj_ptr = unsafe { QObjectPinned::new(&o).get_or_create_cpp_object() };


        let result = Rc::new(RefCell::new(None));
        {
            let result2 = result.clone();
            let fut = unsafe {
                future::wait_on_signal(
                    obj_ptr,
                    o.borrow().sig_with_args.to_cpp_representation(&*o.borrow()),
                )
            };
            future::execute_async(async move {
                let (xx, yy) = fut.await;
                *result2.borrow_mut() = Some(format!("{}={}", yy, xx));
            });
        }


        let engine = Rc::new(QmlEngine::new());
        {
            let fut = unsafe {
                future::wait_on_signal(
                    obj_ptr,
                    o.borrow().sig.to_cpp_representation(&*o.borrow()),
                )
            };
            let engine2 = engine.clone();
            future::execute_async(async move {
                fut.await;
                engine2.quit();
            });
        }

        o.borrow().sig_with_args(88, "Yop".to_owned());
        o.borrow().sig();
        engine.exec();

        assert_eq!(result.borrow().as_ref().unwrap(), "Yop=88");
    });
}

#[test]
fn create_component() {
    let _lock = lock_for_test();
    let qml_text = "
        import QtQuick 2.0

        Item {}
    ";

    let engine = QmlEngine::new();
    let mut component = QmlComponent::new(&engine);

    component.set_data(qml_text.into());

    let obj = component.create();

    assert!(!obj.is_null());
}

#[test]
fn component_status_changed() {
    if_rust_version!(>= 1.39 {
        let _lock = lock_for_test();
        let engine = Rc::new(QmlEngine::new());
        let o = Rc::new(RefCell::new(QmlComponent::new(&engine)));
        let obj_ptr = o.borrow_mut().get_cpp_object();
        let result = Rc::new(RefCell::new(None));

        assert!(!obj_ptr.is_null());

        {
            let result2 = result.clone();
            let engine2 = engine.clone();
            let fut = unsafe {
                future::wait_on_signal(
                    obj_ptr,
                    QmlComponent::status_changed_signal()
                )
            };

            future::execute_async(async move {
                let status = fut.await;

                *result2.borrow_mut() = Some(status);
                engine2.quit();
            });
        }

        single_shot(std::time::Duration::new(0, 0), move || {
            use std::io::Write;
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
            writeln!(tmpfile, "INVALID DATA").unwrap();

            let mut url = "file://".to_string();

            url.push_str(tmpfile.path().to_str().unwrap());

            let qstring_url: QString = url.into();

            assert_eq!(o.borrow().status(), ComponentStatus::Null);
            o.borrow_mut().load_url(qstring_url.into(), CompilationMode::Asynchronous);
        });

        engine.exec();

        assert_eq!(result.borrow().as_ref().unwrap().0, ComponentStatus::Loading);
    });
}

#[test]
fn test_qvariant_qimage_qpixmap() {
    let _lock = lock_for_test();
    // QPixmap need a QApplication
    let _app = QmlEngine::new();

    let mut img = QImage::new(QSize { width: 12, height: 23 }, ImageFormat::ARGB32);
    img.fill(QColor::from_name("red"));
    img.set_pixel_color(2, 2, QColor::from_name("blue"));
    let pix: QPixmap = img.clone().into();
    assert_eq!(pix.size(), QSize { width: 12, height: 23 });
    let img2 = QImage::from_qvariant(pix.clone().to_qvariant()).unwrap();
    assert_eq!(img2.size(), QSize { width: 12, height: 23 });
    assert!(img2.get_pixel_color(2, 2) == QColor::from_rgb_f(0., 0., 1.));
    assert!(img2.get_pixel_color(3, 4) == QColor::from_rgb_f(1., 0., 0.));

    let mut img3: QImage = pix.into();
    assert!(img2 == img3);
    assert_eq!(img3.size(), QSize { width: 12, height: 23 });
    assert!(img3.get_pixel_color(2, 2) == QColor::from_rgb_f(0., 0., 1.));
    assert!(img3.get_pixel_color(8, 4) == QColor::from_rgb_f(1., 0., 0.));
    img3.set_pixel_color(8, 8, QColor::from_name("black"));
    assert!(img2 != img3);
}

#[test]
fn test_application_name() {
    use qmetaobject::qtcore::core_application::*;

    let app_name = QString::from("qmetaobject-rs testing app");
    let app_version = QString::from("0.1");
    let org_domain = QString::from("woboq.com");
    let org_name = QString::from("Woboq");

    QCoreApplication::set_application_name(app_name.clone());
    QCoreApplication::set_application_version(app_version.clone());
    QCoreApplication::set_organization_domain(org_domain.clone());
    QCoreApplication::set_organization_name(org_name.clone());

    let app_name_ = QCoreApplication::application_name();
    let app_version_ = QCoreApplication::application_version();
    let org_domain_ = QCoreApplication::organization_domain();
    let org_name_ = QCoreApplication::organization_name();

    assert_eq!(app_name, app_name_);
    assert_eq!(app_version, app_version_);
    assert_eq!(org_domain, org_domain_);
    assert_eq!(org_name, org_name_);
}

#[test]
fn test_setting_context_object() {
    #[derive(QObject, Default)]
    struct Basic {
        base: qt_base_class!(trait QObject),
        bvalue: qt_property!(i32),
    }
    let qml_txt = "import QtQuick 2.0\nItem { function doTest() { return bvalue  } }";

    let _lock = lock_for_test();
    let mut _app = QmlEngine::new();
    _app.load_data(qml_txt.into());

    let mut obj = Basic::default();
    obj.bvalue = 12;

    let obj_rc = RefCell::new(obj);

    _app.set_object(unsafe { QObjectPinned::new(&obj_rc) });
    let res = _app.invoke_method("doTest".into(), &[]);
    let res: Option<i32> = QMetaType::from_qvariant(res);
    assert_eq!(res, Some(12));
    assert_ne!(res, Some(10));
}
