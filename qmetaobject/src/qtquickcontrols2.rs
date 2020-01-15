/* Copyright (C) 2019 Tim Süberkrüb <dev@timsueberkrueb.io>
   Copyright (C) 2020 Jonah Brüchert <jbb@kaidan.im>

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

use crate::qttypes::QString;

cpp! {{
    #include <QtQuickControls2/QQuickStyle>
}}

cpp_class!(
/// Wrapper around Qt's QQuickStyle class
    pub unsafe struct QQuickStyle as "QQuickStyle"
);

impl QQuickStyle {
    /// Refer to the Qt documentation for QQuickStyle::setStyle.
    pub fn set_style(style: &str) {
        std::env::set_var("QT_QUICK_CONTROLS_STYLE", style);
    }

    /// Refer to the Qt documentation for QQuickStyle::addStylePath.
    pub fn add_style_path(path: QString) {
        cpp!(unsafe [path as "QString"] {
            return QQuickStyle::addStylePath(path);
        })
    }

    /// Refer to the Qt documentation for QQuickStyle::name.
    pub fn name() -> QString {
        cpp!(unsafe [] -> QString as "QString" {
            return QQuickStyle::name();
        })
    }

    /// Refer to the Qt documentation for QQuickStyle::path.
    pub fn path() -> QString {
        cpp!(unsafe [] -> QString as "QString" {
            return QQuickStyle::path();
        })
    }

    /// Refer to the Qt documentation for QQuickStyle::setFallbackStyle.
    pub fn set_fallback_style(style: QString) {
        cpp!(unsafe [style as "QString"] {
            QQuickStyle::setFallbackStyle(style);
        })
    }

    /// Refer to the Qt documentation for QQuickStyle::setStyle.
    pub fn set_style_with_qstring(style: QString) {
        cpp!(unsafe [style as "QString"] {
            return QQuickStyle::setStyle(style);
        })
    }
}

#[test]
fn test_quickstyle_add_style_path() {
    // only tests for crashes right now, because of list limitations
    QQuickStyle::add_style_path("/tmp".into());
}

#[test]
fn test_quickstyle_set_style_with_qstring() {
    QQuickStyle::set_style_with_qstring("Material".into());
    assert_eq!(QQuickStyle::name(), QString::from("Material"))
}
