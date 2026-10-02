//! Runs a ROM twice, with the picture drawn and with it skipped, and compares the whole state after every frame.
//! `skip_check <rom> [frames]` prints the first frame at which the two part, or that they never did.

#[path = "common/load.rs"]
mod load;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("skip_check <rom> [frames]");
    let frames: u32 = args.next().and_then(|f| f.parse().ok()).unwrap_or(600);
    let mut drawn = load::machine(&path);
    let mut skipped = load::machine(&path);
    skipped.sys.ppu.skip = true;
    let (mut a, mut b) = (vec![0u8; drawn.state_size()], vec![0u8; skipped.state_size()]);
    let mut lit = 0u32;
    for frame in 0..frames {
        drawn.run_frame();
        skipped.run_frame();
        drawn.save_state(&mut a).unwrap();
        skipped.save_state(&mut b).unwrap();
        if a != b {
            let at = a.iter().zip(&b).position(|(x, y)| x != y).unwrap();
            println!("PARTED frame {frame} offset {at}\t{path}");
            std::process::exit(1);
        }
        lit += drawn.sys.ppu.picture().chunks_exact(4).any(|px| px[..3] != [0, 0, 0]) as u32;
    }
    println!("same {frames} frames, {lit} with a picture\t{path}");
}
