use crate::types::sle::SleVersion;
use crate::asn1::UnbindReason;
use crate::types::sle::{
    PeerAbortDiagnostic
};

pub trait ProviderNotifier {
    fn bind_succeeded(&self, peer: &str, sii: &str, version: SleVersion);
    fn unbind_succeeded(&self, sii: &str, reason: UnbindReason);
    fn peer_abort(&self, sii: &str, diagnostic: &PeerAbortDiagnostic);
}
