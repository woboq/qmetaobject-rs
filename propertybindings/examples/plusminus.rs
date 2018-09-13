#[macro_use] extern crate propertybindings;
extern crate qmetaobject;
#[macro_use] extern crate cstr;
use std::rc::Rc;

use propertybindings::properties::Property;

#[derive(Default)]
struct PlusMinus {
    counter : Property<'static, i32>,
}

impl propertybindings::items::ItemFactory for PlusMinus {

    fn create() -> Rc<propertybindings::items::Item<'static>> {
        use propertybindings::items::*;
        let model = Rc::new(PlusMinus::default());
        let model1 = model.clone();
        let model2 = model.clone();

        let i = rsml!(
            ColumnLayout {
                MouseArea { on_clicked: model1.counter.set(model1.counter.get() - 1) }
                Text { text: model.counter.get().to_string().into() }
                MouseArea { on_clicked: model2.counter.set(model2.counter.get() + 1) }
            }
        );
        i
    }

}


fn main() {

    qmetaobject::qml_register_type::<propertybindings::items::RSMLItem<PlusMinus>>(cstr!("PlusMinus"), 1, 0, cstr!("PlusMinus"));
    let mut engine = qmetaobject::QmlEngine::new();
    engine.load_data(r#"
import QtQuick 2.0
import QtQuick.Window 2.0
import PlusMinus 1.0

Window {
    width: 800
    height: 400
    visible: true

    PlusMinus {
        anchors.fill: parent
    }

}



        "#.into());
    engine.exec();
}
