use crate::bitreader::BitReader;
use crate::bitwriter::BitWriter;
use std::io::prelude::*;

// https://www.rfc-editor.org/rfc/rfc1951
// https://www.rfc-editor.org/rfc/rfc1952

// Huffman: Frequency of bytes
// LZ: pattern repetition

#[derive(Clone, Copy)]
struct SymbolTable {
    indices: [u16; 274],
    alphabet: [u16; 274],
    count: [u16; 274],
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
            table.indices[table.alphabet[i] as usize] = i as u16;
        }

        table
    }

    pub fn symbol_at(&mut self, index: usize) -> u16 {
        debug_assert!(index < 274);
        self.count[index] += 1;
        self.alphabet[index]
    }

    /// Restore ascending ordering of indices array.
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
        // self.alphabet
        //     .iter_mut()
        //     .enumerate()
        //     .map(|(i, n)| *n = i as u16);

        self.alphabet.sort_by(|&x1, &x2| {
            (self.count[x2 as usize].checked_shl(16).unwrap() + x2)
                .cmp(&(self.count[x1 as usize].checked_shl(16).unwrap() + x1))
        });
        Ok(())
    }
}

pub fn decompress(reader: &mut impl Read) -> Result<Vec<u8>, std::io::Error> {
    let mut bit_reader = BitReader::new(reader);

    let mut decrypt = Vec::<u8>::new();

    let mut symbol_table = SymbolTable::new();

    while let Ok(code) = bit_reader.read_u8(4) {
        const HUFFMAN_TABLE_LENGTH: [u16; 16] = [
            0x2, 0x3, 0x3, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x4, 0x5, 0x5, 0x5,
        ];

        const HUFFMAN_TABLE_VALUE: [u8; 16] = [
            0x0, 0x4, 0xC, 0x14, 0x24, 0x34, 0x44, 0x54, 0x64, 0x74, 0x84, 0x94, 0xA4, 0xB4, 0xD4,
            0xF4,
        ];

        let n_bits = HUFFMAN_TABLE_LENGTH[code as usize]; // number of bits to read

        let byte = bit_reader.read_u8(n_bits as u8)?;

        let mut symbol_table_index = HUFFMAN_TABLE_VALUE[code as usize] as u16;

        symbol_table_index += byte as u16;

        assert!(
            symbol_table_index < 274,
            "Index out of range of the symbol table!"
        );

        // retrieve symbol from alphabet
        let symbol: u16 = symbol_table.symbol_at(symbol_table_index as usize);
        assert!(symbol < 274, "out of sync!");

        let mut n_bytes: usize = 4;

        // execute code word
        match symbol {
            0..=255 => {
                // symbols within one byte are not compressed
                decrypt.push(symbol as u8);

                continue;
            }
            272 => {
                todo!("Not tested yet!");
                // some sort of reset
                symbol_table.rebuild_alphabet().unwrap();

                // divide all counts by two
                // symbol_table.count.iter_mut().map(|count| *count /= 2);

                // parse new huffman table
                let mut n_bits = 0;
                let mut offset = 0;
                let mut length = 0;

                for _ in 0..16 {
                    while let Ok(bit) = bit_reader.read_u8(1) {
                        if bit > 0 {
                            break;
                        } else {
                            n_bits += 1;
                        }
                    }
                    symbol_table.indices[length] = n_bits;
                    symbol_table.indices[length + 1] = offset;
                    length += 2;
                    offset += 1 << n_bits;
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

        let mut bit_value = bit_reader.read_u8(3)? as usize;

        const LZ_LENGHT: [u8; 8] = [1, 1, 2, 3, 4, 5, 6, 7];
        let length = LZ_LENGHT[bit_value as usize];

        const LZ_DISTANCE: [usize; 8] = [0x0, 0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40];
        let base_value = LZ_DISTANCE[bit_value as usize];

        bit_value = bit_reader.read_u8(8)? as usize;

        let copy_offset = bit_value.checked_shl(length as u32).unwrap();

        bit_value = bit_reader.read_u8(length)? as usize;

        let bitmask = (bit_value | copy_offset) as usize;

        let offset = bitmask + base_value.checked_shl(9).unwrap();

        let current_index = decrypt.len() - 1;

        let src_pos = current_index - offset;

        // we need to use single-byte-copy the data case, the src and dest can be overlapped
        for i in src_pos..(src_pos + n_bytes) {
            let prev_byte = decrypt.get(i).unwrap();
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
