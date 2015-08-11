use std::fs::File;
use std::string::String;
use std::io::{self, Read};
use std::collections::HashMap;

pub fn process_tcp<'a>() -> io::Result<HashMap<String, String>> {
    let mut f = try!(File::open("/proc/net/snmp"));
    let mut snmp = String::new();
    let _n = f.read_to_string(&mut snmp).unwrap();


    let mut tcp = snmp.lines()
        .filter(|s| &s[0..3] == "Tcp")
        .map(|s| &s[5..])
        .collect::<Vec<&str>>().into_iter();

    let names = tcp.next().unwrap();
    let values = tcp.next().unwrap();
    let mut hash = HashMap::new();

    for (k, v) in names.split_whitespace().zip(values.split_whitespace()) {
        hash.insert(String::from(k), String::from(v));
    }

    Ok(hash)
}

#[cfg(test)]
mod test {
    use std::process::Command;
    use proc_fs::net::*;
    use proc_fs::ToPid;

    #[test]
    fn test_proc_stack() {
        let tcp = process_tcp();
        assert!(tcp.is_ok());
    }
}
