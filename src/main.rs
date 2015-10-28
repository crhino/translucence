#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
#![feature(convert)]
#![plugin(docopt_macros)]

#[macro_use] extern crate log;
extern crate env_logger;

extern crate iron;
extern crate router;

extern crate serde;
extern crate serde_json;

extern crate time;

extern crate transit;
extern crate marid;

extern crate rustc_serialize;
extern crate docopt;

#[cfg(test)] extern crate chan;

// Write the Docopt usage string with the `docopt!` macro.
docopt!(Config, "
Usage: translucence [options] <metrics-address>

    Run the translucence process with specified parameters.
Options:
    -r SECONDS  The registration interval [default: 10]
");



mod proc_fs;
mod metrics_sender;
mod util;
mod router_runner;

use marid::{launch, Composer, Runner, Signal, Process};
use std::error::Error;
use std::str::FromStr;

fn main() {
    env_logger::init().unwrap();
    let config: Config = Config::docopt().decode().unwrap_or_else(|e| e.exit());

    let router = Box::new(router_runner::RouterRunner::new()) as Box<Runner + Send>;

    let metric_addr = String::from(config.arg_metrics_address.as_str());
    let publish_interval = match u32::from_str(config.flag_r.as_str()) {
        Ok(n) => n,
        Err(_) => 10, // Default to 10 seconds
    };
    let origin = String::from("translucence");
    let metrics = Box::new(metrics_sender::MetricSender::new("0.0.0.0:0",
                                                             metric_addr,
                                                             origin,
                                                             publish_interval).unwrap())
        as Box<Runner + Send>;

    let composer = Composer::new(vec!(router, metrics));
    let process = launch(composer, vec!(Signal::INT, Signal::TERM));
    process.wait().expect("Error while running");
}
