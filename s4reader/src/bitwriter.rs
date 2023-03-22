use std::io::{self, Seek, SeekFrom, Write};
use std::io::{prelude::*, IntoInnerError};

pub struct BitWriter<W> {
    inner: W,
    cached_bits_count: u8,
    cache: u8,
}

impl<W: Write> BitWriter<W> {
    pub fn new(inner: W) -> BitWriter<W> {
        BitWriter {
            inner,
            cached_bits_count: 0,
            cache: 0,
        }
    }

    /// Bit offset from the byte at the starting position of inner.
    pub fn with_offset(offset: u8, inner: W) -> BitWriter<W> {
        BitWriter {
            inner,
            cached_bits_count: offset,
            cache: 0,
        }
    }

    fn write_u8(&mut self, byte: u8, count: u8) -> io::Result<()> {
        assert!(count <= 8, "count is larger than fits inside 8 bits!");

        // fill cache
        self.cache |= byte >> self.cached_bits_count;

        // update count
        self.cached_bits_count += count;

        if self.cached_bits_count >= 8 {
            // flush cache
            self.inner.write(&[self.cache]);

            // update with number of remaining bits
            self.cached_bits_count %= 8;

            if self.cached_bits_count > 0 {
                // new cache with remaining bits
                self.cache = byte << (8 - self.cached_bits_count);
            }
        }

        Ok(())
    }

    pub fn write_bits(&mut self, buf: &[u8], mut count: u64) -> io::Result<()> {
        assert!(
            count <= buf.len() as u64 * 8,
            "bit count exceeds input buffer capacity!"
        );

        // write all filled bytes
        for byte in &buf[..buf.len() - 1] {
            self.write_u8(*byte, 8);
            count -= 8;
        }

        // write trailing
        self.write_u8(buf[buf.len() - 1], count as u8);

        Ok(())
    }

    pub fn into_inner(mut self) -> Result<W, ()> {
        self.flush();
        Ok(self.inner)
    }
}

impl<W: Write> Write for BitWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        buf.iter().for_each(|byte| self.write_u8(*byte, 8).unwrap());
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.write(&[self.cache]);
        self.cached_bits_count = 0;
        Ok(())
    }
}

// impl<W: Write + Seek> Seek for BitWriter<W> {
//     fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
//         self.flush();
//         match self.inner.seek(pos) {
//             Ok(pos) => Ok(pos),
//             Err(err) => Err(err)
//         }
//     }
//     fn stream_position(&mut self) -> io::Result<u64> {
//         match self.inner.stream_position() {
//             Ok(pos) => Ok(pos + self.cached_bits_count as u64),
//             Err(err) => Err(err)
//         }
//     }
// }

#[cfg(test)]
mod test {

    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_u8() {
        let mut output: [u8; 2] = [0; 2];
        let mut output_buf = Cursor::new(&mut output[..]);
        let mut bit_writer = BitWriter::new(output_buf);

        bit_writer.write_u8(0b1111_0000, 4);
        assert_eq!(bit_writer.cached_bits_count, 4, "4 bits in cache");
        assert_eq!(
            bit_writer.cache & 0b1111_0000,
            0b1111_0000,
            "4 bits are added to cache"
        );

        bit_writer.write_u8(0b1110_0000, 3);
        assert_eq!(bit_writer.cached_bits_count, 7, "7 bits in cache");
        assert_eq!(
            bit_writer.cache & 0b1111_1110,
            0b1111_1110,
            "3 bits are added to cache"
        );

        bit_writer.write_u8(0b1000_0000, 1);
        assert_eq!(bit_writer.cached_bits_count, 0, "0 bits in cache");

        assert_eq!(bit_writer.inner.into_inner(), [0b1111_1111, 0b0000_0000]);
    }

    #[test]
    fn test_write_bits() {
        let mut output: [u8; 2] = [0; 2];
        let mut output_buf = Cursor::new(&mut output[..]);
        let mut bit_writer = BitWriter::new(output_buf);

        bit_writer.write_bits(&[0b0000_1111, 0b1110_0000], 10);
        assert_eq!(bit_writer.cached_bits_count, 2, "2 bits in cache");
        assert_eq!(
            bit_writer.cache & 0b1100_0000,
            0b1100_0000,
            "2 bits are cached"
        );

        assert_eq!(bit_writer.inner.into_inner(), [0b0000_1111, 0b0000_0000]);
    }

    #[test]
    #[should_panic]
    fn test_write_bits_output_overflow_panic() {
        let mut output: [u8; 1] = [0; 1];
        let mut output_buf = Cursor::new(&mut output[..]);
        let mut bit_writer = BitWriter::new(output_buf);

        // TODO: Trailing resides in cache without panic although output buffer is full.

        bit_writer.write_bits(&[0b0000_1111, 0b1110_0000], 9);
    }
}
