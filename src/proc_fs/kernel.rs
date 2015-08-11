use std::fs::File;
use std::string::String;
use std::io::{self, Read};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcStack {
    kernel_trace: String,
}

pub fn process_stack(pid: String) -> io::Result<ProcStack> {
    let mut f = try!(File::open(format!("/proc/{}/stack", pid)));
    let mut trace = String::new();
    let _n = f.read_to_string(&mut trace).unwrap();

    Ok(ProcStack {
        kernel_trace: trace,
    })
}

#[cfg(test)]
mod test {
    use std::process::Command;
    use proc_fs::kernel::*;
    use proc_fs::ToPid;

    #[test]
    fn test_proc_stack() {
        let id = Command::new("sh")
            .arg("-c")
            .arg("sleep 1")
            .spawn()
            .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) }).id();

        let stats = process_stack(id.to_pid());
        assert!(stats.is_ok());

        let stats = process_stack("self".to_pid());
        assert!(stats.is_ok());
    }
}
