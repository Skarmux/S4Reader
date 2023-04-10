use crate::bitreader::BitReader;
use std::io::prelude::*;

// https://www.rfc-editor.org/rfc/rfc1951
// https://www.rfc-editor.org/rfc/rfc1952

#[derive(Clone, Copy)]
struct SymbolTable {
    indices: [u32; 274],
    alphabet: [u16; 274],
    count: [u32; 274],
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        let mut table = Self {
            indices: [0; 274],
            alphabet: [0; 274],
            count: [0; 274],
        };

        for i in 0..274 {
            table.alphabet[i] = match i {
                0..=15 => i as u16 + 0x100,
                16 => 0x00,
                17 => 0x20,
                18 => 0x30,
                19 => 0xFF,
                _ => {
                    let mut value = 1;
                    for j in 1..274 {
                        if !table.alphabet.contains(&j) {
                            value = j;
                            break;
                        }
                    }
                    value
                }
            };
            table.indices[table.alphabet[i] as usize] = i as u32;
        }

        table
    }

    pub fn symbol_at(&mut self, index: usize) -> u16 {
        let symbol = self.alphabet[index];
        self.count[symbol as usize] += 1;
        symbol
    }

    // Restore ascending ordering of indices array.
    // pub fn reset_indices(&mut self) -> Result<(), ()> {
    //     self.indices
    //         .iter_mut()
    //         .enumerate()
    //         .map(|(i, x)| *x = (i as u16));
    //     Ok(())
    // }

    /// Replace alphabet with consecutive numbers sorted by count.
    pub fn rebuild_alphabet(&mut self) -> Result<(), ()> {
        self.alphabet = [0; 274];

        // index array
        for (i, symbol) in self.alphabet.iter_mut().enumerate() {
            *symbol = i as u16;
        }

        // sort index by quantities
        self.alphabet.sort_by(|x1, x2| {
            let a = self.count[*x2 as usize].checked_shl(16).unwrap() + *x2 as u32;
            let b = self.count[*x1 as usize].checked_shl(16).unwrap() + *x1 as u32;
            a.cmp(&b)
        });

        // divide all counts by two
        for count in self.count.iter_mut() {
            *count = *count / 2;
        }

        // create new index lookup
        self.indices = [0; 274];
        for (i, symbol) in self.alphabet.iter().enumerate() {
            self.indices[*symbol as usize] = i as u32;
        }

        Ok(())
    }
}

pub fn decompress(reader: &mut impl Read) -> Result<Vec<u8>, std::io::Error> {
    let mut bit_reader = BitReader::new(reader);

    let mut decrypt = Vec::<u8>::new();

    let mut symbol_table = SymbolTable::new();

    const HUFFMAN_TABLE: [(u8,u16); 16] = [
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

        assert!(
            symbol_index < 274,
            "Index out of range of the symbol table!"
        );

        // retrieve symbol from alphabet
        let symbol: u16 = symbol_table.symbol_at(symbol_index as usize);
        assert!(symbol < 274, "out of sync!");

        let mut n_bytes = 4;

        // execute code word
        match symbol {
            0..=255 => {
                // symbols within one byte are not compressed
                decrypt.push(symbol as u8);

                continue;
            }
            272 => {
                // some sort of reset
                symbol_table.rebuild_alphabet().unwrap();

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

                    debug_assert!(*length <= 8, "Length is bigger than anticipated!");

                    tmp_base += (1 as u16) << *length;
                }

                continue;
            }
            273 => {
                // end of data
                return Ok(decrypt);
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
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unexcpected symbol during deflate!",
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
        let copy_offset = (bit_value as u16).checked_shl(length as u32).unwrap();

        let bit_value = bit_reader.read_u8(length)?;
        let bitmask = bit_value as u16 | copy_offset;

        let current_index = decrypt.len();
        let offset = bitmask + (base_value as u16).checked_shl(9).unwrap();
        debug_assert!(current_index >= offset as usize, "Offset out of range");
        let src_pos = current_index - offset as usize;

        // we need to use single-byte-copy the data case, the src and dest can be overlapped
        for i in src_pos..(src_pos + n_bytes) {
            let prev_byte = decrypt
                .get(i)
                .expect("Index points to existing position in decrypt output.");
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

    #[test]
    fn test_decompress() {
        // Chunk @ 1354, size: 24; Type=1; checksum=47560, unknown1=0, unknown2=0
        // let decoded: [u8; 24] = [
        //     0b0000_0001,
        //     0b0000_0000,
        //     0b0000_0000,
        //     0b0000_0000,
        //     0b0000_0100,
        //     0b0000_0000,
        //     0b0000_0000,
        //     0b0000_0000,
        //     0b0000_0010,
        //     0b0000_0000,
        //     0b0000_0000,
        //     0b0000_0000,
        //     0b1000_0000,
        //     0b0000_0010,
        //     0b0000_0000,
        //     0b0000_0000,
        //     0b0000_0000,
        //     0b0000_0000,
        //     0b0000_0000,
        //     0b0000_0000,
        //     0b0100_0000,
        //     0b0000_0000,
        //     0b0000_0000,
        //     0b0000_0000,
        // ];

        // gameType: singlePlayer; playerCount: 4; startResources: medium; size: [640 x 640]; unk5: 0; unk6: 64;
        // let expect = GeneralInformation {
        //     game_type: 1, // singlePlayer
        //     player_count: 4,
        //     start_resources: 2, // medium
        //     map_size: 640,      // 640x640
        // };

        /*
        let slice_of_u8 = &[0b1000_1111];
        let mut reader = BitReader::new(slice_of_u8);

        // You obviously should use try! or some other error handling mechanism here
        let a_single_bit = reader.read_u8(1).unwrap(); // 1
        let more_bits = reader.read_u8(3).unwrap(); // 0
        let last_bits_of_byte = reader.read_u8(4).unwrap(); // 0b1111
        */

        // let mut decrypt: Vec<u8> = Vec::<u8>::new();

        //let mut bit_reader = BitReader::new( &encoded[..] );

        //let mut reader = BufReader::new( &crypt[..] );

        // let mut writer = BufWriter::new( decrypt.as_mut_slice() );

        // decompress(&mut reader, &mut writer );
    }
}
