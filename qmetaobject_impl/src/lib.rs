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
#![cfg_attr(feature = "cargo-clippy", allow(clippy::unreadable_literal))] // Because we copy-paste constants from Qt
#![cfg_attr(feature = "cargo-clippy", allow(clippy::cognitive_complexity))]

#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

extern crate proc_macro;
extern crate proc_macro2;
use proc_macro::TokenStream;

mod qbjs;
mod qobject_impl;
mod qrc_impl;
mod simplelistitem_impl;

/// Get the tokens to refer to the qmetaobject crate. By default, return "::qmetaobject" unless
/// the QMetaObjectCrate is specified
fn get_crate(input: &syn::DeriveInput) -> impl quote::ToTokens {
    for i in input.attrs.iter() {
        if let Ok(x) = i.parse_meta() {
            if x.path().is_ident("QMetaObjectCrate") {
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

/// Implementation of #[derive(QObject)]
#[proc_macro_derive(QObject, attributes(QMetaObjectCrate, qt_base_class))]
pub fn qobject_impl(input: TokenStream) -> TokenStream {
    qobject_impl::generate(input, true)
}

/// Implementation of #[derive(QGadget)]
#[proc_macro_derive(QGadget, attributes(QMetaObjectCrate))]
pub fn qgadget_impl(input: TokenStream) -> TokenStream {
    qobject_impl::generate(input, false)
}

/// Implementation of #[derive(QEnum)]
#[proc_macro_derive(QEnum, attributes(QMetaObjectCrate))]
pub fn qenum_impl(input: TokenStream) -> TokenStream {
    qobject_impl::generate_enum(input)
}

/// Implementation of the qmetaobject::qrc! macro
#[proc_macro]
pub fn qrc_internal(input: TokenStream) -> TokenStream {
    qrc_impl::process_qrc(input)
}

/// Implementation of #[derive(SimpleListItem)]
#[proc_macro_derive(SimpleListItem, attributes(QMetaObjectCrate))]
pub fn simplelistitem(input: TokenStream) -> TokenStream {
    simplelistitem_impl::derive(input)
}
