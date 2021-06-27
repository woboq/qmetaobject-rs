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
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident, Visibility};

pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let crate_ = super::get_crate(&input);

    let values = if let Data::Struct(ref data) = input.data {
        data.fields
            .iter()
            .filter_map(|field| {
                if let Visibility::Public(_) = field.vis {
                    field.ident.clone()
                } else {
                    None
                }
            })
            .collect::<Vec<Ident>>()
    } else {
        panic!("#[derive(SimpleListItem)] is only defined for structs");
    };

    if values.is_empty() {
        panic!("#[derive(SimpleListItem)] only expose public named member, and there are none")
    }

    let arms = values
        .iter()
        .enumerate()
        .map(|(i, ref ident)| {
            let i = i as i32;
            quote! { #i => #crate_::QMetaType::to_qvariant(&self.#ident), }
        })
        .collect::<Vec<_>>();

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote!(
        impl #impl_generics #crate_::listmodel::SimpleListItem for #name #ty_generics #where_clause {
            fn get(&self, idx : i32) -> #crate_::QVariant {
                match idx {
                    #(#arms)*
                    _ => #crate_::QVariant::default()
                }
            }
            fn names() -> Vec<#crate_::QByteArray> {
                vec![ #(#crate_::QByteArray::from(stringify!(#values))),* ]
            }
        }
    ).into()
}
