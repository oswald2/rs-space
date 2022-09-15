use serde::{Deserialize, Serialize};

use crate::ccsds_packet::CcsdsPacket;
use crate::pus_sec_hdr::pus_sec_hdr::*;
use crate::pus_sec_hdr::empty::PUSEmptyHeader;
use crate::pus_types::{CcsdsType, PktID, APID, SSC, HexBytes};

use std::io::Error;

//#[derive(Clone, Serialize, Deserialize)]
//#[derive(Debug)]
pub struct PUSPacket {
    pub pkt_id: PktID,
    pub ssc: SSC,
    pub sec_hdr: Box<dyn PUSSecHeader>,
    pub data: HexBytes,
}

impl PUSPacket {
    pub fn new(typ: CcsdsType, pus_sec_hdr: Box<dyn PUSSecHeader>) -> PUSPacket {
        PUSPacket {
            pkt_id: PktID::new(0, typ, false, APID::new(0)),
            ssc: SSC::new_unseg(0),
            sec_hdr: pus_sec_hdr,
            data: HexBytes::new(),
        }
    }

    pub fn from_ccsds_packet(
        pkt: CcsdsPacket,
        mut pus_sec_hdr: Box<dyn PUSSecHeader>,
    ) -> Result<PUSPacket, Error> {
        if pkt.pkt_id.dfh {
            let len = pus_sec_hdr.len();
            pus_sec_hdr.from_bytes(&pkt.data.0[0..len])?;

            Ok(PUSPacket {
                pkt_id: pkt.pkt_id,
                ssc: pkt.ssc,
                sec_hdr: pus_sec_hdr,
                data: HexBytes(pkt.data.0[len..].to_vec()),
            })
        } else {
            Ok(PUSPacket {
                pkt_id: pkt.pkt_id,
                ssc: pkt.ssc,
                sec_hdr: Box::new(PUSEmptyHeader{}),
                data: pkt.data,
            })
        }
    }
}
