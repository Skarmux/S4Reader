use settlers;

mod ara_crypt;
mod bitreader;
mod bitwriter;
mod decompress;
mod map;

fn main() {
    map::file::Map::open("s4reader/map/Aeneas.map").unwrap();
}
