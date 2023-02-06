use crate::pus_types::{HexBytes, PktID, SSC};

use std::io::{Read, Write};

use tokio::io::{AsyncReadExt, AsyncWriteExt, Error};

use serde::{Deserialize, Serialize};

pub enum PacketType {
    TM,
    TC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastCcsdsPacket {
    pub hdr: [u8; 6],
    pub data: HexBytes,
}

impl FastCcsdsPacket {
    pub const HDR_LEN: usize = 6;
    const DEFAULT_CAPACITY: usize = 4096;

    pub fn new() -> FastCcsdsPacket {
        FastCcsdsPacket {
            hdr: [0; 6],
            data: HexBytes(Vec::with_capacity(Self::DEFAULT_CAPACITY)),
        }
    }

    pub fn new_header_only() -> FastCcsdsPacket {
        FastCcsdsPacket {
            hdr: [0; 6],
            data: HexBytes::new(),
        }
    }

    pub fn length(&self) -> u16 {
        let b1 = self.hdr[4];
        let b2 = self.hdr[5];

        // return the real length of the data. The field contains length - 1
        (((b1 as u16) << 8) | (b2 as u16)) + 1
    }

    pub fn total_length(&self) -> usize {
        self.length() as usize + Self::HDR_LEN
    }

    pub fn crc_value(&self) -> u16 {
        let len = self.data.len();
        let b1 = self.data.0[len - 2];
        let b2 = self.data.0[len - 1];

        ((b1 as u16) << 8) | (b2 as u16)
    }

    pub fn calc_crc(&self) -> u16 {
        crate::crc::calc_crc2(&self.hdr, &self.data.0[0..(self.data.len() - 2)])
    }

    pub fn calc_and_append_crc(&mut self) {
        let crc = crate::crc::calc_crc2(&self.hdr, &self.data.0);
        self.data.0.push((crc >> 8) as u8);
        self.data.0.push((crc & 0xFF) as u8);
    }

    pub fn check_crc(&self) -> bool {
        let calc = self.calc_crc();
        calc == self.crc_value()
    }

    pub fn set_crc(&mut self) {
        let len = self.data.len();
        let crc = crate::crc::calc_crc2(&self.hdr, &self.data.0[0..(len - 2)]);
        self.data.0[len - 2] = (crc >> 8) as u8;
        self.data.0[len - 1] = (crc & 0xFF) as u8;
    }

    pub async fn read_from_async<T: AsyncReadExt + Unpin>(
        &mut self,
        reader: &mut T,
    ) -> Result<usize, Error> {
        // read in the header
        reader.read_exact(&mut self.hdr).await?;

        // determine the data length out of the header
        let len = self.length();

        // resize the data to contain the new data
        self.data.0.resize(len as usize, 0);

        // read in the data, returns the size of the data part or Error
        reader.read_exact(&mut self.data.0).await
    }

    pub fn read_from(&mut self, reader: &mut dyn Read) -> Result<(), Error> {
        // read in the header
        reader.read_exact(&mut self.hdr)?;

        // determine the data length out of the header
        let len = self.length();

        // resize the data to contain the new data
        self.data.0.resize(len as usize, 0);

        // read in the data, returns the size of the data part or Error
        reader.read_exact(&mut self.data.0)
    }

    pub async fn write_to_async<T: AsyncWriteExt + Unpin>(
        &self,
        writer: &mut T,
    ) -> Result<(), Error> {
        writer.write_all(&self.hdr).await?;
        writer.write_all(&self.data.0).await?;
        writer.flush().await?;
        Ok(())
    }

    pub fn write_to(&self, writer: &mut dyn Write) -> Result<(), Error> {
        writer.write_all(&self.hdr)?;
        writer.write_all(&self.data.0)?;
        writer.flush()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcsdsPacket {
    pub pkt_id: PktID,
    pub ssc: SSC,
    pub data: HexBytes,
}

impl CcsdsPacket {
    pub fn from_fast_ccsds_pkt(mut pkt: FastCcsdsPacket) -> CcsdsPacket {
        let pkt_id = PktID::new_from_bytes(&pkt.hdr[0..2]);
        let ssc = SSC::new_from_bytes(&pkt.hdr[2..4]);

        // store data without the CRC, so remove the last 2 bytes
        pkt.data.0.truncate(pkt.data.0.len() - 2);

        CcsdsPacket {
            pkt_id,
            ssc,
            data: pkt.data,
        }
    }

    pub fn to_fast_ccsds_pkt(self) -> FastCcsdsPacket {
        let mut pkt = FastCcsdsPacket::new_header_only();

        // set the header fields: pkt ID
        self.pkt_id.to_bytes(&mut pkt.hdr[0..2]);
        // set the SSC
        self.ssc.to_bytes(&mut pkt.hdr[2..4]);

        // remember, in the header, the data length - 1 is stored;
        let enc_len = (self.data.0.len() - 1) as u16;

        pkt.hdr[4] = (enc_len >> 8) as u8;
        pkt.hdr[5] = (enc_len & 0xFF) as u8;
        pkt.data = self.data;

        pkt.calc_and_append_crc();

        pkt
    }
}
