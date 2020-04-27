extern crate qmetaobject;
use qmetaobject::*;
extern crate chrono;
use chrono::Timelike;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

#[derive(Default)]
struct AbortCondVar {
    is_aborted: Mutex<bool>,
    abort_condvar: Condvar,
}

#[allow(non_snake_case)]
#[derive(Default, QObject)]
struct TimeModel {
    base: qt_base_class!(trait QObject),
    hour: qt_property!(u32; NOTIFY timeChanged READ get_hour),
    minute: qt_property!(u32; NOTIFY timeChanged READ get_minute),
    timeChanged: qt_signal!(),

    thread: Option<(JoinHandle<()>, Arc<AbortCondVar>)>,
}

impl Drop for TimeModel {
    fn drop(&mut self) {
        self.thread.as_ref().map(|x| {
            let mut lock = x.1.is_aborted.lock().unwrap();
            *lock = true;
            x.1.abort_condvar.notify_one();
        });
    }
}

impl TimeModel {
    fn lazy_init(&mut self) {
        if self.thread.is_none() {
            let ptr = QPointer::from(&*self);
            let cb = qmetaobject::queued_callback(move |()| {
                ptr.as_ref().map(|x| x.timeChanged());
            });
            let arc = Arc::<AbortCondVar>::new(Default::default());
            let arc2 = arc.clone();
            let thread = std::thread::spawn(move || loop {
                let lock = arc2.is_aborted.lock().unwrap();
                if *lock {
                    break;
                }
                // We just wait on the condition variable for 1 second to simulate a one second timer
                let lock = arc2
                    .abort_condvar
                    .wait_timeout(lock, std::time::Duration::from_millis(1000))
                    .unwrap()
                    .0;
                std::mem::drop(lock);
                cb(());
            });
            self.thread = Some((thread, arc));
        }
    }
    fn get_hour(&mut self) -> u32 {
        self.lazy_init();
        chrono::offset::Local::now().time().hour()
    }
    fn get_minute(&mut self) -> u32 {
        self.lazy_init();
        chrono::offset::Local::now().time().minute()
    }
}

#[derive(Default, QObject)]
struct QExampleQmlPlugin {
    base: qt_base_class!(trait QQmlExtensionPlugin),
    plugin: qt_plugin!("org.qt-project.Qt.QQmlExtensionInterface/1.0"),
}

impl QQmlExtensionPlugin for QExampleQmlPlugin {
    fn register_types(&mut self, uri: &std::ffi::CStr) {
        //assert_eq!(uri, std::ffi::CStr::from_bytes_with_nul(b"TimeExample\0"));
        qml_register_type::<TimeModel>(
            uri,
            1,
            0,
            std::ffi::CStr::from_bytes_with_nul(b"Time\0").unwrap(),
        );
    }
}
