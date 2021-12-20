use anyhow::bail;
use anyhow::{anyhow, Context, Result};
use std::convert::{TryFrom, TryInto};
use std::fs;
use std::process;
use std::time::{self, Duration};

use process::Command;

pub struct ProcessUptime {
    pub pid: u32,
    pub uptime: time::Duration,
}

const PROC_UPTIME: &str = "/proc/uptime";

impl ProcessUptime {
    pub fn new() -> Result<Self> {
        let pid = process::id();

        let uptime = match Self::get_ps_etime(pid) {
            Ok(etime) => time::Duration::from_secs(etime),
            Err(_) => {
                // Fallback to getting from /proc/{pid}/stat
                // NOTE: changed from using /proc/{pid}/uptime file metadata as Linux exposes the
                // proc tree lazily, created on first read.
                Self::proc_stat_uptime(pid)?
            }
        };

        Ok(Self { pid, uptime })
    }

    pub(crate) fn proc_stat_uptime(pid: u32) -> Result<Duration> {
        let content = fs::read_to_string(PROC_UPTIME)?;

        let system_uptime = Self::system_uptime(&content)?;
        let process_uptime = Self::process_uptime(pid)?;

        // starttime  %llu `(22) starttime  %llu
        let sc_clk_tck: u64 = unsafe { libc::sysconf(libc::_SC_CLK_TCK).try_into()? };

        // To calculate process uptime we divide process_uptime (provided in clock ticks after
        // system boot), divided by system's clock ticks per second - libc::_SC_CLK_TCK)
        let process_uptime = Duration::from_secs(system_uptime - process_uptime / sc_clk_tck);
        Ok(process_uptime)
    }

    fn process_uptime(pid: u32) -> Result<u64> {
        // https://man7.org/linux/man-pages/man5/proc.5.html
        let content = fs::read_to_string(format!("/proc/{}/stat", pid))?;
        let process_uptime = content
            .split_whitespace()
            .nth(21)
            .ok_or_else(|| anyhow!("/proc/{}/stat 22nd column not found", pid))?;
        let process_uptime = process_uptime.parse::<f32>()? as u64;

        Ok(process_uptime)
    }

    pub(crate) fn system_uptime(content: &str) -> Result<u64> {
        let uptime = content.split_whitespace().next();

        let uptime = match uptime {
            Some(uptime) => uptime,
            None => bail!("`/proc/uptime` 0th field not present"),
        };

        Ok(uptime.parse::<f32>()? as u64)
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

#[cfg(test)]
mod test {
    use std::{process, time::Duration};

    use crate::ProcessUptime;

    #[test]
    fn test_system_uptime() {
        assert_eq!(
            ProcessUptime::system_uptime("609773.79 19153047.14\n").unwrap(),
            609773
        )
    }

    #[ignore]
    #[test]
    fn test_this_process_uptime() {
        std::thread::sleep(Duration::from_secs(3));
        let pid = process::id();
        assert_eq!(
            ProcessUptime::proc_stat_uptime(pid).unwrap(),
            Duration::from_secs(3)
        );
    }
}
