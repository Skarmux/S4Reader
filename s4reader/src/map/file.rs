#![allow(dead_code, unused)]

use byteorder::ReadBytesExt;
use byteorder::{ByteOrder, LittleEndian};
use std::default::Default;
use std::ffi::CStr;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader, SeekFrom};
use std::path::Path;
use std::str;

use crate::ara_crypt::AraCrypt;
use crate::decompress::decompress;
use crate::settlers::*;

use super::segments::{SegmentHeader, SegmentType};

#[derive(Default, Debug)]
pub struct Map {
    game_type: u32,
    player_count: u32,
    start_resources: u32,
    map_size: u32,
}

impl Map {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
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
            if let Ok(header) = SegmentHeader::from_le_bytes(&header_buffer) {
                let pos = reader.stream_position().unwrap(); // TODO: Should become unnecessary!

                let mut crypt_reader = reader.take(header.n_bytes_encrypted as u64);

                match header.segment_type {
                    Some(SegmentType::MapInfo) => {
                        println!("MapInfo:");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        map.game_type = LittleEndian::read_u32(&decrypt[..4]);
                        map.player_count = LittleEndian::read_u32(&decrypt[4..8]);
                        map.start_resources = LittleEndian::read_u32(&decrypt[8..12]);
                        map.map_size = LittleEndian::read_u32(&decrypt[12..16]);
                    }
                    Some(SegmentType::PlayerInfo) => {
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        let mut player_info = Vec::<Player>::new();
                        let mut chunks = decrypt.chunks_exact_mut(45);
                        for buffer in chunks.into_iter() {
                            let player = Player::from_le_bytes(buffer).unwrap();
                            dbg!(&player);
                            player_info.push(player);
                        }
                    }
                    Some(SegmentType::MissionInfoDE) => {
                        print!("MissionInfoDE: ");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        let text = unsafe { CStr::from_ptr(decrypt.as_ptr() as *const i8) };
                        println!("{}", text.to_string_lossy());
                    }
                    Some(SegmentType::Buildings) => {
                        print!("Buildings: ");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        println!("[OK]");
                    }
                    Some(SegmentType::Ground) => {
                        print!("Ground: ");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        let mut grounds = Vec::<Ground>::new();
                        for chunk in decrypt.chunks_exact(4) {
                            let ground = Ground::from_le_bytes(chunk).unwrap();
                            dbg!(&ground);
                            grounds.push(ground);
                        }
                        println!("[OK]");
                    }
                    Some(SegmentType::LuaScript) => {
                        println!("LuaScript: ");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        let text = unsafe { CStr::from_ptr(decrypt.as_ptr() as *const i8) };
                        println!("{}", text.to_string_lossy());
                    }
                    Some(SegmentType::MissionHintDE) => {
                        print!("MissionHintDE: ");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        let text = unsafe { CStr::from_ptr(decrypt.as_ptr() as *const i8) };
                        println!("{}", text.to_string_lossy());
                    }
                    Some(SegmentType::MissionHintEN) => {
                        print!("MissionHintEN: ");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        let text = unsafe { CStr::from_ptr(decrypt.as_ptr() as *const i8) };
                        println!("{}", text.to_string_lossy());
                    }
                    Some(SegmentType::MissionInfoEN) => {
                        print!("MissionInfoEN: ");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        let text = unsafe { CStr::from_ptr(decrypt.as_ptr() as *const i8) };
                        println!("{}", text.to_string_lossy());
                    }
                    Some(SegmentType::Objects) => {
                        print!("Objects: ");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        println!("[OK]");
                    }
                    Some(SegmentType::Preview) => {
                        print!("Preview: ");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        println!("[OK]");
                    }
                    Some(SegmentType::Settlers) => {
                        print!("Settlers: ");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        println!("[OK]");
                    }
                    Some(SegmentType::Stacks) => {
                        print!("Stacks: ");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        println!("[OK]");
                    }
                    Some(SegmentType::TeamInfo) => {
                        println!("TeamInfo:");
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        let team_info = TeamInfo::from_le_bytes(&decrypt).unwrap();
                        dbg!(team_info);
                    }
                    Some(SegmentType::VictoryCond) => {
                        let mut decrypt = decompress(&mut crypt_reader)?;
                        let victory_cond = VictoryCondition::from_le_bytes(&decrypt).unwrap();
                        println!("{victory_cond:#?}");
                    }
                    None => {}
                }

                reader = crypt_reader.into_inner();

                /* move stream position behind data content of current segment */
                reader.seek(SeekFrom::Start(pos + header.n_bytes_encrypted as u64))?;
                // TODO should become unneccessary
            }
        }

        Ok(map)
    }
}
