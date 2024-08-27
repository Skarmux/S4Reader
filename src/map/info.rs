
use byteorder::{ByteOrder, LittleEndian as LE};
use std::io::Result;

#[derive(Clone, Debug)]
pub struct Info {
    game_type: u32,
    player_count: u32,
    start_resources: u32,
    map_size: u32,
}

impl Info {
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Info {
            game_type: LE::read_u32(&bytes[..4]),
            player_count: LE::read_u32(&bytes[4..8]),
            start_resources: LE::read_u32(&bytes[8..12]),
            map_size: LE::read_u32(&bytes[12..16]),
        })
    }
}
