

extern crate gcc;

extern crate cpp_build;



fn main() {


    cpp_build::Config::new()
//         .file("src/ffi.rs")
        .include("/usr/include/qt")
        .include("../qmetaobject")
//         .flag("-lQt5Core").flag("-lQt5Gui").flag("-lQt5Widgets")
        .build("src/main.rs");


//     cpp_build::build("src/ffi.rs");
/*
    gcc::Build::new().cpp(true)
        .file("src/ffi.cc")
        .include("/usr/include/qt")
        .flag("-lQt5Core").flag("-lQt5Gui").flag("-lQt5Widgets").flag("-fno-inline")
        .static_flag(false)
        .compile("libffi.a");
*/
    println!("cargo:rustc-link-lib=Qt5Widgets");
    println!("cargo:rustc-link-lib=Qt5Gui");
    println!("cargo:rustc-link-lib=Qt5Core");
    println!("cargo:rustc-link-lib=Qt5Quick");
    println!("cargo:rustc-link-lib=Qt5Qml");
}
