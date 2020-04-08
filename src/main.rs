mod hosts;
mod ports;

use crate::ports::{probe_port, PortStatus};
use std::net::ToSocketAddrs;
use std::time::Duration;

fn main() {
    for port in 0..=65535 {
        let status = probe_port(
            &format!("192.168.1.20:{}", port).to_socket_addrs().unwrap().next().unwrap(),
            Duration::from_millis(1000)
        ).unwrap();

        if status != PortStatus::Closed {
            println!("{:5}: {}", port, status);
        }
    }
}
