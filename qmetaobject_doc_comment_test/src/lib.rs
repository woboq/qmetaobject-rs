/* Copyright (C) 2020 ivan tkachenko a.k.a. ratijas <me@ratijas.tk>

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

//! Showcase of `#[qt_doc(...)]` custom proc macro attribute.
#![deny(missing_docs)]

#[macro_use]
extern crate qmetaobject;

#[qt_doc(qt = "qstring.html", cls = "QString")]
/// Custom explanations...
pub struct QSomething;

impl QSomething {
    #[qt_doc(qt = "qstring.html#capacity", method = "int QString::capacity() const")]
    #[doc = "attr docs"]
    /// Normal `docs`
    pub fn capacity(&self) -> i32 { 0 }

    #[qt_doc(qt = "qbytearray.html#fromHex", member = "QByteArray QByteArray::fromHex(const QByteArray &hexEncoded)")]
    pub fn from_hex(hex_encoded: &[u8]) -> QSomething { QSomething {} }
}

#[qt_doc(qt = "qbytearray.html#qstrnlen", related = "uint qstrnlen(const char *str, uint maxlen)")]
pub fn qstrlen(str: *const u8, maxlen: usize) {}

#[allow(non_camel_case_types)]
#[qt_doc(qt = "qtglobal.html#qreal-typedef", typedef = "qreal")]
pub type qreal = f32;
