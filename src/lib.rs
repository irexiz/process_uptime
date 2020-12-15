use std::convert::TryFrom;
use std::error;
use std::fs;
use std::process;
use std::time;

pub struct ProcessUptime {
    pub pid: u32,
    pub uptime: time::Duration,
}

impl ProcessUptime {
    pub fn new() -> Result<Self, Box<dyn error::Error>> {
        let pid = process::id();
        let metadata = fs::metadata(format!("/proc/{}", pid))?;
        let uptime = metadata.modified()?.elapsed()?;

        Ok(Self { pid, uptime })
    }
}

impl TryFrom<u32> for ProcessUptime {
    type Error = Box<dyn error::Error>;
    fn try_from(pid: u32) -> Result<Self, Self::Error> {
        let metadata = fs::metadata(format!("/proc/{}", pid))?;
        let uptime = metadata.modified()?.elapsed()?;

        Ok(Self { pid, uptime })
    }
}
