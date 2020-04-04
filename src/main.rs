mod hosts;
mod ports;

use crate::hosts::probe_host;
use crate::ports::probe_port;
use std::net::ToSocketAddrs;
use std::time::Duration;

fn main() {
    println!(
        "{:?}",
        probe_port(
            &"google.com:81".to_socket_addrs().unwrap().next().unwrap(),
            Duration::from_secs(3)
        )
    );

    println!(
        "{:?}",
        probe_host(
            &"google.com:0"
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap()
                .ip(),
            Duration::from_secs(3)
        )
    );
}
