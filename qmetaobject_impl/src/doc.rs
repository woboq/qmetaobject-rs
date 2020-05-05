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

use darling::FromMeta;
use proc_macro::TokenStream;
use syn::{AttributeArgs};

#[derive(Default, FromMeta)]
#[darling(default)]
pub struct QtDocMetaArgs {
    qt: String,
    // exactly one of them must be present at a time.
    cls: Option<String>,
    method: Option<String>,
    member: Option<String>,
    related: Option<String>,
    typedef: Option<String>,
}

fn count<T>(it: &Option<T>) -> usize {
    if it.is_some() { 1 } else { 0 }
}

impl QtDocMetaArgs {
    fn count_optionals(&self) -> usize {
        0
            + count(&self.cls)
            + count(&self.method)
            + count(&self.member)
            + count(&self.related)
            + count(&self.typedef)
    }

    fn kind_and_item(&self) -> (&str, &str) {
        if self.count_optionals() != 1 {
            panic!(r#"Exactly one of attributes must be specified: cls, method, member, ..."#);
        }
        if let Some(cls) = &self.cls {
            ("class", cls)
        } else if let Some(method) = &self.method {
            ("method", method)
        } else if let Some(member) = &self.member {
            ("static member", member)
        } else if let Some(related) = &self.related {
            ("related non-member", related)
        } else if let Some(typedef) = &self.typedef {
            ("typedef", typedef)
        } else {
            unreachable!();
        }
    }

    fn doc(&self) -> String {
        let (kind, item) = self.kind_and_item();
        format!("Wrapper for [`{item}`][qt] {kind}.\n\n[qt]: {base}/{path}",
                base = BASE_URL_QT5, path = self.qt, item = item, kind = kind)
    }
}

const BASE_URL_QT5: &'static str = "https://doc.qt.io/qt-5";

pub(crate) fn qt_doc_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    // workaround a compiler bug that was fixed in 1.43.0
    let input = input.into_iter().collect::<TokenStream>();
    // workaround for quote! which accepts only types from proc_macro2
    let input = proc_macro2::TokenStream::from(input);

    let attr_args = parse_macro_input!(args as AttributeArgs);
    let args: QtDocMetaArgs = match QtDocMetaArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let docstring = args.doc();

    let q = quote!(
        #[doc = #docstring]
        #input
    );

    // println!("Expanded TokenStream2:");
    // println!("{:#?}", q);

    q.into()
}
