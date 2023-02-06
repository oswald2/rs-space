
use rasn::AsnType;

use crate::common::*;

#[derive(Debug, AsnType)]
pub struct SleBindInvocation {
    #[rasn(tag(context, 100))]
    pub credentials: Option<Credentials>,
    pub initiator: AuthorityIdentifier,
    pub port_id: PortID,
    pub service_type: ApplicationIdentifier,
    pub version: VersionNumber,
    pub service_instance_id: ServiceInstanceID,
}
