use std::io::Error;

use bytes::Bytes;
use hmac_sha256::Hash;
use rasn::{
    types::{OctetString, Utf8String, VisibleString},
    AsnType, Decode, Encode,
};
use sha1_smol::Sha1;

use crate::sle::config::HashToUse;

use super::sle::TimeCCSDS;

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
pub struct HashInput {
    time: TimeCCSDS,
    random: i32,
    user_name: VisibleString,
    password: OctetString,
}

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
pub struct ISP1Credentials {
    time: TimeCCSDS,
    random: i32,
    the_protected: Vec<u8>,
}

impl HashInput {
    pub fn new(
        time: &TimeCCSDS,
        random: i32,
        user: &str,
        passwd: &str,
    ) -> Result<HashInput, Error> {
        let passwd = match hex::decode(passwd) {
            Ok(v) => v,
            Err(_) => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Password is not Hex-ASCII encoded",
                ));
            }
        };

        Ok(HashInput {
            time: time.clone(),
            random,
            user_name: VisibleString::new(Utf8String::from(user)),
            password: Bytes::copy_from_slice(&passwd),
        })
    }

    pub fn the_protected(&self, mode: HashToUse) -> Result<Vec<u8>, rasn::ber::enc::Error> {
        let out = rasn::der::encode(self)?;
        match mode {
            HashToUse::SHA1 => {
                let sha1 = Sha1::from(&out);
                Ok(Vec::from(sha1.digest().bytes()))
            }
            HashToUse::SHA256 => {
                let sha256 = Hash::hash(&out);
                Ok(Vec::from(sha256))
            }
        }
    }
}

pub fn check_credentials(
    credentials: &ISP1Credentials,
    authority_identifier: &str,
    password: &str,
) -> bool {
    match HashInput::new(
        &credentials.time,
        credentials.random,
        authority_identifier,
        password,
    ) {
        Ok(hi) => {
            let len = credentials.the_protected.len();
            let prot = if len == 20 {
                match hi.the_protected(HashToUse::SHA1) {
                    Ok(res) => res,
                    Err(_) => {
                        return false;
                    }
                }
            } else {
                match hi.the_protected(HashToUse::SHA256) {
                    Ok(res) => res,
                    Err(_) => {
                        return false;
                    }
                }
            };

            prot == credentials.the_protected
        }
        Err(_) => false,
    }
}
