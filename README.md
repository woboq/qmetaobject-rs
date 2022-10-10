# QMetaObject crate for Rust

[![Crates.io](https://img.shields.io/crates/v/qmetaobject.svg)](https://crates.io/crates/qmetaobject)
[![Documentation](https://docs.rs/qmetaobject/badge.svg)](https://docs.rs/qmetaobject/)

A framework empowering everyone to create Qt/QML applications with Rust.
It does so by building `QMetaObject`s at compile time, registering QML types (optionally via exposing `QQmlExtensionPlugin`s) and providing idiomatic wrappers.

## Objectives

 - Rust procedural macro (custom derive) to generate a `QMetaObject` at compile time.
 - Bindings for the main Qt types using the `cpp!` macro from the [`cpp`](https://crates.io/crates/cpp) crate.
 - Users of this crate should not require to type any line of C++ or use another build system beyond cargo.
 - Performance: Avoid any unnecessary conversion or heap allocation.

Presentation Blog Post: https://woboq.com/blog/qmetaobject-from-rust.html

## Overview

```rust
use cstr::cstr;
use qmetaobject::prelude::*;

// The `QObject` custom derive macro allows to expose a class to Qt and QML
#[derive(QObject, Default)]
struct Greeter {
    // Specify the base class with the qt_base_class macro
    base: qt_base_class!(trait QObject),
    // Declare `name` as a property usable from Qt
    name: qt_property!(QString; NOTIFY name_changed),
    // Declare a signal
    name_changed: qt_signal!(),
    // And even a slot
    compute_greetings: qt_method!(fn compute_greetings(&self, verb: String) -> QString {
        format!("{} {}", verb, self.name.to_string()).into()
    })
}

fn main() {
    // Register the `Greeter` struct to QML
    qml_register_type::<Greeter>(cstr!("Greeter"), 1, 0, cstr!("Greeter"));
    // Create a QML engine from rust
    let mut engine = QmlEngine::new();
    // (Here the QML code is inline, but one can also load from a file)
    engine.load_data(r#"
        import QtQuick 2.6
        import QtQuick.Window 2.0
        // Import our Rust classes
        import Greeter 1.0

        Window {
            visible: true
            // Instantiate the rust struct
            Greeter {
                id: greeter;
                // Set a property
                name: "World"
            }
            Text {
                anchors.centerIn: parent
                // Call a method
                text: greeter.compute_greetings("hello")
            }
        }
    "#.into());
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

### `webengine`

Enables `QtWebEngine` functionality. For more details see the [example](./examples/webengine).

This feature is disabled by default.

## What if a wrapper for the Qt C++ API is missing?

It is quite likely that you would like to call a particular Qt function which
is not wrapped by this crate.

In this case, it is always possible to access C++ directly from your rust code
using the `cpp!` macro.

We strive to increase coverage of wrapped API, so whenever there is something
you need but currently missing, you are welcome to open a feature request on
GitHub issues or send a Pull Request right away.

### Tutorial: Adding Rust wrappers for Qt C++ API

This section teaches how to make your own crate with new Qt wrappers, and walk
through a Graph example provided with this repository.

First things first, set up your _Cargo.toml_ and _build.rs_:

1. Add `qttypes` to dependencies.
   Likely, you would just stick to recent versions published on [crates.io](versions).
   ```toml
   [dependencies]
   qttypes = { version = "0.2", features = [ "qtquick" ] }
   ```
   Add more Qt modules you need to the features array.
   Refer to [qttypes crate documentation](docs.qttypes) for a full list of supported modules.
   <br/>
   If you _absolutely need_ latest unreleased changes, use this instead of `version = "..."`:
    * `path = "../path/to/qmetaobject-rs/qttypes"` or
    * `git = "https://github.com/woboq/qmetaobject-rs"`

2. Add `cpp` to dependencies and `cpp_build` to build-dependencies.
   You can find up-to-date instructions on [`cpp` documentation](https://docs.rs/cpp) page.
   ```toml
   [dependencies]
   cpp = "0.5"

   [build-dependencies]
   cpp_build = "0.5"
   ```

3. Copy _build.rs_ script from [_qmetaobject/build.rs_](./qmetaobject/build.rs).
   It will run `cpp_build` against you package, using environment provided by
   [_qttypes/build.rs_](./qttypes/build.rs).

Now, every time you build your package, content of `cpp!` macros will be
collected in one big C++ file and compiled into a static library which will
later be linked into a final binary. You can find this _cpp_closures.cpp_
file buried inside Cargo target directory. Understanding its content might be
useful for troubleshooting.

There are two forms of `cpp!` macro.

* The one with double curly `{{` braces `}}` appends its content verbatim to
  the C++ file. Use it to `#include` headers, define C++ structs & classes etc.

* The other one is for calling expressions at runtime. It is usually written
  with `(` parenthesis `)`, it takes `[` arguments `]` list and requires an
  `unsafe` marker (either surrounding block or as a first keyword inside).

Order of macros invocations is preserved on a per-file (Rust module) basis;
but processing order of files is not guaranteed by the order of `mod`
declarations. So don't assume visibility — make sure to `#include` everything
needed on top of every Rust module.

Check out [documentation of `cpp`](https://docs.rs/cpp) to read more about how
it works internally.

Now that we are all set, let's take a look at the Graph example's code. It is
located in [_examples/graph_](./examples/graph) directory.

Before adding wrappers, we put relevant `#include` lines inside a `{{` double
curly braced `}}` macro:

```rust
cpp! {{
    #include <QtQuick/QQuickItem>
}}
```

If you need to include you own local C++ headers, you can do that too! Check
out how main qmetaobject crate includes _qmetaobject_rust.hpp_ header in
every Rust module that needs it.

Next, we declare a custom QObject, just like in the [overview](#overview), but
this time it derives from `QQuickItem`. Despite its name, `#[derive(QObject)]`
proc-macro can work with more than one base class, as long as it is properly
wrapped and implements the [`QObject`](trait.QObject) trait.

```rust
#[derive(Default, QObject)]
struct Graph {
    base: qt_base_class!(trait QQuickItem),

    // ...
}
```

We wish to call [`QQuickItem::setFlag`] method which is currently not
exposed in the qmetaobject-rs API, so let's call it directly:

```rust
impl Graph {
    fn appendSample(&mut self, value: f64) {
        // ...
        let obj = self.get_cpp_object();
        cpp!(unsafe [obj as "QQuickItem *"] {
            obj->setFlag(QQuickItem::ItemHasContents);
        });
        // ...
    }
}
```

Alternatively, we could add a proper method wrapper, and call it without `unsafe`:

```rust
#[repr(C)]
enum QQuickItemFlag {
    ItemClipsChildrenToShape = 0x01,
    ItemAcceptsInputMethod = 0x02,
    ItemIsFocusScope = 0x04,
    ItemHasContents = 0x08,
    ItemAcceptsDrops = 0x10,
}

impl Graph {
    fn set_flag(&mut self, flag: QQuickItemFlag) {
        let obj = self.get_cpp_object();
        assert!(!obj.is_null());
        cpp!(unsafe [obj as "QQuickItem *", flag as "QQuickItem::Flag"] {
            obj->setFlag(flag);
        });
    }

    fn appendSample(&mut self, value: f64) {
        // ...
        self.set_flag(QQuickItemFlag::ItemHasContents);
        // ...
    }
}
```

Note that C++ method takes optional second argument, but since optional
arguments are not supported by Rust nor by FFI glue, it is always left out
(and defaults to `true`) in this case. To improve on this situation, we could
have added second required argument to Rust function, or implement
two "overloads" with slightly different names, e.g. `set_flag(Flag, bool)` &
`set_flag_on(Flag)` or `enable_flag(Flag)` etc.

Assert for not-null should not be needed if object is guaranteed to be
properly instantiated and initialized before usage. This applies to the
following situations:

- Call [`QObject::cpp_construct()`] directly and store the result in immovable
  memory location;

- Construct [`QObjectPinned`] instance: any access to pinned object or
  conversion to [`QVariant`] ensures creation of C++ object;

- Instantiate object as a QML component. They are always properly
  default-initialized by a QML engine before setting any properties or
  calling any signals/slots.

And that's it! You have just implemented a new wrapper for a Qt C++ class
method. Now send us a Pull Request. 🙂

[versions]: https://crates.io/crates/qmetaobject/versions
[trait.QObject]: https://docs.rs/qmetaobject/latest/qmetaobject/trait.QObject.html
[`QQuickItem::setFlag`]: https://doc.qt.io/qt-5/qquickitem.html#setFlag
[`QObject::cpp_construct()`]: https://docs.rs/qmetaobject/latest/qmetaobject/trait.QObject.html#tymethod.cpp_construct
[`QObjectPinned`]: https://docs.rs/qmetaobject/latest/qmetaobject/struct.QObjectPinned.html
[`QVariant`]: https://docs.rs/qmetaobject/latest/qmetaobject/struct.QVariant.html
[docs.qttypes]: https://docs.rs/qttypes/latest/qttypes/#cargo-features

## Comparison with other projects

This crate objective is to make idiomatic Rust bindings for QML (and only QML, no QWidgets or other
non-graphical Qt API) in a way that doesn't need you to know or use C++ and other build system.
This crates is the best achieving this.

* **[CXX-Qt](https://github.com/KDAB/cxx-qt/)** still makes you to write a bit of boiler-plate code
  in C++ and use extra build step to compile the C++.
  CXX-Qt is ideal to bring some Rust in an existing C++ project. But less so when you just want to
  make an UI for a Rust-only application.

  The CXX-Qt is also  more recent that this crate and make use of Rust features such as attribute
  macro, that did not  exist when the qmetaobject crate was designed.
  (Only derive procedural macro were available in stable rust rust at the time)

* Similarly, the **[Rust Qt Binding Generator](https://invent.kde.org/sdk/rust-qt-binding-generator)**
  is another project that helps to integrate Rust logic in an existing C++/Qt project. This was also
  created before rust had procedural macros, so it uses an external .json file to generate C++ and
  Rust code.

* There exist also a bunch of older crates that tries to provide Rust binding around the Qt C++ API.
  Often automatically generated, these bindings are not idiomatic Rust, require unsafe code to use,
  and are not maintained anymore.

* **[Slint](https://slint-ui.com)** is a project created by the same author of this crate.
  It is not a QML or Qt binding at all, but rather a new language similar to QML, entirely
  implemented in Rust.
  It has the same goal as providing a new to add a UI to a Rust project with idiomatic Rust API,
  but instead of using QML for the UI, it uses its own language.
