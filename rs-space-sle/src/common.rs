use rasn::types::{Integer, Tag};
use rasn::AsnType;
use rasn::{Decode, Decoder, Encode, Encoder};

use num_derive::FromPrimitive;
use num_traits::cast::ToPrimitive;

#[derive(Debug, FromPrimitive)]
pub enum ApplicationIdentifier {
    RtnAllFrames = 0,
    RtnInsert = 1,
    RtnChFrames = 2,
    RtnChFsh = 3,
    RtnChOcf = 4,
    RtnBitstr = 5,
    RtnSpacePkt = 6,
    FwdAosSpacePkt = 7,
    FwdAosVca = 8,
    FwdBitstr = 9,
    FwdProtoVcdu = 10,
    FwdInsert = 11,
    FwdCVcdu = 12,
    FwdTcSpacePkt = 13,
    FwdTcVca = 14,
    FwdTcFrame = 15,
    FwdCltu = 16,
    Unknown,
}

impl AsnType for ApplicationIdentifier {
    const TAG: Tag = Tag::INTEGER;
}

impl Encode for ApplicationIdentifier {
    fn encode_with_tag<E: Encoder>(&self, encoder: &mut E, tag: Tag) -> Result<(), E::Error> {
        self.encode(encoder)?;
        Ok(())
    }
}

impl Decode for ApplicationIdentifier {
    fn decode_with_tag<D: Decoder>(decoder: &mut D, tag: Tag) -> Result<Self, D::Error> {
        let val = Integer::decode(decoder)?;
        match val.to_u16() {
            Some(val) => match num::FromPrimitive::from_u16(val) {
                Some(val) => Ok(val),
                None => Ok(ApplicationIdentifier::Unknown),
            },
            None => Ok(ApplicationIdentifier::Unknown),
        }
    }
}

#[derive(Debug)]
pub struct AuthorityIdentifier(String);

impl AsnType for AuthorityIdentifier {
    const TAG: Tag = Tag::VISIBLE_STRING;
}

impl Encode for AuthorityIdentifier {
    fn encode_with_tag<E: Encoder>(&self, encoder: &mut E, tag: Tag) -> Result<(), E::Error> {
        self.0.encode(encoder)?;
        Ok(())
    }
}

impl Decode for AuthorityIdentifier {
    fn decode_with_tag<D: Decoder>(decoder: &mut D, tag: Tag) -> Result<Self, D::Error> {
        let str = decoder.decode_utf8_string(tag)?;
        Ok(AuthorityIdentifier(str))
    }
}

#[derive(Debug)]
pub struct PortID(String);

impl AsnType for PortID {
    const TAG: Tag = Tag::VISIBLE_STRING;
}

impl Encode for PortID {
    fn encode_with_tag<E: Encoder>(&self, encoder: &mut E, tag: Tag) -> Result<(), E::Error> {
        self.0.encode(encoder)?;
        Ok(())
    }
}

impl Decode for PortID {
    fn decode_with_tag<D: Decoder>(decoder: &mut D, tag: Tag) -> Result<Self, D::Error> {
        let str = decoder.decode_utf8_string(tag)?;
        Ok(PortID(str))
    }
}

#[derive(Debug)]
pub struct VersionNumber(u16);

impl AsnType for VersionNumber {
    const TAG: Tag = Tag::INTEGER;
}

impl Encode for VersionNumber {
    fn encode_with_tag<E: Encoder>(&self, encoder: &mut E, tag: Tag) -> Result<(), E::Error> {
        self.encode(encoder)?;
        Ok(())
    }
}

impl Decode for VersionNumber {
    fn decode_with_tag<D: Decoder>(decoder: &mut D, tag: Tag) -> Result<Self, D::Error> {
        let val = Integer::decode(decoder)?;
        match val.to_u16() {
            Some(val) => Ok(VersionNumber(val)),
            None => Err(rasn::de::Error::custom(
                "VersionNumber filed contains illegal value",
            )),
        }
    }
}

pub const RSP_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 40];
pub const FCLTU_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 7];
pub const SPACK_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 53];
pub const RCF_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 46];
pub const TCVA_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 16];
pub const RSLFG_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 38];
pub const RAF_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 22];
pub const FSLFG_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 14];
pub const FSP_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 10];
pub const SAGR_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 52];
pub const ROCF_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 49];
pub const TCF_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 12];
pub const RCFSH_OID: &[u32] = &[1, 3, 112, 4, 3, 1, 2, 44];

#[derive(Debug, Clone, Copy)]
pub enum ServiceID {
    RSP,
    FCLTU,
    SPACK,
    RCF,
    TCVA,
    RSLFG,
    RAF,
    FSLFG,
    FSP,
    SAGR,
    ROCF,
    TCF,
    RCFSH,
}

impl From<ServiceID> for &[u32] {
    fn from(value: ServiceID) -> Self {
        match value {
            ServiceID::RSP => RSP_OID,
            ServiceID::FCLTU => FCLTU_OID,
            ServiceID::SPACK => SPACK_OID,
            ServiceID::RCF => RCF_OID,
            ServiceID::TCVA => TCVA_OID,
            ServiceID::RSLFG => RSLFG_OID,
            ServiceID::RAF => RAF_OID,
            ServiceID::FSLFG => FSLFG_OID,
            ServiceID::FSP => FSP_OID,
            ServiceID::SAGR => SAGR_OID,
            ServiceID::ROCF => ROCF_OID,
            ServiceID::TCF => TCF_OID,
            ServiceID::RCFSH => RCFSH_OID,
        }
    }
}

#[derive(Debug)]
pub struct ServiceAttribute {
    attribute: ServiceID,
    value: String,
}

impl AsnType for ServiceAttribute {
    const TAG: Tag = Tag::SET;
}

impl Encode for ServiceAttribute {
    fn encode_with_tag<E: Encoder>(&self, encoder: &mut E, tag: Tag) -> Result<(), E::Error> {
        encoder.encode_set(tag, |encoder| {
            Ok({
                encoder.encode_sequence(Tag::SEQUENCE, |encoder| {
                    let v: &[u32] = ServiceID::from(self.attribute).into();
                    encoder.encode_object_identifier(Tag::OBJECT_IDENTIFIER, v)?;
                    encoder.encode_utf8_string(Tag::VISIBLE_STRING, &self.value)?;
                    Ok(())
                })?;
            })
        })?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ServiceInstanceID {
    attributes: Vec<ServiceAttribute>,
}

#[derive(Debug)]
pub struct Credentials();
