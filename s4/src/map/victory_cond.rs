use crate::map::building::BuildingType;
use crate::stack::StackType;
use byteorder::{ByteOrder, LittleEndian};
use std::fmt;

type Pos = (u16, u16);

pub struct VictoryCondition {
    players_defeated: PlayersDefeated,
    buildings_destroyed: BuildingsDestroyed,
    grounds_claimed: GroundsClaimed,
    time_endured: TimesEndured,
    resources_acquired: ResourcesAcquired,
}

impl VictoryCondition {
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        Ok(VictoryCondition {
            players_defeated: PlayersDefeated::from_le_bytes(&bytes[0..9])?, // 9 bytes
            buildings_destroyed: BuildingsDestroyed::from_le_bytes(&bytes[9..30])?, // 21 bytes
            grounds_claimed: GroundsClaimed::from_le_bytes(&bytes[30..56])?, // 26 bytes
            time_endured: TimesEndured::from_le_bytes(&bytes[56..73])?,      // 17 bytes
            resources_acquired: ResourcesAcquired::from_le_bytes(&bytes[73..83])?, // 10 bytes
        })
    }
}

impl fmt::Debug for VictoryCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VictoryCondition")
            .field("players_defeated", &self.players_defeated)
            .field("buildings_destroyed", &self.buildings_destroyed)
            .field("grounds_claimed", &self.grounds_claimed)
            .field("time_endured", &self.time_endured)
            .field("resources_acquired", &self.resources_acquired)
            .finish()
    }
}

struct PlayersDefeated {
    active: bool,
    players: [bool; 8],
}

impl fmt::Debug for PlayersDefeated {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_list()
            .entries(
                self.players
                    .iter()
                    .enumerate()
                    .map(|(i, should_be_defeated)| {
                        format!("Player: {}, To be defeated: {}", i + 1, should_be_defeated)
                    }),
            )
            .finish()
    }
}

impl PlayersDefeated {
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut players = [false; 8];

        for (i, should_be_defeated) in bytes[1..9].iter().enumerate() {
            players[i] = *should_be_defeated == 1;
        }

        Ok(PlayersDefeated {
            active: bytes[0] == 1,
            players,
        })
    }
}

struct BuildingsDestroyed {
    active: bool,
    buildings: [(Option<u8>, Option<BuildingType>); 10],
}

impl fmt::Debug for BuildingsDestroyed {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_list()
            .entries(self.buildings.iter().map(|(player, building)| {
                format!("Player: {:?}, Building: {:?}", player, building)
            }))
            .finish()
    }
}

impl BuildingsDestroyed {
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut buildings = [(None, None); 10];

        for (i, chunk) in bytes[1..21].chunks_exact(2).into_iter().enumerate() {
            let player = match chunk[0] {
                255 => None,
                n => Some(n),
            };
            buildings[i] = (player, BuildingType::try_from(chunk[1]).ok());
        }

        Ok(BuildingsDestroyed {
            active: bytes[0] == 1,
            buildings,
        })
    }
}

struct GroundsClaimed {
    active: bool,
    grounds: [(bool, Pos); 5],
}

impl fmt::Debug for GroundsClaimed {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_list()
            .entries(self.grounds.iter().map(|(to_be_claimed, pos)| {
                format!(
                    "ToBeClaimed: {}, Pos: [{}x, {}y]",
                    to_be_claimed, pos.0, pos.1
                )
            }))
            .finish()
    }
}

impl GroundsClaimed {
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut grounds = [(false, (0, 0)); 5];

        for (i, chunk) in bytes[1..9].chunks_exact(5).into_iter().enumerate() {
            grounds[i] = (
                chunk[0] == 1,
                (
                    LittleEndian::read_u16(&chunk[1..3]),
                    LittleEndian::read_u16(&chunk[3..5]),
                ),
            );
        }

        Ok(GroundsClaimed {
            active: bytes[0] == 1,
            grounds,
        })
    }
}

struct TimesEndured {
    active: bool,
    times_per_player: [u16; 8],
}

impl fmt::Debug for TimesEndured {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_list()
            .entries(
                self.times_per_player
                    .iter()
                    .enumerate()
                    .map(|(i, time)| format!("Player: {}, TimeEndured: {} minutes", i + 1, time)),
            )
            .finish()
    }
}

impl TimesEndured {
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut times_per_player = [0; 8];

        for (i, chunk) in bytes[1..17].chunks_exact(2).into_iter().enumerate() {
            times_per_player[i] = LittleEndian::read_u16(chunk);
        }

        Ok(TimesEndured {
            active: bytes[0] == 1,
            times_per_player,
        })
    }
}

struct ResourcesAcquired {
    active: bool,
    amounts_needed: [(u16, Option<StackType>); 3],
}

impl fmt::Debug for ResourcesAcquired {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_list()
            .entries(
                self.amounts_needed
                    .iter()
                    .map(|(amount, stack)| format!("Amount: {}, Resource: {:?}", amount, stack)),
            )
            .finish()
    }
}

impl ResourcesAcquired {
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut amounts_needed = [(0, None); 3];

        for (i, chunk) in bytes.chunks_exact(3).into_iter().enumerate() {
            amounts_needed[i] = (
                LittleEndian::read_u16(&chunk[0..2]),
                StackType::try_from(chunk[2]).ok(),
            );
        }

        Ok(ResourcesAcquired {
            active: bytes[0] == 1,
            amounts_needed,
        })
    }
}
