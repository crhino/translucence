use transit::udp::*;
use std::net::{ToSocketAddrs};
use std::error::Error;

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

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Metric {
    Network(TcpStat),
    Process(ProcStatm),
}

pub struct MetricSender {
    origin: String,
    transit: Transit,
}

impl MetricSender {
    pub fn new<A>(addr: A, origin: String) -> MetricSender where A: ToSocketAddrs {
        let transit = Transit::new(addr).unwrap();
        MetricSender {
            origin: origin,
            transit: transit,
        }
    }

    pub fn send_to<A>(&mut self, data: Metric, addr: A) -> Result<(), TransitError> where A: ToSocketAddrs {
        // let address = try!(self.transit.local_addr());
        let time = get_time().sec;
        let pkt = MetricPacket {
            origin: self.origin.clone(),
            timestamp: time,
            // ip: address.ip(),
            data: data,
        };
        self.transit.send_to(&pkt, addr)
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
        let mut metric_sender = MetricSender::new(metric_addr, String::from("test-sender"));
        let mut listener = Transit::new(listen_addr).unwrap();

        let data = Metric::Process(process_statm(String::from("self")).unwrap());

        let res = metric_sender.send_to(data.clone(), listen_addr);
        assert!(res.is_ok());
        let res = listener.recv_from();
        println!("{:?}", res);
        assert!(res.is_ok());
        let (net_data, _addr): (MetricPacket, _) = res.unwrap();
        assert_eq!(data, net_data.data);
    }
}
