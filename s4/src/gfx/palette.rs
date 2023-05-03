use byteorder::{LittleEndian as LE, ReadBytesExt};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;

// .p24 .p25 .p26 .p44 .p45 .p46
// pa5 pa6

#[derive(Debug, Default)]
pub struct Palette {
    colors: Vec<u16>,
}

impl Palette {
    pub fn from_reader(mut reader: impl Read) -> io::Result<Self> {
        let mut palette = Palette { colors: Vec::new() };

        while let Ok(color) = reader.read_u16::<LE>() {
            palette.colors.push(color);
        }

        Ok(palette)
    }
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).open(path.as_ref())?;
        let mut reader = BufReader::<File>::new(file);

        let _magic = reader.read_u32::<LE>()?;
        let _unknown0 = reader.read_u32::<LE>()?;
        let _unknown1 = reader.read_u32::<LE>()?;
        let _unknown2 = reader.read_u32::<LE>()?;
        let _unknown3 = reader.read_u32::<LE>()?;

        Palette::from_reader(reader)
    }
}
