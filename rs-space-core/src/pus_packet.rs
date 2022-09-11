use serde::{Deserialize, Serialize};

use crate::ccsds_packet::CcsdsPacket;
use crate::pus_sec_hdr::pus_sec_hdr::*;
use crate::pus_types::{CcsdsType, PktID, APID, SSC, HexBytes};

use std::io::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PUSPacket {
    pub pkt_id: PktID,
    pub ssc: SSC,
    pub sec_hdr: PUSSecondaryHeader,
    pub data: HexBytes,
}

impl PUSPacket {
    pub fn new(typ: CcsdsType, pus_sec_hdr: PUSSecondaryHeader) -> PUSPacket {
        PUSPacket {
            pkt_id: PktID::new(0, typ, false, APID::new(0)),
            ssc: SSC::new_unseg(0),
            sec_hdr: pus_sec_hdr,
            data: HexBytes::new(),
        }
    }

    pub fn from_ccsds_packet(
        pkt: CcsdsPacket,
        mut pus_sec_hdr: PUSSecondaryHeader,
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
                sec_hdr: PUSSecondaryHeader::Empty,
                data: pkt.data,
            })
        }
    }
}
