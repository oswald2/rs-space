#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy)]
pub struct APID(u16);

impl APID {
    pub fn new(pkt_id: u16) -> APID {
        APID(pkt_id & 0x7FF)
    }

    pub fn new_from_bytes(arr: &[u8]) -> APID {
        APID((((arr[0] & 0b0000_0111) as u16) << 8) | (arr[1] as u16))
    }

    pub fn raw(&self) -> u16 {
        self.0
    }

    pub fn set(&mut self, val: u16) {
        self.0 = val & 0x7FF;
    }

    pub fn to_bytes(&self, arr: &mut [u8]) {
        arr[0] = arr[0] | ((self.0 & 0b0000_0111_0000_000) >> 8) as u8;
        arr[1] = (self.0 & 0xff) as u8;
    }
}

impl std::fmt::Display for APID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy)]
pub struct SSC {
    pub flags: SegFlags,
    pub ssc: u16,
}

const SSC_MASK: u16 = 0b0011_0000_0000_0000;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum SegFlags {
    First,
    Continuation,
    Last,
    Unsegmented,
}

impl std::fmt::Display for SegFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SegFlags::First => write!(f, "FIRST"),
            SegFlags::Continuation => write!(f, "CONT"),
            SegFlags::Last => write!(f, "LAST"),
            SegFlags::Unsegmented => write!(f, "UNSEGMENTED"),
        }
    }
}

impl SSC {
    pub fn new(flags: SegFlags, val: u16) -> SSC {
        SSC {
            flags: flags,
            ssc: val & SSC_MASK,
        }
    }

    pub fn new_unseg(val: u16) -> SSC {
        SSC {
            flags: SegFlags::Unsegmented,
            ssc: val & SSC_MASK,
        }
    }

    pub fn new_from_raw(val: u16) -> SSC {
        let fl = match val & 0b1100_0000_0000_0000 {
            0b0000_0000_0000_0000 => SegFlags::Continuation,
            0b0100_0000_0000_0000 => SegFlags::First,
            0b1000_0000_0000_0000 => SegFlags::Last,
            0b1100_0000_0000_0000 => SegFlags::Unsegmented,
            _ => panic!("seg_flags: cannot happen"),
        };
        SSC {
            flags: fl,
            ssc: val & SSC_MASK,
        }
    }

    pub fn new_from_bytes(arr: &[u8]) -> SSC {
        let val: u16 = ((arr[0] as u16) << 8) | arr[1] as u16;
        Self::new_from_raw(val)
    }

    pub fn flags(&self) -> SegFlags {
        self.flags
    }

    pub fn ssc(&self) -> u16 {
        self.ssc
    }
}

impl std::fmt::Display for SSC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.ssc(), self.flags())
    }
}

#[derive(Debug, Clone)]
pub enum CcsdsType {
    TC,
    TM,
}

#[derive(Debug, Clone)]
pub struct PktID {
    pub version: u8,
    pub ccsds_type: CcsdsType,
    pub dfh: bool,
    pub apid: APID,
}

impl PktID {
    pub fn new(version: u8, typ: CcsdsType, dfh: bool, apid: APID) -> PktID {
        PktID {
            version: version,
            ccsds_type: typ,
            dfh: dfh,
            apid: apid,
        }
    }

    pub fn new_from_raw(val: u16) -> PktID {
        let vers = ((val & 0b1110_0000_0000_0000) >> 13) as u8;
        let t = if (val & 0b0001_0000_0000_0000) != 0 {
            CcsdsType::TC
        } else {
            CcsdsType::TM
        };
        let d = (val & 0b0000_1000_0000_0000) != 0;
        PktID {
            version: vers,
            ccsds_type: t,
            dfh: d,
            apid: APID(val),
        }
    }

    pub fn new_from_bytes(arr: &[u8]) -> PktID {
        let vers: u8 = (arr[0] & 0b1110_0000) >> 5;
        let t = if (arr[0] & 0b0001_0000) != 0 {
            CcsdsType::TC
        } else {
            CcsdsType::TM
        };
        let d: bool = (arr[0] & 0b0000_1000) != 0;

        PktID {
            version: vers,
            ccsds_type: t,
            dfh: d,
            apid: APID::new_from_bytes(arr),
        }
    }

    pub fn raw(&self) -> u16 {
        let v = (self.version as u16) << 13;
        let t: u16 = match self.ccsds_type {
            CcsdsType::TM => 0b0000_0000_0000_0000,
            CcsdsType::TC => 0b0001_0000_0000_0000,
        };
        let d: u16 = if self.dfh { 0b0000_1000_0000_0000 } else { 0 };
        v | t | d | self.apid.raw()
    }

    pub fn to_bytes(&self, arr: &mut [u8]) {
        let t: u8 = match self.ccsds_type {
            CcsdsType::TC => 0b0001_0000,
            CcsdsType::TM => 0,
        };
        let d: u8 = if self.dfh { 0b0000_1000 } else { 0 };
        
        self.apid.to_bytes(arr);
        arr[0] = (self.version << 5) | t | d;
    }
}
