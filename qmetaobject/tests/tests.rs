extern crate qmetaobject;
use qmetaobject::*;
use qmetaobject::test::do_test;


/*
#[derive(QObject,Default)]
struct MyObject {
    base: qt_base_object!(trait QObject),
    prop_x: qt_property(u32; NOTIFY prop_x_changed),
    prop_x_changed: qt_signal!(),
    prop_y: qt_property(String; NOTIFY prop_y_changed),
    prop_y_changed: qt_signal!(),
    prop_z: qt_property(QString; NOTIFY prop_z_changed),
    prop_z_changed: qt_signal!(),

    multiply_and_add1: qt_method(fn multiply_and_add1(a: u32, b:u32) -> u32 { a*b + 1 })
}*/

#[test]
fn self_test() {

    #[derive(QObject,Default)]
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

