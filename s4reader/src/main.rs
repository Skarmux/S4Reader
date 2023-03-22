use settlers;

mod ara_crypt;
mod bitreader;
mod bitwriter;
mod decompress;
mod map_file;

fn main() {
    let map_file = map_file::MapFile::open("map/Aeneas.map").unwrap();
    dbg!(&map_file);
    // let map = map_file.load().unwrap();
    // dbg!(&map);
}
