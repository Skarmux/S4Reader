#![allow(dead_code, unused)]
use byteorder::ReadBytesExt;
use std::io::{Read, Result, Error};

pub struct BitReader<R> {
    inner: R,
    cached_bits_count: u8, // cached bits
    cache: u16,            // store for loaded bits
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
    pub fn with_offset(offset: u8, inner: R) -> BitReader<R> {
        assert!(offset < 8, "offset out of range!");

        let mut reader = BitReader::new(inner);

        reader.read_u8(8);
        reader.cached_bits_count = 8 - offset;
        reader.cache = reader.cache << 8 + offset;

        reader
    }

    pub fn read_n_bits(&mut self, n: usize) -> Result<&[u8]> {
        todo!("Not implemented")
    }

    pub fn read_u8(&mut self, count: u8) -> Result<u8> {
        assert!(count <= 8, "Number of bits can't be more than 8!");
        assert!(count > 0, "Value for count can't be zero!");

        // fill up cache
        if self.cached_bits_count < count {
            let next_byte = self.inner.read_u8()?;

            self.cache |= (next_byte as u16).checked_shl((8 - self.cached_bits_count) as u32).unwrap();

            self.cached_bits_count += 8;
        }

        /*
         *     cached bits > 8
         *            |
         * 1111_1111 1100_0000
         */

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
    use std::io::Cursor;

    #[test]
    fn test_read_u8() {
        let mut input: [u8; 2] = [0b1110_0000, 0b0101_0101];
        let mut bit_reader = BitReader::new(&input[..]);

        let byte = bit_reader.read_u8(3);
        assert!(byte.is_ok(), "Error while reading byte!");
        assert_eq!(byte.unwrap(), 0b0000_0111);
    }

    #[test]
    #[should_panic(expected = "Number of bits can't be more than 8!")]
    fn test_read_u8_larger_than_byte() {
        let mut input: [u8; 2] = [0b1110_0000, 0b0101_0101];
        let mut bit_reader = BitReader::new(&input[..]);

        bit_reader.read_u8(9);
    }

    #[test]
    #[should_panic(expected = "Can't read over end of input!")]
    fn test_read_u8_panic_out_of_range() {
        let mut input: [u8; 1] = [0b1110_0000];
        let mut bit_reader = BitReader::new(&input[..]);

        bit_reader.read_u8(8);
        bit_reader.read_u8(1);
    }

    #[test]
    fn test_with_offset() {
        let mut input: [u8; 2] = [0b1110_0000, 0b0101_1111];
        let mut bit_reader = BitReader::with_offset(4, &input[..]);

        let byte = bit_reader.read_u8(8);
        assert!(byte.is_ok(), "Error while reading byte!");
        assert_eq!(byte.unwrap(), 0b0000_0101);
    }

    #[test]
    fn test_with_offset_larger_than_byte() {
        let mut input: [u8; 2] = [0b1110_0000, 0b0101_1111];
        let mut bit_reader = BitReader::with_offset(12, &input[..]);

        let byte = bit_reader.read_u8(4);
        assert!(byte.is_ok(), "Error while reading byte!");
        assert_eq!(byte.unwrap(), 0b0000_1111);
    }
}
