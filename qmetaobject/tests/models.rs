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

extern crate qmetaobject;
use qmetaobject::*;

mod common;
use common::*;


#[test]
fn simple_model() {
    #[derive(Default, SimpleListItem)]
    struct TM {
        pub a: QString,
        pub b: u32,
    }
    // FIXME! why vec! here?
    let model: qmetaobject::listmodel::SimpleListModel<TM> = (vec![TM {
        a: "hello".into(),
        b: 1,
    }]).into_iter()
        .collect();
    assert!(do_test(
        model,
        "Item {
            Repeater{
                id: rep;
                model:_obj;
                Text {
                    text: a + b;
                }
            }
            function doTest() {
                console.log('simple_model:', rep.count, rep.itemAt(0).text);
                return rep.count === 1 && rep.itemAt(0).text === 'hello1';
            }}"
    ));
}
