
use byteorder::{ByteOrder, LittleEndian};
use std::convert::TryFrom;
use std::ffi::CStr;
use std::fmt;

#[derive(Clone, Default)]
pub struct Player {
    tribe: Tribe,
    start_pos: (u32, u32),
    name: String,
}

impl fmt::Debug for Player {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "Player '{}' ({:?}), [{}x, {}y]",
            self.name, self.tribe, self.start_pos.0, self.start_pos.1
        )
    }
}

impl Player {
    // pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, String> {
    //     use std::mem;
    //     use std::slice;
    //     use std::io::Read;
    //     let mut player: Player = unsafe { mem::zeroed() };

    //     let player_size = mem::size_of::<Player>();

    //     unsafe {
    //         let player_slice = slice::from_raw_parts_mut(&mut player as *mut _ as *mut u8, player_size);
    //         bytes.read_exact(player_slice).unwrap();
    //     }

    //     Ok(player)
    // }

    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, String> {
        let name = unsafe { CStr::from_ptr(bytes[12..44].as_ptr() as *const i8) };

        Ok(Player {
            tribe: Tribe::try_from(LittleEndian::read_u32(&bytes[0..4]))?,
            start_pos: (
                LittleEndian::read_u32(&bytes[4..8]),  // x
                LittleEndian::read_u32(&bytes[8..12]), // y
            ),
            name: name.to_str().unwrap().to_string(),
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
    Free,
    Human,
    Computer,
}

impl TryFrom<u8> for PlayerType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => PlayerType::Free,
            1 => PlayerType::Human,
            2 => PlayerType::Computer,
            _ => return Err("No player type found for given value!"),
        })
    }
}
