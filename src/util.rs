use std::thread;
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::{Arc, Mutex, Condvar};
use marid::{Receiver, Signal};

/// This function will attempt to receive a signal.
/// If a signal is received, shutdown variable is set to true.
pub fn handle_signals_atomic(rc: Receiver<Signal>, shutdown: Arc<AtomicBool>) {
    let _thread = thread::spawn(move || {
        let _sig = rc.recv(); // Blocking call
        shutdown.store(true, Ordering::SeqCst);
    });
}

/// This function will attempt to receive a signal.
/// If a signal is received, shutdown variable is set to true and cond var is
/// notified.
pub fn handle_signals_condvar(rc: Receiver<Signal>, shutdown: Arc<(Mutex<bool>, Condvar)>) {
    let _thread = thread::spawn(move || {
        let _sig = rc.recv(); // Blocking call
        let &(ref lock, ref cvar) = &*shutdown;
        let mut stop = lock.lock().unwrap();
        *stop = true;
        cvar.notify_one();
    });
}
