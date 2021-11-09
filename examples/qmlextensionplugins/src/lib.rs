use std::ffi::CStr;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::{self, RecvTimeoutError};

use chrono::Timelike;
use cstr::cstr;

use qmetaobject::prelude::*;

#[allow(non_snake_case)]
#[derive(Default, QObject)]
struct TimeModel {
    base: qt_base_class!(trait QObject),
    hour: qt_property!(u32; NOTIFY timeChanged READ get_hour),
    minute: qt_property!(u32; NOTIFY timeChanged READ get_minute),
    timeChanged: qt_signal!(),

    drop_notification: Option<SyncSender<()>>,
}

impl Drop for TimeModel {
    fn drop(&mut self) {
        // tell the timer thread to stop
        self.drop_notification.as_ref().map(|x| {
            let _ignored_result = x.send(());
        });
    }
}

impl TimeModel {
    fn lazy_init(&mut self) {
        if self.drop_notification.is_none() {
            let ptr = QPointer::from(&*self);
            let notify_time_changed = qmetaobject::queued_callback(move |()| {
                ptr.as_ref().map(|x| x.timeChanged());
            });

            let (drop_notification_tx, drop_notification_rx) = mpsc::sync_channel(1);
            std::thread::spawn(move || {
                // We just wait on the channel for 1 second to simulate a one second timer
                while drop_notification_rx.recv_timeout(std::time::Duration::from_millis(1000))
                    == Err(RecvTimeoutError::Timeout)
                {
                    notify_time_changed(());
                }
            });

            self.drop_notification = Some(drop_notification_tx);
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
    fn register_types(&mut self, uri: &CStr) {
        //assert_eq!(uri, cstr!("TimeExample"));
        qml_register_type::<TimeModel>(uri, 1, 0, cstr!("Time"));
    }
}
