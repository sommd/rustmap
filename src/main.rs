mod cidr;
mod hosts;
mod icmp;
mod ports;

use crate::cidr::IpAddrRange;
use crate::hosts::{probe_host, HostStatus};
use crate::ports::probe_port;
use parse_duration;
use std::net::SocketAddr;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
/// Scan for hosts and open ports.
///
/// Needs to be run as root, or with the CAP_NET_RAW capability on Linux.
///
/// EXAMPLES:{n}
/// {n}rustmap 127.0.0.1                   Check if a single host is up.
/// {n}rustmap 127.0.0.1 -p                Scan all TCP ports for a single host.
/// {n}rustmap 127.0.0.1 ::1 -p            Scan all TCP ports for multiple hosts.
/// {n}rustmap 127.0.0.0/8 -p 22,80,443    Scan for specific ports in an address range.
struct Opt {
    #[structopt(short, long, require_delimiter = true)]
    /// Probe ports for each host and optionally specify which ports.
    ///
    /// Ports can be specified as a comma-separated list, or left unspecified to scan all ports.
    ports: Option<Vec<u16>>,

    #[structopt(short, long, parse(try_from_str = parse_duration::parse), default_value = "1s")]
    /// Timeout for pinging each host and probing each port.
    ///
    /// Parsing is provided by the 'parse_duration' crate and supports almost any notation.
    /// E.g. '1s', '10 seconds', '1 hour, 15 minutes, 12 seconds', '10m32s112ms'.
    timeout: Duration,

    #[structopt(required = true)]
    /// IP addresses to scan.
    ///
    /// Supports IPv4 notation (e.g. '127.0.0.1'), IPv6 notation (e.g. '::1'), IPv4-mapped IPv6
    /// notation (e.g. '::ffff::1.1.1.1') and CIDR notation (e.g. '192.168.0.0/16', 'fe80::/10').
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
