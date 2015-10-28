use transit::udp::*;
use std::net::{ToSocketAddrs};
use std::error::Error;
use std::fmt;
use std::sync::atomic::{Ordering, AtomicBool};
use std::sync::{Arc};

use time::{get_time};
use proc_fs::stats::{ProcStatm};
use proc_fs::net::{process_tcp, TcpStat};
use util::handle_signals_atomic;

use marid::{MaridError, Runner, Receiver, Signal};


#[derive(Debug, Serialize, Deserialize)]
pub struct MetricPacket {
    origin: String,
    timestamp: i64,
    // ip: IpAddr,
    data: Metric,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Metric {
    Network(TcpStat),
    Process(ProcStatm),
}

#[derive(Debug)]
pub enum MetricError {
    UDPError(Box<Error + Sync + Send>),
}

impl From<TransitError> for MetricError {
    fn from(err: TransitError) -> MetricError {
        MetricError::UDPError(Box::new(err))
    }
}

impl Error for MetricError {
    fn description(&self) -> &str {
        match *self {
            MetricError::UDPError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MetricError::UDPError(ref err) => err.cause(),
        }
    }
}

impl fmt::Display for MetricError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MetricError::UDPError(ref err) =>
                write!(fmt, "UDPError: {}", err),
        }
    }
}

pub struct MetricSender {
    origin: String,
    transit: Transit,
    metric_addr: String,
    publish_interval: u32,
    shutdown: Arc<AtomicBool>,
}

impl MetricSender {
    pub fn new<A>(addr: A, metric_addr: String, origin: String, interval: u32)
        -> Result<MetricSender, MetricError> where A: ToSocketAddrs {
            let transit = try!(Transit::new(addr));
            Ok(MetricSender {
                origin: origin,
                transit: transit,
                metric_addr: metric_addr,
                publish_interval: interval,
                shutdown: Arc::new(AtomicBool::new(false)),
            })
        }

    pub fn send_to<A>(&mut self, data: Metric, addr: A) -> Result<(), MetricError> where A: ToSocketAddrs {
        // let address = try!(self.transit.local_addr());
        let time = get_time().sec;
        let pkt = MetricPacket {
            origin: self.origin.clone(),
            timestamp: time,
            // ip: address.ip(),
            data: data,
        };
        try!(self.transit.send_to(&pkt, addr));
        Ok(())
    }
}

impl Runner for MetricSender {
    fn run(mut self: Box<Self>, signals: Receiver<Signal>) -> Result<(), MaridError> {
        debug!("Running MetricSender");
        handle_signals_atomic(signals, self.shutdown.clone());
        let metric_addr = self.metric_addr.clone();

        loop {
            debug!("Checking for signals");
            if self.shutdown.load(Ordering::SeqCst) {
                info!("Received shutdown signal");
                break
            }

            debug!("Attempting to send metrics...");
            let tcp = match process_tcp() {
                Ok(t) => t,
                Err(ref e) => {
                    warn!("Error getting tcp info: {}", e);
                    continue
                },
            };

            let data = Metric::Network(tcp);
            match self.send_to(data, metric_addr.as_str()) {
                Ok(_) => {},
                Err(ref e) => warn!("Error sending metrics: {}", e),
            }

            debug!("metrics sent");
            ::std::thread::sleep_ms(self.publish_interval * 1000);
        }

        Ok(())
    }

    fn setup(&mut self) -> Result<(), MaridError> {
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use transit::udp::*;
    use super::*;
    use proc_fs::stats::*;
    use chan;
    use std::thread;
    use marid::{Signal, Runner};

    #[test]
    fn test_send_to() {
        let metric_addr = "127.0.0.1:60000";
        let listen_addr = "127.0.0.1:60001";
        let mut metric_sender = MetricSender::new(metric_addr,
                                                  String::from(listen_addr),
                                                  String::from("test-sender"),
                                                  10).unwrap();
        let mut listener = Transit::new(listen_addr).unwrap();

        let data = Metric::Process(process_statm(String::from("self")).unwrap());

        let res = metric_sender.send_to(data.clone(), listen_addr);
        assert!(res.is_ok());
        let res = listener.recv_from();
        assert!(res.is_ok());
        let (net_data, _addr): (MetricPacket, _) = res.unwrap();
        assert_eq!(data, net_data.data);
    }

    #[test]
    fn test_runner() {
        let metric_addr = "127.0.0.1:60002";
        let listen_addr = "127.0.0.1:60003";
        let metric_sender = Box::new(MetricSender::new(metric_addr,
                                              String::from(listen_addr),
                                              String::from("test-sender"),
                                              1).unwrap()) as Box<Runner + Send>;
        let mut listener = Transit::new(listen_addr).unwrap();

        let (sn, rc) = chan::sync(1);
        let (test_sn, test_rc) = chan::sync(0);
        let thread = thread::spawn(move || {
            assert!(metric_sender.run(rc).is_ok());
            test_sn.send(true);
        });

        let res: Result<(MetricPacket,_), _> = listener.recv_from();
        assert!(res.is_ok());

        sn.send(Signal::INT);
        assert!(test_rc.recv().unwrap());
        thread.join().expect("Could not join thread");
    }
}
