/* Copyright (C) 2018 Olivier Goffart <ogoffart@woboq.com>
   Copyright (C) 2021 ivan tkachenko a.k.a. ratijas <me@ratijas.tk>

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
//! Wrappers around [`QtCore/QCoreApplication`][qt] header.
//!
//! [qt]: https://doc.qt.io/qt-5/qcoreapplication.html

use cpp::cpp;

use crate::*;

cpp! {{
    #include <QtCore/QCoreApplication>
}}

/// Wrapper around [`QCoreApplication`][qt] class.
///
/// # Wrapper-specific
///
/// Currently it is uncreatible, non-obtainable, and exists purely as a
/// namespace for public static members (associated functions in Rust).
///
/// [qt]: https://doc.qt.io/qt-5/qcoreapplication.html
pub struct QCoreApplication {
    // Private field makes this dummy struct uncreatable by user. Since
    // there's no way to obtain an instance of it, we won't have to worry
    // about layout compatibility and stuff.
    _private: (),
}

impl QCoreApplication {
    /// Wrapper around [`QString applicationName()`][qt] static method.
    ///
    /// [qt]: https://doc.qt.io/qt-5/qcoreapplication.html#applicationName-prop
    pub fn application_name() -> QString {
        cpp!(unsafe [] -> QString as "QString" {
            return QCoreApplication::applicationName();
        })
    }

    /// Wrapper around [`void setApplicationName(const QString &application)`][qt] static method.
    ///
    /// [qt]: https://doc.qt.io/qt-5/qcoreapplication.html#applicationName-prop
    pub fn set_application_name(application: QString) {
        cpp!(unsafe [application as "QString"] {
            QCoreApplication::setApplicationName(application);
        });
    }

    /// Wrapper around [`QString applicationVersion()`][qt] static method.
    ///
    /// [qt]: https://doc.qt.io/qt-5/qcoreapplication.html#applicationVersion-prop
    pub fn application_version() -> QString {
        cpp!(unsafe [] -> QString as "QString" {
            return QCoreApplication::applicationVersion();
        })
    }

    /// Wrapper around [`void setApplicationVersion(const QString &version)`][qt] static method.
    ///
    /// [qt]: https://doc.qt.io/qt-5/qcoreapplication.html#applicationVersion-prop
    pub fn set_application_version(version: QString) {
        cpp!(unsafe [version as "QString"] {
            QCoreApplication::setApplicationVersion(version);
        });
    }

    /// Wrapper around [`QString organizationDomain()`][qt] static method.
    ///
    /// [qt]: https://doc.qt.io/qt-5/qcoreapplication.html#organizationDomain-prop
    pub fn organization_domain() -> QString {
        cpp!(unsafe [] -> QString as "QString" {
            return QCoreApplication::organizationDomain();
        })
    }

    /// Wrapper around [`void setOrganizationDomain(const QString &orgDomain)`][qt] static method.
    ///
    /// [qt]: https://doc.qt.io/qt-5/qcoreapplication.html#organizationDomain-prop
    pub fn set_organization_domain(org_domain: QString) {
        cpp!(unsafe [org_domain as "QString"] {
            QCoreApplication::setOrganizationDomain(org_domain);
        });
    }

    /// Wrapper around [`QString organizationName()`][qt] static method.
    ///
    /// [qt]: https://doc.qt.io/qt-5/qcoreapplication.html#organizationName-prop
    pub fn organization_name() -> QString {
        cpp!(unsafe [] -> QString as "QString" {
            return QCoreApplication::organizationName();
        })
    }

    /// Wrapper around [`void setOrganizationName(const QString &orgName)`][qt] static method.
    ///
    /// [qt]: https://doc.qt.io/qt-5/qcoreapplication.html#organizationName-prop
    pub fn set_organization_name(org_name: QString) {
        cpp!(unsafe [org_name as "QString"] {
            QCoreApplication::setOrganizationName(org_name);
        });
    }

    /// Wrapper around [`void QCoreApplication::quit()`][qt] static method.
    ///
    /// # Note
    ///
    /// Unlike the C library function of the same name, this function does
    /// return to the caller — it is event processing that stops.
    ///
    /// [qt]: https://doc.qt.io/qt-5/qcoreapplication.html#quit
    pub fn quit() {
        cpp!(unsafe [] {
            QCoreApplication::quit();
        });
    }
}
