use std::fs;
use std::process;

fn fetch_metadata() {
    let pid = process::id();
    let metadata = fs::metadata(format!("/proc/{}", pid)).expect("pid metadata");
    println!("{:?}", pid);
    let time = metadata
        .modified()
        .expect("modified")
        .elapsed()
        .expect("time");
    println!("{:?}", time);
}

