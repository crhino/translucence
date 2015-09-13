#![cfg(test)]

extern crate ferrous_threads;

use ferrous_threads::thread_pool::ThreadPool;

use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::{Arc};
use std::process::{Stdio, Command};
use std::net::UdpSocket;

const ADDR: &'static str = "localhost:60000";

#[test]
fn main_metrics_test() {
    let mut pool = ThreadPool::new(2, 2);
    let test_thread = pool.thread().unwrap_or_else(|e| panic!("{}", e));
    let timer_thread = pool.thread().unwrap_or_else(|e| panic!("{}", e));
    let reg_interval = 10;

    let done = Arc::new(AtomicBool::new(false));
    let test_done = done.clone();
    let timer_done = done.clone();

    let passed = Arc::new(AtomicBool::new(false));
    let test_passed = passed.clone();

    let test_addr = ADDR.clone();

    test_thread.start(Box::new(move || {
        let mut buf = [0; 65536];
        let socket = UdpSocket::bind(test_addr)
            .unwrap_or_else(|e| panic!("{}", e));

        let res = socket.recv_from(&mut buf);
        assert!(res.is_ok());

        test_done.store(true, Ordering::SeqCst);
        test_passed.store(true, Ordering::SeqCst);
    })).unwrap_or_else(|e| panic!("{}", e));

    let child = Command::new("cargo")
        .arg("run")
        .arg(ADDR)
        .stderr(Stdio::null()) // Ignore cargo "unknown error" output
        .spawn().unwrap_or_else(|e| panic!("{}", e));

    timer_thread.start(Box::new(move || {
        ::std::thread::sleep_ms(reg_interval * 1000);
        timer_done.store(true, Ordering::SeqCst);
    })).unwrap_or_else(|e| panic!("{}", e));

    loop {
        if done.load(Ordering::SeqCst) {
            break
        }

        ::std::thread::yield_now();
    }

    let pid = child.id();
    let _output = Command::new("pkill")
        .arg("-TERM")
        .arg("-P")
        .arg(format!("{}", pid))
        .output().unwrap_or_else(|e| panic!("{}", e));
    assert!(passed.load(Ordering::SeqCst));
}
