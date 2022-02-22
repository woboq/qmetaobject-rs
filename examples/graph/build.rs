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
    let qt_include_path = std::env::var("DEP_QT_INCLUDE_PATH").unwrap();
    let qt_version = std::env::var("DEP_QT_VERSION")
        .unwrap()
        .parse::<Version>()
        .expect("Parsing Qt version failed");

    if qt_version >= Version::new(6, 0, 0) {
        // This example is not supported on Qt 6 and above because graphics
        // API used for it were removed.
        println!("cargo:rustc-cfg=no_qt");
        return;
    }

    #[allow(unused_mut)]
    let mut config = cpp_build::Config::new();

    for f in std::env::var("DEP_QT_COMPILE_FLAGS").unwrap().split_terminator(";") {
        config.flag(f);
    }

    config
        .include(&qt_include_path)
        .include(format!("{}/QtQuick", qt_include_path))
        .include(format!("{}/QtCore", qt_include_path))
        // See https://github.com/woboq/qmetaobject-rs/pull/168
        //
        // QSGSimpleMaterial{,Shader} classes ain't going to be removed from Qt5
        // which is on a life support at this point; and we know for sure they are
        // already gone in Qt6. So, there's just no point seeing these warning
        // over and over again.
        .flag_if_supported("-Wno-deprecated-declarations")
        .build("src/main.rs");
}
