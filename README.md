# QMetaObject crate for Rust

The qmetaobject crate is a crate which is used to expose rust object to Qt and QML.

[![Travis Build Status](https://travis-ci.org/woboq/qmetaobject-rs.svg?branch=master)](https://travis-ci.org/woboq/qmetaobject-rs)
[![Appveyor Build status](https://ci.appveyor.com/api/projects/status/8l5te3wlj2ie4njc/branch/master?svg=true)](https://ci.appveyor.com/project/ogoffart/qmetaobject-rs/branch/master)

## Objectives

 - Rust procedural macro (custom derive) to generate a QMetaObject at compile time.
 - Bindings for the main Qt types using the cpp! macro from the cpp crate.
 - Users of this crate should not require to type any line of C++ or use another build system than cargo.
 - Performance: Avoid any unnecessary conversion or heap allocation.

 Presentation Blog Post: https://woboq.com/blog/qmetaobject-from-rust.html

## Overview

```rust
extern crate qmetaobject;
use qmetaobject::*;
#[macro_use] extern crate cstr;

#[derive(QObject,Default)]
struct Greeter {
    base : qt_base_class!(trait QObject),
    name : qt_property!(QString; NOTIFY name_changed),
    name_changed : qt_signal!(),
    compute_greetings : qt_method!(fn compute_greetings(&self, verb : String) -> QString {
        return (verb + " " + &self.name.to_string()).into()
    })
}

fn main() {
    qml_register_type::<Greeter>(cstr!("Greeter"), 1, 0, cstr!("Greeter"));
    let mut engine = QmlEngine::new();
    engine.load_data(r#"import QtQuick 2.6; import QtQuick.Window 2.0;
import Greeter 1.0
Window {
    visible: true;
    Greeter { id: greeter; name: 'World'; }
    Text { anchors.centerIn: parent; text: greeter.compute_greetings('hello'); }
}"#.into());
    engine.exec();

}
```

## Features

 - Create object inheriting from QObject, QQuickItem, QAbstractListModel, QQmlExtensionPlugin, ...
 - Export Qt properties, signals, methods, ...
 - Also support `#[derive(QGadget)]` (same as Q_GADGET)
 - Create Qt plugin (see examples/qmlextensionplugins)
 - Partial scene graph support

Requires Qt >= 5.8
