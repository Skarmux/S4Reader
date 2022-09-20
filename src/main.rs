#![allow(unused)]

use std::error::Error;

use byteorder::ReadBytesExt;
use byteorder::{ByteOrder, LittleEndian};

use std::fs::OpenOptions;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::io::BufReader;

mod crypt;
use crypt::ara_crypt::AraCrypt;

mod decompress;
use decompress::decompress::*;

#[derive(PartialEq)]
enum Segment {
    Unknown,
    General{ position: u64, content: Option<GeneralInformation> },
    Player,
    Team,
    Preview,
    Objects,
    Settlers,
    Buildings,
    Stacks,
    QuestText,
    QuestTip,
    Landscape,
}

#[derive(PartialEq)]
struct GeneralInformation {
    game_type: u32,  
    player_count: u32,  
    start_resources: u32,  
    map_size: u32
}

#[derive(Copy, Clone)]
struct SegmentHeader {
    segment_id: u32,
    encrypted_data_length: u32,
    decrypted_data_length: u32,
    checksum: u32
}

struct Map<T: Read + Seek> {
    reader: T,
    segment_register: Vec<Segment>
}

impl<T> Map<T> where T: Read + Seek {

    fn from( mut reader: T ) -> Result<Map<T>, std::io::Error> {
    
        /* validating file */
        let checksum     = reader.read_u32::<LittleEndian>().unwrap();
        let file_version = reader.read_u32::<LittleEndian>().unwrap();

        /* scraping the file */
        let mut segment_register = Vec::<Segment>::new();

        let mut header = Self::read_header(&mut reader);

        while header.is_ok() {

            let stream_position = reader.stream_position().unwrap();

            segment_register.push( match header.unwrap().segment_id {
                1 => Segment::General{ position: stream_position, content: None },
                _ => Segment::Unknown,
            });

            // move position to after segment data
            reader.seek(SeekFrom::Current( header.unwrap().encrypted_data_length as i64 )).unwrap();

            header = Self::read_header(&mut reader);
        }

        reader.rewind();

        Ok( Self { reader, segment_register } )
    }

    fn read_header( reader: &mut T ) -> Result<SegmentHeader, ()> {

        const HEADER_SIZE: usize = 24;
        const HEADER_CRYPT_KEYS: [u32;3] = [0x30313233, 0x34353637, 0x38393031];

        let mut header_buffer: [u8; HEADER_SIZE] = [0; HEADER_SIZE];

        reader.read_exact( &mut header_buffer );

        /* decrypt buffer */
        let mut ara_crypt = AraCrypt::new( HEADER_CRYPT_KEYS );
        header_buffer.iter_mut().for_each( |x| *x = *x ^ ara_crypt.next() as u8 );

        let mut header_iter = header_buffer.chunks_exact( 4 );

        reader.seek(SeekFrom::Current(HEADER_SIZE as i64));

        Ok ( SegmentHeader { 
            segment_id: LittleEndian::read_u32( &header_iter.next().unwrap() ), 
            encrypted_data_length: LittleEndian::read_u32( &header_iter.next().unwrap() ), 
            decrypted_data_length: LittleEndian::read_u32( &header_iter.next().unwrap() ), 
            checksum: LittleEndian::read_u32( &header_iter.next().unwrap() )
        } )
    }

    pub fn game_type(&mut self) -> Option<u32> {

        let segment: Option<(u64, Option<GeneralInformation>)> = None;

        let position: u64 = 0;
        let content: Option<GeneralInformation> = None;

        for s in self.segment_register.iter() {
            match s {
                Segment::General { position, content } => break,
                _ => continue
            };
        }

        // load on demand
        if content.is_none() {
            self.reader.seek(SeekFrom::Start(position)).unwrap();
            let header = Self::read_header(&mut self.reader).unwrap();
            let mut data_buffer : Vec<u8> = Vec::<u8>::with_capacity(header.decrypted_data_length as usize);
            //self.decompress(&data_buffer[..], self.reader.take(header.encrypted_data_length as u64));
        }

        Some(content.unwrap().game_type)
    }
}

fn main() -> Result<(), Box<dyn Error>> {

    let map_file = OpenOptions::new().read(true).open("map/Aeneas.map")?;

    let buf_reader = BufReader::new(map_file);

    Map::from( buf_reader );

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::assert_eq;

    use super::*;

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