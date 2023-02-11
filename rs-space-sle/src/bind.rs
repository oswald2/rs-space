
use rasn::AsnType;
use rasn::types::{ConstOid, Tag};

use crate::common::*;

#[derive(Debug)]
pub struct SleBindInvocation {
    pub credentials: Option<Credentials>,
    pub initiator: AuthorityIdentifier,
    pub port_id: PortID,
    pub service_type: ApplicationIdentifier,
    pub version: VersionNumber,
    pub service_instance_id: ServiceInstanceID,
}

impl AsnType for SleBindInvocation {
    const TAG: Tag = Tag::SEQUENCE;
}