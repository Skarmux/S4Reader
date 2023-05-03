use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug)]
pub struct GraphicIndexList(Vec<u32>);

impl GraphicIndexList {
    pub fn from_reader(mut reader: impl Read) -> io::Result<Self> {
        let mut indices = Vec::new();

        while let Ok(index) = reader.read_u32::<LittleEndian>() {
            indices.push(index);
        }

        Ok(GraphicIndexList(indices))
    }
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).open(path.as_ref())?;
        let mut reader = BufReader::<File>::new(file);

        let _magic = reader.read_u32::<LittleEndian>().unwrap();
        let _unknown0 = reader.read_u32::<LittleEndian>().unwrap();
        let _unknown1 = reader.read_u32::<LittleEndian>().unwrap();
        let _unknown2 = reader.read_u32::<LittleEndian>().unwrap();
        let _unknown3 = reader.read_u32::<LittleEndian>().unwrap();

        GraphicIndexList::from_reader(reader)
    }
}
