#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct PUSType(pub u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct PUSSubType(pub u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct PUSSrcID(pub u16);

pub trait PUSSecHeader {
    // return the PUS Type of the secondary header
    fn pus_type(&self) -> PUSType;

    // return the PUS SubType of the secondary header
    fn pus_sub_type(&self) -> PUSSubType;

    // return the Source- (or Destination)-ID of the secondary header
    fn pus_src_id(&self) -> PUSSrcID;

    // return the length of the secondary header
    fn len(&self) -> usize;

    // Parse a secondary header from a byte slice
    fn from_bytes(&mut self, arr: &[u8]) -> Result<(), std::io::Error>;

    // Encode a secondary header to a byte slice
    fn to_bytes(&self, arr: &mut [u8]) -> Result<(), std::io::Error>;
}
