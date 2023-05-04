use std::ffi::OsString;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use byteorder::{ReadBytesExt, LE};

#[derive(Debug)]
struct FileDescriptor {
    offset: u32,
    size: u32,
    size_decrypt: u32,
    path: PathBuf,
    compressed: bool,
}

#[derive(Debug)]
pub struct Archive {
    path: PathBuf,
    archive: Vec<FileDescriptor>,
}

impl Archive {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {

        let file = OpenOptions::new().read(true).open(&path)?;

        let mut reader = BufReader::<File>::new(file);

        reader.seek(io::SeekFrom::End(-4))?;
        let header_offset = reader.read_u32::<LE>()?; // 22828737 (Letztes Offset bei: 22824781)
        
        reader.seek(io::SeekFrom::Start(header_offset as u64))?;

        let _length = reader.read_u32::<LE>()?; // 8380
        let _unknown = reader.read_u32::<LE>()?; // 4096 (Readonly flag?)
        let path_list_length = reader.read_u32::<LE>()?;
        let path_count = reader.read_u32::<LE>()? as usize;
        let file_list_length = reader.read_u32::<LE>()?;
        let file_count = reader.read_u32::<LE>()? as usize;

        // path name list [path name list length]
        let mut buf = vec![0; path_list_length as usize];
        reader.read(&mut buf)?;
        let path_names = String::from_utf8(buf).expect("valid utf-8 string");
        let paths: Vec<PathBuf> = path_names
            .split_terminator('\0')
            .map(|s| PathBuf::from(s) )
            .collect();
        if paths.len() != path_count {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid path data"));
        }

        // file name list [file name list length]
        let mut buf = vec![0; file_list_length as usize];
        reader.read(&mut buf)?;
        let file_names = String::from_utf8(buf).expect("valid utf-8 string");
        let files: Vec<OsString> = file_names
            .split_terminator('\0')
            .map(|str| OsString::from_str(str).unwrap())
            .collect();
        if files.len() != file_count {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid file data"));
        }

        let mut archive = Vec::new();

        files.iter().try_for_each(|name| -> io::Result<()> {

            let offset = reader.read_u32::<LE>()?;
            let size = reader.read_u32::<LE>()?;
            let size_decrypt = reader.read_u32::<LE>()?;
            let path_index = reader.read_u16::<LE>()? as usize;
            reader.seek_relative(2)?; // skip unknown
            let compressed = reader.read_u32::<LE>()? == 1;
            
            let base_path = paths.get(path_index).unwrap();
            let path = base_path.join(name.clone());

            let virtual_file = FileDescriptor {
                offset,
                size,
                size_decrypt,
                path,
                compressed,                
            };

            let _checksum = reader.read_u32::<LE>()?;

            archive.push(virtual_file);

            Ok(())
        })?;

        let loader = Archive {
            path: PathBuf::from(path.as_ref()),
            archive,
        };

        Ok(loader)
    }
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<Option<Vec<u8>>> {
        let virtual_file = self.archive.iter().find(|f| f.path == path.as_ref());

        match virtual_file {
            None => Ok(None),
            Some(f) => {
                let file = OpenOptions::new().read(true).open(&self.path)?;
                let mut reader = BufReader::<File>::new(file);
        
                reader.seek(SeekFrom::Start(f.offset as u64))?;
        
                let mut buf = vec![0;f.size as usize];
        
                reader.read_exact(&mut buf)?;
        
                if f.compressed {
                    todo!("decompression not implemented");
                }

                Ok(Some(buf))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn initialize() {
        let install_root: PathBuf = [
            "c:\\",
            "Program Files (x86)",
            "GOG Galaxy",
            "Games",
            "Settlers 4 Gold",
        ]
        .iter()
        .collect();
        let gfx_loader = Archive::new(install_root);
        assert!(gfx_loader.is_ok());
    }
}
