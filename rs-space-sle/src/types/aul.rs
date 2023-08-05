use bytes::Bytes;
use hmac_sha256::Hash;
use rasn::{
    types::{OctetString, Utf8String, VisibleString},
    AsnType, Decode, Encode,
};
use sha1_smol::Sha1;

use crate::sle::config::{HashToUse};

use super::sle::{to_ccsds_time, TimeCCSDS};

#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
pub struct ISP1Credentials {
    pub time: TimeCCSDS,
    pub random: i32,
    pub the_protected: OctetString,
}

impl ISP1Credentials {
    pub fn new(
        hash_to_use: HashToUse,
        time: &rs_space_core::time::Time,
        random: i32,
        name: &str,
        password: &[u8],
    ) -> ISP1Credentials {
        let t = to_ccsds_time(time).expect("Error encoding time to SLE CCSDS Time");
        let name = VisibleString::new(Utf8String::from(name));
        let password = Bytes::copy_from_slice(password.as_ref());
        let hi = HashInput::new(&t, random, &name, password);
        let protected = hi.the_protected(hash_to_use);

        ISP1Credentials {
            time: t,
            random,
            the_protected: protected,
        }
    }
}

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
pub struct HashInput {
    time: TimeCCSDS,
    random: i32,
    user_name: VisibleString,
    password: OctetString,
}

impl HashInput {
    pub fn new(time: &TimeCCSDS, random: i32, user: &VisibleString, password: Bytes) -> HashInput {
        HashInput {
            time: time.clone(),
            random,
            user_name: user.clone(),
            password,
        }
    }

    pub fn the_protected(&self, mode: HashToUse) -> Bytes {
        let out = rasn::der::encode(self).unwrap();
        match mode {
            HashToUse::SHA1 => {
                let sha1 = Sha1::from(&out);
                Bytes::copy_from_slice(&sha1.digest().bytes())
            }
            HashToUse::SHA256 => {
                let sha256 = Hash::hash(&out);
                Bytes::copy_from_slice(&sha256)
            }
        }
    }
}

pub fn check_credentials(
    credentials: &ISP1Credentials,
    authority_identifier: &VisibleString,
    password: &Bytes,
) -> bool {
    let hi = HashInput::new(
        &credentials.time,
        credentials.random,
        authority_identifier,
        password.clone(),
    );

    let len = credentials.the_protected.len();
    let prot = if len == 20 {
        hi.the_protected(HashToUse::SHA1)
    } else {
        hi.the_protected(HashToUse::SHA256)
    };

    prot == credentials.the_protected
}

