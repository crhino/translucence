use proc_fs::stats::*;
use proc_fs::kernel::*;
use proc_fs::net::*;
use proc_fs::ToPid;
use iron::{Iron, IronResult, Request, Response};
use router::Router;

use std::sync::{Arc, Mutex, Condvar};

use serde_json;

use marid::{MaridError, Runner, Signal, Receiver};
use util::handle_signals_condvar;

pub struct RouterRunner {
    router: Option<Router>,
}

impl RouterRunner {
    pub fn new() -> RouterRunner {
        let mut router = Router::new();
        router.get("/proc/:pid/statm", proc_statm_handler);
        router.get("/proc/:pid/io", proc_io_handler);
        router.get("/proc/:pid/stack", proc_stack_handler);
        router.get("/net/tcpstats", proc_tcp_handler);

        RouterRunner {
            router: Some(router),
        }
    }
}

impl Runner for RouterRunner {
    fn setup(&mut self) -> Result<(), MaridError> {
        Ok(())
    }

    fn run(mut self: Box<Self>, signals: Receiver<Signal>) -> Result<(), MaridError> {
        debug!("Running RouterRunner");
        let shutdown = Arc::new((Mutex::new(false), Condvar::new()));

        debug!("Setting up signal handling");
        handle_signals_condvar(signals, shutdown.clone());
        let router = self.router.take().expect("Could not take router");

        let mut listener = Iron::new(router).http("localhost:3000").unwrap();
        debug!("Serving requests...");

        let &(ref lock, ref cvar) = &*shutdown;
        let mut stop = lock.lock().expect("Lock was poisoned");
        while !*stop {
            stop = cvar.wait(stop).expect("Lock was poisoned");
        }

        debug!("Shutting down router thread...");

        match listener.close() {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}

fn proc_statm_handler(req: &mut Request) -> IronResult<Response> {
    let ref pid = req.extensions.get::<Router>().unwrap().find("pid").unwrap_or("/");
    let stats = process_statm((*pid).to_pid()).unwrap();
    let serialized = serde_json::to_string(&stats).unwrap();
    Ok(Response::with(serialized))
}

fn proc_io_handler(req: &mut Request) -> IronResult<Response> {
    let ref pid = req.extensions.get::<Router>().unwrap().find("pid").unwrap_or("/");
    let stats = process_io((*pid).to_pid()).unwrap();
    let serialized = serde_json::to_string(&stats).unwrap();
    Ok(Response::with(serialized))
}

fn proc_stack_handler(req: &mut Request) -> IronResult<Response> {
    let ref pid = req.extensions.get::<Router>().unwrap().find("pid").unwrap_or("/");
    let stack_trace = process_stack((*pid).to_pid()).unwrap();
    let serialized = serde_json::to_string(&stack_trace).unwrap();
    Ok(Response::with(serialized))
}

fn proc_tcp_handler(_req: &mut Request) -> IronResult<Response> {
    let tcp = process_tcp().unwrap();
    let serialized = serde_json::to_string(&tcp).unwrap();
    Ok(Response::with(serialized))
}
