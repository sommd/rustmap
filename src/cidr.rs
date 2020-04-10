use std::convert::TryFrom;
use std::fmt;
use std::iter::IntoIterator;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct IpAddrRange {
    addr: IpAddr,
    mask: u8,
}

impl IpAddrRange {
    pub fn new(addr: IpAddr, mask: u8) -> IpAddrRange {
        Self::try_new(addr, mask).unwrap()
    }

    pub fn try_new(addr: IpAddr, mask: u8) -> Result<Self, String> {
        if mask == 0 {
            Err(String::from("mask cannot be 0"))
        } else {
            match addr {
                IpAddr::V4(_) if mask > 32 => Err(String::from(
                    "mask cannot be more than 32 for an IPv4 address",
                )),
                IpAddr::V6(_) if mask > 128 => Err(String::from(
                    "mask cannot be more than 128 for an IPv6 address",
                )),
                _ => Ok(IpAddrRange { addr, mask }),
            }
        }
    }

    pub fn addr(&self) -> &IpAddr {
        &self.addr
    }

    pub fn set_addr(&mut self, addr: IpAddr) {
        self.addr = addr;
    }

    pub fn mask(&self) -> &u8 {
        &self.mask
    }

    pub fn set_mask(&mut self, mask: u8) {
        self.mask = mask;
    }

    pub fn first_addr(&self) -> IpAddr {
        match self.addr {
            IpAddr::V4(addr) => {
                Ipv4Addr::from(u32::from(addr) & (u32::max_value() << (32 - self.mask))).into()
            }
            IpAddr::V6(addr) => {
                Ipv6Addr::from(u128::from(addr) & (u128::max_value() << (128 - self.mask))).into()
            }
        }
    }

    pub fn last_addr(&self) -> IpAddr {
        match self.addr {
            IpAddr::V4(addr) => {
                if self.mask == 32 {
                    self.addr
                } else {
                    Ipv4Addr::from(u32::from(addr) | (u32::max_value() >> self.mask)).into()
                }
            }
            IpAddr::V6(addr) => {
                if self.mask == 128 {
                    self.addr
                } else {
                    Ipv6Addr::from(u128::from(addr) | (u128::max_value() >> self.mask)).into()
                }
            }
        }
    }

    pub fn iter(&self) -> IntoIter {
        IntoIter {
            cur: self.first_addr(),
            last: self.last_addr(),
            done: false,
        }
    }
}

impl fmt::Display for IpAddrRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.addr, self.mask)
    }
}

impl From<IpAddr> for IpAddrRange {
    fn from(addr: IpAddr) -> Self {
        IpAddrRange {
            addr,
            mask: match addr {
                IpAddr::V4(_) => 32,
                IpAddr::V6(_) => 128,
            },
        }
    }
}

impl TryFrom<(IpAddr, u8)> for IpAddrRange {
    type Error = String;

    fn try_from((addr, mask): (IpAddr, u8)) -> Result<IpAddrRange, String> {
        Self::try_new(addr, mask)
    }
}

impl FromStr for IpAddrRange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let slash = s.find('/');

        let addr = IpAddr::from_str(&s[0..slash.unwrap_or_else(|| s.len())]).map_err(|e| e.to_string())?;

        match slash {
            Some(slash) => Self::try_new(
                addr,
                u8::from_str(&s[slash + 1..]).map_err(|e| e.to_string())?,
            ),
            None => Ok(Self::from(addr)),
        }
    }
}

impl IntoIterator for IpAddrRange {
    type Item = IpAddr;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        self.iter()
    }
}

pub struct IntoIter {
    cur: IpAddr,
    last: IpAddr,
    done: bool,
}

impl Iterator for IntoIter {
    type Item = IpAddr;

    fn next(&mut self) -> Option<IpAddr> {
        if self.done {
            None
        } else if self.cur == self.last {
            self.done = true;
            Some(self.cur)
        } else {
            let next = self.cur;

            self.cur = match self.cur {
                IpAddr::V4(addr) => Ipv4Addr::from(u32::from(addr) + 1).into(),
                IpAddr::V6(addr) => Ipv6Addr::from(u128::from(addr) + 1).into(),
            };

            Some(next)
        }
    }

    fn last(self) -> Option<IpAddr> {
        if self.done {
            None
        } else {
            Some(self.last)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        assert_eq!(
            "192.168.1.1".parse::<IpAddrRange>(),
            Ok(IpAddrRange::new(IpAddr::from([192, 168, 1, 1]), 32))
        );

        assert_eq!(
            "192.168.1.1/12".parse::<IpAddrRange>(),
            Ok(IpAddrRange::new(IpAddr::from([192, 168, 1, 1]), 12))
        );

        assert_eq!(
            "::1/128".parse::<IpAddrRange>(),
            Ok(IpAddrRange::new(Ipv6Addr::LOCALHOST.into(), 128))
        );

        // TODO: Better error messages
        assert_eq!(
            "".parse::<IpAddrRange>(),
            Err(String::from("invalid IP address syntax"))
        );
        assert_eq!(
            "192.168.".parse::<IpAddrRange>(),
            Err(String::from("invalid IP address syntax"))
        );
        assert_eq!(
            "192.168.1.1/".parse::<IpAddrRange>(),
            Err(String::from("cannot parse integer from empty string"))
        );
        assert_eq!(
            "192.168.1.1/0".parse::<IpAddrRange>(),
            Err(String::from("mask cannot be 0"))
        );
        assert_eq!(
            "192.168.1.1/33".parse::<IpAddrRange>(),
            Err(String::from(
                "mask cannot be more than 32 for an IPv4 address"
            ))
        );
        assert_eq!(
            "::1/129".parse::<IpAddrRange>(),
            Err(String::from(
                "mask cannot be more than 128 for an IPv6 address"
            ))
        );
    }

    #[test]
    fn test_first_addr() {
        assert_eq!(
            IpAddrRange::new(Ipv4Addr::from(0xFFFFFFFFu32).into(), 24).first_addr(),
            IpAddr::from(Ipv4Addr::from(0xFFFFFF00u32))
        );

        assert_eq!(
            IpAddrRange::new(
                Ipv6Addr::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128).into(),
                112
            )
            .first_addr(),
            IpAddr::from(Ipv6Addr::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000u128))
        );
    }

    #[test]
    fn test_last_addr() {
        assert_eq!(
            IpAddrRange::new(Ipv4Addr::from(0x00000000u32).into(), 24).last_addr(),
            IpAddr::from(Ipv4Addr::from(0x000000FFu32))
        );

        assert_eq!(
            IpAddrRange::new(
                Ipv6Addr::from(0x00000000000000000000000000000000u128).into(),
                112
            )
            .last_addr(),
            IpAddr::from(Ipv6Addr::from(0x0000000000000000000000000000FFFFu128))
        );
    }

    #[test]
    fn test_iter() {
        let range = IpAddrRange::new(IpAddr::from([192, 168, 1, 128]), 24);

        let first = range.first_addr();
        let last = range.last_addr();

        assert_eq!(range.iter().next(), Some(first));
        assert_eq!(range.iter().last(), Some(last));

        for ip in range {
            assert!(ip >= first && ip <= last);
        }
    }
}
