#![recursion_limit="10240"]


#[macro_use]
extern crate cpp;

// extern crate libc;
use std::os::raw::c_void;
use std::mem;

// #![feature(trace_macros)]
// trace_macros!(true);

cpp!{{
    #include <QtQuick/QtQuick>
    #include <QtWidgets/QtWidgets>

    struct RustObject;
    extern "C" QMetaObject *RustObject_metaObject(const RustObject *);

    struct RustObject : QObject {
        void *data;
        const QMetaObject *metaObject() const override {
            return RustObject_metaObject(this);
        }
        int qt_metacall(QMetaObject::Call _c, int _id, void **_a) override {
            _id = QObject::qt_metacall(_c, _id, _a);
            if (_id < 0)
                return _id;
            const QMetaObject *mo = metaObject();
            if (_c == QMetaObject::InvokeMetaMethod || _c == QMetaObject::RegisterMethodArgumentMetaType) {
                int methodCount = mo->methodCount();
                if (_id < methodCount)
                    mo->d.static_metacall(this, _c, _id, _a);
                _id -= methodCount;
            } else if ((_c >= QMetaObject::ReadProperty && _c <= QMetaObject::QueryPropertyUser)
                || _c == QMetaObject::RegisterPropertyMetaType) {
                int propertyCount = mo->propertyCount();
                if (_id < propertyCount)
                    mo->d.static_metacall(this, _c, _id, _a);
                _id -= propertyCount;
            }
            return _id;
        }
    };
}}

#[no_mangle]
pub extern "C" fn RustObject_metaObject(p: *mut c_void) -> *const QMetaObject {
    let s = MyStruct{};
    return s.metaObject();
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
    name String,
    typ : i32,
}

#[derive(Default)]
struct MetaObject {
    int_data : Vec<i32>,
    string_data : Vec<String>,
}
impl MetaObject {
    fn buildStringData(&self) -> Vec<u8> {
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

    fn computeIntData(&mut self, methods : Vec<MetaMethod>) {

        let offset = 14;


        self.addString("MyClass".to_owned());
        self.addString("".to_owned());

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

        let paramOffset = offset + methods.len() as i32 * 5;

        for ref m in &methods {
            let n = self.addString(m.name.clone());
            self.int_data.extend_from_slice(&[n , m.args.len() as i32, paramOffset, 1, m.flags]);
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
                let n = self.addString(a.name.clone());
                self.int_data.push(n);
            }
        }
    }

    fn addString(&mut self, string : String) -> i32 {
        self.string_data.push(string);
        return self.string_data.len() as i32 - 1;
    }
}

#[repr(C)]
pub struct QMetaObject {
    superdata : *const c_void,
    string_data: *const u8,
    data: *const i32,
    static_metacall: extern fn(o: *mut c_void, c: u32, idx: u32, a: *const *mut c_void),
    r: *const c_void,
    e: *const c_void,
}


trait QObject {
    fn metaObject(&self)->*const QMetaObject;
    fn callFunction(&mut self, idx: usize, args : &[*mut c_void]);
}



struct MyStruct {
}
impl MyStruct {
    fn xx(&self) -> i32 {
        println!("MyStruct.xx Called" );
        return 42;
    }
}

impl QObject for MyStruct {
    fn metaObject(&self)->*const QMetaObject {
        let m = MetaMethod {
            name: "xx".to_owned(),
            args: Vec::new(),
            flags: 0x2,
            ret_type: 2 // int
        };
        let mut mo : MetaObject = Default::default();
        mo.computeIntData(vec![m]);
        let str_data = mo.buildStringData();
        let int_data = mo.int_data;
        let str_data_ptr = str_data.as_ptr();
        mem::forget(str_data);
        let int_data_ptr = int_data.as_ptr();
        mem::forget(int_data);


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




        unsafe {
            let x = Box::new(QMetaObject {
                superdata: cpp!{[] -> *const c_void as "const void*" { return &QObject::staticMetaObject; } },
                string_data: str_data_ptr,
                data: int_data_ptr,
                static_metacall: static_metacall,
                r: std::ptr::null(),
                e: std::ptr::null(),
            });
            return Box::into_raw(x);
        }
    }

    fn callFunction(&mut self, idx: usize, args : &[*mut c_void]) {
    }
}


fn main() {
    unsafe { cpp!{[] {

        int argc = 1;
        char *argv[] = {"hello"};
        QApplication app(argc, argv);
        QQmlApplicationEngine engine;
        RustObject x;
        engine.rootContext()->setContextProperty("_foo", &x);
//        QLabel w("dds");
//        w.show();
        engine.loadData(R"(

import QtQuick 2.0
import QtQuick.Window 2.0

Window {
    visible: true
    width: 320; height: 480
    Rectangle {
        id: page
        color: 'lightgray'
        anchors.fill: parent

        Text {
            id: helloText
            text: 'Hello world!' + _foo.xx()
            y: 30
            anchors.horizontalCenter: page.horizontalCenter
            font.pointSize: 24; font.bold: true
        }
    }
}

        )");
        app.exec();

    }}}
}
