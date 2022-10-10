use std::io::prelude::*;
use std::io::{
    self, Read, Seek, SeekFrom
};

use byteorder::ReadBytesExt;

pub struct BitReader<R> {
    inner: R,
    cached_bits_count: u8, // cached bits
    cache: u16, // store for loaded bits
}

impl<R: Read> BitReader<R> {

    pub fn new(inner: R ) -> BitReader<R> {
        BitReader { inner, cached_bits_count: 0, cache: 0 }
    }

    /// Bit offset from the byte at the starting position of inner.
    pub fn with_offset(offset: u8, inner: R) -> BitReader<R> {
        BitReader { 
            inner, 
            cached_bits_count: 8 - offset, 
            cache: inner.read_u8().unwrap() << offset
        }
    }

    pub fn read_u8(&mut self, count: u8) -> Option<u8> {
        assert!(count <= 8);

        // fill up cache
        if self.cached_bits_count < count {
            let next_byte = self.inner.read_u8().unwrap() as u16;
            self.cache |= next_byte << (8 - self.cached_bits_count);
            self.cached_bits_count += 8;
        }

        let result = self.cache >> (16 - count);
        
        self.cache = self.cache << count;
        self.cached_bits_count -= count;

        Some(result as u8)
    }

}

#[cfg(test)]
mod test {

    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_u8() {

        let mut input: [u8;2] = [0b1110_0000, 0b0101_0101];
        let mut bit_reader = BitReader::new(&input[..]);

        let byte = bit_reader.read_u8(3);
        assert!(byte.is_some());
        assert_eq!(byte.unwrap(), 0b0000_0111);
    }

}
