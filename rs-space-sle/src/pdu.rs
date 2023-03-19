use crate::asn1_2::*;

pub enum PDU {
    SlePduBind(SleBindInvocation),
    SlePduBindReturn(SleBindReturn),
}
