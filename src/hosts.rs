use crate::icmp::IcmpEchoPacket;
use crate::raw_socket::RawSocket;
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
    let socket = if addr.is_ipv4() {
        RawSocket::new_ipv4()
    } else {
        RawSocket::new_ipv6()
    }?;

    let mut buf = Vec::from(&IcmpEchoPacket::new_request(
        12345,
        0,
        format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).as_bytes(),
    ));

    socket.sendto(&buf, &addr)?;

    let (_, from) = socket.recvfrom(&mut buf)?;

    Ok(if from == *addr {
        HostStatus::Up
    } else {
        HostStatus::Down
    })
}
