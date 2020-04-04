use std::fmt;
use std::io::Result;
use std::net::IpAddr;
use std::time::Duration;

#[derive(Debug)]
pub enum HostStatus {
    Up,
    Down,
}

impl fmt::Display for HostStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn probe_host(addr: &IpAddr, timeout: Duration) -> Result<HostStatus> {
    todo!()
}
