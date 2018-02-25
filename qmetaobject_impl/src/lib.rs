#![recursion_limit="256"]

#[macro_use]
extern crate syn;
use syn::synom::Parser;
#[macro_use]
extern crate quote;

use quote::ToTokens;

extern crate proc_macro;
use proc_macro::TokenStream;

use std::iter::Iterator;


#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(dead_code)]
mod MetaObjectCall {
    // QMetaObject::Call
    pub const InvokeMetaMethod               : u32 = 0;
    pub const ReadProperty                   : u32 = 1;
    pub const WriteProperty                  : u32 = 2;
    pub const ResetProperty                  : u32 = 3;
    pub const QueryPropertyDesignable        : u32 = 4;
    pub const QueryPropertyScriptable        : u32 = 5;
    pub const QueryPropertyStored            : u32 = 6;
    pub const QueryPropertyEditable          : u32 = 7;
    pub const QueryPropertyUser              : u32 = 8;
    pub const CreateInstance                 : u32 = 9;
    pub const IndexOfMethod                  : u32 = 10;
    pub const RegisterPropertyMetaType       : u32 = 11;
    pub const RegisterMethodArgumentMetaType : u32 = 12;
}

fn builtin_type(name : &str) -> u32 {
    match name {
        "()" => 43,
        "bool" => 1,
        "i32" => 2,
        "u32" => 3,
        "i64" => 4,
        "u64" => 5,
        "f64" => 6,
        "i16" => 33,
        "i8" => 34,
        "u16" => 36,
        "u8" => 37,
        "f32" => 38,
        //"*c_void" => 31,
        _ => 0
    }
}


fn write_u32(val : i32) -> [u8;4] {
    [(val & 0xff) as u8 , ((val >> 8) & 0xff) as u8, ((val >> 16) & 0xff) as u8, ((val >> 24) & 0xff) as u8]
}

#[derive(Clone)]
struct MetaMethodParameter {
    typ : String,
    name : String
}

#[derive(Clone)]
struct MetaMethod {
    name: String,
    args: Vec<MetaMethodParameter>,
    flags: u32,
    ret_type: String,
}

#[derive(Clone)]
struct MetaProperty {
    name : String,
    typ : String,
    flags : u32,
    notify_signal : Option<String>,
}

#[derive(Default)]
struct MetaObject {
    int_data : Vec<u32>,
    string_data : Vec<String>,
}
impl MetaObject {
    fn build_string_data(&self) -> Vec<u8> {
        let mut result : Vec<u8> = Vec::new();
        let sizeof_qbytearraydata = 24;
        let mut ofs = sizeof_qbytearraydata * self.string_data.len() as i32;
        for ref s in &self.string_data {
            result.extend_from_slice(&write_u32(-1)); // ref (-1)
            result.extend_from_slice(&write_u32(s.len() as i32)); // size
            result.extend_from_slice(&write_u32(0)); // alloc / capacityReserved
            result.extend_from_slice(&write_u32(0)); // padding
            result.extend_from_slice(&write_u32(ofs)); // offset (LSB)
            result.extend_from_slice(&write_u32(0)); // offset (MSB)

            ofs += s.len() as i32 + 1; // +1 for the '\0'
            ofs -= sizeof_qbytearraydata;
        }

        for ref s in &self.string_data {
            result.extend_from_slice(s.as_bytes());
            result.push(0); // null terminated
        }
        return result;
    }

    fn compute_int_data(&mut self, class_name: String, properties : &[MetaProperty],
                        methods : &[MetaMethod], signal_count : usize) {


        let has_notify = properties.iter().any(|p| p.notify_signal.is_some());

        self.add_string(class_name.clone());
        self.add_string("".to_owned());

        let mut offset = 14;
        let property_offset = offset + methods.len() as u32 * 5;
        //...


        self.int_data.extend_from_slice(&[
            7, // revision
            0, // classname
            0, 0, // class info count and offset
            methods.len() as u32, offset, // method count and offset
            properties.len() as u32, property_offset, // properties count and offset
            0, 0, // enum count and offset
            0, 0, // constructor count and offset
            0x4 /* PropertyAccessInStaticMetaCall */,   // flags
            signal_count as u32, // signalCount
        ]);

        offset = property_offset + properties.len() as u32 * (if has_notify {4} else {3});

        for ref m in methods {
            let n = self.add_string(m.name.clone());
            self.int_data.extend_from_slice(&[n , m.args.len() as u32, offset, 1, m.flags]);
            offset += 1 + 2 * m.args.len() as u32;
        }

        for ref p in properties {
            let n = self.add_string(p.name.clone());
            let type_id = self.add_type(&p.typ);
            self.int_data.extend_from_slice(&[n , type_id, p.flags]);
        }
        if has_notify {
            for ref p in properties {
                match p.notify_signal {
                    None => self.int_data.push(0 as u32),
                    Some(ref signal) => self.int_data.push(methods.iter().position(
                        |x| x.name == *signal && (x.flags & 0x4) != 0).expect("Invalid NOTIFY signal") as u32)

                }
                ;
            }
        }

        for ref m in methods {
            // return type
            let ret_type = self.add_type(&m.ret_type);
            self.int_data.push(ret_type);
            // types
            for ref a in &m.args {
                let ty = self.add_type(&a.typ);
                self.int_data.push(ty);
            }
            // names
            for ref a in &m.args {
                let n = self.add_string(a.name.clone());
                self.int_data.push(n);
            }
        }
    }

    fn add_type(&mut self, string : &str) -> u32 {
        let mut type_id = builtin_type(string);
        if type_id == 0 {
            type_id = self.add_string(string.to_owned()) | 0x80000000 /*IsUnresolvedType */;
        }
        type_id
    }

    fn add_string(&mut self, string : String) -> u32 {
        self.string_data.push(string);
        return self.string_data.len() as u32 - 1;
    }
}

fn map_method_parameters(args : &syn::punctuated::Punctuated<syn::FnArg, Token![,]>) -> Vec<MetaMethodParameter> {
    args.iter().map(|x| {
        match x {
            &syn::FnArg::Captured(ref cap) => MetaMethodParameter{
                name: format!("{}",cap.pat.clone().into_tokens()),
                typ: format!("{}",cap.ty.clone().into_tokens())
            },
            _ => MetaMethodParameter{ name: "".to_owned(), typ:"()".to_owned()  }
        }
    }).filter(|x| x.typ != "()" ).collect()
}

#[proc_macro_derive(QObject)]
pub fn qobject_impl(input: TokenStream) -> TokenStream {

    let ast : syn::DeriveInput = syn::parse(input).expect("could not parse struct");
    let name = &ast.ident;

    let mut properties = vec![];
    let mut methods = vec![];
    let mut signals = vec![];
    let mut func_bodies = vec![];

    if let syn::Data::Struct(ref data) = ast.data {
        for f in data.fields.iter() {
            use syn::Type::Macro;
            if let Macro(ref mac) = f.ty {
                if let Some(ref segment) = mac.mac.path.segments.last() {
                    match segment.value().ident.as_ref() {
                        "qt_property" => {
                            #[derive(Debug)]
                            enum Flag {
                                Notify(syn::Ident),
                            }
                            named!(property_flag -> Flag, do_parse!(
                                k: syn!(syn::Ident) >>
                                i: cond_reduce!(&k == "NOTIFY", syn!(syn::Ident)) >>
                                (Flag::Notify(i))));
                            named!(property_parser -> (syn::Type, Vec<Flag>), do_parse!(
                                ty: syn!(syn::Type) >>
                                //trail: call!(syn::punctuated::Punctuated::<syn::FnArg, Token![,]>::parse_separated_with, property_flags) >>
                                trail: many0!(do_parse!(
                                    punct!(;) >>
                                    flag: call!(property_flag) >>
                                    (flag)
                                )) >>
                                ((ty, trail))));
                            let parsed = property_parser.parse(mac.mac.tts.clone().into())
                                .expect("Could not parse property");

                            let mut notify_signal = None;
                            let mut flags = 1 | 2 | 0x00004000 | 0x00001000 | 0x00010000;
                            for it in parsed.1 {
                                match it {
                                    Flag::Notify(signal) => {
                                        assert!(notify_signal.is_none(), "Two NOTIFY for a property");
                                        notify_signal = Some(signal.as_ref().to_string());
                                        flags |= 0x00400000;
                                    }
                                }
                            }
                            properties.push(MetaProperty {
                                name: f.ident.expect("Property does not have a name").as_ref().to_string(),
                                typ: format!("{}", parsed.0.into_tokens()),
                                flags: flags,
                                notify_signal: notify_signal
                            });
                        }
                        "qt_method" => {

                            let method_ast : syn::ItemFn = syn::parse(mac.mac.tts.clone().into())
                                .expect("Could not parse method");
                            // TODO: compare f.ident and method_ast.ident
                            let ret_type = match method_ast.decl.output {
                                syn::ReturnType::Default => "()".to_owned(),
                                syn::ReturnType::Type(_, ref typ) =>  format!("{}",typ.clone().into_tokens())
                            };
                            let args = map_method_parameters(&method_ast.decl.inputs);
                            methods.push(MetaMethod {
                                name: f.ident.expect("Method does not have a name").as_ref().to_string(),
                                args: args,
                                flags: 0x2,
                                ret_type: ret_type,
                            });
                            let tts = &mac.mac.tts;
                            func_bodies.push(quote! { #tts } );
                        }
                        "qt_signal" => {
                            let parser = syn::punctuated::Punctuated::<syn::FnArg, Token![,]>::parse_separated;
                            let args_list = parser.parse(mac.mac.tts.clone().into()).expect("Could not parse signal");
                            let args = map_method_parameters(&args_list);
                            signals.push(MetaMethod {
                                name: f.ident.expect("Signal does not have a name").as_ref().to_string(),
                                args: args,
                                flags: 0x2 | 0x4,
                                ret_type: "()".to_owned(),
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    } else {
        //Nope. This is an Enum. We cannot handle these!
       panic!("#[derive(HelloWorld)] is only defined for structs, not for enums!");
    }

    // prepend the methods in the signal
    let mut methods2 = signals.clone();
    methods2.extend(methods);
    let methods = methods2;

    let mut mo : MetaObject = Default::default();
    mo.compute_int_data(name.to_string(), &properties, &methods, signals.len());

    let str_data = mo.build_string_data();
    let int_data = mo.int_data;

    let crate_ : syn::Ident = "qmetaobject".to_owned().into();
    let base : syn::Ident = "QObject".to_owned().into();


    use MetaObjectCall::*;


    let property_meta_call : Vec<_> = properties.iter().enumerate().map(|(i, prop)| {
        let i = i as u32;
        let property_name : syn::Ident = prop.name.clone().into();
        let typ : syn::Ident = prop.typ.clone().into();
        let mut notify = quote! {};
        if let Some(ref signal) = prop.notify_signal {
            let signal : syn::Ident = signal.clone().into();
            notify = quote!{ obj.#signal() };
        }

        quote! { #i => match c {
            #ReadProperty => unsafe {
                let obj : &mut #name = <#name as #base>::get_rust_object(&mut *o);
                let r = *a as *mut #typ;
                *r = obj.#property_name.clone();
            },
            #WriteProperty => unsafe {
                let obj : &mut #name = <#name as #base>::get_rust_object(&mut *o);
                let r = *a as *mut #typ;
                obj.#property_name = (*r).clone();
                #notify
            },
            #ResetProperty => { /* TODO */},
            #RegisterPropertyMetaType => unsafe {
                let r = *a as *mut i32;
                *r = #crate_::register_metatype::<#typ>(stringify!(#typ));
            },
            _ => {}
        }}
    }).collect();

    let method_meta_call : Vec<_> = methods.iter().enumerate().map(|(i, method)| {
        let i = i as u32;
        let method_name : syn::Ident = method.name.clone().into();
        let args_call : Vec<_> = method.args.iter().enumerate().map(|(i, arg)| {
            let i = i as isize;
            let ty : syn::Ident = arg.typ.clone().into();
            quote! {
                *(*(a.offset(#i + 1)) as *const #ty)
            }
        }).collect();

        if method.ret_type == "()" /* Void */ {
            quote! { #i => obj.#method_name(#(#args_call),*), }
        } else {
            let ret_type : syn::Ident = method.ret_type.clone().into();
            let args_call2 = args_call.clone();
            quote! { #i => {
                    let r = *a as *mut #ret_type;
                    if r.is_null() { obj.#method_name(#(#args_call),*); }
                    else { *r = obj.#method_name(#(#args_call2),*); }
                }
            }
        }
    }).collect();

    func_bodies.extend(signals.iter().enumerate().map(|(i, signal)| {
        let sig_name : syn::Ident = signal.name.clone().into();
        let i = i as u32;
        let args_decl : Vec<_> = signal.args.iter().map(|arg| {
            // FIXME!  we should probably use the signature verbatim
            let n : syn::Ident = arg.name.clone().into();
            let ty : syn::Ident = arg.typ.clone().into();
            quote! { #n : #ty }
        }).collect();
        let args_ptr : Vec<_> = signal.args.iter().map(|arg| {
            let n : syn::Ident = arg.name.clone().into();
            let ty : syn::Ident = arg.typ.clone().into();
            quote! { unsafe { std::mem::transmute::<& #ty , *mut std::os::raw::c_void>(& #n) } }
        }).collect();
        let array_size = signal.args.len() + 1;
        quote! {
            #[allow(non_snake_case)]
            fn #sig_name(&mut self #(, #args_decl)*) {
                let a : [*mut std::os::raw::c_void; #array_size] = [ std::ptr::null_mut() #(, #args_ptr)* ];
                #crate_::invoke_signal(self.get_cpp_object().ptr, #name::static_meta_object(), #i, &a)
            }
        }
    }));

    let body =   quote!{
        impl #name {
            #(#func_bodies)*
        }
        impl QObject for #name {
            fn meta_object(&self)->*const #crate_::QMetaObject {
                Self::static_meta_object()
            }

            fn static_meta_object()->*const #crate_::QMetaObject {

                static STRING_DATA : &'static [u8] = & [ #(#str_data),* ];
                static INT_DATA : &'static [u32] = & [ #(#int_data),* ];

                extern "C" fn static_metacall(o: *mut std::os::raw::c_void, c: u32, idx: u32,
                                              a: *const *mut std::os::raw::c_void) {
                    if c == #InvokeMetaMethod { unsafe {
                        let obj : &mut #name = <#name as #base>::get_rust_object(&mut *o);
                        match idx {
                            #(#method_meta_call)*
                            _ => {}
                        }
                    }} else {
                        match idx {
                            #(#property_meta_call)*
                            _ => {}
                        }
                    }
                }

                lazy_static! { static ref MO: #crate_::QMetaObject = #crate_::QMetaObject {
                    superdata:  <#name as #base>::base_meta_object(),
                    string_data: STRING_DATA.as_ptr(),
                    data: INT_DATA.as_ptr(),
                    static_metacall: static_metacall,
                    r: std::ptr::null(),
                    e: std::ptr::null(),
                };};
                return &*MO;
            }

            fn get_cpp_object<'a>(&'a mut self)->&'a mut #crate_::QObjectCppWrapper {
                &mut self.base
            }
        }

    };

    body.into()
}


