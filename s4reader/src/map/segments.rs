use byteorder::{ByteOrder, LittleEndian};
use std::convert::From;
use std::str;

pub struct General {
    game_mode: u32,
    player_count: u32,
    start_resources: u32,
    map_size: u32,
}

pub struct Chunk<T> {
    segment_type: Segment,
    encrypted_data_length: u32,
    decrypted_data_length: u32,
    checksum: u32,
    data: T,
}

impl Chunk<General> {
    pub fn game_mode(&self) -> u32 {
        self.data.game_mode
    }
    pub fn player_count(&self) -> u32 {
        self.data.player_count
    }
    pub fn start_resources(&self) -> u32 {
        self.data.start_resources
    }
    pub fn map_size(&self) -> u32 {
        self.data.map_size
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SegmentHeader {
    pub segment_type: Segment,
    pub encrypted_data_length: u32,
    pub decrypted_data_length: u32,
    pub checksum: u32,
}

impl SegmentHeader {
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut iter = bytes.chunks_exact(4);
        Ok(SegmentHeader {
            segment_type: Segment::from(LittleEndian::read_u32(&iter.next().unwrap())),
            encrypted_data_length: LittleEndian::read_u32(&iter.next().unwrap()),
            decrypted_data_length: LittleEndian::read_u32(&iter.next().unwrap()),
            checksum: LittleEndian::read_u32(&iter.next().unwrap()),
        })
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Segment {
    GeneralInformation,
    PlayerInformation,
    TeamInformation,
    Preview,
    Objects,
    Settlers,
    Buildings,
    Stacks,
    QuestText,
    QuestTip,
    Landscape,
    Unknown,
}

impl From<u32> for Segment {
    fn from(value: u32) -> Segment {
        match value {
            1 => Self::GeneralInformation,
            2 => Self::PlayerInformation,
            3 => Self::TeamInformation,
            4 => Self::Preview,
            6 => Self::Objects,
            7 => Self::Settlers,
            8 => Self::Buildings,
            9 => Self::Stacks,
            11 => Self::QuestText,
            12 => Self::QuestTip,
            13 => Self::Landscape,
            v => {
                println!("Unknown Segment Type ID: {}", v);
                Self::Unknown
            }
        }
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

struct LandscapePosition {
    height: u8,
    terrain: u8,
    subtype: u8,
}

struct Landscape {
    positions: Vec<LandscapePosition>,
    width: u64,
    height: u64,
}
