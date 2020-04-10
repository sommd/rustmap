use crate::icmp::{IcmpEchoPacket, IcmpEchoType};
use socket2::{Domain, Protocol, Socket, Type};
use std::fmt;
use std::io::{ErrorKind, Result};
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
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
        Socket::new(Domain::ipv4(), Type::raw(), Some(Protocol::icmpv4()))
    } else {
        Socket::new(Domain::ipv6(), Type::raw(), Some(Protocol::icmpv6()))
    }?;

    socket.connect(&SocketAddr::new(*addr, 0).into())?;

    let send_packet = |identifier: u16, sequence_number: u16| {
        let buf = Vec::from(&IcmpEchoPacket::new(
            if addr.is_ipv4() {
                IcmpEchoType::Request
            } else {
                IcmpEchoType::RequestV6
            },
            identifier,
            sequence_number,
            format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).as_bytes(),
        ));

        socket.send(&buf)
    };

    let recv_packet = || -> Result<Option<IcmpEchoPacket>> {
        let mut ip_buf = [0; u16::max_value() as usize];
        let ip_len = socket.recv(&mut ip_buf)?;
        let ip_buf = &ip_buf[0..ip_len];

        Ok(if addr.is_ipv4() {
            // Raw IPv4 sockets always return the header
            IcmpEchoPacket::from_ipv4(ip_buf)
        } else {
            // Raw IPv6 sockets don't return the header by default
            IcmpEchoPacket::from(ip_buf)
        })
    };

    let deadline = Instant::now() + timeout;
    let mut remaining = timeout;

    while remaining > Duration::from_secs(0) {
        send_packet(12345, 0)?;

        remaining = deadline - Instant::now();
        socket.set_read_timeout(remaining.into())?;

        match recv_packet() {
            // Invalid reply, try again
            Ok(None) => continue,
            // Good reply, host up
            Ok(Some(_)) => return Ok(HostStatus::Up),
            Err(e) => {
                return if e.kind() == ErrorKind::WouldBlock {
                    // Timed out, host down (this is the error returned when read times out for some reason)
                    Ok(HostStatus::Down)
                } else {
                    Err(e)
                };
            }
        };
    }

    // Timed out
    Ok(HostStatus::Down)
}
