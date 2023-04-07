use byteorder::{ByteOrder, LittleEndian};
use std::convert::TryFrom;

#[derive(Debug, Default)]
pub struct Player {
    tribe: Tribe,
    start_pos: (u32, u32),
    name: [u8;32],
}

impl Player {
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut name = [0;32];
        name[..].copy_from_slice(&bytes[12..44]);
        Ok(Player {
            tribe: Tribe::try_from(LittleEndian::read_u32(&bytes[0..4]))?,
            start_pos: (
                LittleEndian::read_u32(&bytes[4..8]),  // x
                LittleEndian::read_u32(&bytes[8..12]), // y
            ),
            name
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Tribe {
    Roman,
    Viking,
    Mayan,
    Dark,
    Trojan,
}

impl TryFrom<u32> for Tribe {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Tribe::Roman),
            1 => Ok(Tribe::Viking),
            2 => Ok(Tribe::Mayan),
            3 => Ok(Tribe::Dark),
            4 => Ok(Tribe::Trojan),
            _ => Err("No tribe found for given value!"),
        }
    }
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
