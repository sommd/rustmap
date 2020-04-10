#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IcmpEchoType {
    Reply = 0,
    ReplyV6 = 129,
    Request = 8,
    RequestV6 = 128,
}

impl Default for IcmpEchoType {
    fn default() -> IcmpEchoType {
        IcmpEchoType::Reply
    }
}

#[derive(Debug, Default)]
pub struct IcmpEchoPacket {
    pub type_: IcmpEchoType,
    pub identifier: u16,
    pub sequence_number: u16,
    pub data: Vec<u8>,
}

impl IcmpEchoPacket {
    const HEADER_SIZE: usize = 8;

    pub fn from_ipv4(buf: &[u8]) -> Option<IcmpEchoPacket> {
        let header_len = ((buf[0] & 0x0F) * 4) as usize;
        Self::from(&buf[header_len..])
    }

    pub fn from(buf: &[u8]) -> Option<IcmpEchoPacket> {
        let type_ = match buf[0] {
            0 => IcmpEchoType::Reply,
            129 => IcmpEchoType::ReplyV6,
            8 => IcmpEchoType::Request,
            128 => IcmpEchoType::RequestV6,
            _ => return None,
        };

        if buf[1] != 0 {
            return None;
        }

        Some(IcmpEchoPacket {
            type_,
            identifier: u16::from_be_bytes([buf[4], buf[5]]),
            sequence_number: u16::from_be_bytes([buf[6], buf[7]]),
            data: Vec::from(&buf[Self::HEADER_SIZE..]),
        })
    }

    pub fn new(
        type_: IcmpEchoType,
        identifier: u16,
        sequence_number: u16,
        data: &[u8],
    ) -> IcmpEchoPacket {
        IcmpEchoPacket {
            type_,
            identifier,
            sequence_number,
            data: Vec::from(data),
        }
    }

    pub fn write_to(&self, buf: &mut [u8]) {
        self.write_header_to(buf);
        buf[Self::HEADER_SIZE..].copy_from_slice(&self.data);
    }

    pub fn write_header_to(&self, buf: &mut [u8]) {
        buf[0] = self.type_ as u8;
        buf[1] = 0; // code is always 0
        buf[2..=3].copy_from_slice(&0u16.to_be_bytes()); // checksum placeholder
        buf[4..=5].copy_from_slice(&self.identifier.to_be_bytes());
        buf[6..=7].copy_from_slice(&self.sequence_number.to_be_bytes());

        // Calculating the checksum for ICMPv6 is harder, but the OS does it for us!
        if self.type_ == IcmpEchoType::Reply || self.type_ == IcmpEchoType::Request {
            let checksum = internet_checksum(&buf[0..=7]);
            let checksum = internet_checksum_incremental(checksum, &self.data);
            buf[2..=3].copy_from_slice(&checksum.to_be_bytes());
        }
    }

    pub fn len(&self) -> usize {
        Self::HEADER_SIZE + self.data.len()
    }
}

impl From<&IcmpEchoPacket> for Vec<u8> {
    fn from(packet: &IcmpEchoPacket) -> Self {
        let mut buf = vec![0; packet.len()];
        packet.write_to(&mut buf);
        buf
    }
}

fn internet_checksum(data: &[u8]) -> u16 {
    internet_checksum_incremental(0xFFFF, data)
}

fn internet_checksum_incremental(checksum: u16, data: &[u8]) -> u16 {
    !data
        .chunks(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], *chunk.get(1).unwrap_or(&0)]))
        .fold(!checksum, ones_complement_sum)
}

fn ones_complement_sum(a: u16, b: u16) -> u16 {
    let (s, carry) = a.overflowing_add(b);
    s + carry as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internet_checksum() {
        assert_eq!(
            internet_checksum(&[
                0x45, 0x00, 0x00, 0x73, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11, 0x00, 0x00, 0xc0, 0xa8,
                0x00, 0x01, 0xc0, 0xa8, 0x00, 0xc7, 0x01
            ]),
            0xb761
        );

        assert_eq!(
            internet_checksum(&[
                0x45, 0x00, 0x00, 0x73, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11, 0xb7, 0x61, 0xc0, 0xa8,
                0x00, 0x01, 0xc0, 0xa8, 0x00, 0xc7, 0x01
            ]),
            0x0000
        );
    }
}
