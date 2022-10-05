use serde::{Deserialize, Serialize};
use std::io::{Error, ErrorKind};

use crate::pus_sec_hdr::pus_sec_hdr::*;
use crate::time::{time_length, Time, TimeEncoding};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalSecHdrTC {
    pub pus_type: PUSType,
    pub pus_sub_type: PUSSubType,
    pub pus_src_id: PUSSrcID,
    pub ack_accept: bool,
    pub ack_start: bool,
    pub ack_progress: bool,
    pub ack_complete: bool,
}

impl GalSecHdrTC {
    const HDR_SIZE: usize = 6;

    pub fn new() -> GalSecHdrTC {
        GalSecHdrTC {
            pus_type: PUSType(0),
            pus_sub_type: PUSSubType(0),
            pus_src_id: PUSSrcID(0),
            ack_accept: false,
            ack_start: false,
            ack_progress: false,
            ack_complete: false,
        }
    }
}

impl PUSSecHeader for GalSecHdrTC {
    // return the PUS Type of the secondary header
    fn pus_type(&self) -> PUSType {
        self.pus_type
    }

    // return the PUS SubType of the secondary header
    fn pus_sub_type(&self) -> PUSSubType {
        self.pus_sub_type
    }

    // return the Source- (or Destination)-ID of the secondary header
    fn pus_src_id(&self) -> PUSSrcID {
        self.pus_src_id
    }

    // return the length of the secondary header
    fn len(&self) -> usize {
        return GalSecHdrTC::HDR_SIZE;
    }

    // Parse a secondary header from a byte slice
    fn from_bytes(&mut self, arr: &[u8]) -> Result<(), std::io::Error> {
        let vers = arr[0] & 0xF0;
        if vers != 0b0010_0000 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "GAL PUS-C Secondary Header: wrong version number: {}",
                    (vers >> 4)
                ),
            ));
        }

        let b1 = arr[0];
        self.ack_accept = (b1 & 0b0000_1000) != 0;
        self.ack_start = (b1 & 0b0000_0100) != 0;
        self.ack_progress = (b1 & 0b0000_0010) != 0;
        self.ack_complete = (b1 & 0b0000_0001) != 0;

        self.pus_type = PUSType(arr[1]);
        self.pus_sub_type = PUSSubType(arr[2]);

        let src_id = ((arr[3] as u16) << 8) | (arr[4] as u16);
        self.pus_src_id = PUSSrcID(src_id);

        Ok(())
    }

    // Encode a secondary header to a byte slice
    fn to_bytes(&self, arr: &mut [u8]) -> Result<(), std::io::Error> {
        let mut b1: u8 = if self.ack_accept { 0b0000_1000 } else { 0 };
        b1 = b1 | if self.ack_start { 0b0000_0100 } else { 0 };
        b1 = b1 | if self.ack_progress { 0b0000_0010 } else { 0 };
        b1 = b1 | if self.ack_complete { 0b0000_0001 } else { 0 };

        arr[0] = b1 | 0b0010_0000;

        arr[1] = self.pus_type.0;
        arr[2] = self.pus_sub_type.0;

        arr[3] = (self.pus_src_id.0 >> 8) as u8;
        arr[4] = (self.pus_src_id.0 & 0xFF) as u8;

        // the last byte is reserverd for Galileo
        arr[5] = 0;

        Ok(())
    }

    fn set_pus_type(&mut self, typ: PUSType) {
        self.pus_type = typ;
    }

    fn set_pus_sub_type(&mut self, typ: PUSSubType) {
        self.pus_sub_type = typ;
    }

    fn set_src_id(&mut self, src_id: PUSSrcID) {
        self.pus_src_id = src_id;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalSecHdrTM {
    pub time_reference: u8,
    pub pus_type: PUSType,
    pub pus_sub_type: PUSSubType,
    pub pus_dest_id: PUSSrcID,
    pub msg_type_cntr: u16,
    pub time: Time,
}

impl GalSecHdrTM {
    const HDR_SIZE: usize = 14;
}

impl PUSSecHeader for GalSecHdrTM {
    // return the PUS Type of the secondary header
    fn pus_type(&self) -> PUSType {
        self.pus_type
    }

    // return the PUS SubType of the secondary header
    fn pus_sub_type(&self) -> PUSSubType {
        self.pus_sub_type
    }

    // return the Source- (or Destination)-ID of the secondary header
    fn pus_src_id(&self) -> PUSSrcID {
        self.pus_dest_id
    }

    // return the length of the secondary header
    fn len(&self) -> usize {
        return GalSecHdrTM::HDR_SIZE;
    }

    // Parse a secondary header from a byte slice
    fn from_bytes(&mut self, arr: &[u8]) -> Result<(), std::io::Error> {
        let vers = arr[0] & 0xF0;
        if vers != 0b0010_0000 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "GAL PUS-C Secondary Header: wrong version number: {}",
                    (vers >> 4)
                ),
            ));
        }

        let b1 = arr[0];
        self.time_reference = b1 & 0x0F;

        self.pus_type = PUSType(arr[1]);
        self.pus_sub_type = PUSSubType(arr[2]);

        self.msg_type_cntr = ((arr[3] as u16) << 8) | (arr[4] as u16);
        let dest_id = ((arr[5] as u16) << 8) | (arr[6] as u16);
        self.pus_dest_id = PUSSrcID(dest_id);

        self.time = Time::decode_from_enc(
            TimeEncoding::CUC42,
            &arr[7..(7 + time_length(TimeEncoding::CUC42))],
        )?;

        Ok(())
    }

    // Encode a secondary header to a byte slice
    fn to_bytes(&self, arr: &mut [u8]) -> Result<(), std::io::Error> {
        arr[0] = (self.time_reference & 0x0F) | 0b0010_0000;

        arr[1] = self.pus_type.0;
        arr[2] = self.pus_sub_type.0;

        arr[3] = (self.msg_type_cntr >> 8) as u8;
        arr[4] = (self.msg_type_cntr & 0xFF) as u8;

        arr[6] = (self.pus_dest_id.0 >> 8) as u8;
        arr[6] = (self.pus_dest_id.0 & 0xFF) as u8;

        const END_INDEX: usize = 7 + time_length(TimeEncoding::CUC42);
        self.time.encode(&mut arr[7..END_INDEX])?;

        // the last byte is reserverd for Galileo
        arr[END_INDEX] = 0;
        Ok(())
    }

    fn set_pus_type(&mut self, typ: PUSType) {
        self.pus_type = typ;
    }

    fn set_pus_sub_type(&mut self, typ: PUSSubType) {
        self.pus_sub_type = typ;
    }

    fn set_src_id(&mut self, src_id: PUSSrcID) {
        self.pus_dest_id = src_id;
    }
}
