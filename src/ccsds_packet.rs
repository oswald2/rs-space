use crate::pus_types::{PktID, SSC};

pub enum PacketType {
    TM,
    TC,
}

pub struct FastCcsdsPacket {
    pub hdr: [u8; 6],
    pub data: Vec<u8>,
}

impl FastCcsdsPacket {
    pub const HDR_LEN: usize = 6;

    pub fn length(&self) -> u16 {
        let b1 = self.hdr[4];
        let b2 = self.hdr[5];

        (((b1 as u16) << 8) | (b2 as u16)) + 1
    }

    pub fn crc_value(&self) -> u16 {
        let len = self.data.len();
        let b1 = self.data[len - 2];
        let b2 = self.data[len - 1];

        ((b1 as u16) << 8) | (b2 as u16)
    }

    pub fn calc_crc(&self) -> u16 {
        crate::crc::calc_crc2(&self.hdr, &self.data[0..(self.data.len() - 2)])
    }

    pub fn check_crc(&self) -> bool {
        let calc = self.calc_crc();
        calc == self.crc_value()
    }

    pub fn to_ccsds_packet(mut self) -> CcsdsPacket {
        let pkt_id = PktID::new_from_bytes(&self.hdr[0..2]);
        let ssc = SSC::new_from_bytes(&self.hdr[2..4]);

        self.data.truncate(self.data.len() - 2);

        CcsdsPacket {
            pkt_id,
            ssc,
            length: self.length(),
            data: self.data,
        }
    }
}

pub struct CcsdsPacket {
    pub pkt_id: PktID,
    pub ssc: SSC,
    pub length: u16,
    pub data: Vec<u8>,
}
