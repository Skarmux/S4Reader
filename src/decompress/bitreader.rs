use std::io::prelude::*;
use std::io::{
    self, Read, Seek, SeekFrom
};

pub struct BitReader<R> {
    inner: R,
    cached_bits_count: u8, // number of bits in reserve
    cache: u32, // store for loaded bits
}

impl<R: Read + Seek> BitReader<R> {

    pub fn new(inner: R ) -> BitReader<R> {
        BitReader { inner, cached_bits_count: 0, cache: 0 }
    }

    /// load 1 byte
    fn use_byte(&mut self) -> bool {
        let mut buf: [u8;1] = [0];
        match self.inner.read( &mut buf ) {
            Ok(n) => {
                assert!(n == 1);
                self.cache |= u32::from(buf[0]) << self.cached_bits_count;
                self.cached_bits_count += 8;
                true
            }
            Err(_) => false,
        }
    }

    /// load bits according to amount needed
    fn need(&mut self, n: u8) -> bool {
        if self.cached_bits_count < n {
            // attempt to load a byte
            if !self.use_byte() {
                return false;
            }
            if n > 8 && self.cached_bits_count < n {
                assert!(n <= 16);
                // load another byte
                if !self.use_byte() {
                    return false;
                }
            }
        }
        true
    }

    fn take16(&mut self, n: u8) -> Option<u16> {
        if self.need(n) {
            self.cached_bits_count -= n;
            let v = self.cache & ((1 << n) - 1);
            self.cache >>= n;
            Some(v as u16)
        } else {
            None
        }
    }

    pub fn take(&mut self, n: u8) -> Option<u8> {
        assert!(n <= 8);
        self.take16(n).map(|v: u16| v as u8)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        match self.inner.stream_position() {
            Ok(pos) => Ok(pos + (8 - self.cached_bits_count) as u64),
            Err(e) => Err(e)
        }
    }

    fn seek_bits(&mut self, bit_offset: i64) -> io::Result<()> {
        let byte_offset = bit_offset / 8;
        let bit_count = bit_offset % 8;
        assert!(bit_count >= 0);

        self.inner.seek( SeekFrom::Current(byte_offset));
        self.cached_bits_count = 0;
        self.need(bit_count as u8);

        Ok(())
    }

    // fn fill(&mut self) -> BitState {
    //     while self.state.n + 8 <= 32 && self.use_byte() {}
    //     self.state
    // }

    // fn align_byte(&mut self) {
    //     if self.state.n > 0 {
    //         let n = self.state.n % 8;
    //         self.take(n);
    //     }
    // }

    // fn trailing_bytes(&mut self) -> (u8, [u8; 4]) {
    //     let mut len = 0;
    //     let mut bytes = [0; 4];
    //     self.align_byte();
    //     while self.state.n >= 8 {
    //         bytes[len as usize] = self.state.v as u8;
    //         len += 1;
    //         self.state.n -= 8;
    //         self.state.v >>= 8;
    //     }
    //     (len, bytes)
    // }

}
