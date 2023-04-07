use settlers;

mod ara_crypt;
mod bitreader;
mod bitwriter;
mod decompress;
mod map;

fn main() {
    let map = map::file::Map::open("s4reader/map/Aeneas.map").unwrap();
    dbg!(&map);
    // let map = map_file.load().unwrap();
    // dbg!(&map);
}
