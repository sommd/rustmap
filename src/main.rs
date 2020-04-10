mod hosts;
mod icmp;
mod ip;
mod ports;

use crate::hosts::probe_host;
use crate::ports::{probe_port, PortStatus};
use std::net::{IpAddr, ToSocketAddrs};
use std::str::FromStr;
use std::time::Duration;

fn main() {
    println!(
        "{}",
        probe_host(
            &IpAddr::from_str("192.168.1.20").unwrap(),
            Duration::from_millis(1000),
        )
        .unwrap()
    );

    println!(
        "{}",
        probe_host(
            &IpAddr::from_str("2403:5800:7102:8900:922b:34ff:fe5f:1ef4").unwrap(),
            Duration::from_millis(1000),
        )
        .unwrap()
    );

    println!(
        "{}",
        probe_host(
            &IpAddr::from_str("192.168.1.123").unwrap(),
            Duration::from_millis(1000),
        )
        .unwrap()
    );

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
