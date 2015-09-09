use std::fs::File;
use std::string::String;
use std::str::FromStr;
use std::io::{self, Read};

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct TcpStat {
    rto_algorithm: i8,
    rto_min: isize,
    rto_max: isize,
    max_conn: isize, // -1 means dynamically determined
    active_opens: usize,
    passive_opens: usize,
    attempt_fails: usize,
    establish_resets: usize,
    current_establish: usize,
    segments_received: usize,
    segments_sent: usize,
    segments_retransmitted: usize,
    segments_errors_received: usize,
    resets_sent: usize,
}

pub fn process_tcp<'a>() -> io::Result<TcpStat> {
    let mut f = try!(File::open("/proc/net/snmp"));
    let mut snmp = String::new();
    let _n = f.read_to_string(&mut snmp).unwrap();


    let mut tcp = snmp.lines()
        .filter(|s| &s[0..3] == "Tcp")
        .map(|s| &s[5..])
        .collect::<Vec<&str>>().into_iter();

    let _names = tcp.next().unwrap();
    let values = tcp.next().unwrap().split_whitespace().collect::<Vec<&str>>();
    assert_eq!(values.len(), 15);

    let stats = TcpStat{
        rto_algorithm: i8::from_str(values[0]).unwrap(),
        rto_min: isize::from_str(values[1]).unwrap(),
        rto_max: isize::from_str(values[2]).unwrap(),
        max_conn: isize::from_str(values[3]).unwrap(),
        active_opens: usize::from_str(values[4]).unwrap(),
        passive_opens: usize::from_str(values[5]).unwrap(),
        attempt_fails: usize::from_str(values[6]).unwrap(),
        establish_resets: usize::from_str(values[7]).unwrap(),
        current_establish: usize::from_str(values[8]).unwrap(),
        segments_received: usize::from_str(values[9]).unwrap(),
        segments_sent: usize::from_str(values[10]).unwrap(),
        segments_retransmitted: usize::from_str(values[11]).unwrap(),
        segments_errors_received: usize::from_str(values[12]).unwrap(),
        resets_sent: usize::from_str(values[13]).unwrap(),
    };

    Ok(stats)
}

#[cfg(test)]
mod test {
    use proc_fs::net::*;

    #[test]
    fn test_proc_stack() {
        let tcp = process_tcp();
        assert!(tcp.is_ok());
    }
}
