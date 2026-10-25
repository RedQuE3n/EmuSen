//! Writes stage S20's programs for the referees: `s20 <folder>` makes `<name>.elf` and `<name>.expected` (one record
//! word a line in hexadecimal, `-` where any value is expected) for each, and for each display program
//! `<name>.picture`, the picture the manual's model gives (RGBA rows, its width and height first as two little-endian
//! words), to set beside a referee's (Sapphire_Native.md §29.8); for `s10-fieldmark` the pictures of its two buffers
//! (Sapphire_Native.md §36.8).

#[path = "../tests/s6/mod.rs"]
mod s6;
#[path = "../tests/s11/mod.rs"]
mod s11;
#[path = "../tests/s16/mod.rs"]
mod s16;
#[path = "../tests/s20/mod.rs"]
mod s20;

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).expect("a folder"));
    std::fs::create_dir_all(&dir).expect("the folder");
    for p in s20::all() {
        let name = p.name;
        let (source, expected) = p.finish();
        let words = emusen_mips::assemble(&source, s6::ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        std::fs::write(dir.join(format!("{name}.elf")), sapphire::media::program(&words, s6::ORIGIN)).expect("written");
        let text: String = expected.iter().map(|w| w.map_or("-\n".to_string(), |w| format!("{w:08x}\n"))).collect();
        std::fs::write(dir.join(format!("{name}.expected")), text).expect("written");
        let graded = expected.iter().filter(|w| w.is_some()).count();
        println!("{name}: {} words of code and data, {} record words, {graded} graded", words.len(), expected.len());
    }
    for script in s20::displays() {
        let gs = s20::memory(&script);
        let interlaced = gs.privileged.get(&s16::ga::SMODE2).is_some_and(|v| v & 1 == 1);
        let (_, _, w, _, h) = s20::manual::rectangle(&gs);
        let mut bytes = [w.to_le_bytes(), h.to_le_bytes()].concat();
        bytes.extend(s20::manual::picture(&gs, interlaced));
        std::fs::write(dir.join(format!("{}.picture", script.name)), bytes).expect("written");
    }
    // `s10-fieldmark`: the woven picture of each buffer alone, the even field's and the odd field's, and the
    // rectangle's place (x0 in VCK, its pitch, width, first raster, height), so that a referee's picture is set beside
    // the two ways of laying the fields (Sapphire_Native.md §36.8).
    let mut gs = s20::memory(&s20::fieldmark_script());
    for (fbp, which) in [(0, "even"), (1, "odd")] {
        gs.write_privileged(s16::ga::DISPFB1, s20::dispfb(fbp, 1, s16::manual::CT32, 0, 0));
        let (x0, pitch, w, y0, h) = s20::manual::rectangle(&gs);
        let mut bytes = [w.to_le_bytes(), h.to_le_bytes()].concat();
        bytes.extend(s20::manual::picture(&gs, true));
        std::fs::write(dir.join(format!("s10-fieldmark.{which}.picture")), bytes).expect("written");
        println!("s10-fieldmark.{which}: rectangle x0 {x0} pitch {pitch} width {w} first raster {y0} height {h}");
    }
}
