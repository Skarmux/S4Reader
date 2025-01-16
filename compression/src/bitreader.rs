#![allow(dead_code, unused)]
use byteorder::ReadBytesExt;
use std::io::{Result,Read};

/// Wrapper around anything implementing the std::io::Read trait.
/// Allowing to read a custom amount of bits.
#[derive(Debug, PartialEq)]
pub struct BitReader<R> {
    inner: R,
    /// cached bits
    cached_bits_count: u8,
    /// store for loaded bits
    cache: u16,
}

impl<R: Read> BitReader<R> {
    pub fn new(inner: R) -> BitReader<R> {
        BitReader {
            inner,
            cached_bits_count: 0,
            cache: 0,
        }
    }

    /// Bit offset from the byte at the starting position of inner.
    pub fn with_offset(offset: u8, inner: R) -> Result<BitReader<R>> {
        debug_assert!(offset <= 8, "offset bigger than 8 bits");

        let mut reader = BitReader::new(inner);

        reader.read_u8(8);
        reader.cached_bits_count = 8 - offset;
        reader.cache = reader.cache << 8 + offset;

        Ok(reader)
    }

    /// Read {count} number of bits from the next byte.
    /// Returns bits within byte with leading zeroes.
    /// Errors when there is no byte left
    pub fn read_u8(&mut self, count: u8) -> Result<u8> {
        debug_assert!(count <= 8, "can't read more than 8 bits from byte");
        debug_assert!(count != 0, "value for count can't be zero");
        // fill up cache
        if self.cached_bits_count < count {
            let next_byte = self.inner.read_u8()?;

            self.cache |= (next_byte as u16)
                .checked_shl((8 - self.cached_bits_count) as u32)
                .unwrap();

            self.cached_bits_count += 8;
        }
        self.cache = self.cache.rotate_left(count as u32);
        let result = self.cache as u8;
        self.cache &= 0xFF00;
        self.cached_bits_count -= count;

        Ok(result)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::io::{self, Cursor};

    #[test]
    fn read_u8() {
        let mut input: [u8; 2] = [0b1110_0000, 0b0101_0101];
        let mut bit_reader = BitReader::new(&input[..]);

        let byte = bit_reader.read_u8(3);
        assert!(byte.is_ok(), "Error while reading byte!");
        assert_eq!(byte.unwrap(), 0b0000_0111);
    }

    #[test]
    fn read_u8_err_unexpected_eof() {
        let mut input: [u8; 1] = [0b1110_0000];
        let mut bit_reader = BitReader::new(&input[..]);

        assert!(bit_reader.read_u8(8).is_ok());
        assert_eq!(
            bit_reader.read_u8(1).map_err(|e| e.kind()),
            Err(io::ErrorKind::UnexpectedEof)
        );
    }

    #[test]
    fn offset() {
        let mut input: [u8; 2] = [0b1110_0000, 0b0101_1111];
        let mut bit_reader = BitReader::with_offset(4, &input[..]).unwrap();

        let byte = bit_reader.read_u8(8);
        assert!(byte.is_ok(), "Error while reading byte!");
        assert_eq!(byte.unwrap(), 0b0000_0101);
    }

}
