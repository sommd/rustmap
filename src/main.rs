mod hosts;
mod icmp;
mod ports;
mod raw_socket;

use crate::hosts::probe_host;
use crate::ports::{probe_port, PortStatus};
use std::net::{IpAddr, ToSocketAddrs};
use std::str::FromStr;
use std::time::Duration;

fn main() {
    let status = probe_host(
        &IpAddr::from_str("192.168.1.20").unwrap(),
        Duration::from_millis(1000),
    )
    .unwrap();

    println!("{:?}", status);

    return;

    for port in 0..=65535 {
        let status = probe_port(
            &format!("192.168.1.20:{}", port)
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap(),
            Duration::from_millis(1000),
        )
        .unwrap();

        if status != PortStatus::Closed {
            println!("{:5}: {}", port, status);
        }
    }
}
