extern crate qmetaobject;
use qmetaobject::*;

#[derive(QObject, Default)]
#[allow(non_snake_case)]
struct MyCppClass {
    base: qt_base_class!(trait QObject),

    addTwo: qt_method!(fn addTwo(&self, x: i32) -> i32 { return x+2; }),
    countW: qt_method!(fn countW(&self, x: QString) -> i32 {
        let slice = x.to_slice();
        let mut count = 0;
        for i in slice {
            if *i == ('W' as u16) {
                count+=1;
            }
        }
        return count
    }),
    replaceW: qt_method!(fn replaceW(&self, x: QString) -> QString {
        if self.countW(x.clone()) == 0 {
            return x;
        }
        let x : String = x.into();
        return x.replace("W", ".").into();
    }),

    strProp: qt_property!(QString),
    intProp: qt_property!(i32),
}


fn main() {
    register_metatype::<String>("String");
    let args: Vec<String> = std::env::args().collect();
    let mut engine = QmlEngine::new();
    engine.load_file(args[1].clone().into());
    let mut my = MyCppClass::default();
    my.strProp = "Hello".into();
    my.intProp = 42;
    let myr : &QObject = &my;
    let my_variant = myr.as_qvariant();
    engine.invoke_method("benchmark".into(), &[my_variant]);
}
