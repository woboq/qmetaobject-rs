use cstr::cstr;
use qmetaobject::{
    qml_register_type, qt_base_class, qt_method, qt_property, qt_signal, QObject, QString,
    QmlEngine,
};
use std::cell::RefCell;

// Here we define a custom QObject Person with a property and two methods.
#[derive(QObject, Default)]
struct Person {
    base: qt_base_class!(trait QObject),
    name: qt_property!(QString; NOTIFY name_changed),
    name_changed: qt_signal!(),
}

impl Person {
    fn set_name(&mut self, name: String) {
        self.name = name.into();
        self.name_changed();
    }

    fn get_name(&self) -> String {
        self.name.to_string()
    }
}

// Now we want to use the Person as a property of another QObject.
#[derive(QObject, Default)]
struct Greeter {
    base: qt_base_class!(trait QObject),

    // To store our Person QObject as a property of another QObject, we need to use a RefCell.
    person: qt_property!(RefCell<Person>; NOTIFY person_changed),
    person_changed: qt_signal!(),

    compute_greetings: qt_method!(
        fn compute_greetings(&self, verb: String) -> QString {
            // To access the person, we need to borrow it.
            format!("{} {}", verb, self.person.borrow().get_name()).into()
        }
    ),
    set_person_name: qt_method!(
        fn set_person_name(&mut self, name: String) {
            // To modify the nested object we need to borrow it as mutable
            println!("Person name set to {}", &name);
            self.person.borrow_mut().set_name(name);
            self.person_changed();
        }
    ),
}

fn main() {
    // We need to register our two custom QObjects with the QML engine.
    qml_register_type::<Greeter>(cstr!("Greeter"), 1, 0, cstr!("Greeter"));
    qml_register_type::<Person>(cstr!("Person"), 1, 0, cstr!("Person"));

    let mut engine = QmlEngine::new();
    engine.load_data(
        r#"
        import QtQuick 2.6
        import QtQuick.Window 2.0
        import Greeter 1.0

        Window {
            visible: true
            Greeter {
                id: greeter;
                // Here we can directly set the person name inside the Greeter's Person property
                person.name: "World"
                // or we can use the set_person_name method to set the name
                //Component.onCompleted : {
                //    greeter.set_person_name("foo");
                //}
            }
            Text {
                id: txt
                anchors.centerIn: parent
                text: greeter.compute_greetings("hello")

                // When the person's name changes, we update the text
                Connections {
                    target: greeter
                    function onPersonChanged() {
                        txt.text = greeter.compute_greetings("hello")
                    }
                }
            }

            
        }
    "#
        .into(),
    );
    engine.exec();
}
