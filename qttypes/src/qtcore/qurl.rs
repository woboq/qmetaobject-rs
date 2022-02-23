use crate::internal_prelude::*;
use crate::QString;

cpp! {{
    #include <QtCore/QString>
    #include <QtCore/QUrl>
}}

cpp_class!(
    /// Wrapper around [`QUrl`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qurl.html
    #[derive(PartialEq, PartialOrd, Eq, Ord)]
    pub unsafe struct QUrl as "QUrl"
);
impl QUrl {
    /// Wrapper around [`QUrl::fromUserInput(const QString &userInput)`][method] static method.
    ///
    /// [method]: https://doc.qt.io/qt-5/qurl.html#fromUserInput
    pub fn from_user_input(user_input: QString) -> QUrl {
        cpp!(unsafe [user_input as "QString"] -> QUrl as "QUrl" {
            return QUrl::fromUserInput(user_input);
        })
    }
}
impl From<QString> for QUrl {
    fn from(qstring: QString) -> QUrl {
        cpp!(unsafe [qstring as "QString"] -> QUrl as "QUrl" {
            return QUrl(qstring);
        })
    }
}
