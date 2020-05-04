//! Logging facilities and forwarding

use std::os::raw::c_char;

#[cfg(feature = "log")]
use log::{Level, logger, Record, RecordBuilder};

use crate::QString;

cpp! {{
    #include <qmetaobject_rust.hpp>
}}

cpp_class!(
    /// Wrapper for [`QMessageLogContext`] class.
    ///
    /// [`QMessageLogContext`]: https://doc.qt.io/qt-5/qmessagelogcontext.html
    pub unsafe struct QMessageLogContext as "QMessageLogContext"
);

impl QMessageLogContext {
    /// Wrapper for `QMessageLogContext::line`.
    pub fn line(&self) -> i32 {
        cpp!(unsafe [self as "QMessageLogContext*"] -> i32 as "int" { return self->line; })
    }

    /// Wrapper for `QMessageLogContext::file`.
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

    /// Wrapper for `QMessageLogContext::function`.
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

    /// Wrapper for `QMessageLogContext::category`.
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

/// Wrapper for  [`Qt::QtMsgType`][] enum.
///
/// [`Qt::QtMsgType`]: https://doc.qt.io/qt-5/qtglobal.html#QtMsgType-enum
#[repr(C)]
// XXX: Do NOT derive Ord and PartialOrd.
// XXX: Variants are not ordered by severity.
// XXX: Also, levels ordering is not implemented in Qt, only == equality.
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

#[cfg(feature = "log")]
impl From<QtMsgType> for Level {
    /// Mapping from Qt logging levels to Rust logging facade's levels.
    ///
    /// Due to the limited range of levels from both sides,
    /// [`QtCriticalMsg`][`Qt::QtMsgType`] and [`QtFatalMsg`][`Qt::QtMsgType`]
    /// both map to [`log::Level::Error`][Level],
    /// while [`log::Level::Trace`][Level] is never returned.
    ///
    /// [Level]: https://docs.rs/log/0.4.10/log/enum.Level.html
    /// [`Qt::QtMsgType`]: https://doc.qt.io/qt-5/qtglobal.html#QtMsgType-enum
    fn from(lvl: QtMsgType) -> Self {
        match lvl {
            QtMsgType::QtDebugMsg => Level::Debug,
            QtMsgType::QtInfoMsg => Level::Info,
            QtMsgType::QtWarningMsg => Level::Warn,
            QtMsgType::QtCriticalMsg => Level::Error,
            QtMsgType::QtFatalMsg => Level::Error,
            // XXX: What are the external guarantees about possible values of QtMsgType?
            // XXX: Are they promised to be limited to the valid enum variants?
        }
    }
}

#[cfg(feature = "log")]
impl From<Level> for QtMsgType {
    /// Mapping back from Rust logging facade's levels to Qt logging levels.
    ///
    /// Not used internally, exists just for completeness of API.
    ///
    /// Due to the limited range of levels from both sides,
    /// [`log::Level::Debug`][Level] and [`log::Level::Trace`][Level]
    /// both map to [`QtDebugMsg`][`Qt::QtMsgType`],
    /// while [`QtFatalMsg`][`Qt::QtMsgType`] is never returned.
    ///
    /// [Level]: https://docs.rs/log/0.4.10/log/enum.Level.html
    /// [`Qt::QtMsgType`]: https://doc.qt.io/qt-5/qtglobal.html#QtMsgType-enum
    fn from(lvl: Level) -> Self {
        match lvl {
            Level::Error => QtMsgType::QtCriticalMsg,
            Level::Warn => QtMsgType::QtWarningMsg,
            Level::Info => QtMsgType::QtInfoMsg,
            Level::Debug => QtMsgType::QtDebugMsg,
            Level::Trace => QtMsgType::QtDebugMsg,
        }
    }
}

/// Wrapper for [`QtMessageHandler`][] typedef.
///
/// [`QtMessageHandler`]: https://doc.qt.io/qt-5/qtglobal.html#QtMessageHandler-typedef
pub type QtMessageHandler = Option<extern "C" fn(QtMsgType, &QMessageLogContext, &QString)>;

/// Wrapper for [`qInstallMessageHandler`] function.
///
/// # Wrapper-specific behavior
///
/// To restore the message handler, call `install_message_handler(None)`.
///
/// [`qInstallMessageHandler`]: https://doc.qt.io/qt-5/qtglobal.html#qInstallMessageHandler
pub fn install_message_handler(logger: QtMessageHandler) -> QtMessageHandler {
    cpp!(unsafe [logger as "QtMessageHandler"] -> QtMessageHandler as "QtMessageHandler" {
        return qInstallMessageHandler(logger);
    })
}

// Logging middleware, pass-though, or just proxy function.
// It is called from Qt code, then it converts Qt logging data
// into Rust logging facade's log::Record object, and sends it
// to the currently active logger.
#[cfg(feature = "log")]
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
    // Match expression with single all-capturing arm is a hack that allows
    // to extend the lifetime of a matched object for "a little longer".
    // Passing an expression with temporaries as a function argument
    // works exactly the same way.
    // Basically, it retains bounded temporary values together with their
    // intermediate values etc. This is also the way how println! macro works.
    match context.function() {
        "" => finish(record, format_args!("{}", message)),
        f => finish(record, format_args!("[in {}] {}", f, message)),
    }
    fn finish<'a>(mut record: RecordBuilder<'a>, args: std::fmt::Arguments<'a>) {
        logger().log(&record.args(args).build())
    }
}

/// Installs into [Qt logging system][qt-log] a function which forwards messages to the
/// [Rust logging facade][log].
///
/// Most metadata from Qt logging context is retained and passed to [`log::Record`][].
/// Logging levels are mapped with the default [`From`][lvl] implementation.
///
/// This function may be called more than once.
///
/// [qt-log]: https://doc.qt.io/qt-5/qtglobal.html#qInstallMessageHandler
/// [log]: https://crates.io/crates/log
/// [`log::Record`]: https://docs.rs/log/0.4.10/log/struct.Record.html
/// [lvl]: ./struct.QtMsgType.html
#[cfg(feature = "log")]
pub fn init_qt_to_rust() {
    // The reason it is named so complex instead of simple `init` is that
    // such descriptive name is future-proof. Consider if someone someday
    // would want to implement the opposite forwarding logger?
    install_message_handler(Some(log_capture));
}

#[cfg(test)]
#[cfg(feature = "log")]
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
        assert_eq!(Level::from(QtMsgType::QtInfoMsg), Level::Info);
        assert_eq!(QtMsgType::QtCriticalMsg, QtMsgType::from(Level::Error))
    }
}
