
use std::io::prelude::*;
use std::io::Error;
use std::ops::Index;

use byteorder::{ByteOrder, LittleEndian};

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

pub fn decompress<T: Read, U: Write>(reader: &mut T, writer: &mut U) -> Result<(), std::io::Error> {

    let mut huffman_table: [Item<u32,u8>;16] = [Default::default();16];
    
    huffman_table.copy_from_slice(&HUFFMAN_TABLE[..]);

    let mut code_table = generate_code_table();

    let mut buf: [u8;4] = [0;4];

    while reader.read_exact(&mut buf).is_ok() {

        let code_type = LittleEndian::read_i32(&buf);

        if code_type < 0 {
            panic!("CodeType out of sync!");
        }

        let mut huffman_item = &mut huffman_table[code_type as usize];

        if huffman_item.value > 0 {
            reader.read_exact(&mut buf);
            huffman_item.length += LittleEndian::read_u32(&buf);
            if huffman_item.length >= 274 {
                panic!("CodeType out of sync!");
            }
        }

        let word = code_table[huffman_item.length as usize].value;

        // execute code word
        match word {
            0..=255 => {
                // this is a normal letter
                //output[0] = code_word; // ???
            }
            272 => {
                reset_code_table( &mut code_table );

                // update huffman table
                let mut base = 0;
                let mut length: i32 = 0;

                for item in huffman_table.iter_mut() {
                    
                    item.length -= 1;

                    loop {
                        item.length += 1;
                        reader.read_exact(&mut buf);
                        let bit_value = LittleEndian::read_i32(&buf);
                        if bit_value != 0 {
                            break;
                        }
                    }

                    base += (1 << item.length);
                }
            }
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
            _ => panic!("Bad dictionary entry!")
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::assert_eq;

    use super::*;

    #[test]
    fn test_decompress() {

        // let file = OpenOptions::new().read(true).open("map/Aeneas.map").unwrap();

        // let mut buf_reader = BufReader::new(file);

        // let map_file = MapFile::new( buf_reader );

        // assert_eq!( map_file.segment_register[0].segment_type, SegmentType::MapGeneralInformation );
        // assert_eq!( map_file.segment_register[0].offset, 1354 );
        // assert_eq!( map_file.segment_register[0].encrypted_data_length, 21 );
        // assert_eq!( map_file.segment_register[0].decrypted_data_length, 24 );
    }

}