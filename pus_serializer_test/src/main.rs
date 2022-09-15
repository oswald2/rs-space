use rs_space_core::pus_packet::PUSPacket;
use rs_space_core::pus_sec_hdr::gal_pus_c::GalSecHdrTM;
use rs_space_core::pus_sec_hdr::pus_sec_hdr::*;
use rs_space_core::pus_types::{CcsdsType, HexBytes};
use rs_space_core::time::{Time, TimeEncoding};

fn main() {
    let t = Time::now(TimeEncoding::CUC42);

    let sec_hdr = Box::new(GalSecHdrTM {
        time_reference: 0,
        pus_type: PUSType(3),
        pus_sub_type: PUSSubType(25),
        pus_dest_id: PUSSrcID(0),
        msg_type_cntr: 0,
        time: t,
    });
    let mut pkt = PUSPacket::new(CcsdsType::TM, sec_hdr);

    pkt.data = HexBytes((0..255).collect());

    match serde_json::to_string_pretty(&pkt) {
        Ok(str) => {
            println!("{}", str);
        }
        Err(err) => {
            println!("Error returned: {}", err);
        }
    }
}
