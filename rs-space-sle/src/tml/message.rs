use byteorder::ReadBytesExt;
#[allow(unused)]
use byteorder::{BigEndian, WriteBytesExt};
use std::io::{Cursor, IoSlice, Read, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt, Error, ErrorKind};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TMLMessageType {
    SlePduMessage = 1,
    ContextMessage = 2,
    HeartBeat = 3,
}

impl TryFrom<u8> for TMLMessageType {
    type Error = ();

    fn try_from(tp: u8) -> Result<Self, Self::Error> {
        match tp {
            1 => Ok(TMLMessageType::SlePduMessage),
            2 => Ok(TMLMessageType::ContextMessage),
            3 => Ok(TMLMessageType::HeartBeat),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct TMLMessage {
    pub msg_type: TMLMessageType,
    length: u32,
    pub data: Vec<u8>,
}

impl TMLMessage {
    pub fn new(t: TMLMessageType) -> TMLMessage {
        TMLMessage {
            msg_type: t,
            length: 0,
            data: Vec::new(),
        }
    }

    pub fn new_with_data(data: Vec<u8>) -> TMLMessage {
        TMLMessage {
            msg_type: TMLMessageType::SlePduMessage,
            length: 0,
            data: data,
        }
    }

    pub fn new_with_len(t: TMLMessageType, len: usize) -> TMLMessage {
        let mut v = Vec::with_capacity(len);
        v.resize(len, 0);
        TMLMessage {
            msg_type: t,
            length: len as u32,
            data: v,
        }
    }

    pub fn heartbeat_message() -> TMLMessage {
        TMLMessage::new(TMLMessageType::HeartBeat)
    }

    pub fn is_heartbeat(&self) -> bool {
        self.msg_type == TMLMessageType::HeartBeat
    }

    pub fn context_message(interval: u16, dead_factor: u16) -> TMLMessage {
        let mut data: Vec<u8> = Vec::new();

        data.resize(12, 0);

        let mut wrtr = Cursor::new(&mut data[0..]);

        Write::write_all(&mut wrtr, b"ISP1").unwrap();
        Write::write_all(&mut wrtr, &[0, 0, 0, 1]).unwrap();
        WriteBytesExt::write_u16::<BigEndian>(&mut wrtr, interval).unwrap();
        WriteBytesExt::write_u16::<BigEndian>(&mut wrtr, dead_factor).unwrap();

        TMLMessage {
            msg_type: TMLMessageType::ContextMessage,
            length: 12,
            data: data,
        }
    }

    pub fn check_context(&self) -> Result<(u16, u16), String> {
        match self.msg_type {
            TMLMessageType::ContextMessage => {
                let mut rdr = Cursor::new(&self.data);
                let mut buf = [0; 8];
                if let Err(err) = Read::read_exact(&mut rdr, &mut buf[..]) {
                    return Err(format!("Error parsing TML Context Message: {}", err));
                }

                // Check for the ISP1 Value
                let comp = b"ISP1\0\0\01";
                if comp != &buf {
                    return Err(
                        "Error parsing TML Context Message: ISP1 value not found or wrong version"
                            .to_string(),
                    );
                }

                let interval = ReadBytesExt::read_u16::<BigEndian>(&mut rdr).map_err(|_| {
                    format!("Error parsing TML Context Message: could not read interval value")
                })?;
                let dead_factor = ReadBytesExt::read_u16::<BigEndian>(&mut rdr).map_err(|_| {
                    format!("Error parsing TML Context Message: could not read dead factor value")
                })?;

                Ok((interval, dead_factor))
            }
            x => Err(format!("Expected TML Context Message, got {:?}", x)),
        }
    }

    pub async fn read_from_async<T: AsyncReadExt + Unpin>(
        &mut self,
        reader: &mut T,
    ) -> Result<(), Error> {
        let t = reader.read_u8().await?;

        if let Ok(t1) = TMLMessageType::try_from(t) {
            // assign the message type
            self.msg_type = t1;

            // skip the next 3 bytes
            let mut dummy = [0, 0, 0];
            reader.read(&mut dummy[..]).await?;

            // read in the length
            self.length = reader.read_u32().await?;

            // now read in the data
            self.data.resize(self.length as usize, 0);
            reader.read_exact(&mut self.data[0..]).await?;
            Ok(())
        } else {
            let msg = format!("TML Message: invalid message type {}", t);
            return Err(Error::new(ErrorKind::InvalidInput, msg));
        }
    }

    pub async fn async_read<T: AsyncReadExt + Unpin>(reader: &mut T) -> Result<TMLMessage, Error> {
        let mut buf = [0, 0, 0, 0];
        let _ = reader.read_exact(&mut buf[..]).await?;

        if let Ok(t1) = TMLMessageType::try_from(buf[0]) {
            // read in the length
            let length = reader.read_u32().await?;

            // now read in the data
            let mut data = vec![0; length as usize];
            reader.read_exact(&mut data[..]).await?;
            Ok(TMLMessage {
                msg_type: t1,
                length: length,
                data: data,
            })
        } else {
            let msg = format!("TML Message: invalid message type {}", buf[0]);
            return Err(Error::new(ErrorKind::InvalidInput, msg));
        }
    }

    pub async fn write_to_async<T: AsyncWriteExt + Unpin>(
        &self,
        writer: &mut T,
    ) -> Result<(), Error> {
        let mut buf = [0; 8];

        buf[0] = self.msg_type as u8;

        let mut cursor = Cursor::new(&mut buf[4..]);
        let len: u32 = self.data.len() as u32;
        WriteBytesExt::write_u32::<BigEndian>(&mut cursor, len).unwrap();

        writer
            .write_vectored(&[IoSlice::new(&buf), IoSlice::new(&self.data)])
            .await?;
        writer.flush().await?;
        Ok(())
    }
}
