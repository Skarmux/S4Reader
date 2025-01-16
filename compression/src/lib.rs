#[allow(warnings)]
mod bindings;
use bindings::exports::s4::compression::decompress::Guest;

mod bitreader;
use crate::bitreader::BitReader;

use std::io::Cursor;

struct SymbolTable {
    /// contains each value between 0 and 273
    symbols: [u16; 274],
    /// the alphabet values index into counter
    usage_counter: [u32; 274],
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        let mut symbols = [0; 274];
        symbols[..20].copy_from_slice(&[
            256, 257, 258, 259, 260, 261, 262, 263, 264, 265, 266, 267, 268, 269, 270, 271, 0, 32,
            48, 255,
        ]);
        // place all remaining values in order
        for (i, symbol) in (20..274).zip((1..32).chain(33..48).chain(49..255).chain(272..274)) {
            symbols[i] = symbol;
        }

        Self {
            symbols,
            usage_counter: [0; 274],
        }
    }

    /// Get symbol at index and increment its usage count by 1
    pub fn symbol_at(&mut self, index: usize) -> u16 {
        let symbol = self.symbols[index];
        self.usage_counter[symbol as usize] += 1;
        symbol
    }

    /// Replace alphabet with consecutive numbers sorted by count first and symbol second
    /// NOTE: `rebuild` gives off the false impression
    pub fn rebuild_alphabet(&mut self) {
        self.symbols.sort_by(|a, b| {
            let count_a = self.usage_counter[*a as usize];
            let count_b = self.usage_counter[*b as usize];
            match count_a.cmp(&count_b).reverse() {
                std::cmp::Ordering::Equal => a.cmp(&b).reverse(),
                ord => ord,
            }
        });
        // preventing overflow of counts
        // NOTE: unlikely with u32?
        for count in self.usage_counter.iter_mut() {
            *count /= 2;
        }
    }
}

struct Component;

impl Guest for Component {
    /// https://www.rfc-editor.org/rfc/rfc1951
    /// https://www.rfc-editor.org/rfc/rfc1952
    fn decompress(input: Vec<u8>) -> Result<Vec<u8>, String> {
        let cursor = Cursor::new(input);
        let mut bit_reader = BitReader::new(cursor);
        let mut output = Vec::<u8>::new();
        let mut symbol_table = SymbolTable::new();

        // (length,symbol_index)
        let mut huffman: [(u8, u16); 16] = [
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

        while let Ok(code_4bit) = bit_reader.read_u8(4) {
            let (length, mut symbol_index) = huffman[code_4bit as usize];

            if length > 0 {
                let byte = bit_reader.read_u8(length).map_err(|err| err.to_string())?;
                symbol_index += byte as u16;
            }

            assert!(symbol_index < 274, "index out of range of the symbol table");

            // retrieve symbol from alphabet
            let symbol: u16 = symbol_table.symbol_at(symbol_index as usize);

            let mut n_bytes = 4;

            match symbol {
                0..=255 => {
                    // symbols within one byte are not compressed
                    output.push(symbol as u8);

                    continue;
                }
                256..=263 => {
                    let diff = symbol - 256;
                    n_bytes += diff as usize;
                }
                264..=271 => {
                    let length = (symbol - 263) as u8;
                    n_bytes += bit_reader.read_u8(length).map_err(|err| err.to_string())? as usize;

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
                            if bit_reader.read_u8(1).map_err(|err| err.to_string())? == 1 {
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
                    return Ok(output);
                }
                _ => {
                    return Err("unexcpected symbol during deflate".to_string());
                }
            }

            let bit_value = bit_reader.read_u8(3).map_err(|err| err.to_string())?;
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

            let bit_value = bit_reader.read_u8(8).map_err(|err| err.to_string())?;
            let copy_offset = (bit_value as usize).checked_shl(length as u32).unwrap();

            let bit_value = bit_reader.read_u8(length).map_err(|err| err.to_string())?;

            let bitmask = bit_value as usize | copy_offset;

            let current_index = output.len();
            let offset = bitmask + (base_value as usize).checked_shl(9).unwrap();

            let src_pos = current_index - offset;

            for i in src_pos..(src_pos + n_bytes) {
                let prev_byte = output.get(i).expect("index points to existing position");
                output.push(prev_byte.clone());
            }
        }

        Ok(output)
    }
}

bindings::export!(Component with_types_in bindings);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_decompress() {
        // chunk from `info` segment of singleplayer/Aeneas.map at offset 1354(d)
        let compressed: Vec<u8> = vec![
            0x30, 0x28, 0x50, 0xA1, 0x99, 0x42, 0x85, 0x0C, 0x4A, 0x14, 0x29, 0x5A, 0x62, 0x50,
            0x10, 0x01, 0x6D, 0x28, 0x50, 0xA7, 0xF4,
        ];
        let bytes = Component::decompress(compressed).unwrap();
        assert_eq!(
            bytes,
            vec![
                0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x80, 0x02,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00
            ],
            "decompression result differs from Settler.ts implementation"
        );
        let gamemode = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(gamemode, 1);
        let player_count = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(player_count, 4);
        let resource_richness = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        assert_eq!(resource_richness, 2);
        let map_size = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        assert_eq!(map_size, 640);
    }
}
