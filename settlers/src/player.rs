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
