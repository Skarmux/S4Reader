use byteorder::ReadBytesExt;
use byteorder::{ByteOrder, LittleEndian};
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader, SeekFrom};
use std::path::Path;
use std::fmt;

use crate::ara_crypt::AraCrypt;
use crate::decompress::decompress;

#[derive(Copy, Clone, Debug)]
struct SegmentHeader {
    segment_type: u32,
    encrypted_data_length: usize,
    decrypted_data_length: usize,
    checksum: u32,
    offset: u64,
}

pub struct MapFile {
    name: String,
    game_type: u32,
    player_count: u32,
    start_resources: u32,
    map_size: u32,
    // player: Player,
    // team: Vec<Vec<PlayerType>>,
    // preview: u32,
    // mission: String,
    // hint: String,
    // inner: BufReader<File>,
    
}

impl fmt::Debug for MapFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapFile")
            .field("game_type", &self.game_type)
            .field("player_count", &self.player_count)
            .field("start_resources", &self.start_resources)
            .field("map_size", &self.map_size)
            .finish()
    }
}

impl MapFile {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let name = path.as_ref().file_name().unwrap().to_str();

        /* opening file */
        let file = OpenOptions::new().read(true).open(path.as_ref())?;
        let mut inner = BufReader::<File>::new(file);

        /* validating file */
        let _checksum = inner.read_u32::<LittleEndian>().unwrap();
        let _file_version = inner.read_u32::<LittleEndian>().unwrap();

        // TODO: File Validation missing

        /* read header */
        const HEADER_SIZE: usize = 24;
        const HEADER_CRYPT_KEYS: [u32; 3] = [0x30313233, 0x34353637, 0x38393031];

        /* fill header buffer */
        let mut header_buffer: [u8; HEADER_SIZE] = [0; HEADER_SIZE];

        //let mut segment_header = Vec::<SegmentHeader>::new();
        let mut general: GeneralInformation = Default::default();

        while inner.read_exact(&mut header_buffer).is_ok() {
            /* decrypt */
            let mut ara_crypt = AraCrypt::new(HEADER_CRYPT_KEYS);
            header_buffer
                .iter_mut()
                .for_each(|x| *x = *x ^ ara_crypt.next() as u8);

            /* interprete header segment */
            let mut header_iter = header_buffer.chunks_exact(4);
            let header = SegmentHeader {
                segment_type: LittleEndian::read_u32(&header_iter.next().unwrap()),
                encrypted_data_length: LittleEndian::read_u32(&header_iter.next().unwrap())
                    as usize,
                decrypted_data_length: LittleEndian::read_u32(&header_iter.next().unwrap())
                    as usize,
                checksum: LittleEndian::read_u32(&header_iter.next().unwrap()),
                offset: inner.stream_position().unwrap(),
            };

            //segment_header.push(header);
            if header.segment_type == 1 {
                let mut decrypt = Vec::<u8>::with_capacity(header.decrypted_data_length);
                decrypt.resize(header.decrypted_data_length, 0);
                if decompress(&mut inner, &mut decrypt).is_ok() {
                    //general = GeneralInformation::from_le_bytes(&decrypt[..]);
                    assert!(decrypt.len() == 24, "Input slice is 4 bytes!");

                    return Ok(MapFile {
                        name: path.as_ref().file_name().unwrap().to_str().unwrap().into(),
                        game_type: LittleEndian::read_u32(&decrypt[..4]),
                        player_count: LittleEndian::read_u32(&decrypt[4..8]),
                        start_resources: LittleEndian::read_u32(&decrypt[8..12]),
                        map_size: LittleEndian::read_u32(&decrypt[12..16]),
                        // player: Default::default(),
                        // team: Default::default(),
                        // preview: Default::default(),
                        // mission: String::from("hint"),
                        // hint: String::from("hint"),
                        // inner,
                    })
                }
            }

            /* move stream position behind data content of current segment */
            if let Err(err) = inner.seek(SeekFrom::Current(header.encrypted_data_length as i64)) {
                return Err(err);
            }
        }

        Err()
    }

    pub fn game_type(&self) -> GameMode {
        GameMode::from(self.game_type)
    }

    pub fn player_count(&self) -> u32 {
        self.player_count
    }

    pub fn resource_amount(&self) -> ResourceAmount {
        ResourceAmount::from(self.start_resources)
    }

    pub fn map_size(&self) -> u32 {
        self.map_size
    }

    pub fn load(&self) -> Map {
        Map {
            objects: Default::default(),
            settlers: Default::default(),
            buildings: Default::default(),
            stacks: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Map {
    objects: Vec<u32>,
    settlers: Vec<u32>,
    buildings: Vec<u32>,
    stacks: Vec<u32>,
}

#[derive(Debug, Default)]
struct Player {
    name: String,
    tribe: Tribe,
    start_pos: (u32, u32),
}

#[derive(Debug, Clone, Copy)]
pub enum Tribe {
    Roman,
    Viking,
    Mayan,
    Dark,
    Trojan,
}

impl Default for Tribe {
    fn default() -> Self {
        Tribe::Roman
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerType {
    Human,
    Computer,
}

#[derive(Debug, Clone, Copy)]
pub enum GameMode {
    Multiplayer,
    Singleplayer,
    Cooperation,
}

impl Default for GameMode {
    fn default() -> Self {
        GameMode::Singleplayer
    }
}

impl From<u32> for GameMode {
    fn from(value: u32) -> Self {
        match value {
            0 => GameMode::Multiplayer,
            1 => GameMode::Singleplayer,
            2 => GameMode::Cooperation,
            x => panic!("Invalid GameType '{}'", x),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceAmount {
    Low,
    Medium,
    High,
}

impl From<u32> for ResourceAmount {
    fn from(value: u32) -> Self {
        match value {
            0 => ResourceAmount::Low,
            1 => ResourceAmount::Medium,
            2 => ResourceAmount::High,
            x => panic!("Invalid ResourceAmount '{}'", x),
        }
    }
}

impl Default for ResourceAmount {
    fn default() -> Self {
        ResourceAmount::Medium
    }
}

#[derive(Debug, Default)]
struct GeneralInformation {
    pub game_type: GameMode,
    pub player_count: u32,
    pub start_resources: ResourceAmount,
    pub map_size: u32,
}

impl GeneralInformation {
    pub fn from_le_bytes(bytes: &[u8]) -> GeneralInformation {
        assert!(bytes.len() == 24, "Input slice is 4 bytes!");
        Self {
            game_type: GameMode::from(LittleEndian::read_u32(&bytes[..4])),
            player_count: LittleEndian::read_u32(&bytes[4..8]),
            start_resources: ResourceAmount::from(LittleEndian::read_u32(&bytes[8..12])),
            map_size: LittleEndian::read_u32(&bytes[12..16]),
        }
    }
}

// #[cfg(test)]
// mod tests {

//     use std::assert_eq;

//     use super::*;

// }
