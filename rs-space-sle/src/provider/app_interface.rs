use crate::types::sle::SleVersion;

pub trait ProviderNotifier {
    fn bind_succeeded(&self, peer: &str, sii: &str, version: SleVersion);
}
