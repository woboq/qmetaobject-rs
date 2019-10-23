use std::future::Future;
use std::pin::Pin;
use std::os::raw::c_void;
use crate::connections::SignalArgArrayToTuple;

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
///
/// Note that this function returns immediatly. A Qt event loop need to be running
/// on the current thread so the future can be executed. (It is Ok if the Qt event
/// loop hasn't started yet when this function is called)
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

/// Create a future that waits on the emission of a signal.
///
/// The arguments of the signal need to implement `Clone`, and the Output of the future is a tuple
/// containing the arguments of the signal (or the empty tuple if there are none.)
///
/// The future will be ready as soon as the signal is emited.
///
/// This is unsafe for the same reason that connections::connect is unsafe.
pub unsafe fn wait_on_signal<Args : SignalArgArrayToTuple>(sender: *const c_void, signal : crate::connections::CppSignal<Args>)
    -> impl Future<Output = <Args as SignalArgArrayToTuple>::Tuple>
{
    struct F<Args : SignalArgArrayToTuple> {
        started: bool,
        finished: bool,
        handle : crate::connections::ConnectionHandle,
        sender: *const c_void,
        signal : crate::connections::CppSignal<Args>,
        result: Option<<Args as SignalArgArrayToTuple>::Tuple>,
        waker: Option<std::task::Waker>,
    }
    impl<Args : SignalArgArrayToTuple> Drop for F<Args> {
        fn drop(&mut self) {
            self.handle.disconnect();
        }
    }
    impl<Args : SignalArgArrayToTuple> Future for F<Args> {
        type Output = <Args as SignalArgArrayToTuple>::Tuple;
        fn poll(self: Pin<&mut Self>, ctx: &mut std::task::Context) -> std::task::Poll<Self::Output> {
            if self.finished {
                unsafe {
                    return std::task::Poll::Ready(self.get_unchecked_mut().result.take().unwrap());
                }
            }
            if !self.started {
                unsafe {
                    let s_ptr = self.get_unchecked_mut() as *mut F<_>;
                    (*s_ptr).started = true;
                    (*s_ptr).waker = Some(ctx.waker().clone());
                    (*s_ptr).handle = crate::connections::connect((*s_ptr).sender, (*s_ptr).signal, s_ptr);
                    debug_assert!((*s_ptr).handle.is_valid());
                }
            }
            std::task::Poll::Pending
        }
    }

    impl<Args : SignalArgArrayToTuple> crate::connections::Slot<Args> for *mut F<Args> {
        unsafe fn apply(&mut self, a : *const *const c_void) {
            (**self).finished = true;
            (**self).result = Some(Args::args_array_to_tuple(a));
            (**self).handle.disconnect();
            (**self).waker.as_ref().unwrap().wake_by_ref();
        }
    }

    F {
        started: false,
        finished: false,
        handle: crate::connections::ConnectionHandle::default(),
        sender,
        signal,
        result: None,
        waker: None
    }
}
