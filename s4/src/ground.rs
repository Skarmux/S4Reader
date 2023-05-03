
use byteorder::{ByteOrder, LittleEndian};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Ground {
    pub height: u8,
    pub ground_type: GroundType,
    pub flags: u16,
}

impl Ground {
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, String> {
        Ok(Ground {
            height: bytes[0],
            ground_type: GroundType::try_from(bytes[1]).unwrap(),
            flags: LittleEndian::read_u16(&bytes[2..4]),
        })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, TryFromPrimitive)]
pub enum GroundType {
    Water1 = 0,
    Water2,
    Water3,
    Water5,
    Water6,
    Water7,
    Water8,
    WaterBeach,

    Grass = 16,
    GrassMountain,
    GrassIsland,
    GrassDesert = 20,
    GrassSwamp,
    GrassMud = 23,
    GrassDark,
    GrassWeird,
    GrassDusty = 28,
    GrassPavement,

    Mountain = 32,
    MountainGrass,
    MountainSnow = 35,

    Beach = 48,

    Desert = 64,
    DesertGrass,

    Swamp = 80,
    SwampGrass,

    River1 = 96,
    River2,
    River3,
    River4,

    UnidentifiedGrass1 = 112,
    UnidentifiedGrass2,
    UnidentifiedGrass3,

    Snow = 128,
    SnowMountain,

    Mud = 144,
    MudGrass,

    Glitched = 250,
}
