use std::future::Future;
use std::pin::Pin;
use std::os::raw::c_void;

static QTWAKERVTABLE : std::task::RawWakerVTable = unsafe {
    std::task::RawWakerVTable::new(
        |s : *const()|  {
            std::task::RawWaker::new(
                cpp!([s as "Waker*"] -> *const() as "Waker*" {
                    s->ref++;
                    return s;
                }),
                &QTWAKERVTABLE)
        },
        |s : *const() | {
            cpp!([s as "Waker*"] {
                s->wake();
                s->deref();
            })
        },
        |s : *const() | {
            cpp!([s as "Waker*"] {
                s->wake();
            })
        },
        |s : *const() | {
            cpp!([s as "Waker*"] {
                s->deref();
            })
        },
    )
};

cpp!{{
    struct Waker : QObject {
    public:
        TraitObject future;
        bool woken = false;
        QAtomicInt ref = 0;
        bool event(QEvent *e) override {
            if (e->type() != QEvent::User)
                return false;
            woken = false;
            rust!(ProcessQtEvent [this: *const() as "Waker*",
                future : *mut dyn Future<Output=()> as "TraitObject"] {
                poll_with_qt_waker(this, Pin::new_unchecked(&mut *future));
            });
            return true;
        }
        void deref() {
            if (!--ref) {
                deleteLater();
            }
        }
        void wake() {
            if (woken) return;
            woken = true;
            QApplication::postEvent(this, new QEvent(QEvent::User));
        }
        ~Waker() {
            rust!(QtDestroyFuture [future : *mut dyn Future<Output=()> as "TraitObject"] {
                std::mem::drop(Box::from_raw(future))
            });
        }
    };
}}

/// Execute a future on the Qt Event loop
///
/// Waking the waker will post an event to the Qt event loop which will poll the future
/// from the event handler
pub fn execute_async(f: impl Future<Output=()> + 'static) {
    let f = Box::into_raw(Box::new(f)) as *mut dyn Future<Output=()>;
    unsafe {
        let waker = cpp!([f as "TraitObject"] -> *const() as "Waker*" {
            auto w = new Waker;
            w->ref++;
            w->future = f;
            return w;
        });
        poll_with_qt_waker(waker, Pin::new_unchecked(&mut *f))
    }
}

unsafe fn poll_with_qt_waker(waker: *const(), future: Pin<&mut dyn Future<Output = ()>>) {
    let waker = std::task::RawWaker::new(waker, &QTWAKERVTABLE);
    let waker = std::task::Waker::from_raw(waker);
    let mut context = std::task::Context::from_waker(&waker);
    match future.poll(&mut context) {
        std::task::Poll::Ready(()) => {}
        std::task::Poll::Pending => {}
    }
}

type Args = fn();
pub unsafe fn wait_on_signal/*<Args>*/(sender: *const c_void, signal : crate::connections::CppSignal<Args>)
    -> impl Future<Output = ()>
{
    struct F/*<Args>*/ {
        started: bool,
        finished: bool,
        handle : crate::connections::ConnectionHandle,
        sender: *const c_void,
        signal : crate::connections::CppSignal<Args>,
    }
    impl Drop for F {
        fn drop(&mut self) {
            self.handle.disconnect();
        }
    }
    impl Future for F {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, ctx: &mut std::task::Context) -> std::task::Poll<()> {
            if self.finished {
                return std::task::Poll::Ready(());
            }
            if !self.started {
                self.started = true;
                let w = ctx.waker().clone();
                unsafe {
                    let s_ptr = self.get_unchecked_mut() as *mut F;
                    (*s_ptr).handle = crate::connections::connect((*s_ptr).sender, (*s_ptr).signal, move || {
                        (*s_ptr).handle.disconnect();
                        (*s_ptr).finished = true;
                        w.wake_by_ref();
                    });
                    debug_assert!((*s_ptr).handle.is_valid());
                }
            }
            std::task::Poll::Pending
        }
    }

    F { started: false, finished: false, handle: crate::connections::ConnectionHandle::default(), sender, signal }
}
