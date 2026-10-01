//! The header VenusRT chooses for each file given, and every candidate's score and reset-handler evidence
//! (VenusRT_Native.md §12.3 and §14.5): path, chosen map, mode, score, title, then one field per candidate.

fn main() {
    for path in std::env::args().skip(1) {
        let image = std::fs::read(&path).expect("a ROM");
        let rom = if image.len() % 1024 == 512 { &image[512..] } else { &image[..] };
        let Some(c) = venusrt::cart::Cartridge::new(&image) else {
            println!("{path}\tnone");
            continue;
        };
        let evidence: Vec<String> = venusrt::cart::candidates(rom)
            .into_iter()
            .map(|h| {
                let (map, score) = (h.map, h.score);
                let (writes, crashed) = venusrt::machine::Machine::reset_evidence(rom, h, 4000);
                format!("{map:?}:{score}:{writes}:{}", crashed as u8)
            })
            .collect();
        println!("{path}\t{:?}\t{:02X}\t{}\t{}\t{}", c.header.map, c.header.map_mode, c.header.score, c.header.title, evidence.join(" "));
    }
}
