use rasn::types::{ConstOid, Integer, Oid, SetOf, Tag, Class};
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
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

impl ServiceID {
    pub fn from_oid(oid: &Oid) -> ServiceID {
        match oid.get(7) {
            Some(40) => ServiceID::RSP,
            Some(7) => ServiceID::FCLTU,
            Some(53) => ServiceID::SPACK,
            Some(46) => ServiceID::RCF,
            Some(16) => ServiceID::TCVA,
            Some(38) => ServiceID::RSLFG,
            Some(22) => ServiceID::RAF,
            Some(14) => ServiceID::FSLFG,
            Some(10) => ServiceID::FSP,
            Some(52) => ServiceID::SAGR,
            Some(49) => ServiceID::ROCF,
            Some(12) => ServiceID::TCF,
            Some(44) => ServiceID::RCFSH,
            Some(_) => ServiceID::SAGR,
            None => ServiceID::SAGR,
        }
    }
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

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct ServiceAttribute {
    attribute: ServiceID,
    value: String,
}

impl AsnType for ServiceAttribute {
    const TAG: Tag = Tag::SEQUENCE;
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

impl Decode for ServiceAttribute {
    fn decode_with_tag<D: Decoder>(decoder: &mut D, tag: Tag) -> Result<Self, D::Error> {
        decoder.decode_sequence(Tag::SEQUENCE, |decoder| {
            let oid: ServiceID =
                ServiceID::from_oid(&*decoder.decode_object_identifier(Tag::OBJECT_IDENTIFIER)?);
            let str = decoder.decode_utf8_string(Tag::VISIBLE_STRING)?;
            Ok(Self {
                attribute: oid,
                value: str,
            })
        })
    }
}

#[derive(Debug, Clone)]
struct ServiceAttr(Vec<ServiceAttribute>);

impl AsnType for ServiceAttr {
    const TAG: Tag = Tag::SET;
}

impl Decode for ServiceAttr {
    fn decode_with_tag<D: Decoder>(decoder: &mut D, _tag: Tag) -> Result<Self, D::Error> {
        let fields: SetOf<ServiceAttribute> = decoder.decode_set_of(Tag::SET)?;
        let v = fields.into_iter().collect();
        Ok(ServiceAttr(v))
    }
}

#[derive(Debug)]
pub struct ServiceInstanceID {
    attributes: Vec<ServiceAttr>,
}

impl AsnType for ServiceInstanceID {
    const TAG: Tag = Tag::SEQUENCE;
}

impl Decode for ServiceInstanceID {
    fn decode_with_tag<D: Decoder>(decoder: &mut D, _tag: Tag) -> Result<Self, D::Error> {
        let vals = decoder.decode_sequence_of(Tag::SET)?;
        Ok(ServiceInstanceID { attributes: vals })
    }
}

#[derive(Debug)]
pub struct Credentials(Option<String>);


impl AsnType for Credentials {
    const TAG: Tag = Tag::new(Class::Context, 0);
}

impl Decode for Credentials {
    fn decode_with_tag<D: Decoder>(decoder: &mut D, _tag: Tag) -> Result<Self, D::Error> {
        let c = decoder.decode_null(Tag::new(Class::Context, 0));
        
        match c {
            Ok(_) => Ok(Credentials(None)),
            Err(_) => {
                Ok(Credentials(Some(decoder.decode_utf8_string(Tag::new(Class::Context, 1))?)))
            }
        }
    }
}
