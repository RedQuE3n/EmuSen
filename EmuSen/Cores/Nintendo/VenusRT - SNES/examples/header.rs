//! The header VenusRT chooses for each file given: map, map mode, score and title, tab-separated (VenusRT_Native.md §12.3).

fn main() {
    for path in std::env::args().skip(1) {
        let image = std::fs::read(&path).expect("a ROM");
        match venusrt::cart::Cartridge::new(&image) {
            Some(c) => println!("{path}\t{:?}\t{:02X}\t{}\t{}", c.header.map, c.header.map_mode, c.header.score, c.header.title),
            None => println!("{path}\tnone"),
        }
    }
}
