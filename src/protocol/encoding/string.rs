use std::num::TryFromIntError;

use super::VarInt;

#[allow(clippy::module_name_repetitions)]
/// Represents a protocol encoded string, with a fixed encoded length and string bytes.
pub struct EncodedString {
    length: VarInt,
    inner: String,
}

impl EncodedString {
    /// Retrieves a reference to the inner encoded string.
    pub fn as_slice(&self) -> Vec<u8> {
        [self.length.as_slice(), self.inner.as_bytes()].concat()
    }
}

// Allow attempts to convert String -> EncodedString
impl TryFrom<String> for EncodedString {
    type Error = TryFromIntError;
    fn try_from(s: String) -> Result<EncodedString, Self::Error> {
        let str_len = i32::try_from(s.len())?;

        Ok(Self {
            inner: s,
            length: VarInt::from(str_len),
        })
    }
}
