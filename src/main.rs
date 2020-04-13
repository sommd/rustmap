mod cidr;
mod hosts;
mod icmp;
mod ports;

use crate::cidr::IpAddrRange;
use crate::hosts::{probe_host, HostStatus};
use crate::ports::{probe_port, PortStatus};
use parse_duration;
use std::io::{self, Write};
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
    let Opt {
        ports,
        timeout,
        addr_ranges,
    } = Opt::from_args();

    let show_closed_ports = matches!(ports, Some(ref vec) if !vec.is_empty());

    let ports = match ports {
        Some(ref vec) if vec.is_empty() => (0..u16::max_value()).collect(),
        Some(vec) => vec,
        None => Vec::default(),
    };

    addr_ranges
        .iter()
        // Flatten IpAddrRanges into IpAddrs
        .flatten()
        // Probe each host
        .flat_map(|addr| {
            print!("{:<16} ", addr);
            io::stdout().flush().expect("flush stdout");

            let status = probe_host(&addr, timeout);

            match &status {
                Ok(status) => println!("{}", status),
                Err(error) => println!("{}", error),
            }

            // Return address only if host is up
            match status {
                Ok(HostStatus::Up) => Some(addr),
                _ => None,
            }
        })
        // Flat map IpAddrs into SocketAddrs for each port
        .flat_map(|up_addr| {
            ports
                .iter()
                .map(move |port| SocketAddr::new(up_addr, *port))
        })
        // Probe each port
        .for_each(|socket_addr| match probe_port(&socket_addr, timeout) {
            Ok(status) => {
                if show_closed_ports || status == PortStatus::Open {
                    println!("  :{:<5} {}", socket_addr.port(), status);
                }
            }
            Err(error) => println!("{}", error),
        });
}
