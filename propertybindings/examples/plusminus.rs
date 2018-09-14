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
        use qmetaobject::QColor;
        let model = Rc::new(PlusMinus::default());
        let model1 = model.clone();
        let model2 = model.clone();

        rsml!(
            ColumnLayout {
                Container {
                    Rectangle { color: QColor::from_name("grey") }
                    Text {
                        text: "-".into(),
                        vertical_alignment: alignment::VCENTER,
                        horizontal_alignment: alignment::HCENTER,
                    }
                    MouseArea { on_clicked: model1.counter.set(model1.counter.get() - 1) }
                }
                Text {
                    text: model.counter.get().to_string().into(),
                    vertical_alignment: alignment::VCENTER,
                    horizontal_alignment: alignment::HCENTER,
                }
                Container {
                    Rectangle { color: QColor::from_name("grey") }
                    Text {
                        text: "+".into(),
                        vertical_alignment: alignment::VCENTER,
                        horizontal_alignment: alignment::HCENTER,
                    }
                    MouseArea { on_clicked: model2.counter.set(model2.counter.get() + 1) }
                }
            }
        )
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
    width: 300
    height: 400
    visible: true

    PlusMinus {
        anchors.fill: parent
    }

}



        "#.into());
    engine.exec();
}
