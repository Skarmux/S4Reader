use crate::bitreader::BitReader;
use std::io::prelude::*;

// https://www.rfc-editor.org/rfc/rfc1951
// https://www.rfc-editor.org/rfc/rfc1952

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
        assert!(index < 274);
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

pub fn decompress<T: Read>(reader: &mut T, output: &mut Vec<u8>) -> Result<(), std::io::Error> {
    let mut bit_reader = BitReader::new(reader);

    const HUFFMAN_TABLE: [[u32; 16]; 2] = [
        [2, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5],
        [
            0x00, 0x04, 0x0C, 0x14, 0x24, 0x34, 0x44, 0x54, 0x64, 0x74, 0x84, 0x94, 0xA4, 0xB4,
            0xD4, 0xF4,
        ],
    ];

    let mut symbol_table = SymbolTable::new();

    let mut output_pos: usize = 0;

    while let Ok(code) = bit_reader.read_u8(4) {
        // TODO: Passiert das überhaupt?
        if output_pos >= output.len() {
            // reached end of output
            break;
        }

        assert!(code < 128, "out of sync!");

        // read code item
        let bits = HUFFMAN_TABLE[0][code as usize];
        let mut symbol_table_index = HUFFMAN_TABLE[1][code as usize];

        if bits > 0 {
            // read more bits
            symbol_table_index += bit_reader.read_u8(bits as u8).unwrap() as u32;

            assert!(symbol_table_index < 274, "out of sync!");
        }

        let symbol: u16 = symbol_table.symbol_at(symbol_table_index as usize);

        let mut length = 4;

        // execute code word
        match symbol {
            0..=255 => {
                // not compressed
                output[output_pos] = symbol as u8;
                output_pos += 1;
                continue;
            }
            256..=264 => {
                length += symbol - 256;
            }
            265..=271 => {
                let index = (symbol - 264) as usize;
                const LZ_LENGTH_TABLE: [[u16; 8]; 2] = [
                    [0x001, 0x002, 0x003, 0x004, 0x005, 0x006, 0x007, 0x008],
                    [0x008, 0x00A, 0x00E, 0x016, 0x026, 0x046, 0x086, 0x106],
                ];
                let bits = LZ_LENGTH_TABLE[0][(index << 1) as usize];
                let offset = LZ_LENGTH_TABLE[0][((index << 1) + 1) as usize];
                length += offset + (bit_reader.read_u8(bits as u8).unwrap() as u16);
            }
            272 => {
                // some sort of reset
                symbol_table.rebuild_alphabet().unwrap();

                // divide all counts by two
                // symbol_table.count.iter_mut().map(|count| *count /= 2);

                // parse new huffman table
                let mut bits_count = 0;
                let mut offset = 0;
                let mut length = 0;

                for _ in 0..16 {
                    while let Ok(bit) = bit_reader.read_u8(1) {
                        if bit > 0 {
                            break;
                        } else {
                            bits_count += 1;
                        }
                    }
                    symbol_table.indices[length] = bits_count;
                    symbol_table.indices[length + 1] = offset;
                    length += 2;
                    offset += 1 << bits_count;
                }
                continue;
            }
            273 => {
                // end of data
                break;
            }
            _ => {
                panic!("unhandled symbol: {}", symbol);
            }
        }

        let lz_index = bit_reader.read_u8(3).unwrap() as usize;
        const LZ_DISTANCE_TABLE: [[u8; 8]; 2] = [
            [0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
            [0x00, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40],
        ];
        let bits = LZ_DISTANCE_TABLE[0][lz_index] + 1;
        let offset = LZ_DISTANCE_TABLE[1][lz_index] as usize;

        let code = bit_reader.read_u8(8).unwrap();
        let copy_offset = code.checked_shl(bits as u32).unwrap();

        let code = bit_reader.read_u8(bits).unwrap();

        assert!(
            (output_pos + length as usize) < output.len(),
            "output buffer not large enough!"
        );

        // source position in buffer (LZ Jumping)
        let bit_and = (code | copy_offset) as usize; // WAS IST DAS?
        let mut src_pos = output_pos - bit_and + offset.checked_shl(9).unwrap();

        // we need to use single-byte-copy the data case, the src and dest can be overlapped
        for _ in (0..=length).rev() {
            output[output_pos] = output[src_pos];
            output_pos += 1;
            src_pos += 1;
        }
    }

    Ok(())
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
