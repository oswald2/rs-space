use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;
use std::time::{Duration, SystemTime};

pub enum TimeEncoding {
    CUC42,
}

pub const fn time_length(enc: TimeEncoding) -> usize {
    match enc {
        TimeEncoding::CUC42 => 6,
    }
}

pub struct Time {
    time: Duration,
    encoding: TimeEncoding,
}

impl Time {
    pub fn new(enc: TimeEncoding) -> Time {
        Time {
            time: Duration::from_secs(0),
            encoding: enc,
        }
    }

    pub fn now(enc: TimeEncoding) -> Time {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => Time {
                time: n,
                encoding: enc,
            },
            Err(_) => panic!("System Time before UNIX EPOCH!"),
        }
    }

    pub fn len(&self) -> usize {
        match self.encoding {
            TimeEncoding::CUC42 => return 6,
        }
    }

    pub fn encode(&self, arr: &mut [u8]) -> Result<(), std::io::Error> {
        match self.encoding {
            TimeEncoding::CUC42 => {
                let secs = self.time.as_secs();
                let micro = self.time.subsec_micros();

                let mut wrtr = Cursor::new(arr);
                wrtr.write_u32::<BigEndian>(secs as u32)?;
                let subsec = ((micro as f64) * 65536.0 / 1_000_000.0).round() as u16;
                wrtr.write_u16::<BigEndian>(subsec)
            }
        }
    }

    pub fn decode(&mut self, arr: &[u8]) -> Result<(), std::io::Error> {
        match self.encoding {
            TimeEncoding::CUC42 => {
                let mut rdr = Cursor::new(arr);
                let sec = rdr.read_u32::<BigEndian>()?;
                let subsec = rdr.read_u16::<BigEndian>()?;

                let nano = (subsec as f64 * 1_000_000_000.0 / 65536.0).round() as u32;

                self.time = Duration::new(sec as u64, nano);
                Ok(())
            }
        }
    }

    pub fn decode_from_enc(enc: TimeEncoding, arr: &[u8]) -> Result<Time, std::io::Error> {
        match enc {
            TimeEncoding::CUC42 => {
                let mut rdr = Cursor::new(arr);
                let sec = rdr.read_u32::<BigEndian>()?;
                let subsec = rdr.read_u16::<BigEndian>()?;

                let nano = (subsec as f64 * 1_000_000_000.0 / 65536.0).round() as u32;

                Ok(Time {
                    time: Duration::new(sec as u64, nano),
                    encoding: TimeEncoding::CUC42,
                })
            }
        }
    }
}
