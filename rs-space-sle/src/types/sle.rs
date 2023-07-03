use std::{collections::BTreeSet, io::Error};

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case},
    multi::separated_list1,
    sequence::separated_pair,
    IResult,
};
use rasn::{
    types::{Class, ConstOid, Implicit, ObjectIdentifier, OctetString, SetOf, VisibleString},
    AsnType, Decode, Encode, Tag,
};
use serde::{Deserialize, Serialize};

use bytes::Bytes;

use super::aul::ISP1Credentials;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum SleVersion {
    V3 = 3,
    V4 = 4,
    V5 = 5,
}

impl TryFrom<u8> for SleVersion {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(SleVersion::V3),
            4 => Ok(SleVersion::V4),
            5 => Ok(SleVersion::V5),
            x => Err(format!("Illegal number for SLE Version: {}", x)),
        }
    }
}

#[derive(AsnType, Debug, PartialEq, Encode, Decode)]
#[rasn(choice)]
pub enum Time {
    #[rasn(tag(0))]
    CcsdsFormat(TimeCCSDS),
    #[rasn(tag(1))]
    CcsdsPicoFormat(TimeCCSDSpico),
}

//#[derive(AsnType, Debug, Clone, PartialEq, Encode, Decode)]
pub type TimeCCSDS = OctetString;

pub fn null_ccsds_time() -> TimeCCSDS {
    Bytes::copy_from_slice(&[0; 8])
}

pub type TimeCCSDSpico = OctetString;

pub fn to_ccsds_time(time: &rs_space_core::time::Time) -> Result<TimeCCSDS, Error> {
    let mut tmp = [0; 8];
    time.encode_into(Some(rs_space_core::time::TimeEncoding::CDS8), &mut tmp)?;
    Ok(Bytes::copy_from_slice(&tmp))
}

pub fn to_ccsds_time_pico(time: &rs_space_core::time::Time) -> Result<TimeCCSDSpico, Error> {
    let mut tmp = [0; 10];
    time.encode_into(Some(rs_space_core::time::TimeEncoding::CDS8), &mut tmp)?;
    Ok(Bytes::copy_from_slice(&tmp))
}

// ASN1 common types
pub type ConditionalTime = Option<Time>;

#[derive(AsnType, Debug, PartialEq)]
#[rasn(choice)]
pub enum Credentials {
    #[rasn(context, 0)]
    Unused,
    #[rasn(context, 1)]
    Used(ISP1Credentials),
}

// Ok, for Credentials, we need to write our own encoder. This is because SLE does some weird ASN1 things.
impl Encode for Credentials {
    fn encode_with_tag<E: rasn::Encoder>(
        &self,
        encoder: &mut E,
        _tag: rasn::Tag,
    ) -> Result<(), E::Error> {
        match self {
            // Unused is a NULL value tagged with Context 0
            Credentials::Unused => {
                encoder.encode_null(Tag {
                    class: Class::Context,
                    value: 0,
                })?;
                Ok(())
            }
            // Used is an Octet String (the ASN1 encoded ISP1 Credentials) tagged with Context 1
            Credentials::Used(isp1) => {
                let content = rasn::der::encode(&isp1).unwrap();
                encoder.encode_octet_string(
                    Tag {
                        class: Class::Context,
                        value: 1,
                    },
                    &content,
                )?;
                Ok(())
            }
        }
    }
}

impl Decode for Credentials {
    fn decode_with_tag<D: rasn::Decoder>(decoder: &mut D, _tag: Tag) -> Result<Self, D::Error> {
        match decoder.decode_null(Tag {
            class: Class::Context,
            value: 0,
        }) {
            Ok(_) => Ok(Credentials::Unused),
            Err(_err) => {
                let val = decoder.decode_octet_string(Tag {
                    class: Class::Context,
                    value: 1,
                })?;
                match rasn::der::decode(&val) {
                    Ok(isp1) => Ok(Credentials::Used(isp1)),
                    Err(_err) => Err(rasn::de::Error::no_valid_choice("ISP1Credentials")),
                }
            }
        }
    }
}

// #[derive(AsnType, Debug, PartialEq, Encode, Decode)]
pub type ServiceInstanceIdentifier = Vec<ServiceInstanceAttribute>;

#[derive(AsnType, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Encode, Decode)]
pub struct ServiceInstanceAttributeInner {
    pub identifier: ObjectIdentifier,
    pub si_attribute_value: VisibleString,
}

pub type ServiceInstanceAttribute = SetOf<ServiceInstanceAttributeInner>;

pub fn new_service_instance_attribute(id: &ConstOid, value: &str) -> ServiceInstanceAttribute {
    let mut tree = BTreeSet::new();
    tree.insert(ServiceInstanceAttributeInner {
        identifier: ObjectIdentifier::new_unchecked(std::borrow::Cow::Borrowed(id.0)),
        si_attribute_value: Implicit::new(String::from(value)),
    });
    tree
}

pub const SAGR: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 52]);
pub const SPACK: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 53]);
pub const FSL_FG: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 14]);
pub const RSL_FG: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 38]);
pub const CLTU: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 7]);
pub const FSP: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 10]);
pub const RAF: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 22]);
pub const RCF: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 46]);
pub const RCFSH: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 44]);
pub const ROCF: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 49]);
pub const RSP: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 40]);
pub const TCF: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 12]);
pub const TCVA: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 16]);

pub fn service_oid_to_string(oid: &ObjectIdentifier) -> Result<String, String> {
    match oid.as_ref() {
        x if x == SAGR.0 => Ok("sagr".to_owned()),
        x if x == SPACK.0 => Ok("spack".to_owned()),
        x if x == FSL_FG.0 => Ok("fsl-fg".to_owned()),
        x if x == RSL_FG.0 => Ok("rsl-fg".to_owned()),
        x if x == CLTU.0 => Ok("cltu".to_owned()),
        x if x == FSP.0 => Ok("fsp".to_owned()),
        x if x == RAF.0 => Ok("raf".to_owned()),
        x if x == RCF.0 => Ok("rcf".to_owned()),
        x if x == RCFSH.0 => Ok("rcfsh".to_owned()),
        x if x == ROCF.0 => Ok("rocf".to_owned()),
        x if x == RSP.0 => Ok("rsp".to_owned()),
        x if x == TCF.0 => Ok("tcf".to_owned()),
        x if x == TCVA.0 => Ok("tcva".to_owned()),
        x => Err(format!(
            "Could not parse OID for service attribute: {:?}",
            x
        )),
    }
}

pub fn service_instance_identifier_to_string(
    si_identifier: &ServiceInstanceIdentifier,
) -> Result<String, String> {
    let mut si_strings: Vec<String> = Vec::new();

    for attr in si_identifier.iter().flatten() {
        let oid_string = service_oid_to_string(&attr.identifier)?;
        let attr_string = format!("{}={}", oid_string, attr.si_attribute_value.as_str());
        si_strings.push(attr_string);
    }

    Ok(si_strings.join("."))
}

pub fn string_to_service_instance_id(sii: &str) -> Result<ServiceInstanceIdentifier, String> {
    match service_instance_id_parser(sii) {
        Ok((_, val)) => Ok(val),
        Err(err) => Err(format!("Error on parsing SII from string: {err}")),
    }
}

fn service_instance_id_parser(sii: &str) -> IResult<&str, ServiceInstanceIdentifier> {
    let (input, res) = separated_list1(tag("."), attrib_parser)(sii)?;
    Ok((input, res))
}

fn attrib_parser(input: &str) -> IResult<&str, ServiceInstanceAttribute> {
    let (input, (oid, value)) = separated_pair(attr_name_parser, tag("="), is_not("."))(input)?;

    Ok((input, new_service_instance_attribute(oid, value)))
}

fn attr_name_parser(input: &str) -> IResult<&str, &ConstOid> {
    alt((
        sagr_parser,
        spack_parser,
        rsl_fg_parser,
        fsl_fg_parser,
        cltu_parser,
        fsp_parser,
        raf_parser,
        rcf_parser,
        rcfsh_parser,
        rocf_parser,
        rsp_parser,
        tcf_parser,
        tcva_parser,
    ))(input)
}

fn sagr_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("sagr")(input)?;
    Ok((input, &SAGR))
}

fn spack_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("spack")(input)?;
    Ok((input, &SPACK))
}

fn rsl_fg_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("rsl-fg")(input)?;
    Ok((input, &RSL_FG))
}

fn fsl_fg_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("fsl-fg")(input)?;
    Ok((input, &FSL_FG))
}

fn cltu_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("cltu")(input)?;
    Ok((input, &CLTU))
}

fn fsp_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("fsp")(input)?;
    Ok((input, &FSP))
}

fn raf_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("raf")(input)?;
    Ok((input, &RAF))
}

fn rcf_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("rcf")(input)?;
    Ok((input, &RCF))
}

fn rcfsh_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("rcfsh")(input)?;
    Ok((input, &RCFSH))
}

fn rocf_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("rocf")(input)?;
    Ok((input, &ROCF))
}

fn rsp_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("rsp")(input)?;
    Ok((input, &RSP))
}

fn tcf_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("tcf")(input)?;
    Ok((input, &TCF))
}

fn tcva_parser(input: &str) -> IResult<&str, &ConstOid> {
    let (input, _) = tag_no_case("tcva")(input)?;
    Ok((input, &TCVA))
}
