/*
 * Get a process's status from /proc/<pid>/stat
 */
use std::fs::File;
use std::string::String;
use std::str::FromStr;
use std::io::{self, Read};

#[derive(Debug)]
#[allow(dead_code)]
struct ProcStat {
    pid: isize,
    command: String,
    state: char,
    ppid: isize, // PID of the parent
    gprp: isize, // Process group id of process
    session: isize,
    tty_nr: isize, // Controlling terminal
    tpgid: isize, // id of foreground process group of controlling terminal
    flags: usize, // Kernel flags word. See PF_* defines
    minfly: usize, // # of minor faults
    cminflt: usize, // # of minor faults children have made
    majflt: usize, // # of major faults process has made
    cmajflt: usize, // # of major faults children have made
    utime: usize, // Amt of time process has been scheduled in usermode
    stime: usize, // Amt of time process has been scheduled in kernel mode
    cutime: isize, // Amt of time children have been scheduled in usermode
    cstime: isize, // Amt of time children has been scheduled in kernel mode
    priority: isize, // scheduling priority
    nice: isize, // nice value
    num_threads: isize,
    itrealval: isize, // time in jiffies before next SIGALRM
    starttime: usize, // Time started after system boot
    vsize: usize, // virtual memory size in bytes
    rss: isize, // Resident Set Size: # of pages in real memory
    rsslim: usize, // soft limit in bytes on rss of process
    startcode: usize, // Address above which program text can run
    endcode: usize,  // Address below which program text can run
    startstack: usize, // Address of start of stack
    kstkesp: usize, // current EIP (instruction pointer)
    signal: usize, // bitmap of pending signals (Obsolete)
    blocked: usize, // bitmap of blocked signals (Obsolete)
    sigignore: usize, // bitmap of ignored signals (Obsolete)
    sigcatch: usize, // bitmap of caught signals (Obsolete)
    wchan: usize, // "wait channel", address in kernel where process is sleeping
    nswap: usize, // # of pages swapped (not maintained)
    cnswap: usize, // # of pages swapped for child processes (not maintained)
    exit_signal: isize, // Signal to be sent to parent when we die
    processor: usize, // CPU number last executed on
    rt_priority: usize, // real-time scheduling priority (1-99) or 0
    policy: usize, // Scheduling policy, SCHED_* constants
    delayacct_blkio_ticks: usize, // Aggregated block i/o delays in clock ticks
    guest_time: usize, // Guest time of process (time on virtual CPU) in clock ticks
    cguest_time: isize, // Guest time of children (time on virtual CPU) in clock ticks
    start_data: usize, // Address above which program {un,}initialized data is placed
    end_data: usize, // Address below which program {un,}initialized data is placed
    start_brk: usize, // Address above which program can be expanded
    arg_start: usize, // Address above which program command-line args are placed
    arg_end: usize, // Address below which program command-line args are placed
    env_start: usize, // Address above which program environment are placed
    env_end: usize, // Address below which program environment are placed
    exit_code: usize, // thread's exit status
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct ProcStatm {
    size: usize, // total program size
    resident: usize, // resident set size
    share: usize, // shared pages (i.e. backed by file)
    text: usize, // text (code)
    lib: usize, // Library
    data: usize, // Data + stack
    dt: usize, // Dirty pages
}

pub fn process_statm(pid: String) -> io::Result<ProcStatm> {
    let mut f = try!(File::open(format!("/proc/{}/statm", pid)));
    let mut stats_str = String::new();
    let _n = f.read_to_string(&mut stats_str).unwrap();

    let stats: Vec<usize> = stats_str.split_whitespace()
        .map(|s| usize::from_str(s).unwrap())
        .collect();
    assert_eq!(stats.len(), 7);

    Ok(ProcStatm {
        size: stats[0],
        resident: stats[1],
        share: stats[2],
        text: stats[3],
        lib: stats[4],
        data: stats[5],
        dt: stats[6],
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcIo {
    rchar: usize, // bytes passed to read syscalls
    wchar: usize, // bytes passed to write syscalls
    syscr: usize, // read syscall count
    syscw: usize, // write syscall count
    read_bytes: usize, // Count of bytes read from storage layer
    write_bytes: usize, // Count of bytes sent to storage layer
    cancelled_write_bytes: usize, // count of bytes the process caused to not be written
}

pub fn process_io(pid: String) -> io::Result<ProcIo> {
    let mut f = try!(File::open(format!("/proc/{}/io", pid)));
    let mut stats_str = String::new();
    let _n = f.read_to_string(&mut stats_str).unwrap();

    let stats: Vec<usize> = stats_str.split_whitespace()
        .enumerate()
        .filter(|&(i, _s)| i % 2 != 0)
        .map(|(_i, s)| s)
        .map(|s| usize::from_str(s).unwrap())
        .collect();
    assert_eq!(stats.len(), 7);

    Ok(ProcIo {
        rchar: stats[0],
        wchar: stats[1],
        syscr: stats[2],
        syscw: stats[3],
        read_bytes: stats[4],
        write_bytes: stats[5],
        cancelled_write_bytes: stats[6],
    })
}

#[cfg(test)]
mod test {
    use std::process::Command;
    use proc_fs::stats::*;
    use proc_fs::ToPid;

    #[test]
    fn test_proc_statm() {
        let id = Command::new("sh")
            .arg("-c")
            .arg("sleep 1")
            .spawn()
            .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) }).id();

        let stats = process_statm(id.to_pid());
        assert!(stats.is_ok());

        let stats = process_statm("self".to_pid());
        assert!(stats.is_ok());
    }

    #[test]
    fn test_proc_io() {
        let id = Command::new("sh")
            .arg("-c")
            .arg("sleep 1")
            .spawn()
            .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) }).id();

        let stats = process_io(id.to_pid());
        assert!(stats.is_ok());

        let stats = process_io("self".to_pid());
        assert!(stats.is_ok());
    }
}
