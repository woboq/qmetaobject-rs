use std::future::Future;
use std::mem::replace;
use std::os::raw::c_void;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use cpp::cpp;

use crate::connections::SignalArgArrayToTuple;

static QT_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |s: *const ()| {
        RawWaker::new(
            cpp!(unsafe [s as "Waker *"] -> *const() as "Waker *" {
                s->refs++;
                return s;
            }),
            &QT_WAKER_VTABLE,
        )
    },
    |s: *const ()| {
        cpp!(unsafe [s as "Waker *"] {
            s->wake();
            s->deref();
        })
    },
    |s: *const ()| {
        cpp!(unsafe [s as "Waker *"] {
            s->wake();
        })
    },
    |s: *const ()| {
        cpp!(unsafe [s as "Waker *"] {
            s->deref();
        })
    },
);

cpp! {{

    #include <QtCore/QCoreApplication>

    /// Special QObject subclass to glue together internals of Rust's futures and Qt's events.
    /// It's lifetime is determined through reference counting, and its lifecycle is based on
    /// Qt's QObject rather than C++ RAII.
    struct Waker : QObject {
        /// Wrapped Rust's Future as a dynamic trait object.
        TraitObject future;
        /// Guard against redundant processing of multiple consecutive wake-up calls.
        bool woken = false;
        /// Guard against polling a future after it has been completed.
        bool completed = false;
        /// Reference counter.
        QAtomicInt refs = 0;

        // start with refs count of 1, because caller gets the ownership.
        Waker(TraitObject f): future(f), refs(1) {}

        void customEvent(QEvent *e) override {
            Q_UNUSED(e);
            woken = false;
            // future must not be polled after it returned `Poll::Ready`
            if (completed) {
                return;
            }
            completed = rust!(ProcessQtEvent [
                this: *const () as "Waker *",
                future: *mut dyn Future<Output = ()> as "TraitObject"
            ] -> bool as "bool" {
                poll_with_qt_waker(this, Pin::new_unchecked(&mut *future))
            });
            if (completed) {
                deref();
            }
        }

        void deref() {
            if (!--refs) {
                deleteLater();
            }
        }

        void wake() {
            if (woken) {
                return;
            }
            woken = true;
            // This line results in invocation of customEvent(QEvent*) method above.
            // Note that object may be waken multiple times before the wake up call
            // actually gets proceeded by the Qt's event loop.
            QCoreApplication::postEvent(this, new QEvent(QEvent::User));
        }

        ~Waker() {
            rust!(QtDestroyFuture [future: *mut dyn Future<Output = ()> as "TraitObject"] {
                drop(Box::from_raw(future));
            });
        }
    };
}}

/// Execute a future on the Qt Event loop
///
/// Waking the waker will post an event to the Qt event loop which will poll the future
/// from the event handler
///
/// Note that this function returns immediately. A Qt event loop need to be running
/// on the current thread so the future can be executed. (It is Ok if the Qt event
/// loop hasn't started yet when this function is called)
pub fn execute_async(f: impl Future<Output = ()> + 'static) {
    let f: *mut dyn Future<Output = ()> = Box::into_raw(Box::new(f));
    unsafe {
        let waker = cpp!([f as "TraitObject"] -> *const() as "Waker *" {
            return new Waker(f);
        });
        poll_with_qt_waker(waker, Pin::new_unchecked(&mut *f));
    }
}

// SAFETY: caller must ensure that given future hasn't returned Poll::Ready earlier.
unsafe fn poll_with_qt_waker(waker: *const (), future: Pin<&mut dyn Future<Output = ()>>) -> bool {
    cpp!([waker as "Waker *"] { waker->refs++; });
    let waker = RawWaker::new(waker, &QT_WAKER_VTABLE);
    let waker = Waker::from_raw(waker);
    let mut context = Context::from_waker(&waker);
    future.poll(&mut context).is_ready()
}

/// Create a future that waits on the emission of a signal.
///
/// The arguments of the signal need to implement `Clone`, and the Output of the future is a tuple
/// containing the arguments of the signal (or the empty tuple if there are none.)
///
/// The future will be ready as soon as the signal is emitted.
///
/// This is unsafe for the same reason that [`connections::connect`][] is unsafe.
///
/// [`connections::connect`]: ../connections/fn.connect.html
pub unsafe fn wait_on_signal<Args: SignalArgArrayToTuple>(
    sender: *const c_void,
    signal: crate::connections::Signal<Args>,
) -> impl Future<Output = <Args as SignalArgArrayToTuple>::Tuple> {
    enum ConnectionFutureState<Args: SignalArgArrayToTuple> {
        Init { sender: *const c_void, signal: crate::connections::Signal<Args> },
        Started { handle: crate::connections::ConnectionHandle, waker: Waker },
        Finished { result: <Args as SignalArgArrayToTuple>::Tuple },
        Invalid,
    }

    impl<Args: SignalArgArrayToTuple> std::marker::Unpin for ConnectionFutureState<Args> {}

    struct ConnectionFuture<Args: SignalArgArrayToTuple>(ConnectionFutureState<Args>);

    impl<Args: SignalArgArrayToTuple> Drop for ConnectionFuture<Args> {
        fn drop(&mut self) {
            if let ConnectionFutureState::Started { ref mut handle, .. } = &mut self.0 {
                handle.disconnect();
            }
        }
    }

    impl<Args: SignalArgArrayToTuple> Future for ConnectionFuture<Args> {
        type Output = <Args as SignalArgArrayToTuple>::Tuple;
        fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
            let state = &mut self.0;
            *state = match replace(state, ConnectionFutureState::Invalid) {
                ConnectionFutureState::Finished { result } => {
                    return Poll::Ready(result);
                }
                ConnectionFutureState::Init { sender, signal } => {
                    let s_ptr = state as *mut ConnectionFutureState<_>;
                    let handle = unsafe { crate::connections::connect(sender, signal, s_ptr) };
                    debug_assert!(handle.is_valid());
                    ConnectionFutureState::Started { handle, waker: ctx.waker().clone() }
                }
                s @ ConnectionFutureState::Started { .. } => s,
                ConnectionFutureState::Invalid => unreachable!(),
            };
            Poll::Pending
        }
    }

    impl<Args: SignalArgArrayToTuple> crate::connections::Slot<Args>
        for *mut ConnectionFutureState<Args>
    {
        unsafe fn apply(&mut self, a: *const *const c_void) {
            if let ConnectionFutureState::Started { mut handle, waker } = replace(
                &mut **self,
                ConnectionFutureState::Finished { result: Args::args_array_to_tuple(a) },
            ) {
                handle.disconnect();
                waker.wake();
            } else {
                unreachable!();
            }
        }
    }

    ConnectionFuture(ConnectionFutureState::Init { sender, signal })
}
