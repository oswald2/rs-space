use log::{debug, error, info, LevelFilter};

use tokio::io::{Error, ErrorKind};

use rs_space_core::ccsds_packet::{CcsdsPacket, FastCcsdsPacket};
use rs_space_core::pus_packet::PUSPacket;
use rs_space_core::pus_sec_hdr::gal_pus_c::GalSecHdrTC;
use rs_space_core::pus_sec_hdr::pus_sec_hdr::*;

pub fn process_fast_packet(pkt: FastCcsdsPacket) -> Result<(), Error> {
    debug!("Received: {:?}", pkt);
    if pkt.check_crc() {
        let ccsds_pkt = pkt.to_ccsds_packet();

        debug!("CcsdsPacket: {:?}", ccsds_pkt);

        process_ccsds_packet(ccsds_pkt)
    } else {
        Err(Error::new(ErrorKind::Other, "FastCcsdsPacket: CRC Error"))
    }
}

pub fn process_ccsds_packet(pkt: CcsdsPacket) -> Result<(), Error> {
    let res = PUSPacket::from_ccsds_packet(pkt, Box::new(GalSecHdrTC::new()));
    match res {
        Ok(pus_pkt) => {
            debug!("PUS Packet: {:?}", pus_pkt);
            Ok(())
        }
        Err(err) => Err(err),
    }
}
