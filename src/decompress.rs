mod bitreader;
mod bitwriter;

use std::collections::VecDeque;
use std::io::{prelude::*, BufReader, Cursor, Error, SeekFrom};
use std::ops::{Index, IndexMut};
use std::slice;

use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
use byteorder::{ByteOrder, LittleEndian};

use bitreader::BitReader;
use bitwriter::BitWriter;

// https://www.rfc-editor.org/rfc/rfc1951
// https://www.rfc-editor.org/rfc/rfc1952

#[derive(Default, Clone, Copy)]
struct Item<T: Copy, U: Copy> {
    length: T,
    value: U,
}

static LZ_DISTANCE_TABLE: [Item<u8, u8>; 8] = [
    Item {
        length: 0,
        value: 0x00,
    },
    Item {
        length: 0,
        value: 0x01,
    },
    Item {
        length: 1,
        value: 0x02,
    },
    Item {
        length: 2,
        value: 0x04,
    },
    Item {
        length: 3,
        value: 0x08,
    },
    Item {
        length: 4,
        value: 0x10,
    },
    Item {
        length: 5,
        value: 0x20,
    },
    Item {
        length: 6,
        value: 0x40,
    },
];

static LZ_LENGTH_TABLE: [Item<u8, u16>; 8] = [
    Item {
        length: 1,
        value: 0x008,
    },
    Item {
        length: 2,
        value: 0x00A,
    },
    Item {
        length: 3,
        value: 0x00E,
    },
    Item {
        length: 4,
        value: 0x016,
    },
    Item {
        length: 5,
        value: 0x026,
    },
    Item {
        length: 6,
        value: 0x046,
    },
    Item {
        length: 7,
        value: 0x086,
    },
    Item {
        length: 8,
        value: 0x106,
    },
];

static HUFFMAN_TABLE: [Item<u32, u16>; 16] = [
    Item {
        length: 2,
        value: 0x00,
    },
    Item {
        length: 3,
        value: 0x04,
    },
    Item {
        length: 3,
        value: 0x0C,
    },
    Item {
        length: 4,
        value: 0x14,
    },
    Item {
        length: 4,
        value: 0x24,
    },
    Item {
        length: 4,
        value: 0x34,
    },
    Item {
        length: 4,
        value: 0x44,
    },
    Item {
        length: 4,
        value: 0x54,
    },
    Item {
        length: 4,
        value: 0x64,
    },
    Item {
        length: 4,
        value: 0x74,
    },
    Item {
        length: 4,
        value: 0x84,
    },
    Item {
        length: 4,
        value: 0x94,
    },
    Item {
        length: 4,
        value: 0xA4,
    },
    Item {
        length: 5,
        value: 0xB4,
    },
    Item {
        length: 5,
        value: 0xD4,
    },
    Item {
        length: 5,
        value: 0xF4,
    },
];

#[derive(Default, Clone, Copy)]
struct CodeItem {
    index: u32,
    value: u32,
    count: u32,
}

#[derive(Clone, Copy)]
struct SymbolTable {
    indices: [u16; 274],
    alphabet: [u16; 274],
    count: [u16; 274],
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        //let mut table: [CodeItem;274] = [Default::default();274];

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
    pub fn reset_indices(&mut self) -> Result<(), ()> {
        self.indices
            .iter_mut()
            .enumerate()
            .map(|(i, x)| *x = (i as u16));
        Ok(())
    }

    /// Replace alphabet with consecutive numbers sorted by count.
    pub fn rebuild_alphabet(&mut self) -> Result<(), ()> {
        self.alphabet = [0; 274];
        self.alphabet
            .iter_mut()
            .enumerate()
            .map(|(i, n)| *n = i as u16);

        self.alphabet.sort_by(|&x1, &x2| {
            (self.count[x2 as usize].checked_shl(16).unwrap() + x2)
                .cmp(&(self.count[x1 as usize].checked_shl(16).unwrap() + x1))
        });
        Ok(())
    }
}

pub fn decompress<T: Read>(reader: &mut T, output: &mut Vec<u8>) -> Result<(), std::io::Error> {
    let mut bit_reader = BitReader::new(reader);

    let mut huffman_table = HUFFMAN_TABLE.clone(); // symbol index table

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
        let mut code_item = huffman_table[code as usize];
        let bits = code_item.length;
        let mut symbol_table_index = code_item.value;

        if bits > 0 {
            // read more bits
            symbol_table_index += bit_reader.read_u8(bits as u8).unwrap() as u16;

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
                let bits = LZ_LENGTH_TABLE[(index << 1) as usize].length;
                let offset = LZ_LENGTH_TABLE[((index << 1) + 1) as usize].value;
                length += offset + (bit_reader.read_u8(bits as u8).unwrap() as u16);
            }
            272 => {
                // some sort of reset
                symbol_table.rebuild_alphabet();

                // divide all counts by two
                symbol_table.count.iter_mut().map(|count| *count /= 2);

                // parse new huffman table
                let mut bits_count = 0;
                let mut offset = 0;
                let mut length = 0;

                for i in 0..16 {
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
        let lz_item = LZ_DISTANCE_TABLE[lz_index];
        let bits = lz_item.length + 1;
        let offset = lz_item.value as usize;

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
        for i in (0..=length).rev() {
            output[output_pos] = output[src_pos];
            output_pos += 1;
            src_pos += 1;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::io::{BufWriter, Cursor};

    use super::*;

    struct GeneralInformation {
        game_type: u32,
        player_count: u32,
        start_resources: u32,
        map_size: u32,
    }

    #[test]
    fn test_reading_from_output() {
        let mut output: [u8; 2] = [0; 2];

        let mut writer = BitWriter::new(&mut output[..]);
        writer.write_bits(&[0b1111_0000], 4);
        writer.flush();

        let mut reader = BitReader::new(&output[..]);

        assert_eq! {reader.read_u8(8).unwrap(), 0b1111_0000};
    }

    #[test]
    fn test_decompress() {
        // Chunk @ 1354, size: 24; Type=1; checksum=47560, unknown1=0, unknown2=0
        let decoded: [u8; 24] = [
            0b0000_0001,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0100,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0010,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b1000_0000,
            0b0000_0010,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
            0b0100_0000,
            0b0000_0000,
            0b0000_0000,
            0b0000_0000,
        ];

        // gameType: singlePlayer; playerCount: 4; startResources: medium; size: [640 x 640]; unk5: 0; unk6: 64;
        let expect = GeneralInformation {
            game_type: 1, // singlePlayer
            player_count: 4,
            start_resources: 2, // medium
            map_size: 640,      // 640x640
        };

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
