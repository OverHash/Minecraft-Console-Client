use crate::protocol::Packet;

#[derive(Default)]
pub struct Status {}

/// Implement conversion from Status -> Packet
impl From<Status> for Packet {
    fn from(_: Status) -> Self {
        Self::new(0x00, vec![])
    }
}
