extern crate qmetaobject;
use qmetaobject::*;

use std::ffi::CStr;

mod implementation;

fn main() {
    qml_register_type::<implementation::Todos>(CStr::from_bytes_with_nul(b"RustCode\0").unwrap(), 1, 0,
        CStr::from_bytes_with_nul(b"Todos\0").unwrap());
    let mut engine = QmlEngine::new();
    engine.load_data((include_bytes!("../main.qml") as &[u8]).into());
    engine.exec();
}
