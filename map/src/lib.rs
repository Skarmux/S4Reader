#[allow(warnings)]
mod bindings;
use bindings::exports::s4::files::load::{Map, Info, Gamemode, Guest, ResourceRichness};
use bindings::s4::compression::decompress::decompress;
use bindings::s4::encryption::decryption::decrypt;

use byteorder::{ReadBytesExt,LittleEndian as LE};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Cursor, Read};

struct Component;

#[derive(Debug)]
struct Header {
    section_id: u32,
    chunk_size: u32,
    decompressed_size: u32,
    _checksum: u32,
    _unknown0: u32,
    _unknown1: u32,
}

impl Guest for Component {
    fn load_map(_path: String) -> Result<Map, String> {
        todo!()
    }

    fn load_map_info(path: String) -> Result<Info, String> {
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|err| err.to_string())?;
        let mut reader = BufReader::<File>::new(file);

        #[cfg(debug_assertions)]
        {
            // verify that data has been loaded correctly from disc
            let checksum = reader.read_u32::<LE>().map_err(|err| err.to_string())?;
            // version for compatibility checking with base game and addons
            let version = reader.read_u32::<LE>().map_err(|err| err.to_string())?;
            println!("checksum: {:x}\nversion: {}", checksum, version);
        }

        #[cfg(not(debug_assertions))]
        reader.seek_relative(8);

        loop {
            let mut buffer: [u8;24] = [0;24];
            reader.read_exact(&mut buffer).map_err(|err| err.to_string())?;
            let buffer = decrypt(&buffer);

            let mut cursor = Cursor::new(buffer);
            let header = Header {
                section_id: cursor.read_u32::<LE>().map_err(|err| err.to_string())?,
                chunk_size: cursor.read_u32::<LE>().map_err(|err| err.to_string())?,
                decompressed_size: cursor.read_u32::<LE>().map_err(|err| err.to_string())?,
                _checksum: cursor.read_u32::<LE>().map_err(|err| err.to_string())?,
                _unknown0: cursor.read_u32::<LE>().map_err(|err| err.to_string())?,
                _unknown1: cursor.read_u32::<LE>().map_err(|err| err.to_string())?,
            };

            dbg!(&header);

            if header.section_id == 1 {
                let mut chunk: Vec<u8> = Vec::with_capacity(header.chunk_size as usize);
                reader.read_exact(&mut chunk).map_err(|err| err.to_string())?;

                let data = decompress(&mut chunk).map_err(|err| err.to_string())?;
                debug_assert!(data.len() == header.decompressed_size as usize);

                let mut cursor = Cursor::new(data);
                let gamemode = match cursor.read_u32::<LE>().map_err(|err| err.to_string())? {
                    0 => Gamemode::Versus,
                    1 => Gamemode::Single,
                    2 => Gamemode::Coop,
                    v => panic!("invalid gamemode: {}", v),
                };

                let player_limit = cursor.read_u32::<LE>().map_err(|err| err.to_string())? as u8;
                debug_assert!(player_limit > 0 && player_limit < 9);

                let resource_richness =
                    match cursor.read_u32::<LE>().map_err(|err| err.to_string())? {
                        0 => None,
                        1 => Some(ResourceRichness::Low),
                        2 => Some(ResourceRichness::Medium),
                        3 => Some(ResourceRichness::High),
                        v => panic!("invalid resource richness: {}", v),
                    };
                let map_size = cursor.read_u16::<LE>().map_err(|err| err.to_string())? as u32;

                return Ok(Info {
                    gamemode,
                    player_limit,
                    resource_richness,
                    map_size,
                });
            }

            if let Err(e) = reader.seek_relative(header.chunk_size as i64) {
                return Err(e.to_string());
            }
        }
    }
}

bindings::export!(Component with_types_in bindings);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_load_map_info() {
        let info = Component::load_map_info("./Aeneas.map".to_string()).unwrap();
        assert_eq!(info.gamemode, Gamemode::Single);
        assert_eq!(info.player_limit, 8);
        assert_eq!(info.resource_richness, Some(ResourceRichness::Low));
        assert_eq!(info.map_size, 640);
    }
}