extern crate tokio_io;
extern crate bytes;

use tokio_io::codec::{Decoder, Encoder};
use bytes::BytesMut;

mod pixel;
mod sysexclusive;

pub struct OPCCodec;

pub use pixel::Pixels;
pub use sysexclusive::SystemExclusiveData;

pub enum OpcMessageData {
    SetPixelColours(Pixels),
    Other(u8, Vec<u8>),
    SystemExclusive(SystemExclusiveData),
}

impl std::convert::From<OpcMessageData> for Vec<u8> {
    fn from(t: OpcMessageData) -> Vec<u8> {
        match t {
            OpcMessageData::SetPixelColours(pixels) => pixels.into(),
            OpcMessageData::SystemExclusive(sysdata) => sysdata.into(),
            OpcMessageData::Other(_, data) => data,
        }
    }
}

pub struct OpcMessage {
    pub channel: u8,
    pub message: OpcMessageData,
}

impl OpcMessage {
    fn new(channel: u8, msg: OpcMessageData) -> Self {
        OpcMessage {
            channel: channel,
            message: msg,
        }
    }
}

const OPC_HEADER_LENGTH: usize = 4;

pub struct OpcHeader {
    pub channel: u8,
    pub command: u8,
    pub length: u16,
}

impl OpcHeader {
    pub fn new(buf: &[u8]) -> Self {
        OpcHeader {
            channel: buf[0],
            command: buf[1],
            length: (((buf[2] as u16) << 8) + (buf[3] as u16)),
        }
    }

    pub fn read_header<T: std::io::Read>(r: &mut T) -> std::io::Result<Self> {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;

        Ok(OpcHeader {
            channel: buf[0],
            command: buf[1],
            length: (((buf[2] as u16) << 8) + (buf[3] as u16)),
        })
    }

    pub fn write_header<T: std::io::Write>(&self, w: &mut T) -> std::io::Result<()> {

        let len1: u8 = ((self.length >> 8) & 0xff) as u8;
        let len2: u8 = (self.length & 0xff) as u8;

        let buf = [self.channel, self.command, len1, len2];
        w.write_all(&buf)?;

        Ok(())
    }
}

#[cfg(not(feature="failfast"))]
fn verify_vec_size(mut d: Vec<u8>) -> Vec<u8> {
    if d.len() > (std::u16::MAX as usize) {
        d.truncate(std::u16::MAX as usize);
    }

    d
}

#[cfg(feature="failfast")]
fn verify_vec_size(d: Vec<u8>) -> Vec<u8> {
    if d.len() > (std::u16::MAX as usize) {
        panic!("Vector too big for OPC Message")
    }
    d
}

impl From<OpcMessage> for Vec<u8> {
    fn from(m: OpcMessage) -> Vec<u8> {
        let channel: u8 = m.channel;
        let command = match m.message {
            OpcMessageData::SetPixelColours(_) => 0,
            OpcMessageData::SystemExclusive(_) => 255,
            OpcMessageData::Other(cmd, _) => cmd,
        };

        let data: Vec<u8> = m.message.into();
        let data = verify_vec_size(data);
        let datalen = data.len();
        let len1: u8 = ((datalen >> 8) & 0xff) as u8;
        let len2: u8 = (datalen & 0xff) as u8;

        let mut msg = Vec::new();
        msg.reserve_exact(OPC_HEADER_LENGTH + datalen);

        msg.extend_from_slice(&[channel, command, len1, len2]);
        msg.extend(data);
        msg
    }
}


impl Decoder for OPCCodec {
    type Item = OpcMessage;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> std::io::Result<Option<OpcMessage>> {
        if buf.len() >= OPC_HEADER_LENGTH {
            let header = OpcHeader::new(buf);

            let packet_size = OPC_HEADER_LENGTH + header.length as usize;

            if buf.len() >= packet_size {
                //get rid of header, we all ready have it..
                buf.split_to(OPC_HEADER_LENGTH);
                // read the data
                let data = buf.split_to(header.length as usize).to_vec();
                let data = match header.command {
                    0 => OpcMessageData::SetPixelColours(data.into()),
                    255 => OpcMessageData::SystemExclusive(data.into()),
                    _ => OpcMessageData::Other(header.command, data),
                };
                Ok(Some(OpcMessage::new(header.channel, data)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> std::io::Result<Option<OpcMessage>> {
        self.decode(buf)
    }
}

impl Encoder for OPCCodec {
    type Item = OpcMessage;
    type Error = std::io::Error;


    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {

        let data: Vec<u8> = item.into();
        dst.extend(data);

        Ok(())

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes;

    #[test]
    fn vec_of_pixels_deserialized() {
        let mut pixels_vec: BytesMut = vec![1u8,// channel
        0, // command
        0,3, // size
        1,2,3]
            .into();
        let mut codec = OPCCodec;

        let pixelsmsg: OpcMessage = codec.decode(&mut pixels_vec).unwrap().unwrap();

        assert_eq!(pixelsmsg.channel, 1);

        let pixels = match pixelsmsg.message {
            OpcMessageData::SetPixelColours(pixels) => pixels,
            _ => panic!("wrong message"),
        };
        let pixel = pixels.iter().next().unwrap();
        assert_eq!(pixel.r(), 1);
        assert_eq!(pixel.g(), 2);
        assert_eq!(pixel.b(), 3);
    }


    #[test]
    fn vec_of_pixels_serialized() {
        let mut p = Pixels::new(1);
        {
            let mut pixel = p.iter_mut().next().unwrap();
            pixel.set_r(1);
            pixel.set_g(2);
            pixel.set_b(3);
        }
        let msg = OpcMessage::new(1, OpcMessageData::SetPixelColours(p));
        let expect_pixels_vec = vec![1u8,// channel
        0,   // command
        0,3, // size
        1,2,3];

        let mut serialized = bytes::Bytes::new().try_mut().unwrap();
        let mut codec = OPCCodec;

        codec.encode(msg, &mut serialized).unwrap();

        assert_eq!(expect_pixels_vec, serialized.to_vec());

    }
}
