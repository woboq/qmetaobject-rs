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

//! This crates implement the custom derive used by the `qmetaobject` crate.

#![recursion_limit = "256"]

#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

extern crate proc_macro;
use proc_macro::TokenStream;

mod qbjs;
mod qobject_impl;
mod qrc_impl;
mod simplelistitem_impl;

/// Get the tokens to refer to the qmetaobject crate. By default, return "::qmetaobject" unless
/// the QMetaObjectCrate is specified
fn get_crate(input: &syn::DeriveInput) -> impl quote::ToTokens {
    for i in input.attrs.iter() {
        if let Some(x) = i.interpret_meta() {
            if x.name() == "QMetaObjectCrate" {
                if let syn::Meta::NameValue(mnv) = x {
                    use syn::Lit::*;
                    let lit: syn::Path = match mnv.lit {
                        Str(s) => syn::parse_str(&s.value())
                            .expect("Can't parse QMetaObjectCrate Attribute"),
                        _ => panic!("Can't parse QMetaObjectCrate Attribute"),
                    };
                    return quote!( #lit );
                }
            }
        }
    }

    quote!(::qmetaobject)
}

#[proc_macro_derive(QObject, attributes(QMetaObjectCrate, qt_base_class))]
pub fn qobject_impl(input: TokenStream) -> TokenStream {
    qobject_impl::generate(input, true)
}

#[proc_macro_derive(QGadget, attributes(QMetaObjectCrate))]
pub fn qgadget_impl(input: TokenStream) -> TokenStream {
    qobject_impl::generate(input, false)
}

#[proc_macro_derive(QResource_internal, attributes(qrc))]
pub fn qresource_impl(input: TokenStream) -> TokenStream {
    let src = input.to_string();
    let beg = src
        .find("stringify!(")
        .expect("Internal error: no strignify in QResource_internal contents") + 11;
    let end = src
        .rfind("))")
        .expect("Internal error: no '))' in QResource_internal contents");
    qrc_impl::process_qrc(&src[beg..end])
}

#[proc_macro_derive(SimpleListItem, attributes(QMetaObjectCrate))]
pub fn simplelistitem(input: TokenStream) -> TokenStream {
    simplelistitem_impl::derive(input)
}
