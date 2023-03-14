mod ara_crypt;
mod decompress;
mod map;

use map::MapFile;

fn main() {
    let map_file = MapFile::open("map/Aeneas.map").unwrap();
    dbg!(&map_file);
    // let map = map_file.load().unwrap();
    // dbg!(&map);
}
