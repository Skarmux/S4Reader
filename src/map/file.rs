use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::{prelude::*, BufReader};
use std::path::Path;
use std::str;

use byteorder::ReadBytesExt;
use byteorder::{ByteOrder, LittleEndian};
use num_enum::TryFromPrimitive;

use crate::io::ara_crypt::AraCrypt;
use crate::io::decompress::decompress;
use crate::map::info::*;

#[derive(Debug)]
pub struct GameMap {
    checksum: u32,
    version: u32,
    map: Info,
    // player: Player,
    // team: TeamInfo,
    // preview: Box<Preview>,
    // objects: Vec<Object>,
    // settlers: Vec<Settler>,
    // buildings: Vec<Building>,
    // stacks: Vec<Stack>,
    // victory_conditions: VictoryCondition,
    // mission_text_german: String,
    // mission_hint_german: String,
    // landscape: Vec<Ground>,
    // mission_text_english: String,
    // mission_hint_english: String,
    // lua_script: String,
}

impl GameMap {
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).open(path.as_ref())?;
        let mut reader = BufReader::<File>::new(file);

        Ok(GameMap {
            checksum: reader.read_u32::<LittleEndian>().unwrap(),
            version: reader.read_u32::<LittleEndian>().unwrap(),
            map: GameMap::read_info(&mut reader)?,
        })
    }
    fn read_info(reader: &mut BufReader<File>) -> io::Result<Info> {
        while let Ok(header) = GameMap::read_header(reader) {
            //dbg!(&header);
            if Some(SegmentType::MapInfo) == header.segment_type {
                let mut crypt_reader = reader.take(header.n_bytes_encrypted as u64);
                let decrypt = decompress(&mut crypt_reader)?;
                return Info::from_le_bytes(&decrypt)
            } else {
                let _ = reader.seek_relative(header.n_bytes_encrypted as i64);
            }
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "header segment for map info not found",
        ))
    }
    fn read_header(reader: &mut BufReader<File>) -> io::Result<SegmentHeader> {
        let mut header_buffer = [0; 24];

        reader.read_exact(&mut header_buffer)?;

        let mut ara_crypt = AraCrypt::new([0x30313233, 0x34353637, 0x38393031]);
        header_buffer
            .iter_mut()
            .for_each(|x| *x = *x ^ ara_crypt.next() as u8);

        Ok(SegmentHeader::from_le_bytes(&header_buffer)?)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SegmentHeader {
    pub segment_type: Option<SegmentType>,
    pub n_bytes_encrypted: u32,
    pub n_bytes_decrypted: u32,
    pub checksum: u32,
}

impl SegmentHeader {
    /// NOTE: usage of bytes 16..24 is unknown
    pub fn from_le_bytes(bytes: &[u8; 24]) -> io::Result<Self> {
        Ok(SegmentHeader {
            segment_type: SegmentType::try_from(LittleEndian::read_u32(&bytes[0..4])).ok(),
            n_bytes_encrypted: LittleEndian::read_u32(&bytes[4..8]),
            n_bytes_decrypted: LittleEndian::read_u32(&bytes[8..12]),
            checksum: LittleEndian::read_u32(&bytes[12..16]),
        })
    }
}

#[derive(PartialEq, Copy, Clone, Debug, TryFromPrimitive)]
#[repr(u32)]
pub enum SegmentType {
    // EOF,
    MapInfo = 1,
    PlayerInfo,
    TeamInfo,
    Preview,
    // Unknown0,
    Objects = 6,
    Settlers,
    Buildings,
    Stacks,
    VictoryCond,
    MissionInfoDE,
    MissionHintDE,
    Ground,
    MissionInfoEN,
    MissionHintEN,
    LuaScript,
    // EDM = 64,
    // Unknown1,
    // EditorInfo,
    // Unknown2 = 16974621
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum GameMode {
    Multiplayer = 0,
    Singleplayer = 1,
    Cooperation = 2,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
pub enum ResourceAmount {
    Low = 0,
    Medium = 1,
    High = 2,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn loading_map_from_file() {
        let file = GameMap::from_file("data/Settlers 4 Gold/Map/Singleplayer/Aeneas.map");
        assert!(file.is_ok());
    }
}
