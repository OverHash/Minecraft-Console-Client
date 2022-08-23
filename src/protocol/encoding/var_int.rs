/// Variable-integers (var-int) are a variable width integer which have a fixed range of
/// up to 5 `u8` bytes to represent the `i32` value, but they use the minimal amount of bytes
/// necessary.
///
/// For example, `40i32` can be represented in one byte, while a 32-bit value will require the full 5 bytes.
///
/// The first bit of every byte (most significant bit) represents if there is another byte to be read, while
/// the remaining 7 bits represent the value held at that byte.
///
/// See [`https://wiki.vg/VarInt_And_VarLong`] for more details.
pub struct VarInt {
    inner: [u8; 5],
}

impl VarInt {
    /// Retrieves a reference to the inner encoded var-int, only returning the non-zero bytes.
    pub fn as_slice(&self) -> &[u8] {
        let mut max_index = 0;

        for i in 0..5 {
            // if we have reached the end, stop pointer
            if self.inner[i] & 0b1000_0000 == 0 {
                break;
            }

            // increment max_index if there is a next byte
            max_index += 1;
        }

        &self.inner[0..=max_index]
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        // Algorithm is as follow:
        // shift the last 7 bit of the current value
        // if the value is != 0, make the MSB 1 (otherwise 0)
        // push the value to the stack
        // if the value == 0, break

        let mut var_int = Self { inner: [0; 5] };

        let mut n = value;
        for i in 0..5 {
            // get last 7 bits
            // the casting here is allowed, the first bit of the byte represents if there is going to be another byte
            // hence 0b0111_1111
            #[allow(clippy::cast_sign_loss)]
            let mut temp = (n & 0b0111_1111) as u8;
            // shift to the right by 7 bits
            n = (n >> 7) & (i32::MAX >> 6);
            if n != 0 {
                // signify that there is another byte to go
                temp |= 0b1000_0000;
            }

            // push value to var-int constructed stack
            var_int.inner[i] = temp;

            // check if we have fully encoded the value yet
            if n == 0 {
                break;
            }
        }

        var_int
    }
}

impl From<VarInt> for i32 {
    fn from(var_int: VarInt) -> Self {
        let mut result = 0;

        for i in 0..5 {
            // MSB in the value represents if there is more values to read, so we
            // ignore it here
            let value = i32::from(var_int.inner[i] & 0b0111_1111);
            // shift left by 7 * i bits
            result |= value << (7 * i);

            // check if there is no more values to read (MSB = 0)
            if var_int.inner[i] & 0b1000_0000 == 0 {
                break;
            }
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::VarInt;

    struct VarIntTest {
        value: i32,
        buffer_encoded: [u8; 5],
        encoded: Vec<u8>,
    }

    fn get_test_suite() -> Vec<VarIntTest> {
        vec![
            // single-digit positive values
            VarIntTest {
                value: 0,
                buffer_encoded: [0b0000_0000, 0, 0, 0, 0],
                encoded: vec![0b0000_0000],
            },
            VarIntTest {
                value: 1,
                buffer_encoded: [0b0000_0001, 0, 0, 0, 0],
                encoded: vec![0b0000_0001],
            },
            VarIntTest {
                value: 2,
                buffer_encoded: [0b0000_0010, 0, 0, 0, 0],
                encoded: vec![0b0000_0010],
            },
            VarIntTest {
                value: 127,
                buffer_encoded: [0b0111_1111, 0, 0, 0, 0],
                encoded: vec![0b0111_1111],
            },
            // double-digit positive values
            VarIntTest {
                value: 128,
                buffer_encoded: [0b1000_0000, 0b0000_0001, 0, 0, 0],
                encoded: vec![0b1000_0000, 0b0000_0001],
            },
            VarIntTest {
                value: 255,
                buffer_encoded: [0b1111_1111, 0b0000_0001, 0, 0, 0],
                encoded: vec![0b1111_1111, 0b0000_0001],
            },
            VarIntTest {
                value: 291,
                buffer_encoded: [0b1010_0011, 0b0000_0010, 0, 0, 0],
                encoded: vec![0b1010_0011, 0b0000_0010],
            },
            // triple-byte digits
            VarIntTest {
                value: 25565,
                buffer_encoded: [0b1101_1101, 0b1100_0111, 0b0000_0001, 0, 0],
                encoded: vec![0b1101_1101, 0b1100_0111, 0b0000_0001],
            },
            VarIntTest {
                value: 2_097_151,
                buffer_encoded: [0b1111_1111, 0b1111_1111, 0b0111_1111, 0, 0],
                encoded: vec![0b1111_1111, 0b1111_1111, 0b0111_1111],
            },
            // five-byte digits
            VarIntTest {
                value: i32::MAX,
                buffer_encoded: [
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b0000_0111,
                ],
                encoded: vec![
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b0000_0111,
                ],
            },
            // negative values
            VarIntTest {
                value: -1,
                buffer_encoded: [
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b0000_1111,
                ],
                encoded: vec![
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b1111_1111,
                    0b0000_1111,
                ],
            },
            VarIntTest {
                value: i32::MIN,
                buffer_encoded: [
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b0000_1000,
                ],
                encoded: vec![
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b1000_0000,
                    0b0000_1000,
                ],
            },
        ]
    }

    #[test]
    fn can_encode() {
        for test in get_test_suite() {
            assert_eq!(VarInt::from(test.value).inner, test.buffer_encoded);
        }
    }

    #[test]
    fn can_decode() {
        for test in get_test_suite() {
            assert_eq!(
                i32::from(VarInt {
                    inner: test.buffer_encoded
                }),
                test.value
            );
        }
    }

    #[test]
    fn handles_range() {
        // should be able to go to and from i32 values
        // try for i16::MIN to i16::MAX
        let range = (i32::MIN >> 16)..=(i32::MAX >> 16);
        let decoded = range.clone().map(VarInt::from).map(i32::from);
        assert!(range.eq(decoded));
    }

    #[test]
    fn does_give_slice() {
        for value in get_test_suite() {
            assert_eq!(VarInt::from(value.value).as_slice(), value.encoded);
        }
    }
}
