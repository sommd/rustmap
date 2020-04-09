use nix;
use nix::libc;
use nix::sys::socket::{
    connect, recv, recvfrom, send, sendto, socket, AddressFamily, InetAddr, MsgFlags, SockAddr,
    SockFlag, SockProtocol, SockType,
};
use nix::unistd::close;
use std::io;
use std::net;
use std::os::unix::io::RawFd;

fn nix_error_to_std(error: nix::Error) -> io::Error {
    match error {
        nix::Error::Sys(errno) => io::Error::from_raw_os_error(errno as i32),
        nix::Error::InvalidPath => io::Error::new(io::ErrorKind::InvalidInput, error),
        nix::Error::InvalidUtf8 => io::Error::new(io::ErrorKind::InvalidData, error),
        nix::Error::UnsupportedOperation => io::Error::new(io::ErrorKind::Other, error),
    }
}

fn ipaddr_to_nix_sock_addr(addr: &net::IpAddr) -> SockAddr {
    SockAddr::new_inet(InetAddr::from_std(&net::SocketAddr::new(addr.clone(), 0)))
}

pub struct RawSocket {
    fd: RawFd,
}

impl RawSocket {
    pub fn new_ipv4() -> io::Result<RawSocket> {
        Self::new(AddressFamily::Inet)
    }

    pub fn new_ipv6() -> io::Result<RawSocket> {
        Self::new(AddressFamily::Inet6)
    }

    fn new(domain: AddressFamily) -> io::Result<RawSocket> {
        Ok(RawSocket {
            fd: socket(domain, SockType::Raw, SockFlag::empty(), unsafe {
                *(&libc::IPPROTO_ICMP as *const i32 as *const SockProtocol)
            })
            .map_err(nix_error_to_std)?,
        })
    }

    pub fn connect(&self, addr: &net::IpAddr) -> io::Result<()> {
        connect(self.fd, &ipaddr_to_nix_sock_addr(&addr)).map_err(nix_error_to_std)
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        recv(self.fd, buf, MsgFlags::empty()).map_err(nix_error_to_std)
    }

    pub fn recvfrom(&self, buf: &mut [u8]) -> io::Result<(usize, net::IpAddr)> {
        recvfrom(self.fd, buf)
            .map(|(len, addr)| {
                if let SockAddr::Inet(addr) = addr.unwrap() {
                    (len, addr.to_std().ip())
                } else {
                    panic!("SockAddr was not SockAddr::Inet")
                }
            })
            .map_err(nix_error_to_std)
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        send(self.fd, &buf, MsgFlags::empty()).map_err(nix_error_to_std)
    }

    pub fn sendto(&self, buf: &[u8], addr: &net::IpAddr) -> io::Result<usize> {
        sendto(
            self.fd,
            &buf,
            &ipaddr_to_nix_sock_addr(&addr),
            MsgFlags::empty(),
        )
        .map_err(nix_error_to_std)
    }
}

impl Drop for RawSocket {
    fn drop(&mut self) {
        close(self.fd).unwrap()
    }
}
