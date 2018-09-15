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
#![recursion_limit="256"]

#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

extern crate proc_macro;
use proc_macro::TokenStream;

mod qbjs;
mod qrc_impl;
mod qobject_impl;

#[proc_macro_derive(QObject, attributes(QMetaObjectCrate,qt_base_class))]
pub fn qobject_impl(input: TokenStream) -> TokenStream {
    qobject_impl::generate(input, true)
}

#[proc_macro_derive(QGadget)]
pub fn qgadget_impl(input: TokenStream) -> TokenStream {
    qobject_impl::generate(input, false)
}

#[proc_macro_derive(QResource_internal, attributes(qrc))]
pub fn qresource_impl(input: TokenStream) -> TokenStream {
    let src = input.to_string();
    let beg = src.find("stringify!(").expect("Internal error: no strignify in QResource_internal contents") + 11;
    let end = src.rfind("))").expect("Internal error: no '))' in QResource_internal contents");
    qrc_impl::process_qrc(&src[beg..end])
}

