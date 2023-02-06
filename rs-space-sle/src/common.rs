use rasn::types::ConstOid;

#[derive(Debug)]
pub enum ApplicationIdentifier {
    RtnAllFrames,
    RtnInsert,
    RtnChFrames,
    RtnChFsh,
    RtnChOcf,
    RtnBitstr,
    RtnSpacePkt,
    FwdAosSpacePkt,
    FwdAosVca,
    FwdBitstr,
    FwdProtoVcdu,
    FwdInsert,
    FwdCVcdu,
    FwdTcSpacePkt,
    FwdTcVca,
    FwdTcFrame,
    FwdCltu,
}

#[derive(Debug)]
pub struct AuthorityIdentifier(String);

#[derive(Debug)]
pub struct PortID(String);

#[derive(Debug)]
pub struct VersionNumber(u16);

pub const RSP_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 40]);
pub const FCLTU_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 7]);
pub const SPACK_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 53]);
pub const RCF_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 46]);
pub const TCVA_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 16]);
pub const RSLFG_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 38]);
pub const RAF_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 22]);
pub const FSLFG_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 14]);
pub const FSP_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 10]);
pub const SAGR_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 52]);
pub const ROCF_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 49]);
pub const TCF_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 12]);
pub const RCFSH_OID: ConstOid = ConstOid(&[1, 3, 112, 4, 3, 1, 2, 44]);

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ServiceAttribute {
    attribute: ServiceID,
    value: String,
}

#[derive(Debug)]
pub struct ServiceInstanceID {
    attributes: Vec<ServiceAttribute>,
}

#[derive(Debug)]
pub struct Credentials();
