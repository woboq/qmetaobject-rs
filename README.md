# QMetaObject crate for Rust

The qmetaobject crate is a crate which is used to expose rust object to Qt and QML.

[![Travis Build Status](https://travis-ci.org/woboq/qmetaobject-rs.svg?branch=master)](https://travis-ci.org/woboq/qmetaobject-rs)
[![Appveyor Build status](https://ci.appveyor.com/api/projects/status/8l5te3wlj2ie4njc/branch/master?svg=true)](https://ci.appveyor.com/project/ogoffart/qmetaobject-rs/branch/master)
[![Crates.io](https://img.shields.io/crates/v/qmetaobject.svg)](https://crates.io/crates/qmetaobject)
[![Documentation](https://docs.rs/qmetaobject/badge.svg)](https://docs.rs/qmetaobject/)

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
    qmetaobject::log::init_qt_to_rust();
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

## Cargo features

Cargo provides a way to enable (or disable default) optional [features](https://doc.rust-lang.org/cargo/reference/features.html).

### `log`

By default, Qt's logging system is not initialized, and messages from e.g. QML's `console.log` don't go anywhere.
The "log" feature enables integration with [`log`](https://crates.io/crates/log) crate, the Rust logging facade.

The feature is enabled by default. To activate it, execute the following code as early as possible in `main()`:

```rust
fn main() {
    qmetaobject::log::init_qt_to_rust();
    // don't forget to set up env_logger or any other logging backend.
}
```

### `chrono_qdatetime`

Enables interoperability of `QDate` and `QTime` with Rust [`chrono`](https://crates.io/crates/chrono) package.

This feature is disabled by default.

## What if a binding for the Qt C++ API you want to use is missing?

It is quite likely that you would like to call a particular Qt function which is not wrapped by
this crate.

In this case, it is always possible to access C++ directly from your rust code using the cpp! macro.

Example: from [`examples/graph/src/main.rs`](https://github.com/woboq/qmetaobject-rs/blob/a5b7456bdd22b856dfae49d513c06ecddd6499fc/examples/graph/src/main.rs#L37), the struct Graph is a QObject deriving from QQuickItem,
QQuickItem::setFlag is currently not exposed in the API but we wish to call it anyway.

```rust
impl Graph {
    fn appendSample(&mut self, value: f64) {
        // ...
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "QQuickItem*"] { obj->setFlag(QQuickItem::ItemHasContents); });
        // ...
    }
}
```

But ideally, we should wrap as much as possible so this would not be needed. You can request API
as a github issue, or contribute via a pull request.
