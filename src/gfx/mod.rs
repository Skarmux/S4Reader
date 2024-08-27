mod direction_index_list;
pub mod graphic;
mod graphic_index_list;
mod job_index_list;
mod palette;
mod palette_index_list;

use std::io;
use super::io::archive::Archive;

type Image = Vec<u8>;

pub enum Resource {
    Gold,
}

pub enum Job {
    Carrier(Resource),
    Swordsman{health: i8},
    Bowman,
}

pub enum Direction {
    East, SouthEast, SouthWest, West, NorthWest, NorthEast
}

pub enum Animation {
    Move(Direction, Vec<Image>)
}

pub struct Settler {
    tribe: Tribe,
    job: Option<Job>,
}

pub enum Tribe {
    Roman, Viking, Mayan, Dark
}

pub enum Graphic {
    Image { top: Image },
    Animation(Vec<Image>),
}

pub enum Menu {
    Base,
    MissionCD,
    AddOn,
}

#[repr(u8)]
pub enum Kind {
    Menu = 0,
    Flags = 1,

    Stack = 3,
    Object = 5,
    Marker = 7,
    Animal = 8,

    AlphaEffect = 4,
    SolidEffect = 6,

    Interface(Tribe) = 9,
    // RomanInterface = 9,
    // VikingInterface = 19,
    // MayanInterface = 29,
    // DarkInterface = _, (missing)
    // TrojanInterface = 39,

    Building(Tribe) = 10,
    // RomanBuilding = 10,
    // VikingBuilding = 11,
    // MayaBuilding = 12,
    // DarkBuilding = 13,
    // TrojanBuilding = 14,

    MissionCDMenu = 18,
    AddOnMenu = 28,

    Settler(Tribe) = 20,
    // Roman(Settler) = 20,
    // Viking(Settler) = 21,
    // Mayan(Settler) = 22,
    // Dark(Settler) = 23,
    // Trojan(Settler) = 24,

    Vehicle(Tribe) = 30,
    // RomanVehicle = 30,
    // VikingVehicle = 31,
    // MayanVehicle = 32,
    // DarkVehicle = 33, (missing)
    // TrojanVehicle = 34,

    AddOnFlags = 35,
    AddOnAlphaEffect = 36,
    AddOnSolidEffect = 37,
}

pub fn load(_kind: Kind) -> io::Result<Resource> {

    // gfx.lib stores a register of all available graphical assets
    let _archive = Archive::new("g:\\GOG\\Settlers 4 Gold\\gfx.lib").unwrap();

    todo!()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_load() {
        let _ = load(Kind::Settler(Tribe::Roman));
    }
}
