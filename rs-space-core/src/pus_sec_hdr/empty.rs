use serde::{Serialize, Deserialize};

use crate::pus_sec_hdr::pus_sec_hdr::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PUSEmptyHeader;

impl PUSSecHeader for PUSEmptyHeader {
    // return the PUS Type of the secondary header
    fn pus_type(&self) -> PUSType {
        PUSType(0)
    }

    // return the PUS SubType of the secondary header
    fn pus_sub_type(&self) -> PUSSubType {
        PUSSubType(0)
    }

    // return the Source- (or Destination)-ID of the secondary header
    fn pus_src_id(&self) -> PUSSrcID {
        PUSSrcID(0)
    }

    // return the length of the secondary header
    fn len(&self) -> usize {
        0
    }

    // Parse a secondary header from a byte slice
    fn from_bytes(&mut self, _arr: &[u8]) -> Result<(), std::io::Error> {
        Ok(())
    }

    // Encode a secondary header to a byte slice
    fn to_bytes(&self, _arr: &mut [u8]) -> Result<(), std::io::Error> {
        Ok(())
    }
}
