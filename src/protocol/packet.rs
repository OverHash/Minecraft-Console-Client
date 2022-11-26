use std::num::TryFromIntError;

use super::encoding::VarInt;

pub struct Packet {
    id: VarInt,
    data: Vec<u8>,
}

impl Packet {
    /// Creates a new packet with the given `id` and `data` fields.
    pub fn new<T: Into<VarInt>>(id: T, data: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            data,
        }
    }
}

impl TryFrom<Packet> for Vec<u8> {
    type Error = TryFromIntError;

    fn try_from(p: Packet) -> Result<Self, Self::Error> {
        let full_data = [p.id.as_slice(), p.data.as_slice()].concat();
        let data_len = i32::try_from(full_data.len())?;

        Ok(vec![VarInt::from(data_len).as_slice(), full_data.as_slice()].concat())
    }
}
