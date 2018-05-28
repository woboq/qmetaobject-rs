extern crate qmetaobject;
use qmetaobject::*;


#[allow(non_snake_case)]
#[derive(Default,QObject)]
struct TimeModel
{
    base: qt_base_class!(trait QObject),
    hour: qt_property!(u32; NOTIFY timeChanged),
    minute: qt_property!(u32; NOTIFY timeChanged),
    timeChanged: qt_signal!(),

}


#[derive(Default, QObject)]
struct QExampleQmlPlugin {
    base: qt_base_class!(trait QQmlExtensionPlugin),
    plugin: qt_plugin!("org.qt-project.Qt.QQmlExtensionInterface/1.0")
}

impl QQmlExtensionPlugin for QExampleQmlPlugin {
    fn register_types(&mut self, uri : &std::ffi::CStr) {
        //assert_eq!(uri, std::ffi::CStr::from_bytes_with_nul(b"TimeExample\0"));
        qml_register_type::<TimeModel>(uri, 1, 0, std::ffi::CStr::from_bytes_with_nul(b"Time\0").unwrap());
    }
}
