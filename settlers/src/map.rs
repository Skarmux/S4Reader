#[derive(Debug)]
pub struct Map {
    pub objects: Vec<u32>,
    pub settlers: Vec<u32>,
    pub buildings: Vec<u32>,
    pub stacks: Vec<u32>,
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
