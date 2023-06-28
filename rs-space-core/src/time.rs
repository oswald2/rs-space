use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::time::{Duration, SystemTime};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TimeEncoding {
    CUC42,
    CDS8,
    CDS10,
}

pub const fn time_length(enc: TimeEncoding) -> usize {
    match enc {
        TimeEncoding::CUC42 => 6,
        TimeEncoding::CDS8 => 8,
        TimeEncoding::CDS10 => 10,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn from_sytem_time(time: SystemTime, enc: TimeEncoding) -> Time {
        match time.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => Time {
                time: n,
                encoding: enc,
            },
            Err(_) => panic!("System Time before UNIX EPOCH!"),
        }
    }

    pub fn from_duration(dur: Duration, enc: TimeEncoding) -> Time {
        Time {
            time: dur,
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
        time_length(self.encoding)
    }

    pub fn encode_into(
        &self,
        enc: Option<TimeEncoding>,
        arr: &mut [u8],
    ) -> Result<(), std::io::Error> {
        let enc = enc.unwrap_or(self.encoding);
        match enc {
            TimeEncoding::CUC42 => {
                let secs = self.time.as_secs();
                let micro = self.time.subsec_micros();

                let mut wrtr = Cursor::new(arr);
                wrtr.write_u32::<BigEndian>(secs as u32)?;
                let subsec = ((micro as f64) * 65536.0 / 1_000_000.0).round() as u16;
                wrtr.write_u16::<BigEndian>(subsec)
            }
            TimeEncoding::CDS8 => {
                let secs = self.time.as_secs();
                let micro = self.time.subsec_micros();

                let epoch_secs = secs + 378691200;
                let days = (epoch_secs / 86400) as u16;
                let milli = ((epoch_secs % 86400) * 1_000 + micro as u64 / 1_000) as u32;
                let mic: u16 = (micro % 1_000) as u16;

                let mut wrtr = Cursor::new(arr);
                wrtr.write_u16::<BigEndian>(days)?;
                wrtr.write_u32::<BigEndian>(milli)?;
                wrtr.write_u16::<BigEndian>(mic)?;
                Ok(())
            }
            TimeEncoding::CDS10 => {
                let secs = self.time.as_secs();
                let nano = self.time.subsec_nanos();

                let epoch_secs = secs + 378691200;
                let days = (epoch_secs / 86400) as u16;
                let milli = ((epoch_secs % 86400) * 1_000 + nano as u64 / 1_000_000) as u32;
                let pico: u32 = (nano % 1_000_000) * 1_000;

                let mut wrtr = Cursor::new(arr);
                wrtr.write_u16::<BigEndian>(days)?;
                wrtr.write_u32::<BigEndian>(milli)?;
                wrtr.write_u32::<BigEndian>(pico)?;
                Ok(())
            }
        }
    }

    pub fn encode(&self, enc: Option<TimeEncoding>) -> Result<Vec<u8>, std::io::Error> {
        let enc = enc.unwrap_or(self.encoding);
        let mut res = vec![0; time_length(enc)];
        self.encode_into(Some(enc), &mut res)?;
        Ok(res)
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
            TimeEncoding::CDS8 => {
                let mut rdr = Cursor::new(arr);
                let days = rdr.read_u16::<BigEndian>()?;
                let milli = rdr.read_u32::<BigEndian>()?;
                let micro = rdr.read_u16::<BigEndian>()?;

                let secs = days as u64 * 86400 - 378691200 + milli as u64 / 1_000;
                let nano = ((milli % 1_000) * 1_000 + micro as u32) * 1_000;

                self.time = Duration::new(secs, nano);
                Ok(())
            }
            TimeEncoding::CDS10 => {
                let mut rdr = Cursor::new(arr);
                let days = rdr.read_u16::<BigEndian>()?;
                let milli = rdr.read_u32::<BigEndian>()?;
                let pico = rdr.read_u32::<BigEndian>()?;

                let secs = days as u64 * 86400 - 378691200 + milli as u64 / 1_000;
                let nano = (milli % 1_000) * 1_000_000 + pico / 1_000;

                self.time = Duration::new(secs, nano);
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
            TimeEncoding::CDS8 => {
                let mut rdr = Cursor::new(arr);
                let days = rdr.read_u16::<BigEndian>()?;
                let milli = rdr.read_u32::<BigEndian>()?;
                let micro = rdr.read_u16::<BigEndian>()?;

                let secs = days as u64 * 86400 - 378691200 + milli as u64 / 1_000;
                let nano = ((milli % 1_000) * 1_000 + micro as u32) * 1_000;

                Ok(Time {
                    time: Duration::new(secs, nano),
                    encoding: TimeEncoding::CDS8,
                })
            }
            TimeEncoding::CDS10 => {
                let mut rdr = Cursor::new(arr);
                let days = rdr.read_u16::<BigEndian>()?;
                let milli = rdr.read_u32::<BigEndian>()?;
                let pico = rdr.read_u32::<BigEndian>()?;

                let secs = days as u64 * 86400 - 378691200 + milli as u64 / 1_000;
                let nano = (milli % 1_000) * 1_000_000 + pico / 1_000;

                Ok(Time {
                    time: Duration::new(secs, nano),
                    encoding: TimeEncoding::CDS10,
                })
            }
        }
    }
}
