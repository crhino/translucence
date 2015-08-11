#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate iron;
extern crate router;

extern crate serde;
extern crate serde_json;

mod proc_fs;

use proc_fs::stats::*;
use proc_fs::kernel::*;
use proc_fs::net::*;
use proc_fs::ToPid;
use iron::{Iron, IronResult, Request, Response};
use router::Router;

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

fn main() {
    let mut router = Router::new();
    router.get("/proc/:pid/statm", proc_statm_handler);
    router.get("/proc/:pid/io", proc_io_handler);
    router.get("/proc/:pid/stack", proc_stack_handler);
    router.get("/net/tcpstats", proc_tcp_handler);

    Iron::new(router).http("localhost:3000").unwrap();
}
