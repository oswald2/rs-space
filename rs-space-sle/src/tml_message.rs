use byteorder::{BigEndian, WriteBytesExt};

use std::io::{Cursor, Write};

use tokio::io::{AsyncReadExt, AsyncWriteExt, Error, ErrorKind};


#[repr(u8)]
#[derive(Debug, Clone, Copy)]
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
            _ => Err(()) 
        }
    }


}


#[derive(Debug)]
pub struct TMLMessage {
    pub msg_type: TMLMessageType,
    pub length: u32,
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

    pub async fn read_from_async<T: AsyncReadExt + Unpin>(&mut self, reader: &mut T) -> Result<(), Error> {
        let t = reader.read_u8().await?;

        if let Ok(t1) = TMLMessageType::try_from(t) {
            // assign the message type 
            self.msg_type = t1;

            // skip the next 3 bytes
            let mut dummy = [0,0,0];
            reader.read(&mut dummy[0..]).await?;

            // read in the length
            self.length = reader.read_u32().await?;
    
            // now read in the data 
            self.data.resize(self.length as usize, 0);
            reader.read_exact(&mut self.data[0..]).await?;
            Ok(())
        }    
        else  {
            let msg = format!("TML Message: invalid message type {}", t);
            return Err(Error::new(ErrorKind::InvalidInput, msg))
        }
    }

    pub async fn write_to_async<T: AsyncWriteExt + Unpin>(&self, writer: &mut T) -> Result<(), Error> {
        writer.write_u8(self.msg_type as u8).await?;
        writer.write(&[0,0,0]).await?;
        let len: u32 = self.data.len() as u32;
        writer.write_u32(len).await?;
        writer.write_all(&self.data).await?;
        writer.flush().await?;
        Ok(())
    }

}