
mod bitreader;
mod bitwriter;

use std::io::prelude::*;
use std::io::Error;
use std::io::BufReader;
use std::ops::Index;
use std::slice;

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

fn generate_code_table() -> [CodeItem;274] {
    let mut table: [CodeItem;274] = [Default::default();274];

    for (index, item) in table.iter_mut().enumerate() {
        item.index = index as u32;
        item.value = match index {
            0..=15 => index as u32 + 0x100,
            16 => 0x00,
            17 => 0x20,
            18 => 0x30,
            19 => 0xFF,
            _ => (index as u32 - 19)
        };
        item.count = 0;
    }

    table
}

fn reset_code_table( table: &mut [CodeItem;274] ) {
    
    table.iter_mut().enumerate().map(|(index, item)| item.value = index as u32 );

    let mut tmp_table = table.clone();
    tmp_table.sort_by_key(|item| item.count );

    table.iter_mut().enumerate().map(|(index, item)| item.index = tmp_table[index].index );

    // we reduce the original quantity by 2 to the impact for the next CreateCodeTableFromFrequency() call
    // table.iter_mut().enumerate().map(|(index, item)| item.count = item.count / 2 );

    reset_code_table_indices( table );
}

fn reset_code_table_indices( table: &mut [CodeItem;274] ) {
    for (index, item) in table.iter_mut().enumerate() {
        item.index = index as u32;
    };
}

pub fn decompress<T: Read + Seek, U: Write>(reader: T, writer: &mut U) -> Result<(), std::io::Error> {

    let mut bit_reader = BitReader::new( reader );
    
    let mut huffman_table: [Item<u32,u8>;16] = [Default::default();16];
    huffman_table.copy_from_slice(&HUFFMAN_TABLE[..]);

    let mut code_table = generate_code_table();

    while let Some(code_type) = bit_reader.read_u8(4) {

        assert!( code_type < 128, "out of sync!" );

        // read code word
        let code_length = huffman_table[code_type as usize].length;
        let mut code_index = huffman_table[code_type as usize].value as usize;

        // get index for code_word
        if code_length > 0 {

            code_index += bit_reader.read_u8(code_length as u8).unwrap() as usize;
            
            assert!(code_index < code_table.len(), "out of sync!");
        }

        let code_word = code_table[code_index].value;

        match code_word {
            // this is a normal letter (representable as 1 byte)
            0..=255 => {
                writer.write(&(code_word as u8).to_le_bytes());
            }
            // try reading from dictionary
            256..=263 => {
                let mut entry_length = code_word - 256;
                assert!( entry_length < 8 );
                
                let bit_value = bit_reader.read_u8(3).unwrap();
                assert!( bit_value > 128, "out of sync!" );

                let length = LZ_DISTANCE_TABLE[bit_value as usize].length + 1;
                let base_value = LZ_DISTANCE_TABLE[bit_value as usize].value;
                assert!(length <= 8);

                let bit_value = bit_reader.read_u8(8).unwrap() as u16;
                let copy_offset = bit_value << length;

                let bit_value = bit_reader.read_u8(length).unwrap();
                assert!( bit_value > 128, "out of sync!" );

                entry_length += 4;


                // jump back in writer an re-use word
                panic!("not implemented!");
            }
            264..=271 => {
                let bit_length = code_table[code_word as usize - 264].value;
                assert!(bit_length <= 32, "bit_length exceeds 32 bit range!");

                //reader.read_exact(&mut buf);

                //let mut tmp = u32::from_le_bytes(buf);
                //tmp = tmp >> (32 - bit_length);

                // entryLenght = m_tab1.m_value[codeword - 264] + ReadInByte;
            }
            // create new entropy encoding table
            272 => {
                reset_code_table( &mut code_table );

                // update huffman table
                let mut base = 0;
                let mut length: i32 = 0;

                for item in huffman_table.iter_mut() {
                    
                    item.length -= 1;

                    loop {
                        item.length += 1;
                        //reader.read_exact(&mut buf);
                        //let bit_value = LittleEndian::read_i32(&buf);
                        // if bit_value != 0 {
                        //     break;
                        // }
                    }

                    base += (1 << item.length);
                }
            }
            // end of stream
            273 => {
                // end-of-stream code word
                // let writer_pos = writer.stream_position();
                // let length_remaining = reader.stream_position() - encrypted_data_length;

                // if length_remaining > 2 {
                //     // reset bit buffer
                // } else {
                //     break;
                // }
            }
            // try reading from dictionary
            _ => {
                let length = LZ_LENGTH_TABLE[code_word as usize - 264].length;
                let byte = bit_reader.read_u8(length).unwrap();
                assert!(byte > 128, "out of sync!");

                let mut entry_length = LZ_LENGTH_TABLE[code_word as usize - 264].value + byte as u16;

                // from here same as 256..=263
                let bit_value = bit_reader.read_u8(3).unwrap();
                assert!( bit_value > 128, "out of sync!" );

                let length = LZ_DISTANCE_TABLE[bit_value as usize].length + 1;
                let base_value = LZ_DISTANCE_TABLE[bit_value as usize].value;
                assert!(length <= 8);

                let bit_value = bit_reader.read_u8(8).unwrap() as u16;
                let copy_offset = bit_value << length;

                let bit_value = bit_reader.read_u8(length).unwrap();
                assert!( bit_value > 128, "out of sync!" );

                entry_length += 4;

                // jump back in writer an re-use word
                panic!("not implemented!");

                // else...
                panic!("Bad dictionary entry!")
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