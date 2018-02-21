extern crate cpp_build;

fn main() {
    cpp_build::Config::new()
        .include("/usr/include/qt")
        .build("src/lib.rs");

    println!("cargo:rustc-link-lib=Qt5Widgets");
    println!("cargo:rustc-link-lib=Qt5Gui");
    println!("cargo:rustc-link-lib=Qt5Core");
    println!("cargo:rustc-link-lib=Qt5Quick");
    println!("cargo:rustc-link-lib=Qt5Qml");
}
