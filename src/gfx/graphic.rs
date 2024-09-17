use byteorder::{LittleEndian as LE, ReadBytesExt};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug, Default)]
pub struct Graphic {
    width: u16,
    height: u16,
    left: u16,
    top: u16,
    graphic_type: u8,
    flag0: u16,
    flag1: u32,
    data: Vec<u8>,
}

impl Graphic {
    pub fn from_reader(mut reader: impl Read) -> io::Result<Self> {
        let mut graphic = Graphic {
            ..Default::default()
        };

        let check_value = reader.read_u16::<LE>()?;

        if check_value > 860 {
            graphic.width = (check_value & 0xFF00) >> 8;
            graphic.height = check_value & 0x00FF;
            graphic.left = reader.read_u8()? as u16;
            graphic.top = reader.read_u8()? as u16;
            graphic.flag0 = reader.read_u16::<LE>()?;
            graphic.flag1 = reader.read_u16::<LE>()? as u32;
        } else {
            graphic.width = check_value;
            graphic.height = reader.read_u16::<LE>()?;
            graphic.left = reader.read_u16::<LE>()?;
            graphic.top = reader.read_u16::<LE>()?;
            graphic.graphic_type = reader.read_u8()?;
            graphic.flag0 = reader.read_u8()? as u16;
            graphic.flag1 = reader.read_u32::<LE>()?;
        }

        let data_length = (graphic.width as usize)
            .checked_mul(graphic.height as usize)
            .unwrap()
            .checked_mul(4)
            .unwrap();
        graphic.data.resize(data_length, 0);
        reader.read(&mut graphic.data)?; // TODO: Why does read_exact panic here?

        Ok(graphic)
    }
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).open(path.as_ref())?;
        let mut reader = BufReader::<File>::new(file);

        let _magic = reader.read_u32::<LE>()?;
        let _unknown0 = reader.read_u32::<LE>()?;
        let _unknown1 = reader.read_u32::<LE>()?;
        let _unknown2 = reader.read_u32::<LE>()?;
        let _unknown3 = reader.read_u32::<LE>()?;

        Graphic::from_reader(reader)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::env;

    #[test]
    fn load_file() {
        let install_root = env::var("INSTALL_ROOT").unwrap();
        let file_path = format!("{0}/{1}",install_root, "Gfx/0.gfx");
        let gfx_file = Graphic::from_file(file_path).unwrap();
    }
}
