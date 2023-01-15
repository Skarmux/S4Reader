use byteorder::ReadBytesExt;
use byteorder::{ByteOrder, LittleEndian};


#[derive(PartialEq, Debug)]
pub struct GeneralInformation {
    pub game_type: u32,  
    pub player_count: u32,  
    pub start_resources: u32,  
    pub map_size: u32
}

impl GeneralInformation {
    pub fn from_le_bytes(bytes: &[u8]) -> GeneralInformation {
        assert!(bytes.len() == 24);
        Self {
            game_type: LittleEndian::read_u32(&bytes[..4]),
            player_count: LittleEndian::read_u32(&bytes[4..8]),
            start_resources: LittleEndian::read_u32(&bytes[8..12]),
            map_size: LittleEndian::read_u32(&bytes[12..16])
        }
    }
}