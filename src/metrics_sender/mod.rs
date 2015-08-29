use transit::udp::*;
use std::net::{ToSocketAddrs};
use std::io;
use std::io::{ErrorKind};
use serde_json;
use std::error::Error;

use time::{get_time};


#[derive(Debug, Serialize, Deserialize)]
pub struct MetricPacket {
    origin: String,
    timestamp: i64,
    // ip: IpAddr,
    data: Metric,
}

impl FromTransit for MetricPacket {
    fn from_transit(buf: &[u8]) -> io::Result<MetricPacket> {
        let res = serde_json::de::from_slice(buf);
        match res {
            Ok(metric) => Ok(metric),
            Err(json_err) => {
                Err(io::Error::new(ErrorKind::Other, json_err.description()))
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Metric {
    Network(String),
    Process(String),
    Kernel(String),
}

pub struct MetricSender {
    origin: String,
    transit: Transit<String>,
}

impl MetricSender {
    pub fn new<A>(addr: A, origin: String) -> MetricSender where A: ToSocketAddrs {
        let transit = Transit::new(addr).unwrap();
        MetricSender {
            origin: origin,
            transit: transit,
        }
    }

    pub fn send_to<A>(&self, data: Metric, addr: A) -> io::Result<()> where A: ToSocketAddrs {
        // let address = try!(self.transit.local_addr());
        let time = get_time().sec;
        let pkt = MetricPacket {
            origin: self.origin.clone(),
            timestamp: time,
            // ip: address.ip(),
            data: data,
        };
        let serialized = serde_json::to_string(&pkt) .unwrap();
        println!("{:?}", serialized);
        self.transit.send_to(&serialized, addr)
    }
}

#[cfg(test)]
mod test {
    use transit::udp::*;
    use super::*;

    #[test]
    fn test_send_to() {
        let metric_addr = "127.0.0.1:60000";
        let listen_addr = "127.0.0.1:60001";
        let metric_sender = MetricSender::new(metric_addr, String::from("test-sender"));
        let listener: Transit<MetricPacket> = Transit::new(listen_addr).unwrap();

        let data = Metric::Kernel(String::from("this is a test"));

        let res = metric_sender.send_to(data.clone(), listen_addr);
        assert!(res.is_ok());
        let res = listener.recv_from();
        println!("{:?}", res);
        assert!(res.is_ok());
        let (net_data, _addr) = res.unwrap();
        assert_eq!(data, net_data.data);
    }
}
