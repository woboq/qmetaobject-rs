extern crate qmetaobject;
use qmetaobject::*;
use qmetaobject::test::do_test;

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


#[derive(QObject,Default)]
struct MyObject {
    base: qt_base_class!(trait QObject),
    prop_x: qt_property!(u32; NOTIFY prop_x_changed),
    prop_x_changed: qt_signal!(),
    prop_y: qt_property!(String; NOTIFY prop_y_changed),
    prop_y_changed: qt_signal!(),
    prop_z: qt_property!(QString; NOTIFY prop_z_changed),
    prop_z_changed: qt_signal!(),

    multiply_and_add1: qt_method!(fn multiply_and_add1(&self, a: u32, b:u32) -> u32 { a*b + 1 }),

    concatenate_strings: qt_method!(fn concatenate_strings(
            &self, a: QString, b:QString, c: QByteArray) -> QString {
        let a = a.to_string();
        QString::from_str(&(a + &(b.to_string()) + &(c.to_string())))
    })
}


#[test]
fn property_read_write_notify() {

    let obj = MyObject::default();
    assert!(do_test(obj, "Item {
        property int yo: _obj.prop_x;
        function doTest() {
            _obj.prop_x = 123;
            return yo === 123;
        }}"));

    let obj = MyObject::default();
    assert!(do_test(obj, "Item {
        property string yo: _obj.prop_y + ' ' + _obj.prop_z;
        function doTest() {
            _obj.prop_y = 'hello';
            _obj.prop_z = 'world';
            return yo === 'hello world';
        }}"));
}

#[test]
fn call_method() {

    let obj = MyObject::default();
    assert!(do_test(obj, "Item {
        function doTest() {
            return _obj.multiply_and_add1(45, 76) === 45*76+1;
        }}"));

    let obj = MyObject::default();
    assert!(do_test(obj, "Item {
        function doTest() {
            return _obj.concatenate_strings('abc', 'def', 'hij') == 'abcdefhij';
        }}"));

    let obj = MyObject::default();
    assert!(do_test(obj, "Item {
        function doTest() {
            return _obj.concatenate_strings(123, 456, 789) == '123456789';
        }}"));
}



#[test]
fn simple_model() {

    #[derive(Default)]
    struct TM {
        a: QString,
        b: u32,
    }
    impl qmetaobject::listmodel::SimpleListItem for TM {
        fn get(&self, idx : i32) -> QVariant {
            match idx {
                0 => self.a.clone().into(),
                1 => self.b.clone().into(),
                _ => QVariant::default()
            }
        }
        fn names() -> Vec<QByteArray> {
            vec![ QByteArray::from_str("a"), QByteArray::from_str("b") ]
        }
    }
    let model : qmetaobject::listmodel::SimpleListModel<TM> = Default::default();
    assert!(do_test(model, "Item {
            Repeater{
                id: rep;
                model:_obj;
                Text {
                    text: a + b;
                }
            }
            function doTest() {
                return rep.count === 0;
            }}"));
}
