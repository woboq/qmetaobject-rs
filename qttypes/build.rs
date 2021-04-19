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

use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;

fn qmake_query(var: &str) -> String {
    let qmake = std::env::var("QMAKE").unwrap_or("qmake".to_string());
    String::from_utf8(
        Command::new(qmake)
            .env("QT_SELECT", "qt5")
            .args(&["-query", var])
            .output()
            .expect("Failed to execute qmake. Make sure 'qmake' is in your path")
            .stdout,
    )
    .expect("UTF-8 conversion failed")
}

// qreal is a double, unless QT_COORD_TYPE says otherwise:
// https://doc.qt.io/qt-5/qtglobal.html#qreal-typedef
fn detect_qreal_size(qt_include_path: &str) {
    let path = Path::new(qt_include_path).join("QtCore").join("qconfig.h");
    let f = std::fs::File::open(&path).expect(&format!("Cannot open `{:?}`", path));
    let b = BufReader::new(f);

    // Find declaration of QT_COORD_TYPE
    for line in b.lines() {
        let line = line.expect("qconfig.h is valid UTF-8");
        if line.contains("QT_COORD_TYPE") {
            if line.contains("float") {
                println!("cargo:rustc-cfg=qreal_is_float");
                return;
            } else {
                panic!("QT_COORD_TYPE with unknown declaration {}", line);
            }
        }
    }
}

fn main() {
    let qt_include_path = qmake_query("QT_INSTALL_HEADERS");
    let qt_library_path = qmake_query("QT_INSTALL_LIBS");
    let qt_version = qmake_query("QT_VERSION");

    let mut config = cpp_build::Config::new();

    if cfg!(target_os = "macos") {
        config.flag("-F");
        config.flag(qt_library_path.trim());
    }

    detect_qreal_size(&qt_include_path.trim());

    config.include(qt_include_path.trim()).build("src/lib.rs");

    println!("cargo:VERSION={}", qt_version.trim());
    println!("cargo:LIBRARY_PATH={}", qt_library_path.trim());
    println!("cargo:INCLUDE_PATH={}", qt_include_path.trim());

    let macos_lib_search = if cfg!(target_os = "macos") { "=framework" } else { "" };
    let macos_lib_framework = if cfg!(target_os = "macos") { "" } else { "5" };

    let debug = std::env::var("DEBUG").ok().map_or(false, |s| s == "true");
    let windows_dbg_suffix = if debug && cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=msvcrtd");
        "d"
    } else {
        ""
    };
    println!("cargo:rustc-link-search{}={}", macos_lib_search, qt_library_path.trim());

    let link_lib = |lib: &str| {
        println!(
            "cargo:rustc-link-lib{search}=Qt{vers}{lib}{suffix}",
            search = macos_lib_search,
            vers = macos_lib_framework,
            lib = lib,
            suffix = windows_dbg_suffix
        )
    };
    link_lib("Core");
    link_lib("Gui");
    link_lib("Widgets");
    #[cfg(feature = "qtquick")]
    link_lib("Quick");
    #[cfg(feature = "qtquick")]
    link_lib("Qml");
    #[cfg(feature = "qtwebengine")]
    link_lib("WebEngine");
}
