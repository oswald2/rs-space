use serde::{Deserialize, Serialize};

use crate::pus_sec_hdr::gal_pus_c::{GalSecHdrTC, GalSecHdrTM};
use crate::time::Time;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PUSType(pub u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PUSSubType(pub u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PUSSrcID(pub u16);

pub trait PUSSecHeader {
    /// return the PUS Type of the secondary header
    fn pus_type(&self) -> PUSType;

    /// return the PUS SubType of the secondary header
    fn pus_sub_type(&self) -> PUSSubType;

    /// return the Source- (or Destination)-ID of the secondary header
    fn pus_src_id(&self) -> PUSSrcID;

    /// return the length of the secondary header
    fn len(&self) -> usize;

    /// Parse a secondary header from a byte slice
    fn from_bytes(&mut self, arr: &[u8]) -> Result<(), std::io::Error>;

    /// Encode a secondary header to a byte slice
    fn to_bytes(&self, arr: &mut [u8]) -> Result<(), std::io::Error>;

    /// Set the PUS Type
    fn set_pus_type(&mut self, typ: PUSType);

    /// Set the PUS Sub Type
    fn set_pus_sub_type(&mut self, typ: PUSSubType);

    /// Set the Source/Destination ID in the secondary header
    fn set_src_id(&mut self, src_id: PUSSrcID);
}


pub trait PUSSecTCHeader: PUSSecHeader {
    fn acceptance_flag(&self) -> bool;
    fn start_flag(&self) -> bool;
    fn progress_flag(&self) -> bool;
    fn completion_flag(&self) -> bool;

    fn set_acceptance_flag(&mut self, flag: bool);
    fn set_start_flag(&mut self, flag: bool);
    fn set_progress_flag(&mut self, flag: bool);
    fn set_completion_flag(&mut self, flag: bool);
}

pub trait PUSSecTMHeader {
    fn time() -> Time;
    fn set_time(&mut self, time: Time);
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PUSSecondaryHeader {
    Empty,
    GALTC(GalSecHdrTC),
    GALTM(GalSecHdrTM),
}

impl PUSSecHeader for PUSSecondaryHeader {
    fn pus_type(&self) -> PUSType {
        match self {
            PUSSecondaryHeader::Empty => PUSType(0),
            PUSSecondaryHeader::GALTC(hdr) => hdr.pus_type(),
            PUSSecondaryHeader::GALTM(hdr) => hdr.pus_type(),
        }
    }

    fn pus_sub_type(&self) -> PUSSubType {
        match self {
            PUSSecondaryHeader::Empty => PUSSubType(0),
            PUSSecondaryHeader::GALTC(hdr) => hdr.pus_sub_type(),
            PUSSecondaryHeader::GALTM(hdr) => hdr.pus_sub_type(),
        }
    }

    fn pus_src_id(&self) -> PUSSrcID {
        match self {
            PUSSecondaryHeader::Empty => PUSSrcID(0),
            PUSSecondaryHeader::GALTC(hdr) => hdr.pus_src_id(),
            PUSSecondaryHeader::GALTM(hdr) => hdr.pus_src_id(),
        }
    }

    fn len(&self) -> usize {
        match self {
            PUSSecondaryHeader::Empty => 0,
            PUSSecondaryHeader::GALTC(hdr) => hdr.len(),
            PUSSecondaryHeader::GALTM(hdr) => hdr.len(),
        }
    }

    fn from_bytes(&mut self, arr: &[u8]) -> Result<(), std::io::Error> {
        match self {
            PUSSecondaryHeader::Empty => Ok(()),
            PUSSecondaryHeader::GALTC(hdr) => hdr.from_bytes(arr),
            PUSSecondaryHeader::GALTM(hdr) => hdr.from_bytes(arr),
        }
    }

    fn to_bytes(&self, arr: &mut [u8]) -> Result<(), std::io::Error> {
        match self {
            PUSSecondaryHeader::Empty => Ok(()),
            PUSSecondaryHeader::GALTC(hdr) => hdr.to_bytes(arr),
            PUSSecondaryHeader::GALTM(hdr) => hdr.to_bytes(arr),
        }
    }

    fn set_pus_type(&mut self, typ: PUSType) {
        match self {
            PUSSecondaryHeader::Empty => (),
            PUSSecondaryHeader::GALTC(hdr) => hdr.set_pus_type(typ),
            PUSSecondaryHeader::GALTM(hdr) => hdr.set_pus_type(typ),
        }
    }

    fn set_pus_sub_type(&mut self, typ: PUSSubType) {
        match self {
            PUSSecondaryHeader::Empty => (),
            PUSSecondaryHeader::GALTC(hdr) => hdr.set_pus_sub_type(typ),
            PUSSecondaryHeader::GALTM(hdr) => hdr.set_pus_sub_type(typ),
        }
    }

    fn set_src_id(&mut self, src_id: PUSSrcID) {
        match self {
            PUSSecondaryHeader::Empty => (),
            PUSSecondaryHeader::GALTC(hdr) => hdr.set_src_id(src_id),
            PUSSecondaryHeader::GALTM(hdr) => hdr.set_src_id(src_id),
        }
    }
}
