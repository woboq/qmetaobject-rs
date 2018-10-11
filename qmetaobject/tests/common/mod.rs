/* Copyright (C) 2018 Olivier Goffart <ogoffart@woboq.com>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

#![allow(dead_code)]

use qmetaobject::*;
use std::sync::Mutex;
use std::cell::RefCell;

lazy_static! {
    pub static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

pub fn do_test<T: QObject + Sized>(obj: T, qml: &str) -> bool {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let qml_text = "import QtQuick 2.0\n".to_owned() + qml;

    let obj = RefCell::new(obj);

    let mut engine = QmlEngine::new();
    engine.set_object_property("_obj".into(), unsafe { QObjectPinned::new(&obj) });
    engine.load_data(qml_text.into());
    engine.invoke_method("doTest".into(), &[]).to_bool()
}

pub fn do_test_variant(obj: QVariant, qml: &str) -> bool {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let qml_text = "import QtQuick 2.0\n".to_owned() + qml;

    let mut engine = QmlEngine::new();
    engine.set_property("_obj".into(), obj);
    engine.load_data(qml_text.into());
    engine.invoke_method("doTest".into(), &[]).to_bool()
}
