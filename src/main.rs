#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate iron;
extern crate router;

extern crate serde;
extern crate serde_json;

mod proc_fs;

use proc_fs::stats::*;
use proc_fs::ToPid;
use iron::{Iron, IronResult, Request, Response};
use router::Router;

fn proc_fs_handler(req: &mut Request) -> IronResult<Response> {
    let ref pid = req.extensions.get::<Router>().unwrap().find("pid").unwrap_or("/");
    let stats = process_statm((*pid).to_pid()).unwrap();
    let serialized = serde_json::to_string(&stats).unwrap();
    Ok(Response::with(serialized))
}

fn main() {
    let mut router = Router::new();
    router.get("/proc/:pid", proc_fs_handler);

    Iron::new(router).http("localhost:3000").unwrap();
}
