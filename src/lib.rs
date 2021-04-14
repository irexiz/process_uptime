use anyhow::{anyhow, Context, Result};
use std::convert::TryFrom;
use std::fs;
use std::process;
use std::time;

use process::Command;

pub struct ProcessUptime {
    pub pid: u32,
    pub uptime: time::Duration,
}

impl ProcessUptime {
    pub fn new() -> Result<Self> {
        let pid = process::id();

        let uptime = match Self::get_ps_etime(pid) {
            Ok(etime) => time::Duration::from_secs(etime),
            Err(_) => {
                // Fallback to getting from /proc/{}
                let metadata = fs::metadata(format!("/proc/{}", pid))?;
                metadata.modified()?.elapsed()?
            }
        };

        Ok(Self { pid, uptime })
    }

    fn get_ps_etime(pid: u32) -> Result<u64> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("ps -o etimes -p {} --no-headers", pid).as_str())
            .output()?;

        let mut uptime_string = std::str::from_utf8(output.stdout.as_slice())?.to_string();

        if uptime_string.ends_with('\n') {
            uptime_string.pop();
            if uptime_string.ends_with('\r') {
                uptime_string.pop();
            }
        }

        uptime_string
            .parse::<u64>()
            .with_context(|| anyhow!("Failed to parse {} to u64", uptime_string))
    }
}

impl TryFrom<u32> for ProcessUptime {
    type Error = anyhow::Error;
    fn try_from(pid: u32) -> Result<Self, Self::Error> {
        let metadata = fs::metadata(format!("/proc/{}", pid))?;
        let uptime = metadata.modified()?.elapsed()?;

        Ok(Self { pid, uptime })
    }
}
