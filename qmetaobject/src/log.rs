//! Logging facilities and forwarding

use log::{Level, logger, Record};

use crate::{install_message_handler, QMessageLogContext, QString, QtMsgType};
use crate::QtMsgType::*;

/// Mapping from Qt logging levels to Rust logging facade's levels.
///
/// Due to the limited range of levels from both sides,
/// [`QtCriticalMsg`][`QtMsgType`] and [`QtFatalMsg`][`QtMsgType`]
/// both map to [`log::Level::Error`][Level],
/// while [`log::Level::Trace`][Level] is never returned.
///
/// [Level]: https://docs.rs/log/0.4.10/log/enum.Level.html
/// [`QtMsgType`]: https://doc.qt.io/qt-5/qtglobal.html#QtMsgType-enum
pub fn map_level(lvl: &QtMsgType) -> Level {
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

// Logging middleware, pass-though, or just proxy function.
// It is called from Qt code, then it converts Qt logging data
// into Rust logging facade's log::Record object, and sends it
// to the currently active logger.
extern "C" fn log_capture(msg_type: QtMsgType,
                          context: &QMessageLogContext,
                          message: &QString) {
    let level = map_level(&msg_type);
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

/// Installs into Qt logging system a function which forwards messages to the
/// [Rust logging facade][log].
///
/// Most metadata from Qt logging context is retained and passed to [`log::Record`][].
/// Logging levels are mapped with [this][map_level] algorithm.
///
/// This function may be called more than once.
///
/// [log]: https://docs.rs/log
/// [`log::Record`]: https://docs.rs/log/0.4.10/log/struct.Record.html
/// [map_level]: ./fn.map_level.html
pub fn init_qt_to_rust() {
    // The reason it is named so complex instead of simple `init` is that
    // such descriptive name is future-proof. Consider if someone someday
    // would want to implement the opposite forwarding logger?
    install_message_handler(log_capture);
}
