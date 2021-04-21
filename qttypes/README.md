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