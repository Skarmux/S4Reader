use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug)]
pub struct JobIndexList {
    indices: Vec<u32>,
}

impl JobIndexList {
    pub fn from_reader(mut reader: impl Read) -> io::Result<Self> {
        let mut job_index_list = JobIndexList {
            indices: Vec::new(),
        };

        while let Ok(index) = reader.read_u32::<LittleEndian>() {
            job_index_list.indices.push(index);
        }

        Ok(job_index_list)
    }
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).open(path.as_ref())?;
        let mut reader = BufReader::<File>::new(file);

        let _magic = reader.read_u32::<LittleEndian>().unwrap();
        let _unknown0 = reader.read_u32::<LittleEndian>().unwrap();
        let _unknown1 = reader.read_u32::<LittleEndian>().unwrap();
        let _unknown2 = reader.read_u32::<LittleEndian>().unwrap();
        let _unknown3 = reader.read_u32::<LittleEndian>().unwrap();

        Self::from_reader(reader)
    }
}
