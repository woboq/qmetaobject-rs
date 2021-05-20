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

use semver::Version;

fn main() {
    eprintln!("cargo:warning={:?}", std::env::vars().collect::<Vec<_>>());

    let qt_include_path = std::env::var("DEP_QT_INCLUDE_PATH").unwrap();
    let qt_library_path = std::env::var("DEP_QT_LIBRARY_PATH").unwrap();
    let qt_version = std::env::var("DEP_QT_VERSION")
        .unwrap()
        .parse::<Version>()
        .expect("Parsing Qt version failed");

    let mut config = cpp_build::Config::new();
    if cfg!(target_os = "macos") {
        config.flag("-F");
        config.flag(qt_library_path.trim());
    }
    if qt_version >= Version::new(6, 0, 0) {
        config.flag_if_supported("-std=c++17");
        config.flag_if_supported("/std:c++17");
    }
    config.include(qt_include_path.trim()).build("src/lib.rs");

    for minor in 7..=15 {
        if qt_version >= Version::new(5, minor, 0) {
            println!("cargo:rustc-cfg=qt_{}_{}", 5, minor);
        }
    }
    let mut minor = 0;
    while qt_version >= Version::new(6, minor, 0) {
        println!("cargo:rustc-cfg=qt_{}_{}", 6, minor);
        minor += 1;
    }
}
