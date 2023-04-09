use crate::internal_prelude::*;
use crate::QString;

cpp! {{
    #include <QtCore/QSettings>
    #include <QtCore/QString>
}}

cpp_class!(
    /// Wrapper around [`QSettings`][class] class.
    ///
    /// [class]: https://doc.qt.io/qt-5/qsettings.html
    #[derive(Default)]
    pub unsafe struct QSettings as "QSettings"
);

impl QSettings {
    /// Wrapper around [`QSettings(const QString &organization, const QString &application = QString(), QObject *parent = nullptr)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qsettings.html#QSettings-3
    pub fn new(organization: &str, application: &str) -> *mut Self {
        let organization = QString::from(organization);
        let application = QString::from(application);
        cpp!(
            unsafe [organization as "QString", application as "QString"] -> *mut QSettings as "QSettings*" {
                QSettings* settings = new QSettings(organization, application);
                return settings;
            }
        )
    }

    /// Wrapper around [`QSettings(const QString &fileName, QSettings::Format format, QObject *parent = nullptr)`][ctor] constructor.
    ///
    /// [ctor]: https://doc.qt.io/qt-5/qsettings.html#QSettings
    pub fn from_path(file_name: &str) -> *mut Self {
        let file_name = QString::from(file_name);
        cpp!(
            unsafe [file_name as "QString"] -> *mut QSettings as "QSettings*" {
                QSettings* settings = new QSettings(file_name, QSettings::NativeFormat);
                return settings;
            }
        )
    }

    pub fn filename(&self) -> String {
        let filename: QString = cpp!(
            unsafe [self as "QSettings *"] -> QString as "QString" {
                return self->fileName();
            }
        );
        filename.to_string()
    }

    pub fn contains(&self, key: &str) -> bool {
        let key = QString::from(key);
        unsafe {
            cpp!([self as "QSettings *", key as "QString"] -> bool as "bool" {
                return self->contains(key);
            })
        }
    }

    pub fn value_bool(&self, key: &str) -> bool {
        let key = QString::from(key);
        unsafe {
            cpp!([self as "QSettings *", key as "QString"] -> bool as "bool" {
                return self->value(key).toBool();
            })
        }
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        let key = QString::from(key);
        unsafe {
            cpp!([self as "QSettings *", key as "QString", value as "bool"] {
                self->setValue(key, value);
            })
        };
    }

    pub fn value_string(&self, key: &str) -> String {
        let key = QString::from(key);
        let val = unsafe {
            cpp!([self as "QSettings *", key as "QString"] -> QString as "QString" {
                return self->value(key).toString();
            })
        };
        val.into()
    }

    pub fn set_string(&mut self, key: &str, value: &str) {
        let key = QString::from(key);
        let value = QString::from(value);
        unsafe {
            cpp!([self as "QSettings *", key as "QString", value as "QString"] {
                self->setValue(key, value);
            })
        };
    }

    pub fn sync(&self) {
        unsafe {
            cpp!([self as "QSettings *"] {
                self->sync();
            })
        };
    }
}

#[test]
fn test_qsettings_filename() {
    let inner = QSettings::new("qmetaobject", "qsettings");
    let qsettings = unsafe { inner.as_ref().unwrap() };
    assert!(qsettings.filename().ends_with("qmetaobject/qsettings.conf"));

    drop(qsettings);
    drop(inner);
}

#[test]
fn test_qsettings_new_from_path() {
    let inner = QSettings::from_path("/tmp/my_settings.conf");
    let qsettings = unsafe { inner.as_ref().unwrap() };

    assert_eq!(qsettings.filename(), "/tmp/my_settings.conf");

    drop(qsettings);
    drop(inner);
}

#[test]
fn test_qsettings_values() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_pathbuf = temp_dir.path().join("qsettings.conf");
    let config_file = config_pathbuf.to_str().unwrap();

    let inner = QSettings::from_path(config_file);
    let qsettings = unsafe { inner.as_mut().unwrap() };

    qsettings.set_bool("test_true", false);
    qsettings.set_bool("test_false", true);
    qsettings.set_string("test_empty", "");
    qsettings.set_string("test_string", "Lorem Ipsum");
    qsettings.set_string("test_emoji", "🦀");

    qsettings.sync();

    assert_eq!(qsettings.value_bool("test_true"), false);
    assert_eq!(qsettings.value_bool("test_false"), true);
    assert_eq!(qsettings.value_string("test_empty"), "");
    assert_eq!(qsettings.value_string("test_string"), "Lorem Ipsum");
    assert_eq!(qsettings.value_string("test_emoji"), "🦀");

    drop(qsettings);
    drop(inner);

    let inner = QSettings::from_path(config_file);
    let qsettings = unsafe { inner.as_mut().unwrap() };

    assert_eq!(qsettings.value_bool("test_true"), false);
    assert_eq!(qsettings.value_bool("test_false"), true);
    assert_eq!(qsettings.value_string("test_empty"), "");
    assert_eq!(qsettings.value_string("test_string"), "Lorem Ipsum");
    assert_eq!(qsettings.value_string("test_emoji"), "🦀");

    drop(qsettings);
    drop(inner);

    drop(temp_dir);
    assert!(!config_pathbuf.as_path().exists());
}
