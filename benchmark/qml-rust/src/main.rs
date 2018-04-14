#![allow(non_snake_case)]
#[macro_use]
extern crate qml;
use qml::*;


pub struct Test {
    ret : QVariant,
}

impl Test {

    pub fn addTwo(&mut self, x: i32) -> Option<&QVariant> {
        self.ret = (x as f32 +2.).into();
        return Some(&self.ret);
    }

    pub fn countW(&mut self, x: String) -> Option<&QVariant> {
        let mut count = 0;
        for i in x.chars() {
            if i == 'W' {
                count+=1;
            }
        }
        self.ret = (count as f32).into();
        return Some(&self.ret);
    }

    pub fn replaceW(&mut self, x: String) -> Option<&QVariant> {
        /*if self.countW(x.clone()) == 0 {
            return x;
        }*/
        self.ret = x.replace("W", ".").into();
        return Some(&self.ret);
    }
}

Q_OBJECT!(
pub Test as QTest{
    signals:
//        fn testname (a: i32, b: i32, f: f32, d: f64, bo: bool, list: QVariantList);
    slots:
        fn addTwo(i: i32);
        fn countW(x: String);
        fn replaceW(x: String);
    properties:
        strProp: String; read: get_strProp, write: set_strProp, notify: strProp_changed;
        intProp: i32; read: get_intProp, write: set_intProp, notify: intProp_changed;
});


fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut engine = QmlEngine::new();

    let q = QVariant::from(42);
    let t = QTest::new(Test { ret: q }, "Hello".into(), 42);
    engine.set_and_store_property("testObj", t.get_qobj());
    engine.load_file(&*args[1]);
    engine.exec();
    engine.quit();


}

