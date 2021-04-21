# qttypes

This crate contains manually generated bindings to Qt basic value types.
It is meant to be used by other crates, such as the `qmetaobject` crate which re-expose them

The Qt types are basically exposed using the `cpp` crate. They have manually writen rust idiomatic
API which expose the C++ API.
These types are the direct equivalent of the Qt types and are exposed on the stack.

In addition, the build script of this crate expose some metadata to downstream crate that also
want to use Qt's C++ API:
 - `DEP_QT_VERSION`: The Qt version as given by qmake
 - `DEP_QT_INCLUDE_PATH`: The include directory to give to the `cpp_build` crate to locate the Qt headers
 - `DEP_QT_LIBRARY_PATH`: The path containing the Qt libraries.

See the [crate documentation](https://docs.rs/qttypes) for more info.

## Philosophy

The goal of this crate is to expose a idiomatic Qt API for the core value type classes.
The API is manually generated to expose required feature in the most rust-like API, while
still keeping the similarities with the Qt API itself.

It is not meant to expose all of the Qt API exhaustively, but only the part which is
relevant for the usage in other crate.
If you see a feature missing, feel free to write a issue or a pull request.

Note that this crate concentrate on the value types, not the widgets or the
the `QObject`.  For that, there is the `qmetaobject` crate.