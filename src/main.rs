#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
#![feature(convert, result_expect)]
#![plugin(docopt_macros)]

#[macro_use] extern crate log;
extern crate env_logger;

extern crate iron;
extern crate router;

extern crate serde;
extern crate serde_json;

extern crate time;

extern crate transit;
extern crate ferrous_threads;

extern crate rustc_serialize;
extern crate docopt;

// Write the Docopt usage string with the `docopt!` macro.
docopt!(Config, "
Usage: translucence [options] <metrics-address>

    Run the translucence process with specified parameters.

Options:
    -r SECONDS  The registration interval [default: 10]
");



mod proc_fs;
mod metrics_sender;

use proc_fs::stats::*;
use proc_fs::kernel::*;
use proc_fs::net::*;
use proc_fs::ToPid;
use metrics_sender::*;
use iron::{Iron, IronResult, Request, Response};
use router::Router;

use ferrous_threads::thread_pool::{ThreadPool};
use std::error::Error;
use std::str::FromStr;

fn main() {
    env_logger::init().unwrap();
    let config: Config = Config::docopt().decode().unwrap_or_else(|e| e.exit());
    let mut pool = ThreadPool::new(2, 2);

    let  router_thread = pool.thread().expect("Could not request router thread");
    router_thread.start(Box::new(move || {
        debug!("Starting router thread...");

        let mut router = Router::new();
        router.get("/proc/:pid/statm", proc_statm_handler);
        router.get("/proc/:pid/io", proc_io_handler);
        router.get("/proc/:pid/stack", proc_stack_handler);
        router.get("/net/tcpstats", proc_tcp_handler);

        Iron::new(router).http("localhost:3000").unwrap();

        debug!("Router thread finished");
    })).expect("Could not start router thread");

    let  metrics_thread = pool.thread().expect("Could not request metrics thread");
    metrics_thread.start(Box::new(move || {
        debug!("Starting metrics thread...");
        metrics_reporting(config);
        debug!("Metrics thread finished");
    })).expect("Could not start metrics thread");

    router_thread.join().expect("Failed to join router thread");
    metrics_thread.join().expect("Failed to join metrics thread");
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

fn metrics_reporting(config: Config) {
    let mut metrics_sender = MetricSender::new("0.0.0.0:0", String::from("translucence"))
        .expect("Could not create metric sender");
    let metric_addr = config.arg_metrics_address.as_str();
    let publish_interval = match u32::from_str(config.flag_r.as_str()) {
        Ok(n) => n,
        Err(_) => 10, // Default to 10 seconds
    };

    loop {
        debug!("Attempting to send metrics...");

        let tcp = match process_tcp() {
            Ok(t) => t,
            Err(ref e) => {
                warn!("Error getting tcp info: {}", e);
                continue
            },
        };

        let data = Metric::Network(tcp);
        match metrics_sender.send_to(data, metric_addr) {
            Ok(_) => {},
            Err(ref e) => warn!("Error sending metrics: {}", e),
        }

        debug!("metrics sent");
        ::std::thread::sleep_ms(publish_interval * 1000);
    }
}
