
use std::io::prelude::*;
use std::io::Error;

use byteorder::{ByteOrder, LittleEndian};

// https://www.rfc-editor.org/rfc/rfc1951

// const LZ_DISTANCE_TABLE: [[u8;8];2] = [
//     [0, 0, 1, 2, 3, 4, 5, 6],
//     [0x00, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40]
// ];

// const LZ_LENGTH_TABLE: [[u16;8];2] = [
//     [1, 2, 3, 4, 5, 6, 7, 8],
//     [0x008, 0x00A, 0x00E, 0x016, 0x026, 0x046, 0x086, 0x106]
// ];

// let mut huffman_table: [[u8;16];2] = [
//     [2, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5],
//     [0x00, 0x04, 0x0C, 0x14, 0x24, 0x34, 0x44, 0x54, 0x64, 0x74, 0x84, 0x94, 0xA4, 0xB4, 0xD4, 0xF4]
// ];

const LZ_DISTANCE_TABLE: [(u8,u8);8] = [
    (0, 0x00),
    (0, 0x01),
    (1, 0x02),
    (2, 0x04),
    (3, 0x08),
    (4, 0x10),
    (5, 0x20),
    (6, 0x40)
];

const LZ_LENGTH_TABLE: [(u8,u16);8] = [
    (1, 0x008),
    (2, 0x00A),
    (3, 0x00E),
    (4, 0x016),
    (5, 0x026),
    (6, 0x046),
    (7, 0x086),
    (8, 0x106)
];

struct CodeTable {
    table: [u32;274],
    indices: [u32;274],
    quantities: [i32;274],
}

impl CodeTable {
    fn new() -> Self {

        let mut table: [u32;274] = [0;274];

        for i in 0..16 {
            table[i as usize] = i + 0x100;
        }
        
        table[16] = 0x00;
        table[17] = 0x20;
        table[18] = 0x30;
        table[19] = 0xFF;

        let mut j = 20;
        for i in 1..274 {
            if table.iter().any(|&val| val == i) {
                table[j] = i;
                j += 1;
            }
        }

        let indices = Self::new_indices(&table);
        CodeTable { 
            table, 
            indices, 
            quantities: [0;274]
        }
    }

    fn new_indices( table: &[u32;274] ) -> [u32;274] {
        let indices: [u32;274];
        for i in 0..274 {
            let index = table[i as usize] as usize;
            indices[index] = i;
        }
        indices
    }

    fn word_at(&mut self, index: usize ) -> u32 {
        let word = self.table[index];
        self.quantities[word as usize] += 1;
        word
    }

    fn reset(&mut self) {
        // create new entropy encoding table
        self.table.iter_mut().enumerate().map(|(i, &mut v)| v = i as u32 );

        // sort index by quantities... to be stable we use "quantities + index" as value
        //self.table.sort_by(|a, b| ((self.quantities[b as usize] << 16) as u32 + b) - ((self.quantities[a as usize] << 16) as u32 + a) > 0 );
        let comparator = |&a, &b| -> std::cmp::Ordering {
            let result =  (self.quantities[b as usize] << 16) + b as i32 - (self.quantities[a as usize] << 16) + a as i32;
            0.cmp(&result)
        };
        self.table.sort_by(|a, b| comparator(a, b) );

        // we reduce the original quantity by 2 to the impact for the next CreateCodeTableFromFrequency() call
        for i in 0..274 {
            self.quantities[i] = self.quantities[i] / 2;
        }

        self.indices = Self::new_indices( &self.table );
    }
}

pub fn decompress<T: Read, U: Write>(reader: &mut T, writer: &mut U) -> Result<(), std::io::Error> {

    let mut huffman_table: [(u32,u8);16] = [
        (2, 0x00),
        (3, 0x04),
        (3, 0x0C),
        (4, 0x14),
        (4, 0x24),
        (4, 0x34),
        (4, 0x44),
        (4, 0x54),
        (4, 0x64),
        (4, 0x74),
        (4, 0x84),
        (4, 0x94),
        (4, 0xA4),
        (5, 0xB4),
        (5, 0xD4),
        (5, 0xF4)
    ];

    let mut code_table = CodeTable::new();

    let mut buf: [u8;4] = [0;4];

    while reader.read_exact(&mut buf).is_ok() {

        let code_type = LittleEndian::read_i32(&buf);

        if code_type < 0 {
            panic!("CodeType out of sync!");
        }

        let (index, value) = huffman_table[code_type as usize];

        if value > 0 {
            reader.read_exact(&mut buf);
            index += LittleEndian::read_u32(&buf);
            if index >= 274 {
                panic!("CodeType out of sync!");
            }
        }

        let word = code_table.word_at(index as usize);

        // execute code word
        match word {
            0..=255 => {
                // this is a normal letter
                //output[0] = code_word; // ???
            }
            272 => {
                code_table.reset();

                // update huffman table
                let mut base = 0;
                let mut length: i32 = 0;

                for (i, v) in huffman_table.iter_mut() {
                    
                    length -= 1;

                    loop {
                        length += 1;
                        reader.read_exact(&mut buf);
                        let bit_value = LittleEndian::read_i32(&buf);
                        if bit_value != 0 {
                            break;
                        }
                    }

                    *i = length;
                    *v = base;

                    base += (1 << length);
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