use std::io::{Error, ErrorKind};

use crate::pus_sec_hdr::*;

pub struct PUSCSecHdrTC {
    pub pus_type: PUSType,
    pub pus_sub_type: PUSSubType,
    pub pus_dest_id: PUSSrcID,
    pub ack_accept: bool,
    pub ack_start: bool,
    pub ack_progress: bool,
    pub ack_complete: bool,
}

impl PUSSecHeader for PUSCSecHdrTC {
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
        return 5;
    }

    // Parse a secondary header from a byte slice
    fn from_bytes(&mut self, arr: &[u8]) -> Result<(), std::io::Error> {
        let vers = arr[0] & 0xF0;
        if vers != 0b1010_0000 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Could not parse EDEN type",
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
        self.pus_dest_id = PUSSrcID(src_id);

        Ok(())
    }

    // Encode a secondary header to a byte slice
    fn to_bytes(&self, arr: &mut [u8]) {
        let mut b1: u8 = if self.ack_accept { 0b0000_1000 } else { 0 };
        b1 = b1 | if self.ack_start { 0b0000_0100 } else { 0 };
        b1 = b1 | if self.ack_progress { 0b0000_0010 } else { 0 };
        b1 = b1 | if self.ack_complete { 0b0000_0001 } else { 0 };

        arr[0] = b1 | 0b1010_0000;

        arr[1] = self.pus_type.0;
        arr[2] = self.pus_sub_type.0;

        arr[3] = (self.pus_dest_id.0 >> 8) as u8;
        arr[4] = (self.pus_dest_id.0 & 0xFF) as u8;
    }
}

pub struct PUSCSecHdrTM {}
