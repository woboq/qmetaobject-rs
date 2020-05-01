//! Logging facilities and forwarding

use std::os::raw::c_char;

use log::{Level, logger, Record};

use crate::QString;

use self::QtMsgType::*;

cpp! {{
    #include <qmetaobject_rust.hpp>
}}

cpp_class!(
    /// Wrapper for Qt's QMessageLogContext
    pub unsafe struct QMessageLogContext as "QMessageLogContext"
);

impl QMessageLogContext {
    // Return QMessageLogContext::line
    pub fn line(&self) -> i32 {
        cpp!(unsafe [self as "QMessageLogContext*"] -> i32 as "int" { return self->line; })
    }
    // Return QMessageLogContext::file
    pub fn file(&self) -> &str {
        unsafe {
            let x = cpp!([self as "QMessageLogContext*"] -> *const c_char as "const char*" {
                return self->file;
            });
            if x.is_null() {
                return "";
            }
            std::ffi::CStr::from_ptr(x).to_str().unwrap()
        }
    }
    // Return QMessageLogContext::function
    pub fn function(&self) -> &str {
        unsafe {
            let x = cpp!([self as "QMessageLogContext*"] -> *const c_char as "const char*" {
                return self->function;
            });
            if x.is_null() {
                return "";
            }
            std::ffi::CStr::from_ptr(x).to_str().unwrap()
        }
    }
    // Return QMessageLogContext::category
    pub fn category(&self) -> &str {
        unsafe {
            let x = cpp!([self as "QMessageLogContext*"] -> *const c_char as "const char*" {
                return self->category;
            });
            if x.is_null() {
                return "";
            }
            std::ffi::CStr::from_ptr(x).to_str().unwrap()
        }
    }
}

/// Wrap Qt's QtMsgType enum
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum QtMsgType {
    QtDebugMsg,
    QtWarningMsg,
    QtCriticalMsg,
    QtFatalMsg,
    QtInfoMsg,
    // there is also one level defined in C++ code:
    // QtSystemMsg = QtCriticalMsg
}

/// Mapping from Qt logging levels to Rust logging facade's levels.
///
/// Due to the limited range of levels from both sides,
/// [`QtCriticalMsg`][`QtMsgType`] and [`QtFatalMsg`][`QtMsgType`]
/// both map to [`log::Level::Error`][Level],
/// while [`log::Level::Trace`][Level] is never returned.
///
/// [Level]: https://docs.rs/log/0.4.10/log/enum.Level.html
/// [`QtMsgType`]: https://doc.qt.io/qt-5/qtglobal.html#QtMsgType-enum
pub fn map_level(lvl: QtMsgType) -> Level {
    match lvl {
        QtDebugMsg => Level::Debug,
        QtInfoMsg => Level::Info,
        QtWarningMsg => Level::Warn,
        QtCriticalMsg => Level::Error,
        QtFatalMsg => Level::Error,
        // XXX: What are the external guarantees about possible values of QtMsgType?
        // XXX: Are they promised to be limited to the valid enum variants?
    }
}

/// Mapping back from Rust logging facade's levels to Qt logging levels.
///
/// Not used internally, exists just for completeness of API.
///
/// Due to the limited range of levels from both sides,
/// [`log::Level::Debug`][Level] and [`log::Level::Trace`][Level]
/// both map to [`QtDebugMsg`][`QtMsgType`],
/// while [`QtFatalMsg`][`QtMsgType`] is never returned.
///
/// [Level]: https://docs.rs/log/0.4.10/log/enum.Level.html
/// [`QtMsgType`]: https://doc.qt.io/qt-5/qtglobal.html#QtMsgType-enum
pub fn unmap_level(lvl: Level) -> QtMsgType {
    match lvl {
        Level::Error => QtCriticalMsg,
        Level::Warn => QtWarningMsg,
        Level::Info => QtInfoMsg,
        Level::Debug => QtDebugMsg,
        Level::Trace => QtDebugMsg,
    }
}

impl From<QtMsgType> for Level {
    /// Delegates to [default][] mapping algorithm.
    ///
    /// [default]: ./fn.map_level.html
    fn from(lvl: QtMsgType) -> Self {
        map_level(lvl)
    }
}

impl From<Level> for QtMsgType {
    /// Delegates to [default][] mapping algorithm.
    ///
    /// [default]: ./fn.unmap_level.html
    fn from(lvl: Level) -> Self {
        unmap_level(lvl)
    }
}

/// Wrap qt's qInstallMessageHandler.
/// Useful in order to forward the log to a rust logging framework
pub fn install_message_handler(logger: extern "C" fn(QtMsgType, &QMessageLogContext, &QString)) {
    cpp!(unsafe [logger as "QtMessageHandler"] { qInstallMessageHandler(logger); })
}

// Logging middleware, pass-though, or just proxy function.
// It is called from Qt code, then it converts Qt logging data
// into Rust logging facade's log::Record object, and sends it
// to the currently active logger.
extern "C" fn log_capture(msg_type: QtMsgType,
                          context: &QMessageLogContext,
                          message: &QString) {
    let level = msg_type.into();
    let target = match context.category() {
        "" => "default",
        x => x,
    };
    let file = match context.file() {
        "" => None,
        x => Some(x),
    };
    let line = match context.line() {
        // In Qt, line numbers start from 1, while 0 is just a placeholder
        0 => None,
        x => Some(x as _),
    };
    let mut record = Record::builder();
    record.level(level)
        .target(target)
        .file(file)
        .line(line)
        .module_path(None);
    // (inner) match with single all-capturing arm is a hack that allows us
    // to extend the lifetime of a matched object for "a little longer".
    // Basically, it retains bounded temporary values together with their
    // intermediate values etc. This is also the way how println! macro works.
    match context.function() {
        "" => match format_args!("{}", message) {
            args => logger().log(&record.args(args).build()),
        },
        f => match format_args!("[in {}] {}", f, message) {
            args => logger().log(&record.args(args).build()),
        }
    }
}

/// Installs into [Qt logging system][qt-log] a function which forwards messages to the
/// [Rust logging facade][log].
///
/// Most metadata from Qt logging context is retained and passed to [`log::Record`][].
/// Logging levels are mapped with the [default][map_level] algorithm.
///
/// This function may be called more than once.
///
/// [qt-log]: https://doc.qt.io/qt-5/qtglobal.html#qInstallMessageHandler
/// [log]: https://docs.rs/log
/// [`log::Record`]: https://docs.rs/log/0.4.10/log/struct.Record.html
/// [map_level]: ./fn.map_level.html
pub fn init_qt_to_rust() {
    // The reason it is named so complex instead of simple `init` is that
    // such descriptive name is future-proof. Consider if someone someday
    // would want to implement the opposite forwarding logger?
    install_message_handler(log_capture);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_init() {
        // must not crash
        init_qt_to_rust();
        init_qt_to_rust();
    }

    #[test]
    fn test_convert() {
        assert_eq!(Level::from(QtInfoMsg), Level::Info);
        assert_eq!(QtCriticalMsg, QtMsgType::from(Level::Error))
    }
}
