use std::{collections::HashMap, fmt::Display};

use bytes::Bytes;
use rasn::types::{Utf8String, VisibleString};
use rs_space_core::pus_types::HexBytes;
use serde::{Deserialize, Serialize};

use crate::{tml::config::TMLConfig, types::sle::SleVersion};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum HashToUse {
    SHA1,
    SHA256,
}

impl Default for HashToUse {
    fn default() -> Self {
        HashToUse::SHA256
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum SleAuthType {
    AuthNone,
    AuthBind,
    AuthAll,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthorityID(pub String);

impl Display for AuthorityID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub authority_id: AuthorityID,
    pub password: HexBytes,
}

#[derive(Debug, Clone)]
pub struct PeerASN1 {
    pub authority_id: VisibleString,
    pub password: Bytes,
}

#[derive(Debug, Clone)]
pub struct CommonConfig {
    pub tml: TMLConfig,
    pub authority_identifier: VisibleString,
    pub password: Bytes,
    pub auth_type: SleAuthType,
    pub hash_to_use: HashToUse,
    pub version: SleVersion,
    pub peer_map: HashMap<VisibleString, PeerASN1>,
}

impl CommonConfig {
    pub fn from(conf: CommonConfigExt) -> Self {
        CommonConfig {
            tml: conf.tml.clone(),
            authority_identifier: VisibleString::new(Utf8String::from(conf.authority_identifier.0)),
            password: Bytes::copy_from_slice(conf.password.as_ref()),
            auth_type: conf.auth_type,
            hash_to_use: conf.hash_to_use,
            version: conf.version,
            peer_map: peer_set(&conf.peers),
        }
    }

    pub fn get_peer(&self, identifier: &VisibleString) -> Option<&PeerASN1> {
        self.peer_map.get(identifier)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonConfigExt {
    pub tml: TMLConfig,
    pub authority_identifier: AuthorityID,
    pub password: HexBytes,
    pub peers: Vec<Peer>,
    pub auth_type: SleAuthType,
    pub hash_to_use: HashToUse,
    pub version: SleVersion,
}

impl CommonConfigExt {
    pub fn new() -> Self {
        CommonConfigExt::default()
    }
}

fn peer_set(peers: &Vec<Peer>) -> HashMap<VisibleString, PeerASN1> {
    peers
        .into_iter()
        .map(|peer| {
            let name = VisibleString::new(Utf8String::from(&peer.authority_id.0));
            (
                name.clone(),
                PeerASN1 {
                    authority_id: name,
                    password: Bytes::copy_from_slice(peer.password.as_ref()),
                },
            )
        })
        .collect()
}

impl Default for CommonConfigExt {
    fn default() -> Self {
        let peer_vec = vec![
            Peer {
                authority_id: AuthorityID("EGSCC".to_string()),
                password: HexBytes(vec![0x12, 0x34, 0x56, 0x78]),
            },
            Peer {
                authority_id: AuthorityID("SLETT".to_string()),
                password: HexBytes(vec![0xaa, 0xbb, 0xcc, 0xdd, 0xaa, 0xbb, 0xcc, 0xdd]),
            },
        ];

        CommonConfigExt {
            tml: TMLConfig::default(),
            authority_identifier: AuthorityID("PARAGONTT".to_string()),
            password: HexBytes(vec![
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
                0x0e, 0x0f,
            ]),
            peers: peer_vec,
            auth_type: SleAuthType::AuthNone,
            hash_to_use: HashToUse::SHA256,
            version: SleVersion::V3,
        }
    }
}
