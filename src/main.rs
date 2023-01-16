#![allow(unused)]

mod ara_crypt;
mod decompress;
mod map;

use map::GeneralInformation;
use decompress::decompress;

use byteorder::ReadBytesExt;
use byteorder::{ByteOrder, LittleEndian};

use std::fs::OpenOptions;

use std::io;
use std::io::{SeekFrom, BufWriter, prelude::*, BufReader, Error, ErrorKind, Cursor};
use std::fs::File;
use std::ops::Index;
use std::path::Path;

use ara_crypt::AraCrypt;

// #[derive(PartialEq, Debug)]
// enum Segment {
//     General(GeneralInformation) = 1,
//     // Player = 2,
//     // Team = 3,
//     // Preview = 4,
//     // Objects = 6,
//     // Settlers = 7,
//     // Buildings = 8,
//     // Stacks = 9,
//     // QuestText = 11,
//     // QuestTip = 12,
//     // Landscape = 13,
// }

#[derive(Copy, Clone, Debug)]
struct SegmentHeader {
    segment_type: u32,
    encrypted_data_length: usize,
    decrypted_data_length: usize,
    checksum: u32,
    offset: u64
}

struct MapFile {
    inner: BufReader<File>,
    segment_header: Vec<SegmentHeader>
}

impl MapFile {

    /// same as std::fs::File
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<MapFile> {

        /* opening file */
        let file = OpenOptions::new().read(true).open(path.as_ref())?;
        let mut inner =  BufReader::new(file);

        /* validating file */
        let checksum     = inner.read_u32::<LittleEndian>().unwrap();
        let file_version = inner.read_u32::<LittleEndian>().unwrap();
        // TODO: File Validation missing
        
        Ok(MapFile { inner, segment_header: Vec::new() })
    }

    /// preload each segments header information
    pub fn index(&mut self) -> io::Result<()> {

        /* read header */
        const HEADER_SIZE: usize = 24;
        const HEADER_CRYPT_KEYS: [u32;3] = [0x30313233, 0x34353637, 0x38393031];

        /* fill header buffer */
        let mut header_buffer: [u8; HEADER_SIZE] = [0; HEADER_SIZE];
        
        while self.inner.read_exact( &mut header_buffer ).is_ok() {

            /* decrypt */
            let mut ara_crypt = AraCrypt::new( HEADER_CRYPT_KEYS );
            header_buffer.iter_mut().for_each( |x| *x = *x ^ ara_crypt.next() as u8 );

            /* interprete header segment */
            let mut header_iter = header_buffer.chunks_exact( 4 );
            let header = SegmentHeader {
                segment_type: LittleEndian::read_u32( &header_iter.next().unwrap() ),
                encrypted_data_length: LittleEndian::read_u32( &header_iter.next().unwrap() ) as usize, 
                decrypted_data_length: LittleEndian::read_u32( &header_iter.next().unwrap() ) as usize, 
                checksum: LittleEndian::read_u32( &header_iter.next().unwrap() ),
                offset: self.inner.stream_position().unwrap()
            };

            self.segment_header.push(header);

            /* move stream position behind data content of current segment */
            if let Err(err) = self.inner.seek(SeekFrom::Current( header.encrypted_data_length as i64 )) {
                return Err(err);
            }

        }

        self.inner.rewind();

        Ok(())
    }

    pub fn general_information(&mut self) -> Result<GeneralInformation,()> {

        for header in self.segment_header.iter() {
            if header.segment_type == 1 {
                //dbg!(header);
                self.inner.seek(SeekFrom::Start(header.offset));
                let mut decrypt = Vec::<u8>::with_capacity(header.decrypted_data_length);
                decrypt.resize(header.decrypted_data_length, 0);
                if decompress(&mut self.inner, &mut decrypt).is_ok() {
                    return Ok(GeneralInformation::from_le_bytes(&decrypt[..]));
                }
            }
        }
        Err(())
    }
}

fn main() {
    let mut map_file = MapFile::open( "map/Aeneas.map" ).unwrap();
    map_file.index();
    let general_information = map_file.general_information().unwrap();
    dbg!(general_information);
}

#[cfg(test)]
mod tests {

    use std::assert_eq;

    use super::*;

    // #[test]
    // fn test_read_header() {
    //     let map_file = OpenOptions::new().read(true).open("map/Aeneas.map").unwrap();

    //     let buf_reader = BufReader::new(map_file);

    //     let map = MapFile::from( buf_reader ).unwrap();
    //     let general_information = map[Segment::General];

    //     dbg!(map.segments);
    // }

    #[test]
    fn test_map_file() {

        // let file = OpenOptions::new().read(true).open("map/Aeneas.map").unwrap();

        // let mut buf_reader = BufReader::new(file);

        // let map_file = MapFile::new( buf_reader );

        // assert_eq!( map_file.segment_register[0].segment_type, SegmentType::MapGeneralInformation );
        // assert_eq!( map_file.segment_register[0].offset, 1354 );
        // assert_eq!( map_file.segment_register[0].encrypted_data_length, 21 );
        // assert_eq!( map_file.segment_register[0].decrypted_data_length, 24 );
    }

}