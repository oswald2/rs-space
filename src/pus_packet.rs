use crate::ccsds_packet::CcsdsPacket;
use crate::pus_sec_hdr::*;
use crate::pus_types::{CcsdsType, PktID, APID, SSC};

pub struct PUSPacket<'a> {
    pub pkt_id: PktID,
    pub ssc: SSC,
    pub sec_hdr: Box<dyn PUSSecHeader + 'a>,
    pub data: Vec<u8>,
}

impl<'a> PUSPacket<'a> {
    pub fn new(typ: CcsdsType, pus_sec_hdr: Box<dyn PUSSecHeader + 'a>) -> PUSPacket {
        PUSPacket {
            pkt_id: PktID::new(0, typ, false, APID::new(0)),
            ssc: SSC::new_unseg(0),
            sec_hdr: pus_sec_hdr,
            data: Vec::new(),
        }
    }

    pub fn from_ccsds_packet(
        pkt: CcsdsPacket,
        pus_sec_hdr: Box<dyn PUSSecHeader + 'a>,
    ) -> PUSPacket {
        let range = 0 .. pus_sec_hdr.len();
        let mut sec_hdr2 = Box::new(*pus_sec_hdr.clone());
        sec_hdr2.from_bytes(&pkt.data[range]);
        pkt.data.drain(range);

        PUSPacket {
            pkt_id: pkt.pkt_id, 
            ssc: pkt.ssc,
            sec_hdr: pus_sec_hdr,
            data: pkt.data
        }
    }
}
