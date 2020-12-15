use std::error;
use std::fs;
use std::io;
use std::time;

pub struct ProcessMetadata {
    pid: u32,
    metadata: fs::Metadata,
}

impl ProcessMetadata {
    pub fn new(pid: u32) -> Result<Self, io::Error> {
        let metadata = fs::metadata(format!("/proc/{}", pid))?;

        Ok(Self { pid, metadata })
    }

    pub fn uptime(&self) -> Result<time::Duration, Box<dyn error::Error>> {
        Ok(self.metadata.modified()?.elapsed()?)
    }
    pub fn pid(&self) -> u32 {
        self.pid
    }
}
