use crate::bitreader::BitReader;
use std::io::prelude::*;

// https://www.rfc-editor.org/rfc/rfc1951
// https://www.rfc-editor.org/rfc/rfc1952

#[derive(Clone, Copy)]
struct SymbolTable {
    alphabet: [(u16, u32); 274], // contains values between 0 and 273 and their usage counts
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        let mut table = Self {
            alphabet: [(0, 0); 274],
        };

        let mut alphabet_iter = table.alphabet.iter_mut();

        const PREDEFINED: [u16; 20] = [
            256, 257, 258, 259, 260, 261, 262, 263, 264, 265, 266, 267, 268, 269, 270, 271, 0, 32,
            48, 255,
        ];
        for (predefined, (symbol, _count)) in PREDEFINED.iter().zip(&mut alphabet_iter) {
            *symbol = *predefined;
        }

        for (i, (symbol, _count)) in (1..274)
            .filter(|x| !&PREDEFINED.contains(x))
            .zip(&mut alphabet_iter)
        {
            *symbol = i;
        }

        table
    }

    /// Get symbol at index and increment its usage count by 1
    pub fn symbol_at(&mut self, index: usize) -> u16 {
        self.alphabet[index].1 += 1;
        self.alphabet[index].0
    }

    /// Replace alphabet with consecutive numbers sorted by count and symbol
    pub fn rebuild_alphabet(&mut self) {
        self.alphabet
            .sort_by(|(symbol_a, count_a), (symbol_b, count_b)| {
                match count_a.cmp(&count_b).reverse() {
                    std::cmp::Ordering::Equal => symbol_a.cmp(&symbol_b).reverse(),
                    ord => ord,
                }
            });

        for (_symbol, count) in self.alphabet.iter_mut() {
            *count /= 2;
        }
    }
}

pub fn decompress(reader: &mut impl Read) -> Result<Vec<u8>, std::io::Error> {
    let mut bit_reader = BitReader::new(reader);

    let mut decrypt = Vec::<u8>::new();

    let mut symbol_table = SymbolTable::new();

    const HUFFMAN_TABLE: [(u8, u16); 16] = [
        (0x2, 0x0),
        (0x3, 0x4),
        (0x3, 0xC),
        (0x4, 0x14),
        (0x4, 0x24),
        (0x4, 0x34),
        (0x4, 0x44),
        (0x4, 0x54),
        (0x4, 0x64),
        (0x4, 0x74),
        (0x4, 0x84),
        (0x4, 0x94),
        (0x4, 0xA4),
        (0x5, 0xB4),
        (0x5, 0xD4),
        (0x5, 0xF4),
    ];

    let mut huffman = HUFFMAN_TABLE;

    while let Ok(code) = bit_reader.read_u8(4) {
        let (length, mut symbol_index) = huffman[code as usize];

        if length > 0 {
            let byte = bit_reader.read_u8(length)?;
            symbol_index += byte as u16;
        }

        assert!(symbol_index < 274, "index out of range of the symbol table");

        // retrieve symbol from alphabet
        let symbol: u16 = symbol_table.symbol_at(symbol_index as usize);

        let mut n_bytes = 4;

        match symbol {
            0..=255 => {
                // symbols within one byte are not compressed
                decrypt.push(symbol as u8);

                continue;
            }
            256..=263 => {
                let diff = symbol - 256;
                n_bytes += diff as usize;
            }
            264..=271 => {
                let length = (symbol - 263) as u8;
                n_bytes += bit_reader.read_u8(length)? as usize;

                const OFFSET: [u16; 8] = [0x8, 0xA, 0xE, 0x16, 0x26, 0x46, 0x86, 0x106];
                let offset = OFFSET[(length - 1) as usize];
                n_bytes += offset as usize;
            }
            272 => {
                // some sort of reset
                symbol_table.rebuild_alphabet();

                // rebuild huffman table
                let mut tmp_length: i8 = 0;
                let mut tmp_base = 0;

                for (length, symbol_index) in huffman.iter_mut() {
                    tmp_length -= 1;
                    // count zeroes
                    loop {
                        tmp_length += 1;
                        if bit_reader.read_u8(1)? == 1 {
                            break;
                        }
                    }

                    *length = tmp_length as u8;
                    *symbol_index = tmp_base;

                    tmp_base += (1 as u16) << *length;
                }

                continue;
            }
            273 => {
                // end of data
                return Ok(decrypt);
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "unexcpected symbol during deflate",
                ));
            }
        }

        let bit_value = bit_reader.read_u8(3)?;
        const LZ_DIST: [(u8, u8); 8] = [
            (1, 0x0),
            (1, 0x1),
            (2, 0x2),
            (3, 0x4),
            (4, 0x8),
            (5, 0x10),
            (6, 0x20),
            (7, 0x40),
        ];
        let (length, base_value) = LZ_DIST[bit_value as usize];

        let bit_value = bit_reader.read_u8(8)?;
        let copy_offset = (bit_value as usize).checked_shl(length as u32).unwrap();

        let bit_value = bit_reader.read_u8(length)?;

        let bitmask = bit_value as usize | copy_offset;

        let current_index = decrypt.len();
        let offset = bitmask + (base_value as usize).checked_shl(9).unwrap();

        let src_pos = current_index - offset;

        for i in src_pos..(src_pos + n_bytes) {
            let prev_byte = decrypt.get(i).expect("index points to existing position");
            decrypt.push(prev_byte.clone());
        }
    }

    Ok(decrypt)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_reading_from_output() {
        let mut output: [u8; 2] = [0; 2];

        let mut writer = crate::bitwriter::BitWriter::new(&mut output[..]);
        writer.write_bits(&[0b1111_0000], 4).unwrap();
        writer.flush().unwrap();

        let mut reader = BitReader::new(&output[..]);

        assert_eq! {reader.read_u8(8).unwrap(), 0b1111_0000};
    }
}
