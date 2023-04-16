#![allow(dead_code, unused)]

use crate::player::*;

use std::ffi::CStr;
use std::fmt;

#[derive(Debug)]
pub struct TeamInfo {
    constellation_name: String,
    team_player_data: Vec<TeamPlayerData>,
}

impl TeamInfo {
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let constellation_name = unsafe { CStr::from_ptr(bytes.as_ptr() as *const i8) };

        let mut team_player_data = Vec::<TeamPlayerData>::new();
        let chunk_iter = bytes[33..].chunks_exact(2);
        for chunk in chunk_iter {
            let data = TeamPlayerData::from_le_bytes(chunk)?;
            team_player_data.push(data);
        }

        Ok(TeamInfo {
            constellation_name: constellation_name.to_str().unwrap().to_string(),
            team_player_data,
        })
    }
}

#[derive(Clone, Copy)]
struct TeamPlayerData {
    team: u8,
    player_type: PlayerType,
}

impl fmt::Debug for TeamPlayerData {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "Team {}: {:?}", self.team, self.player_type)
    }
}

impl TeamPlayerData {
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        Ok(TeamPlayerData {
            team: bytes[0],
            player_type: PlayerType::try_from(bytes[1])?,
        })
    }
}
