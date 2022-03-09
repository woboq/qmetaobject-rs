use crate::{cpp, QString, QStringList};

cpp! {{
    #include <QtCore/QStandardPaths>
}}

/// Bindings for [`QStandardPaths::StandardLocation`][enum] enum.
///
/// [enum]: https://doc.qt.io/qt-5/qstandardpaths.html#StandardLocation-enum
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum QStandardPathLocation {
    DesktopLocation = 0,
    DocumentsLocation = 1,
    FontsLocation = 2,
    ApplicationsLocation = 3,
    MusicLocation = 4,
    MoviesLocation = 5,
    PicturesLocation = 6,
    TempLocation = 7,
    HomeLocation = 8,
    AppLocalDataLocation = 9,
    CacheLocation = 10,
    GenericDataLocation = 11,
    RuntimeLocation = 12,
    ConfigLocation = 13,
    DownloadLocation = 14,
    GenericCacheLocation = 15,
    GenericConfigLocation = 16,
    AppDataLocation = 17,
    AppConfigLocation = 18,
}

impl QStandardPathLocation {
    pub fn display_name(t: Self) -> Option<QString> {
        cpp!(unsafe [t as "QStandardPaths::StandardLocation"] -> QString as "QString" {
            return QStandardPaths::displayName(t);
        })
        .to_option()
    }

    pub fn standard_locations(t: Self) -> QStringList {
        cpp!(unsafe [t as "QStandardPaths::StandardLocation"] -> QStringList as "QStringList" {
            return QStandardPaths::standardLocations(t);
        })
    }

    pub fn writable_location(t: Self) -> Option<QString> {
        cpp!(unsafe [t as "QStandardPaths::StandardLocation"] -> QString as "QString" {
            return QStandardPaths::writableLocation(t);
        })
        .to_option()
    }
}
