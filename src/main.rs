mod cidr;
mod hosts;
mod icmp;
mod ip;
mod ports;

use crate::cidr::IpAddrRange;
use crate::hosts::{probe_host, HostStatus};
use crate::ports::{probe_port, PortStatus};
use parse_duration;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rustmap", about = "Scan for hosts or open ports.")]
struct Opt {
    #[structopt(short = "p", long = "--ports")]
    ports: Option<Vec<u16>>,

    #[structopt(short, long, parse(try_from_str = parse_duration::parse), default_value = "1s")]
    timeout: Duration,

    #[structopt(required = true)]
    addr_ranges: Vec<IpAddrRange>,
}

fn main() {
    let mut opt = Opt::from_args();

    opt.ports = opt.ports.map(|ports| {
        if ports.is_empty() {
            (0..u16::max_value()).collect()
        } else {
            ports
        }
    });

    opt.addr_ranges
        .iter()
        .flat_map(|range| range.iter())
        .for_each(|addr| {
            let host_status = probe_host(&addr, opt.timeout);
            let host_up = matches!(host_status, Ok(HostStatus::Up));

            println!(
                "{:<16} {}",
                addr,
                host_status
                    .map(|status| status.to_string())
                    .unwrap_or_else(|e| e.to_string())
            );

            if host_up {
                if let Some(ref ports) = opt.ports {
                    ports.iter().for_each(|port| {
                        let port_status = probe_port(&SocketAddr::new(addr, *port), opt.timeout);

                        println!(
                            "  :{:<5} {}",
                            port,
                            port_status
                                .map(|status| status.to_string())
                                .unwrap_or_else(|e| e.to_string())
                        );
                    });
                }
            }
        });
}
