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

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::iter::FromIterator;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream, Result};
use syn::LitStr;

#[derive(Debug)]
struct Resource {
    prefix: String,
    files: Vec<File>,
}

impl Parse for Resource {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Resource {
            prefix: input.parse::<LitStr>()?.value(),
            files: {
                let content;
                braced!(content in input);
                content.parse_terminated::<File, Token![,]>(File::parse)?.into_iter().collect()
            },
        })
    }
}

#[derive(Debug)]
struct File {
    file: String,
    alias: Option<String>,
}

impl Parse for File {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(File {
            file: input.parse::<LitStr>()?.value(),
            alias: input.parse::<Option<Token![as]>>()?.map_or(Ok(None), |_| -> Result<_> {
                Ok(Some(input.parse::<LitStr>()?.value()))
            })?,
        })
    }
}

struct QrcMacro {
    func: syn::Ident,
    data: Vec<Resource>,
}

impl Parse for QrcMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(QrcMacro {
            func: input.parse()?,
            data: {
                input.parse::<Option<Token![,]>>()?;
                input
                    .parse_terminated::<Resource, Token![,]>(Resource::parse)?
                    .into_iter()
                    .collect()
            },
        })
    }
}

fn qt_hash(key: &str) -> u32 {
    let mut h = 0u32;

    for p in key.chars() {
        assert_eq!(p.len_utf16(), 1, "Surrogate pair not supported by the hash function");
        h = (h << 4) + p as u32;
        h ^= (h & 0xf0000000) >> 23;
        h &= 0x0fffffff;
    }
    h
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone)]
struct HashedString {
    hash: u32,
    string: String,
}
impl HashedString {
    fn new(string: String) -> HashedString {
        let hash = qt_hash(&string);
        HashedString { hash, string }
    }
}

enum TreeNode {
    File(String), // The FileName
    Directory(BTreeMap<HashedString, TreeNode>, u32),
}
impl TreeNode {
    fn new_dir() -> TreeNode {
        TreeNode::Directory(Default::default(), 0)
    }
    fn new_file(file: String) -> TreeNode {
        TreeNode::File(file)
    }

    fn insert_node(&mut self, rel_path: &str, node: TreeNode) {
        let contents = match self {
            TreeNode::Directory(ref mut contents, _) => contents,
            _ => panic!("root not a dir?"),
        };

        if rel_path == "" {
            // insert into iteself
            contents.extend(match node {
                TreeNode::Directory(contents, _) => contents,
                _ => panic!("merge file and directory?"),
            });
            return;
        }

        match rel_path.find('/') {
            Some(idx) => {
                let (name, rest) = rel_path.split_at(idx);
                let hashed = HashedString::new(name.into());
                contents
                    .entry(hashed)
                    .or_insert_with(TreeNode::new_dir)
                    .insert_node(&rest[1..], node);
            }
            None => {
                let hashed = HashedString::new(rel_path.into());
                contents
                    .insert(hashed, node)
                    .and_then(|_| -> Option<()> { panic!("Several time the same file?") });
            }
        };
    }

    fn compute_offsets(&mut self, mut offset: u32) -> u32 {
        if let TreeNode::Directory(ref mut dir, ref mut o) = self {
            *o = offset;
            offset += dir.len() as u32;
            for node in dir.values_mut() {
                offset = node.compute_offsets(offset);
            }
        }
        offset
    }
}

// remove duplicate, or leading '/'
fn simplify_prefix(mut s: String) -> String {
    let mut last_slash = true; // so we remove the first '/'
    s.retain(|x| {
        let r = last_slash && x == '/';
        last_slash = x == '/';
        !r
    });
    if last_slash {
        s.pop();
    }
    s
}

#[test]
fn simplify_prefix_test() {
    assert_eq!(simplify_prefix("/".into()), "");
    assert_eq!(simplify_prefix("///".into()), "");
    assert_eq!(simplify_prefix("/foo//bar/d".into()), "foo/bar/d");
    assert_eq!(simplify_prefix("hello/".into()), "hello");
}

fn build_tree(resources: Vec<Resource>) -> TreeNode {
    let mut root = TreeNode::new_dir();
    for r in resources {
        let mut node = TreeNode::new_dir();
        for f in r.files {
            node.insert_node(
                f.alias.as_ref().unwrap_or(&f.file),
                TreeNode::new_file(f.file.clone()),
            );
        }
        root.insert_node(&simplify_prefix(r.prefix), node);
    }
    root
}

fn push_u32_be(v: &mut Vec<u8>, val: u32) {
    v.extend_from_slice(&[
        ((val >> 24) & 0xff) as u8,
        ((val >> 16) & 0xff) as u8,
        ((val >> 8) & 0xff) as u8,
        (val & 0xff) as u8,
    ]);
}

fn push_u16_be(v: &mut Vec<u8>, val: u16) {
    v.extend_from_slice(&[((val >> 8) & 0xff) as u8, (val & 0xff) as u8]);
}

#[derive(Default, Debug)]
struct Data {
    payload: Vec<u8>,
    names: Vec<u8>,
    tree_data: Vec<u8>,
    files: Vec<String>,
}
impl Data {
    fn insert_file(&mut self, filename: &str) {
        let mut filepath = PathBuf::new();
        if let Ok(cargo_manifest) = env::var("CARGO_MANIFEST_DIR") {
            filepath.push(cargo_manifest);
        }

        filepath.push(filename);

        let mut data = fs::read(&filepath)
            .unwrap_or_else(|_| panic!("Cannot open file {}", filepath.display()));
        push_u32_be(&mut self.payload, data.len() as u32);
        self.payload.append(&mut data);
        self.files.push(filepath.to_str().expect("File path contains invalid Unicode").into());
    }

    fn insert_directory(&mut self, contents: &BTreeMap<HashedString, TreeNode>) {
        for (ref name, ref val) in contents {
            let name_off = self.insert_name(name);
            push_u32_be(&mut self.tree_data, name_off);
            match val {
                TreeNode::File(ref filename) => {
                    push_u16_be(&mut self.tree_data, 0); // flags
                    push_u16_be(&mut self.tree_data, 0); // country
                    push_u16_be(&mut self.tree_data, 1); // lang (C)
                    let offset = self.payload.len();
                    push_u32_be(&mut self.tree_data, offset as u32);
                    self.insert_file(filename);
                }
                TreeNode::Directory(ref c, offset) => {
                    push_u16_be(&mut self.tree_data, 2); // directory flag
                    push_u32_be(&mut self.tree_data, c.len() as u32);
                    push_u32_be(&mut self.tree_data, *offset);
                }
            }
            // modification time (64 bit) FIXME
            push_u32_be(&mut self.tree_data, 0);
            push_u32_be(&mut self.tree_data, 0);
        }
        for val in contents.values() {
            if let TreeNode::Directory(ref c, _) = val {
                self.insert_directory(c)
            }
        }
    }

    fn insert_name(&mut self, name: &HashedString) -> u32 {
        let offset = self.names.len();
        push_u16_be(&mut self.names, name.string.len() as u16);
        push_u32_be(&mut self.names, name.hash);

        for p in name.string.chars() {
            assert_eq!(p.len_utf16(), 1, "Surrogate pair not supported");
            push_u16_be(&mut self.names, p as u16);
        }
        //println!("NAME {} -> {}", offset, name.string);
        offset as u32
    }
}

fn generate_data(root: &TreeNode) -> Data {
    let mut d = Data::default();

    let contents = match root {
        TreeNode::Directory(ref contents, _) => contents,
        _ => panic!("root not a dir?"),
    };

    // first item
    push_u32_be(&mut d.tree_data, 0); // fake name
    push_u16_be(&mut d.tree_data, 2); // flag
    push_u32_be(&mut d.tree_data, contents.len() as u32);
    push_u32_be(&mut d.tree_data, 1); // first offset

    // modification time (64 bit) FIXME
    push_u32_be(&mut d.tree_data, 0);
    push_u32_be(&mut d.tree_data, 0);

    d.insert_directory(contents);
    d
}

fn expand_macro(func: &syn::Ident, data: Data) -> TokenStream {
    let Data { payload, names, tree_data, files } = data;

    // Workaround performance issue with proc_macro2 and Rust 1.29:
    // quote!(#(#payload),*) uses proc_macro2::TokenStream::extend, which is O(n²) with rust 1.29
    // since the payload array can be quite large, this is completely unacceptable.
    let payload = ::proc_macro2::TokenStream::from_iter(payload.iter().map(|x| quote!(#x,)));

    let q = quote! {
        fn #func() {
            use ::std::sync::Once;
            static INIT_RESOURCES: Once = Once::new();
            INIT_RESOURCES.call_once(|| {
                static PAYLOAD : &'static [u8] = & [ #payload ];
                static NAMES : &'static [u8] = & [ #(#names),* ];
                static TREE_DATA : &'static [u8] = & [ #(#tree_data),* ];
                unsafe { ::qmetaobject::qrc::register_resource_data(2, TREE_DATA, NAMES, PAYLOAD) };
                // Because we want that the macro re-compiles if the contents of the file changes!
                #({ const _X: &'static [ u8 ] = include_bytes!(#files); })*
            });
        }
    };
    //println!("{}", q.to_string());
    q.into()
}

pub fn process_qrc(source: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(source as QrcMacro);
    let mut tree = build_tree(parsed.data);
    tree.compute_offsets(1);
    let d = generate_data(&tree);
    expand_macro(&parsed.func, d)
}
