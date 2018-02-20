#![recursion_limit="256"]
#[macro_use]
extern crate proc_macro_hack;

// #[macro_use]
// extern crate synstructure;
extern crate syn;
#[macro_use]
extern crate quote;

extern crate proc_macro;
use proc_macro::TokenStream;

proc_macro_expr_impl! {
    /// Add one to an expression.
    pub fn add_one_impl(input: &str) -> String {
        format!("1 + {}", input)
    }
}

fn write_u32(val : i32) -> [u8;4] {
    [(val & 0xff) as u8 , ((val >> 8) & 0xff) as u8, ((val >> 16) & 0xff) as u8, ((val >> 24) & 0xff) as u8]
}

struct MetaMethodParameter {
    typ : i32,
    name : String
}

struct MetaMethod {
    name: String,
    args: Vec<MetaMethodParameter>,
    flags: i32,
    ret_type: i32,
}

struct MetaProperty {
    name : String,
    typ : i32,
    flags : i32,
}

#[derive(Default)]
struct MetaObject {
    int_data : Vec<i32>,
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

    fn compute_int_data(&mut self, class_name: String, properties : &[MetaProperty], methods : &[MetaMethod]) {


        let has_notify = false;

        self.add_string(class_name.clone());
        self.add_string("".to_owned());

        let offset = 14;
        let property_offset = offset + methods.len() as i32 * 5;
        //...
        let param_offest = property_offset + properties.len() as i32 * (if has_notify {4} else {3});

        self.int_data.extend_from_slice(&[
            7, // revision
            0, // classname
            0, 0, // class info count and offset
            methods.len() as i32, offset, // method count and offset
            properties.len() as i32, property_offset, // properties count and offset
            0, 0, // enum count and offset
            0, 0, // constructor count and offset
            0x4 /* PropertyAccessInStaticMetaCall */,   // flags
            0, // signalCount
        ]);


        for ref m in methods {
            let n = self.add_string(m.name.clone());
            self.int_data.extend_from_slice(&[n , m.args.len() as i32, param_offest, 1, m.flags]);
        }

        for ref p in properties {
            let n = self.add_string(p.name.clone());
            self.int_data.extend_from_slice(&[n , p.typ, p.flags]);
        }

        for ref m in methods {
            // return type
            self.int_data.push(m.ret_type);
            // types
            for ref a in &m.args {
                self.int_data.push(a.typ);
            }
            // names
            for ref a in &m.args {
                let n = self.add_string(a.name.clone());
                self.int_data.push(n);
            }
        }
    }

    fn add_string(&mut self, string : String) -> i32 {
        self.string_data.push(string);
        return self.string_data.len() as i32 - 1;
    }
}


#[proc_macro_derive(QObject)]
pub fn qobject_impl(input: TokenStream) -> TokenStream {

    let ast : syn::DeriveInput = syn::parse(input).expect("could not parse struct");
    let name = &ast.ident;

    let mut properties = vec![];

    if let syn::Data::Struct(ref data) = ast.data {
        for f in data.fields.iter() {
            use syn::Type::Macro;
            if let Macro(ref mac) = f.ty {
                if let Some(ref segment) = mac.mac.path.segments.last() {
                    if segment.value().ident == "qt_property" {
                        properties.push(MetaProperty {
                            name: f.ident.expect("Property does not have a name").as_ref().to_string(),
                            typ: 3,
                            flags: 1 | 2 | 0x00004000 | 0x00001000,
                        });
                    }
                }
            }
        }
    } else {
        //Nope. This is an Enum. We cannot handle these!
       panic!("#[derive(HelloWorld)] is only defined for structs, not for enums!");
    }

    let m = MetaMethod {
            name: "xx".to_owned(),
            args: Vec::new(),
            flags: 0x2,
            ret_type: 2 // int
    };
    let mut mo : MetaObject = Default::default();
    mo.compute_int_data(name.to_string(), &properties, &vec![m]);

    let str_data = mo.build_string_data();
    let int_data = mo.int_data;

    let body =   quote!{
        impl QObject for #name {
            fn meta_object(&self)->*const QMetaObject {

                static STRING_DATA : &'static [u8] = & [ #(#str_data),* ];
                static INT_DATA : &'static [i32] = & [ #(#int_data),* ];

                extern "C" fn static_metacall(o: *mut
                c_void, c: u32, idx: u32, a: *const *mut c_void) {
                    // get the actual object
                    //std::mem::transmute::<*mut c_void, *mut u8>(*a)
                    let obj = unsafe { std::mem::transmute::<*mut c_void, &mut #name>(
                        o.offset(8/*virtual_table*/ + 8 /* d_ptr */)) }; // FIXME

                    if c == 0 /*QMetaObject::InvokeMetaMethod*/ {
                        match idx {
                            0 => {
                                unsafe {
                                    let r = std::mem::transmute::<*mut c_void, *mut i32>(*a);
                                    *r = obj.xx();
                                    //*r = foobar(*a);
                                }
                            },
                            _ => {}
                        }
                    } else if c == 1 /*QMetaObject::ReadProperty*/ {
                    }
                }

                lazy_static! { static ref MO: QMetaObject = QMetaObject {
                    superdata: #name::base_meta_object(),
                    string_data: STRING_DATA.as_ptr(),
                    data: INT_DATA.as_ptr(),
                    static_metacall: static_metacall,
                    r: std::ptr::null(),
                    e: std::ptr::null(),
                };};
                return &*MO;
            }
        }

    };
    body.into()
}


