
mod bitreader;
mod bitwriter;

use std::io::prelude::*;
use std::io::Error;
use std::io::BufReader;
use std::io::SeekFrom;
use std::ops::Index;
use std::slice;

use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
use byteorder::{ByteOrder, LittleEndian};

use bitreader::BitReader;
use bitwriter::BitWriter;

// https://www.rfc-editor.org/rfc/rfc1951
// https://www.rfc-editor.org/rfc/rfc1952

#[derive(Default, Clone, Copy)]
struct Item<T: Copy, U: Copy> { length: T, value: U }

static LZ_DISTANCE_TABLE: [Item<u8,u8>;8] = [
    Item{ length:0, value:0x00 },
    Item{ length:0, value:0x01 },
    Item{ length:1, value:0x02 },
    Item{ length:2, value:0x04 },
    Item{ length:3, value:0x08 },
    Item{ length:4, value:0x10 },
    Item{ length:5, value:0x20 },
    Item{ length:6, value:0x40 }
];

static LZ_LENGTH_TABLE: [Item<u8,u16>;8] = [
    Item{ length:1, value:0x008 },
    Item{ length:2, value:0x00A },
    Item{ length:3, value:0x00E },
    Item{ length:4, value:0x016 },
    Item{ length:5, value:0x026 },
    Item{ length:6, value:0x046 },
    Item{ length:7, value:0x086 },
    Item{ length:8, value:0x106 }
];

static HUFFMAN_TABLE: [Item<u32,u8>;16] = [
    Item{ length:2, value:0x00 },
    Item{ length:3, value:0x04 },
    Item{ length:3, value:0x0C },
    Item{ length:4, value:0x14 },
    Item{ length:4, value:0x24 },
    Item{ length:4, value:0x34 },
    Item{ length:4, value:0x44 },
    Item{ length:4, value:0x54 },
    Item{ length:4, value:0x64 },
    Item{ length:4, value:0x74 },
    Item{ length:4, value:0x84 },
    Item{ length:4, value:0x94 },
    Item{ length:4, value:0xA4 },
    Item{ length:5, value:0xB4 },
    Item{ length:5, value:0xD4 },
    Item{ length:5, value:0xF4 }
];

#[derive(Default, Clone, Copy)]
struct CodeItem {
    index: u32,
    value: u32,
    count: u32
}

#[derive(Clone, Copy)]
struct SymbolTable<const N: usize> {
    indices:  [u16;N],
    alphabet: [u16;N],
    count:    [u16;N]
}

impl<const N: usize> SymbolTable<N> {

    pub fn new() -> SymbolTable<N> {

        let mut table: [CodeItem;274] = [Default::default();274];

        let mut table = Self {
            indices:  [0;N],
            alphabet: [0;N],
            count:    [0;N]
        };

        for i in 0..N {
            table.indices[i] = i as u16;
            table.alphabet[i] = match i {
                0..=15 => i as u16 + 0x100,
                16 => 0x00,
                17 => 0x20,
                18 => 0x30,
                19 => 0xFF,
                _ => (i as u16 - 19)
            }
        }

        table
    }

    pub fn symbol_at(&mut self, index: usize) -> u16 {
        assert!(index < N);
        self.count[index] += 1;
        self.alphabet[index]
    }

    /// Restore ascending ordering of indices array.
    pub fn reset_indices(&mut self) -> Result<(),()> {
        self.indices.iter_mut().enumerate().map(|(i, x)| *x = (i as u16) );
        Ok(())
    }

    /// Replace alphabet with consecutive numbers sorted by count.
    pub fn rebuild_alphabet(&mut self) -> Result<(),()> {
        self.alphabet = [0;N];
        self.alphabet.iter_mut().enumerate().map(|(i, n)| *n = i as u16);
        self.alphabet.sort_by(|&x1, &x2| {
            ((self.count[x2 as usize] << 16) + x2).cmp(&((self.count[x1 as usize] << 16) + x1))
        });
        Ok(())
    }
    
}

pub fn decompress<T: Read, U: Read + Write + Seek>(reader: T, mut writer: U) -> Result<(), std::io::Error> {

    let mut bit_reader = BitReader::new( reader );
    
    let mut huffman_table: [Item<u32,u8>;16] = [Default::default();16]; // symbol index table
    huffman_table.copy_from_slice(&HUFFMAN_TABLE[..]);

    let mut symbol_table = SymbolTable::<274>::new();

    while let Ok(code) = bit_reader.read_u8(4) {

        assert!( code < 128, "out of sync!" );

        // read code item
        let mut code_item = huffman_table[code as usize];
        let bits = code_item.length;
        let offset = code_item.value;
        let index = (bit_reader.read_u8(code_item.length as u8).unwrap() + offset) as usize;

        let symbol = symbol_table.symbol_at(index);

        //let mut write_pos: usize = 0;

        // execute code word
        match symbol {
            0..=255 => {
                // uncompressed
                writer.write_u8(symbol as u8);
                //dest[write_pos] = symbol as u8;
                //write_pos += 1;
            }
            256..=271 => {
                // symbol from dictionary
                let mut length = 4;

                if symbol < 264 {
                    length += symbol - 256;
                }
                else {
                    let index = (symbol - 264) as usize;
                    let bits = LZ_LENGTH_TABLE[(index << 1) as usize].length;
                    let offset = LZ_LENGTH_TABLE[((index << 1) + 1) as usize].value;
                    length += offset + (bit_reader.read_u8(bits as u8).unwrap() as u16);
                }

                let code = bit_reader.read_u8(3).unwrap();
                let bits = LZ_DISTANCE_TABLE[(code << 1) as usize].length;
                let offset = LZ_DISTANCE_TABLE[((code << 1) + 1) as usize].value;
                let distance = (offset << 9) + bit_reader.read_u8(bits + 9).unwrap();

                // LZ77 jumping
                let write_pos = writer.stream_position().unwrap();
                let jump_pos = write_pos - distance as u64;
                for i in 0..(length as usize) {
                    writer.seek(SeekFrom::Start(jump_pos));
                    let prev_symbol = writer.read_u8().unwrap(); //dest[(index + i)];
                    writer.seek(SeekFrom::Start(write_pos));
                    writer.write_u8(prev_symbol);
                }
            }
            272 => {
                // some sort of reset
                symbol_table.rebuild_alphabet();

                // divide all counts by two
                symbol_table.count.iter_mut().map(|count| *count /= 2 );

                // parse new huffman table
                let mut bits_count = 0;
                let mut offset = 0;
                let mut length = 0;

                for i in 0..16 {

                    while let Ok(bit) = bit_reader.read_u8(1) {
                        if bit > 0 {
                            break;
                        }
                        else {
                            bits_count += 1;
                        }
                    }

                    symbol_table.indices[length] = bits_count;
                    symbol_table.indices[length + 1] = offset;
                    length += 2;
                    offset += 1 << bits_count;

                }
            }
            _ => {
                // end-of-stream
                break;
            }
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

        let mut output: [u8;2] = [0;2];

        let mut writer = BitWriter::new(&mut output[..]);
        writer.write_bits(&[0b1111_0000], 4);
        writer.flush();

        let mut reader = BitReader::new(&output[..]);

        assert_eq!{reader.read_u8(8).unwrap(), 0b1111_0000};
    }

    #[test]
    fn test_decompress() {

        // Chunk @ 1354, size: 24; Type=1; checksum=47560, unknown1=0, unknown2=0
        let decoded: [u8;24] = [
            0b0000_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000, 
            0b0000_0100, 0b0000_0000, 0b0000_0000, 0b0000_0000, 
            0b0000_0010, 0b0000_0000, 0b0000_0000, 0b0000_0000, 
            0b1000_0000, 0b0000_0010, 0b0000_0000, 0b0000_0000, 
            0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000, 
            0b0100_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000
        ];

        // gameType: singlePlayer; playerCount: 4; startResources: medium; size: [640 x 640]; unk5: 0; unk6: 64;
        let expect = GeneralInformation {
            game_type: 1, // singlePlayer
            player_count: 4,  
            start_resources: 2, // medium
            map_size: 640 // 640x640
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