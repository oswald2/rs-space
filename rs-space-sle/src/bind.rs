
use rasn::{AsnType, Encode, Decode};
use rasn::types::{Tag};


use crate::common::*;

#[derive(Debug, Decode)]
pub struct SleBindInvocation {
    pub credentials: Credentials,
    pub initiator: AuthorityIdentifier,
    pub port_id: PortID,
    pub service_type: ApplicationIdentifier,
    pub version: VersionNumber,
    pub service_instance_id: ServiceInstanceID,
}

impl AsnType for SleBindInvocation {
    const TAG: Tag = Tag::new(rasn::types::Class::Context, 100);
}


