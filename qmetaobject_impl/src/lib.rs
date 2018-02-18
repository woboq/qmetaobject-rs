#![recursion_limit="256"]
#[macro_use]
extern crate proc_macro_hack;

#[macro_use]
extern crate synstructure;
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
/*
struct MetaProperty {
    name : String,
    typ : i32,
}
*/
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

    fn compute_int_data(&mut self, methods : Vec<MetaMethod>) {

        let offset = 14;


        self.add_string("MyClass".to_owned());
        self.add_string("".to_owned());

        self.int_data.extend_from_slice(&[
            7, // revision
            0, // classname
            0, 0, // class info count and offset
            methods.len() as i32, offset, // method count and offset
            0, 0, // properties count and offset
            0, 0, // enum count and offset
            0, 0, // constructor count and offset
            0x4 /* PropertyAccessInStaticMetaCall */,   // flags
            0, // signalCount
        ]);

        let param_offest = offset + methods.len() as i32 * 5;

        for ref m in &methods {
            let n = self.add_string(m.name.clone());
            self.int_data.extend_from_slice(&[n , m.args.len() as i32, param_offest, 1, m.flags]);
        }

        for ref m in &methods {
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



//decl_derive!([QObject] => qobject_impl);
//fn qobject_impl(_s: synstructure::Structure) -> quote::Tokens {

#[proc_macro_derive(QObject)]
pub fn qobject_impl(input: TokenStream) -> TokenStream {


    //s.bound_impl(
   //     quote!{QObject},

   println!("{:?}", input.to_string());

   let m = MetaMethod {
            name: "xx".to_owned(),
            args: Vec::new(),
            flags: 0x2,
            ret_type: 2 // int
    };
    let mut mo : MetaObject = Default::default();
    mo.compute_int_data(vec![m]);

    //let str_data = data.iter().map(|x| x.to_string()).join(", ");
    //let int_data = mo.int_data.iter().map(|x| x.to_string()).join(", ");

    let str_data = mo.build_string_data();//.iter().map(|x| x.to_string()).fold("".to_owned(), |a,b| a + &b + ", ");
    let int_data = mo.int_data;//.iter().map(|x| x.to_string()).fold("".to_owned(), |a,b| a + &b + ", ");

   // let int_size = mo.int_data.size();

  // println!("[{}] - [{}]", str_data, int_data);


    let body =   quote!{
impl QObject for MyStruct {
    fn meta_object(&self)->*const QMetaObject {

        static STRING_DATA : &'static [u8] = & [ #(#str_data),* ];
        static INT_DATA : &'static [i32] = & [ #(#int_data),* ];


        extern "C" fn static_metacall(o: *mut
        c_void, c: u32, idx: u32, a: *const *mut c_void) {
            // get the actual object
            //std::mem::transmute::<*mut c_void, *mut u8>(*a)
            let obj = unsafe { std::mem::transmute::<*mut c_void, &mut MyStruct>(
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
            }
            /*//println!("MyStruct.foo Called {}, {}", c, idx );
            unsafe {
                cpp!{[a as "int**"]{ *a[0] = 42; }}
            }*/
        }


        let x = Box::new(QMetaObject {
            superdata: Self::base_meta_object(),
            string_data: STRING_DATA.as_ptr(),
            data: INT_DATA.as_ptr(),
            static_metacall: static_metacall,
            r: std::ptr::null(),
            e: std::ptr::null(),
        });
        return Box::into_raw(x);
    }
}

        };
  //  )

  body.into()
}



/*
#[proc_macro_derive(QObject)]
pub fn qobject_impl(input: TokenStream) -> TokenStream {

}*/

