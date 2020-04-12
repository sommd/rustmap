# rustmap

Rustmap is a very simple "Nmap-like" program that can scan for hosts and open TCP ports. It is mostly written for educational purposes (I wanted to learn Rust, and learn a bit more about how Nmap works) so it's quite slow and doesn't have many features.

## Installation

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. `cargo install rustmap`

## Examples

Check if a single host is up:

```sh
sudo rustmap 127.0.0.1
```

Scan all TCP ports for a single host:

```sh
sudo rustmap 127.0.0.1 -p
```

Scan all TCP ports for multiple hosts:

```sh
sudo rustmap 127.0.0.1 ::1 -p
```

Scan for specific ports in an address range:

```sh
sudo rustmap 127.0.0.0/8 -p 22,80,443
```

## Usage

```sh
rustmap [OPTIONS] <addr-ranges>...
```

### Flags

#### `-h, --help`

Prints help information

#### `-V, --version`

Prints version information

### Options

#### `-p, --ports <ports>...`

Probe ports for each host and optionally specify which ports.

Ports can be specified as a comma-separated list, or left unspecified to scan all ports.

#### `-t, --timeout <timeout>`

Timeout for pinging each host and probing each port.

Parsing is provided by the [parse_duration](https://crates.io/crates/parse_duration) crate and supports almost any notation. E.g. `1s`, `10 seconds`, `1 hour, 15 minutes, 12 seconds`, `10m32s112ms`. [default: `1s`]

### Args

#### `<addr-ranges>...`

IP addresses to scan.

Supports IPv4 notation (e.g. `127.0.0.1`), IPv6 notation (e.g. `::1`), IPv4-mapped IPv6 notation (e.g. `::ffff::1.1.1.1`) and CIDR notation (e.g. `192.168.0.0/16`, `fe80::/10`).

## License
[Apache 2.0](./LICENSE)
