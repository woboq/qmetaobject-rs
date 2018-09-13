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
//         use qmetaobject::{QString, QColor};

        let model = Rc::new(PlusMinus::default());

        //let y : Rc<Rectangle<'a>> = rsml!( Rectangle { color: QColor::from_name("yellow") } );
        let mouse1 = rsml!( MouseArea { } );
        let mouse2 = rsml!( MouseArea { } );

        mouse1.pressed.on_notify({
            let model = model.clone();
            move |x| if !*x { model.counter.set(model.counter.get() + 1) }
        });
        mouse2.pressed.on_notify({
            let model = model.clone();
            move |x| if !*x { model.counter.set(model.counter.get() - 1) }
        });

        let i : Rc<ColumnLayout> = rsml!(
            ColumnLayout {
                geometry.width : 110.,
                geometry.height : 90.,
            }
        );
        i.add_child(mouse1);
        i.add_child(rsml!( Text { text: model.counter.get().to_string().into() } ));
        i.add_child(mouse2);
        i
    }

}


/*
#[cfg(test)]
mod test {
    #[test]
    fn test() {
        use super::*;
        use std::rc::{Rc};

        enum MyItem {}
        impl ItemFactory for MyItem {
            fn create() -> Rc<Item<'static>> {
                //let y : Rc<Rectangle<'a>> = rsml!( Rectangle { color: QColor::from_name("yellow") } );
                let m : Rc<MouseArea> = rsml!( MouseArea {  } );
                let t : Rc<Text> = rsml!( Text { text: QString::from("Hello world") } );
                let m_weak = Rc::downgrade(&m);
                let b : Rc<Rectangle> =  rsml!( Rectangle {
                    color: QColor::from_name(if m_weak.upgrade().map_or(false, |x| x.pressed.get()) { "blue" } else { "yellow" })
                } );

                let i : Rc<ColumnLayout> = rsml!(
                    ColumnLayout {
                        geometry.width : 110.,
                        geometry.height : 90.,
                    }
                );
                i.add_child(b);
                i.add_child(t);
                //i.add_child(y);
                i.add_child(m);
                i
            }

        }



}
*/


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
