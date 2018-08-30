#![recursion_limit="512"]

#[cfg(test)]
#[macro_use] extern crate cstr;


#[macro_use]
extern crate qmetaobject;

#[macro_use] extern crate cpp;

#[macro_use]
pub mod properties;
pub use properties::*;
pub mod anchors;
pub mod items;


