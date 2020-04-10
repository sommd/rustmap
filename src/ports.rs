use nix::errno::Errno;
use std::fmt;
use std::io::{ErrorKind, Result};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortStatus {
    Open,
    Closed,
    Filtered,
    HostDown,
}

impl fmt::Display for PortStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn probe_port(addr: &SocketAddr, timeout: Duration) -> Result<PortStatus> {
    match TcpStream::connect_timeout(&addr, timeout) {
        Ok(_) => Ok(PortStatus::Open),
        Err(e) => match e.kind() {
            ErrorKind::TimedOut => Ok(PortStatus::Filtered),
            ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset => Ok(PortStatus::Closed),
            _ => match e.raw_os_error().map(Errno::from_i32) {
                Some(Errno::ENETUNREACH) | Some(Errno::EHOSTUNREACH) => Ok(PortStatus::HostDown),
                _ => Err(e),
            },
        },
    }
}
