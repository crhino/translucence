use transit::udp::*;
use std::net::{ToSocketAddrs};
use std::error::Error;
use std::fmt;

use time::{get_time};
use proc_fs::stats::{ProcStatm};
use proc_fs::net::{TcpStat};


#[derive(Debug, Serialize, Deserialize)]
pub struct MetricPacket {
    origin: String,
    timestamp: i64,
    // ip: IpAddr,
    data: Metric,
}

impl MetricPacket {
    pub fn origin(&self) -> &str {
        self.origin.as_str()
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Metric {
    Network(TcpStat),
    Process(ProcStatm),
}

pub struct MetricSender {
    origin: String,
    transit: Transit,
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

impl MetricSender {
    pub fn new<A>(addr: A, origin: String) -> Result<MetricSender, MetricError> where A: ToSocketAddrs {
        let transit = try!(Transit::new(addr));
        Ok(MetricSender {
            origin: origin,
            transit: transit,
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

#[cfg(test)]
mod test {
    use transit::udp::*;
    use super::*;
    use proc_fs::stats::*;

    #[test]
    fn test_send_to() {
        let metric_addr = "127.0.0.1:60000";
        let listen_addr = "127.0.0.1:60001";
        let mut metric_sender = MetricSender::new(metric_addr, String::from("test-sender")).unwrap();
        let mut listener = Transit::new(listen_addr).unwrap();

        let data = Metric::Process(process_statm(String::from("self")).unwrap());

        let res = metric_sender.send_to(data.clone(), listen_addr);
        assert!(res.is_ok());
        let res = listener.recv_from();
        assert!(res.is_ok());
        let (net_data, _addr): (MetricPacket, _) = res.unwrap();
        assert_eq!(data, net_data.data);
    }
}
