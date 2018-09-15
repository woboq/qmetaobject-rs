//! This crate is work in progress.
//! The Idea is to have develop a QML-inspired macros in rust.
//!
//! Behind the scene, this uses the QML scene graph. But there is
//! only one QQuickItem. All rust Item are just node in the scene
//! graphs.
//! (For some node such as text node, there is an hidden QQuickItem
//! because there is no public API to get a text node)
//! only the `items` module depends on Qt.

#![recursion_limit="512"]

#[macro_use] extern crate cstr;


#[macro_use]
extern crate qmetaobject;

#[macro_use] extern crate cpp;

#[macro_use]
pub mod properties;
pub use properties::*;
pub mod anchors;
#[macro_use]
pub mod rslm;
pub mod items;
pub mod quick;


