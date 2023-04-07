#![allow(dead_code, unused)]

use byteorder::ReadBytesExt;
use byteorder::{ByteOrder, LittleEndian};
use std::default::Default;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader, SeekFrom};
use std::path::Path;
use std::str;

use crate::ara_crypt::AraCrypt;
use crate::decompress::decompress;
use crate::settlers::map::*;
use crate::settlers::player::Player;

use super::segments::{Segment, SegmentHeader};

#[derive(Default, Debug)]
pub struct Map {
    game_type: u32,
    player_count: u32,
    start_resources: u32,
    map_size: u32,
    quest_text: String,
}

impl Map {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let _name = String::from(
            path.as_ref()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .split_terminator('.')
                .next()
                .unwrap(),
        );
        let file = OpenOptions::new().read(true).open(path.as_ref())?;
        Map::from_file(file)
    }

    pub fn from_file(mut file: File) -> std::io::Result<Self> {
        let mut reader = BufReader::<File>::new(file);

        /* validating file */
        let _checksum = reader.read_u32::<LittleEndian>().unwrap();
        let _file_version = reader.read_u32::<LittleEndian>().unwrap();

        /* read header */
        let mut ara_crypt = AraCrypt::new([0x30313233, 0x34353637, 0x38393031]);

        /* fill header buffer */
        let mut header_buffer: [u8; 24] = [0; 24];

        let mut map = Map {
            ..Default::default()
        };

        while reader.read_exact(&mut header_buffer).is_ok() {
            /* decrypt */
            header_buffer
                .iter_mut()
                .for_each(|x| *x = *x ^ ara_crypt.next() as u8);
            ara_crypt.reset();

            /* interprete header segment */
            let header = SegmentHeader::from_le_bytes(&header_buffer).unwrap();

            //let mut decrypt = Vec::<u8>::with_capacity(header.decrypted_data_length as usize);
            //decrypt.resize(header.decrypted_data_length as usize, 0);

            let pos = reader.stream_position().unwrap();

            if header.segment_type == Segment::PlayerInformation {
                let mut crypt_reader = reader.take(header.encrypted_data_length as u64);
                let decrypt = decompress(&mut crypt_reader)?;
                reader = crypt_reader.into_inner();
                let player_a = Player::from_le_bytes(&decrypt[0..45]).unwrap();
                let player_b = Player::from_le_bytes(&decrypt[45..90]).unwrap();
                dbg!(player_b);
                // assert!(
                //     decrypt.len() == 24,
                //     "Input slice is bigger than 4 bytes! {}",
                //     decrypt.len()
                // );
                // map.game_type = LittleEndian::read_u32(&decrypt[..4]);
                // map.player_count = LittleEndian::read_u32(&decrypt[4..8]);
                // map.start_resources = LittleEndian::read_u32(&decrypt[8..12]);
                // map.map_size = LittleEndian::read_u32(&decrypt[12..16]);
            }

            // let mut crypt_reader = reader.take(header.encrypted_data_length as u64);
            // let decrypt = decompress(&mut crypt_reader)?;
            // reader = crypt_reader.into_inner();

            // match header.segment_type {
            //     Segment::GeneralInformation => {
            //         assert!(
            //             decrypt.len() == 24,
            //             "Input slice is bigger than 4 bytes! {}",
            //             decrypt.len()
            //         );
            //         map.game_type = LittleEndian::read_u32(&decrypt[..4]);
            //         map.player_count = LittleEndian::read_u32(&decrypt[4..8]);
            //         map.start_resources = LittleEndian::read_u32(&decrypt[8..12]);
            //         map.map_size = LittleEndian::read_u32(&decrypt[12..16]);
            //     }
            //     Segment::QuestText => {
            //         map.quest_text = str::from_utf8(&decrypt).unwrap().to_owned();
            //     }
            //     _ => {}
            // }

            /* move stream position behind data content of current segment */
            reader.seek(SeekFrom::Start(pos + header.encrypted_data_length as u64))?;
        }

        Ok(map)
    }
}
