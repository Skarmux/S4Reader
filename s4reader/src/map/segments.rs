use byteorder::{ByteOrder, LittleEndian};
use std::convert::From;
use std::convert::TryFrom;
use std::str;
use std::io;

#[derive(Copy, Clone, Debug)]
pub struct SegmentHeader {
    pub segment_type: Option<SegmentType>,
    pub n_bytes_encrypted: u32,
    pub n_bytes_decrypted: u32,
    pub checksum: u32,
}

impl SegmentHeader {
    /// NOTE: usage of bytes 16..24 is unknown
    pub fn from_le_bytes(bytes: &[u8;24]) -> io::Result<Self> {
        Ok(SegmentHeader {
            segment_type: SegmentType::try_from(LittleEndian::read_u32(&bytes[0..4])).ok(),
            n_bytes_encrypted: LittleEndian::read_u32(&bytes[4..8]),
            n_bytes_decrypted: LittleEndian::read_u32(&bytes[8..12]),
            checksum: LittleEndian::read_u32(&bytes[12..16]),
        })
    }
}

#[repr(u32)]
#[derive(PartialEq, Copy, Clone, Debug)]
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

impl TryFrom<u32> for SegmentType {
    type Error = &'static str;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::MapInfo,
            2 => Self::PlayerInfo,
            3 => Self::TeamInfo,
            4 => Self::Preview,
            6 => Self::Objects,
            7 => Self::Settlers,
            8 => Self::Buildings,
            9 => Self::Stacks,
            10 => Self::VictoryCond,
            11 => Self::MissionInfoDE,
            12 => Self::MissionHintDE,
            13 => Self::Ground,
            14 => Self::MissionInfoEN,
            15 => Self::MissionHintEN,
            16 => Self::LuaScript,
            _ => return Err("Unimplemented segment type!"),
        })
    }
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
