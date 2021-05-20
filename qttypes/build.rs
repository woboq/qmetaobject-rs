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
use std::path::{Path, PathBuf};
use std::process::Command;

fn qmake_query(var: &str) -> Result<String, std::io::Error> {
    let qmake = std::env::var("QMAKE").unwrap_or("qmake".to_string());
    Ok(String::from_utf8(
        Command::new(qmake).env("QT_SELECT", "qt5").args(&["-query", var]).output()?.stdout,
    )
    .expect("UTF-8 conversion failed"))
}

fn open_header(file: &str, qt_include_path: &str, qt_library_path: &str) -> std::fs::File {
    let mut path = PathBuf::from(qt_include_path);
    path.push("QtCore");
    path.push(file);
    if cfg!(target_os = "macos") {
        if !path.exists() {
            path = Path::new(qt_library_path).join("QtCore.framework/Headers");
            path.push(file);
        }
    }
    std::fs::File::open(&path).expect(&format!("Cannot open `{:?}`", path))
}

// qreal is a double, unless QT_COORD_TYPE says otherwise:
// https://doc.qt.io/qt-5/qtglobal.html#qreal-typedef
fn detect_qreal_size(qt_include_path: &str, qt_library_path: &str) {
    let f = open_header("qconfig.h", qt_include_path, qt_library_path);
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

fn detect_version_from_header(qt_include_path: &str, qt_library_path: &str) -> String {
    let f = open_header("qtcoreversion.h", qt_include_path, qt_library_path);
    let b = BufReader::new(f);

    // Find declaration of QTCORE_VERSION_STR
    for line in b.lines() {
        let line = line.expect("qtcoreversion.h is valid UTF-8");
        if line.contains("QTCORE_VERSION_STR") {
            return line.split('\"').nth(1).expect("Parsing QTCORE_VERSION_STR").into();
        }
    }
    panic!("Could not detect Qt version from include paths")
}

fn main() {
    println!("cargo:rerun-if-env-changed=QT_INCLUDE_PATH");
    println!("cargo:rerun-if-env-changed=QT_LIBRARY_PATH");
    let (qt_version, qt_include_path, qt_library_path) = match (
        std::env::var("QT_INCLUDE_PATH").ok().filter(|x| !x.is_empty()),
        std::env::var("QT_LIBRARY_PATH").ok().filter(|x| !x.is_empty()),
    ) {
        (Some(qt_include_path), Some(qt_library_path)) => {
            let qt_version = detect_version_from_header(&qt_include_path, &qt_library_path);
            (qt_version, qt_include_path, qt_library_path)
        }
        (Some(_), None) | (None, Some(_)) => {
            panic!("QT_INCLUDE_PATH and QT_LIBRARY_PATH env variable must be either both empty or both set ")
        }
        (None, None) => {
            let qt_version = match qmake_query("QT_VERSION") {
                Ok(v) => v,
                Err(_err) => {
                    #[cfg(feature = "required")]
                    panic!(
                        "Error: Failed to execute qmake. Make sure 'qmake' is in your path!\n{:?}",
                        _err
                    );
                    #[cfg(not(feature = "required"))]
                    {
                        println!("cargo:rerun-if-env-changed=QMAKE");
                        println!("cargo:rustc-cfg=no_qt");
                        println!("cargo:FOUND=0");
                        return;
                    }
                }
            };
            let qt_include_path = qmake_query("QT_INSTALL_HEADERS").unwrap();
            let qt_library_path = qmake_query("QT_INSTALL_LIBS").unwrap();
            println!("cargo:rerun-if-env-changed=QMAKE");
            (qt_version, qt_include_path, qt_library_path)
        }
    };

    let mut config = cpp_build::Config::new();

    if cfg!(target_os = "macos") {
        config.flag("-F");
        config.flag(qt_library_path.trim());
    }

    detect_qreal_size(&qt_include_path.trim(), qt_library_path.trim());

    if qt_version.trim().starts_with("6.") {
        config.flag_if_supported("-std=c++17");
        config.flag_if_supported("/std:c++17");
    }
    config.include(qt_include_path.trim()).build("src/lib.rs");

    println!("cargo:VERSION={}", qt_version.trim());
    println!("cargo:LIBRARY_PATH={}", qt_library_path.trim());
    println!("cargo:INCLUDE_PATH={}", qt_include_path.trim());
    println!("cargo:FOUND=1");

    let macos_lib_search = if cfg!(target_os = "macos") { "=framework" } else { "" };
    let vers_suffix = if cfg!(target_os = "macos") {
        String::new()
    } else {
        qt_version.split(".").next().unwrap_or("").to_string()
    };

    let debug = std::env::var("DEBUG").ok().map_or(false, |s| s == "true");
    let windows_dbg_suffix = if debug && cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=msvcrtd");
        "d"
    } else {
        ""
    };

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-rpath,{}", qt_library_path.trim());
    }

    println!("cargo:rustc-link-search{}={}", macos_lib_search, qt_library_path.trim());

    let link_lib = |lib: &str| {
        println!(
            "cargo:rustc-link-lib{search}=Qt{vers}{lib}{suffix}",
            search = macos_lib_search,
            vers = vers_suffix,
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
    #[cfg(feature = "qtquickcontrols2")]
    link_lib("QuickControls2");
    #[cfg(feature = "qtmultimedia")]
    link_lib("Multimedia");
    #[cfg(feature = "qtmultimediawidgets")]
    link_lib("MultimediaWidgets");
    #[cfg(feature = "qtsql")]
    link_lib("Sql");
    #[cfg(feature = "qttest")]
    link_lib("Test");

    println!("cargo:rerun-if-changed=src/lib.rs");
}
