use syn;
use syn::parse::{Parse, Parser, ParseStream, Result};
use quote::ToTokens;
use proc_macro::TokenStream;
use ::std::iter::Iterator;
use super::qbjs;

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

fn builtin_type(name : &syn::Type) -> u32 {
    match name.clone().into_token_stream().to_string().as_ref() {
        "()" | "( )" => 43,
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
        "QString" => 10,
        "QByteArray" => 12,
        "QVariant" => 41,
        _ => 0
    }
}

fn write_u32(val : i32) -> [u8;4] {
    [(val & 0xff) as u8 , ((val >> 8) & 0xff) as u8, ((val >> 16) & 0xff) as u8, ((val >> 24) & 0xff) as u8]
}

#[derive(Clone)]
struct MetaMethodParameter {
    typ : syn::Type,
    name : Option<syn::Ident>
}

#[derive(Clone)]
struct MetaMethod {
    name: syn::Ident,
    args: Vec<MetaMethodParameter>,
    flags: u32,
    ret_type: syn::Type,
}

#[derive(Clone)]
struct MetaProperty {
    name : syn::Ident,
    typ : syn::Type,
    flags : u32,
    notify_signal : Option<syn::Ident>,
    getter : Option<syn::Ident>,
    setter : Option<syn::Ident>,
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
            let n = self.add_string(m.name.to_string());
            self.int_data.extend_from_slice(&[n , m.args.len() as u32, offset, 1, m.flags]);
            offset += 1 + 2 * m.args.len() as u32;
        }

        for ref p in properties {
            let n = self.add_string(p.name.to_string());
            let type_id = self.add_type(p.typ.clone());
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
            let ret_type = self.add_type(m.ret_type.clone());
            self.int_data.push(ret_type);
            // types
            for ref a in &m.args {
                let ty = self.add_type(a.typ.clone());
                self.int_data.push(ty);
            }
            // names
            for ref a in &m.args {
                let n = self.add_string(a.name.clone().into_token_stream().to_string());
                self.int_data.push(n);
            }
        }
    }

    fn add_type(&mut self, ty : syn::Type) -> u32 {
        let mut type_id = builtin_type(&ty);
        let string = ty.into_token_stream().to_string();
        if type_id == 0 {
            type_id = self.add_string(string) | 0x80000000 /*IsUnresolvedType */;
        }
        type_id
    }

    fn add_string(&mut self, string : String) -> u32 {
        self.string_data.push(string);
        return self.string_data.len() as u32 - 1;
    }
}

fn map_method_parameters(args : &syn::punctuated::Punctuated<syn::FnArg, Token![,]>) -> Vec<MetaMethodParameter> {
    args.iter().filter_map(|x| {
        match x {
            &syn::FnArg::Captured(ref cap) => Some(MetaMethodParameter {
                name: if let syn::Pat::Ident(ref id) = cap.pat { Some(id.ident.clone()) } else { None },
                typ: cap.ty.clone()
            }),
            _ => None
        }
    }).collect()
}

fn map_method_parameters2(args : &syn::punctuated::Punctuated<syn::BareFnArg, Token![,]>) -> Vec<MetaMethodParameter> {
    args.iter().filter_map(|x| {
        if let Some(ref name) = x.name {
            Some(MetaMethodParameter{
                name: if let syn::BareFnArgName::Named(ref id) = name.0 { Some(id.clone()) } else { None },
                typ: x.ty.clone()
            })
        } else {
            None
        }
    }).collect()
}

pub fn generate(input: TokenStream, is_qobject : bool) -> TokenStream {

    let ast = parse_macro_input!(input as syn::DeriveInput);

    let name = &ast.ident;

    let mut properties = vec![];
    let mut methods = vec![];
    let mut signals = vec![];
    let mut func_bodies = vec![];
    let mut is_plugin = false;
    let mut plugin_iid : Option<syn::LitStr> = None;

    let crate_ = super::get_crate(&ast);
    let mut base : syn::Ident = parse_quote!(QGadget);
    let mut base_prop : syn::Ident = parse_quote!(missing_base_class_property);

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    if let syn::Data::Struct(ref data) = ast.data {
        for f in data.fields.iter() {
            use syn::Type::Macro;
            if let Macro(ref mac) = f.ty {
                if let Some(ref segment) = mac.mac.path.segments.last() {
                    match segment.value().ident.to_string().as_ref() {
                        "qt_property" => {
                            #[derive(Debug)]
                            enum Flag {
                                Notify(syn::Ident),
                                Read(syn::Ident),
                                Write(syn::Ident),
                                Const,
                            }
                            impl Parse for Flag {
                                fn parse(input: ParseStream) -> Result<Self> {
                                    let k = input.parse::<syn::Ident>()?;
                                    if &k == "NOTIFY" {
                                        Ok(Flag::Notify(input.parse()?))
                                    } else if &k == "CONST" {
                                        Ok(Flag::Const)
                                    } else if &k == "READ" {
                                        Ok(Flag::Read(input.parse()?))
                                    } else if &k == "WRITE" {
                                        Ok(Flag::Write(input.parse()?))
                                    } else {
                                        Err(input.error("expected a property keyword"))
                                    }
                                }

                            }

                            let property_parser = |input: ParseStream| -> Result<(syn::Type, Vec<Flag>)> {
                                Ok((input.parse()?,
                                    input.parse::<Option<Token![;]>>()?.map(|_| -> Result<Vec<Flag>> {
                                        let mut r = Vec::<Flag>::new();
                                        while !input.is_empty() {
                                            r.push(input.parse()?)
                                        }
                                        Ok(r)
                                    }).unwrap_or(Ok(Default::default()))?))
                            };

                            let parsed = property_parser.parse(mac.mac.tts.clone().into())
                                .expect("Could not parse property");

                            let mut notify_signal = None;
                            let mut getter = None;
                            let mut setter = None;
                            let mut flags = 1 | 2 | 0x00004000 | 0x00001000 | 0x00010000;
                            for it in parsed.1 {
                                match it {
                                    Flag::Notify(signal) => {
                                        assert!(notify_signal.is_none(), "Two NOTIFY for a property");
                                        notify_signal = Some(signal);
                                        flags |= 0x00400000;
                                    }
                                    Flag::Const => {
                                        flags |= 0x00000400; // Constant
                                        flags &= !2; // Writable
                                    }
                                    Flag::Read(i) => {
                                        assert!(getter.is_none(), "Two READ for a property");
                                        getter = Some(i);
                                    }
                                    Flag::Write(i) => {
                                        assert!(setter.is_none(), "Two READ for a property");
                                        setter = Some(i);
                                    }
                                }
                            }
                            properties.push(MetaProperty {
                                name: f.ident.clone().expect("Property does not have a name"),
                                typ: parsed.0,
                                flags: flags,
                                notify_signal: notify_signal,
                                getter: getter,
                                setter: setter,
                            });
                        }
                        "qt_method" => {

                            let name = f.ident.clone().expect("Method does not have a name");

                            let (output, args) =
                                if let Ok(method_ast) = syn::parse::<syn::ItemFn>(mac.mac.tts.clone().into()) {
                                    assert_eq!(method_ast.ident, name);
                                    let tts = &mac.mac.tts;
                                    func_bodies.push(quote! { #tts } );
                                    let args = map_method_parameters(&method_ast.decl.inputs);
                                    (method_ast.decl.output, args)
                                } else { if let Ok(method_decl) = syn::parse::<syn::TypeBareFn>(mac.mac.tts.clone().into()) {
                                    let args = map_method_parameters2(&method_decl.inputs);
                                    (method_decl.output, args)
                                } else {
                                    panic!("Cannot parse qt_method {}", name);
                                }};

                            let ret_type = match output {
                                syn::ReturnType::Default => parse_quote!{()},
                                syn::ReturnType::Type(_, ref typ) =>  (**typ).clone()
                            };
                            methods.push(MetaMethod {
                                name: name,
                                args: args,
                                flags: 0x2,
                                ret_type: ret_type,
                            });
                        }
                        "qt_signal" => {
                            let parser = syn::punctuated::Punctuated::<syn::FnArg, Token![,]>::parse_terminated;
                            let args_list = parser.parse(mac.mac.tts.clone().into()).expect("Could not parse signal");
                            let args = map_method_parameters(&args_list);
                            signals.push(MetaMethod {
                                name: f.ident.clone().expect("Signal does not have a name"),
                                args: args,
                                flags: 0x2 | 0x4,
                                ret_type: parse_quote!{()},
                            });
                        }
                        "qt_base_class" => {
                            let parser = |input: ParseStream| -> Result<syn::Ident> {
                                input.parse::<Token![trait]>()?;
                                input.parse()
                            };
                            base = parser.parse(mac.mac.tts.clone().into()).expect("Could not parse base trait");
                            base_prop = f.ident.clone().expect("base prop needs a name");
                        }
                        "qt_plugin" => {
                            is_plugin = true;
                            let iid : syn::LitStr = syn::parse(mac.mac.tts.clone().into()).expect("Could not parse q_plugin iid");
                            plugin_iid = Some(iid);
                        }
                        _ => {}
                    }
                }
            }
            for i in f.attrs.iter() {
                if let Some(x) = i.interpret_meta() {
                    match x.name().to_string().as_ref()  {
                        "qt_base_class" => {
                            if let syn::Meta::NameValue(mnv) = x {
                                if let syn::Lit::Str(s) = mnv.lit {
                                    base = syn::parse_str(&s.value()).expect("invalid qt_base_class");
                                    base_prop = f.ident.clone().expect("base prop needs a name");
                                } else { panic!("Can't parse qt_base_class"); }
                            } else { panic!("Can't parse qt_base_class"); }
                        }
                        _ => {}
                    }
                }
            }
        }
    } else {
        //Nope. This is an Enum. We cannot handle these!
       panic!("#[derive(QObject)] is only defined for structs, not for enums!");
    }

    // prepend the methods in the signal
    let mut methods2 = signals.clone();
    methods2.extend(methods);
    let methods = methods2;

    let mut mo : MetaObject = Default::default();
    mo.compute_int_data(name.to_string(), &properties, &methods, signals.len());

    let str_data = mo.build_string_data();
    let int_data = mo.int_data;


    use self::MetaObjectCall::*;

    let get_object = if is_qobject {
        quote!{ <#name #ty_generics as #base>::get_rust_object(&mut *o) }
    } else {
        quote!{ ::std::mem::transmute::<*mut ::std::os::raw::c_void, &mut #name #ty_generics>(o) }
    };


    let property_meta_call : Vec<_> = properties.iter().enumerate().map(|(i, prop)| {
        let i = i as u32;
        let property_name = &prop.name;
        let typ = &prop.typ;


        let mut notify = quote! {};
        if let Some(ref signal) = prop.notify_signal {
            let signal : syn::Ident = signal.clone().into();
            notify = quote!{ obj.#signal() };
        }

        let register_type = if builtin_type(&prop.typ) == 0 {
            let typ_str = typ.clone().into_token_stream().to_string();
            let typ_str = typ_str.as_bytes();
            quote! {
                #RegisterPropertyMetaType => unsafe {
                    let r = *a as *mut i32;
                    *r = <#typ as #crate_::PropertyType>::register_type(
                        ::std::ffi::CStr::from_bytes_with_nul_unchecked(&[#(#typ_str ,)* 0u8]) );
                }
            }
        } else { quote!{} };

        let getter = if let Some(ref getter) = prop.getter {
            let getter_ident : syn::Ident = getter.clone().into();
            quote!{
                let mut tmp : #typ = obj.#getter_ident();
                <#typ as #crate_::PropertyType>::pass_to_qt(&mut tmp, *a);
            }
        } else {
            quote!{ <#typ as #crate_::PropertyType>::pass_to_qt(&mut obj.#property_name, *a); }
        };

        let setter = if let Some(ref setter) = prop.setter {
            let setter_ident : syn::Ident = setter.clone().into();
            quote!{
                obj.#setter_ident(<#typ as #crate_::PropertyType>::read_from_qt(*a));
            }
        } else {
            quote! {
                obj.#property_name = <#typ as #crate_::PropertyType>::read_from_qt(*a);
                #notify
            }
        };

        quote! { #i => match c {
            #ReadProperty => unsafe {
                let obj : &mut #name = #get_object;
                #getter
            },
            #WriteProperty => unsafe {
                let obj : &mut #name = #get_object;
                #setter
            },
            #ResetProperty => { /* TODO */},
            #register_type
            _ => {}
        }}
    }).collect();

    let method_meta_call : Vec<_> = methods.iter().enumerate().map(|(i, method)| {
        let i = i as u32;
        let method_name : syn::Ident = method.name.clone().into();
        let args_call : Vec<_> = method.args.iter().enumerate().map(|(i, arg)| {
            let i = i as isize;
            let ty = &arg.typ;
            quote! {
                (*(*(a.offset(#i + 1)) as *const #ty)).clone()
            }
        }).collect();

        fn is_void(ret_type : &syn::Type) -> bool {
            if let syn::Type::Tuple(ref tuple) = ret_type {
                return tuple.elems.len() == 0;
            } else {
                return false;
            }
        }

        if is_void(&method.ret_type) {
            quote! { #i => obj.#method_name(#(#args_call),*), }
        } else {
            let ret_type = &method.ret_type;
            let args_call2 = args_call.clone();
            quote! { #i => {
                    let r = *a as *mut #ret_type;
                    if r.is_null() { obj.#method_name(#(#args_call),*); }
                    else { *r = obj.#method_name(#(#args_call2),*); }
                }
            }
        }
    }).collect();

    let register_arguments : Vec<_> = methods.iter().enumerate().map(|(i, method)| {
        let i = i as u32;
        let args : Vec<_> = method.args.iter().enumerate().map(|(i, arg)| {
            let i = i as u32;
            if builtin_type(&arg.typ) == 0 {
                let typ = &arg.typ;
                let typ_str = arg.typ.clone().into_token_stream().to_string();
                let typ_str = typ_str.as_bytes();
                quote! {
                    #i => { unsafe { *(*a as *mut i32) = <#typ as #crate_::QMetaType>::register(
                        Some(::std::ffi::CStr::from_bytes_with_nul_unchecked(&[#(#typ_str ,)* 0u8]))) }; }
                }
            } else {
                quote! {}
            }
        }).collect();

        quote! { #i => {
            match unsafe {*(*(a.offset(1)) as *const u32)} {
                #(#args)*
                _ => {}
            }
        }}
    }).collect();

    func_bodies.extend(signals.iter().enumerate().map(|(i, signal)| {
        let sig_name  = &signal.name;
        let i = i as u32;
        let args_decl : Vec<_> = signal.args.iter().map(|arg| {
            // FIXME!  we should probably use the signature verbatim
            let n = &arg.name;
            let ty = &arg.typ;
            quote! { #n : #ty }
        }).collect();
        let args_ptr : Vec<_> = signal.args.iter().map(|arg| {
            let n = &arg.name;
            let ty = &arg.typ;
            quote! { unsafe { ::std::mem::transmute::<& #ty , *mut ::std::os::raw::c_void>(& #n) } }
        }).collect();
        let array_size = signal.args.len() + 1;
        quote! {
            #[allow(non_snake_case)]
            fn #sig_name(&mut self #(, #args_decl)*) {
                let a : [*mut ::std::os::raw::c_void; #array_size] = [ ::std::ptr::null_mut() #(, #args_ptr)* ];
                unsafe { #crate_::invoke_signal((self as &mut #crate_::QObject).get_cpp_object(), #name::static_meta_object(), #i, &a) }
            }
        }
    }));

    let index_of_method = signals.iter().enumerate().map(|(i, signal)| {
        let sig_name = &signal.name;
        // if *a[1] == offset_of(signal field)  =>  *a[0] = index and return.
        quote! {
            unsafe {
                let null = ::std::ptr::null() as *const #name #ty_generics;
                let offset = &(*null).#sig_name as *const _ as usize - (null as usize);
                if (*(*(a.offset(1)) as *const usize)) == offset  {
                    *(*a as *mut i32) = #i as i32;
                    return;
                }
            }
        }
    });

    let base_meta_object = if is_qobject {
        quote!{ <#name #ty_generics as #base>::get_object_description().meta_object }
    } else {
        quote!{ ::std::ptr::null() }
    };

    let mo = if ast.generics.params.is_empty() {
        quote! {
            qmetaobject_lazy_static! { static ref MO: #crate_::QMetaObject = #crate_::QMetaObject {
                superdata: #base_meta_object,
                string_data: STRING_DATA.as_ptr(),
                data: INT_DATA.as_ptr(),
                static_metacall: static_metacall,
                r: ::std::ptr::null(),
                e: ::std::ptr::null(),
            };};
            return &*MO;
        }
    } else {
        let turbo_generics = ty_generics.as_turbofish();
        let (ty_generics, turbo_generics) = if ast.generics.type_params().count() != 0 {
            (quote!(#ty_generics), quote!(#turbo_generics))
        } else {
            (quote!(), quote!() )
        };
        quote! {
            use ::std::sync::Mutex;
            use ::std::collections::HashMap;
            use ::std::any::TypeId;

            // FIXME! this could be global
            qmetaobject_lazy_static! {
                static ref HASHMAP: Mutex<HashMap<TypeId, Box<#crate_::QMetaObject>>> =
                    Mutex::new(HashMap::new());
            };
            let mut h = HASHMAP.lock().unwrap();
            let mo = h.entry(TypeId::of::<#name #ty_generics>()).or_insert_with(
                || Box::new(#crate_::QMetaObject {
                    superdata: #base_meta_object,
                    string_data: STRING_DATA.as_ptr(),
                    data: INT_DATA.as_ptr(),
                    static_metacall: static_metacall #turbo_generics,
                    r: ::std::ptr::null(),
                    e: ::std::ptr::null(),
            }));
            return &**mo;
        }
    };

    let qobject_spec_func = if is_qobject {
        quote!{
            fn get_cpp_object(&self)-> *mut ::std::os::raw::c_void {
                self.#base_prop.get()
            }
            unsafe fn get_from_cpp(ptr: *const ::std::os::raw::c_void) -> *const Self {
                if ptr.is_null() { return ::std::ptr::null(); }
                <#name #ty_generics as #base>::get_rust_object(&mut *(ptr as *mut ::std::os::raw::c_void)) as *const Self
            }

            unsafe fn cpp_construct(&mut self) -> *mut ::std::os::raw::c_void {
                assert!(self.#base_prop.get().is_null());
                let trait_object : *const #base = self;
                let trait_object_ptr : *const *const #base = &trait_object;
                let n = (<#name #ty_generics as #base>::get_object_description().create)
                    (trait_object_ptr as *const ::std::os::raw::c_void);
                self.#base_prop.set(n);
                n
            }

            unsafe fn qml_construct(&mut self, mem : *mut ::std::os::raw::c_void,
                                    extra_destruct : extern fn(*mut ::std::os::raw::c_void)) {
                let trait_object : *const #base = self;
                let trait_object_ptr : *const *const #base = &trait_object;
                self.#base_prop.set(mem);
                (<#name #ty_generics as #base>::get_object_description().qml_construct)(
                    mem, trait_object_ptr as *const ::std::os::raw::c_void, extra_destruct);
            }

            fn cpp_size() -> usize {
                <#name #ty_generics as #base>::get_object_description().size
            }
        }
    } else {
        quote!{ }
    };

    let trait_name = if is_qobject { quote!{ QObject } } else { quote!{ QGadget } };

    let mut body =   quote!{
        #[allow(non_snake_case)]
        impl #impl_generics #name #ty_generics #where_clause {
            #(#func_bodies)*
        }
        impl #impl_generics #crate_::#trait_name for #name #ty_generics #where_clause {
            fn meta_object(&self)->*const #crate_::QMetaObject {
                Self::static_meta_object()
            }

            fn static_meta_object()->*const #crate_::QMetaObject {

                static STRING_DATA : &'static [u8] = & [ #(#str_data),* ];
                static INT_DATA : &'static [u32] = & [ #(#int_data),* ];

                #[allow(unused_variables)]
                extern "C" fn static_metacall #impl_generics (o: *mut ::std::os::raw::c_void, c: u32, idx: u32,
                                              a: *const *mut ::std::os::raw::c_void) #where_clause {
                    if c == #InvokeMetaMethod { unsafe {
                        let obj : &mut #name #ty_generics = #get_object;
                        match idx {
                            #(#method_meta_call)*
                            _ => { let _ = obj; }
                        }
                    }} else if c == #IndexOfMethod {
                        #(#index_of_method)*
                    } else if c == #RegisterMethodArgumentMetaType {
                        match idx {
                            #(#register_arguments)*
                            _ => {}
                        }
                    } else {
                        match idx {
                            #(#property_meta_call)*
                            _ => {}
                        }
                    }
                }
                #mo
            }

            #qobject_spec_func

        }

    };

    if is_qobject {
        body = quote! { #body
            impl #impl_generics #crate_::PropertyType for #name #ty_generics #where_clause {
                const READ_ONLY : bool = true;
                fn register_type(_name : &::std::ffi::CStr) -> i32 {
                    #crate_::register_metatype_qobject::<Self>()
                }
                unsafe fn pass_to_qt(&mut self, a: *mut ::std::os::raw::c_void) {
                    let r = a as *mut *const ::std::os::raw::c_void;
                    let obj = (self as &mut #crate_::QObject).get_cpp_object();
                    *r = if !obj.is_null() { obj }
                        else { (self as  &mut #crate_::QObject).cpp_construct() };
                }

                unsafe fn read_from_qt(_a: *const ::std::os::raw::c_void) -> Self {
                    panic!("Cannot write into an Object property");
                }
            }
        }
    }

    if is_plugin {
        use qbjs::Value;
        let mut object_data : Vec<(&'static str, Value)> = vec![
            ("IID", Value::String(plugin_iid.unwrap().value())),
            ("className", Value::String(name.to_string())),
            ("version", Value::Double(0x050100 as f64)),
            ("debug", Value::Double(0.0 as f64)),
//            ("MetaData"] = CDef->Plugin.MetaData;
        ];
        object_data.sort_by(|a, b| a.0.cmp(b.0));


        let plugin_data = qbjs::serialize(&object_data);
        let plugin_data_size = plugin_data.len();
        body = quote! { #body
            #[link_section = ".qtmetadata"]
            #[no_mangle]
            #[allow(non_upper_case_globals)]
            pub static qt_pluginMetaData: [u8 ; 20 + #plugin_data_size] = [
                b'Q', b'T', b'M', b'E', b'T', b'A', b'D', b'A', b'T', b'A', b' ', b' ',
                b'q', b'b', b'j', b's', 1, 0, 0, 0,
                #(#plugin_data),*
            ];

            #[no_mangle]
            pub extern fn qt_plugin_query_metadata() -> *const u8
            {  qt_pluginMetaData.as_ptr() }

            #[no_mangle]
            pub extern fn qt_plugin_instance() -> *mut ::std::os::raw::c_void
            {
                #crate_::into_leaked_cpp_ptr(#name::default())
            }

        }
    }
    body.into()
}
