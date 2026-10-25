//! Stage S30's runner (Sapphire_Native.md §38): the state's layout listing, and states on the player's BIOS.
//!
//! `cargo run --release --example s30 -- layout <file>` writes version 2's listing, a console's at power-on.
//!
//! `cargo run --release --example s30 -- bios <bios> <americas|europe|japan> <span> <frame>...` starts one console on
//! a BIOS given by path and runs it past the last frame named; at each, its state is saved and loaded into a console
//! just made on the same BIOS, both run `span` frames, and every frame's picture and sound and the state at the end are
//! compared. Only whether they are equal is printed, and where states differ the names of the blocks' fields, never a
//! value, an offset into a memory, a program counter or an instruction (Sapphire_Plan.md §1.4). The image is handed to
//! the machines and never read here. The times of each save and load are printed.

use std::time::Instant;

use emusen_native::core::Core;
use emusen_native::ffi::StateMachine;
use sapphire::machine::{Machine, Market};
use sapphire::media::Media;

fn console(market: Market, bios: Option<&[u8]>) -> Machine {
    let mut m = Machine::new(Media::read(&sapphire::media::program(&[0], 0x0010_0000)).unwrap(), market, [Some("ps2.dualshock2"), None]);
    if let Some(b) = bios {
        m.insert_bios(b);
    }
    m
}

fn save(m: &Machine) -> (Vec<u8>, f64) {
    let t = Instant::now();
    let mut s = vec![0; m.state_size()];
    m.save_state(&mut s).expect("the state saves");
    (s, t.elapsed().as_secs_f64() * 1e3)
}

/// The listing's fields whose bytes differ between two states of one layout: names only.
fn differing(layout: &str, a: &[u8], b: &[u8]) -> Vec<String> {
    layout
        .lines()
        .filter_map(|l| {
            let mut f = l.split(' ');
            let (at, len, name) = (f.next()?.parse::<usize>().ok()?, f.next()?.parse::<usize>().ok()?, f.nth(1)?);
            (a.get(at..at + len) != b.get(at..at + len)).then(|| name.to_string())
        })
        .collect()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a[1].as_str() {
        "layout" => std::fs::write(&a[2], console(Market::Americas, None).layout()).expect("the listing written"),
        "bios" => bios(&a[2..]),
        other => panic!("no mode {other}"),
    }
}

fn bios(a: &[String]) {
    let image = std::fs::read(&a[0]).expect("the BIOS");
    let market = match a[1].as_str() {
        "europe" => Market::Europe,
        "japan" => Market::Japan,
        _ => Market::Americas,
    };
    let span: i64 = a[2].parse().unwrap();
    let points: Vec<i64> = a[3..].iter().map(|p| p.parse().unwrap()).collect();
    let mut m = console(market, Some(&image));
    let started = Instant::now();
    let mut chunk = vec![0i16; 1 << 16];
    for &p in &points {
        while m.frames < p {
            m.advance();
            m.drain_audio(&mut chunk, usize::MAX);
        }
        let (s, saved_ms) = save(&m);
        let mut b = console(market, Some(&image));
        let t = Instant::now();
        b.load_state(&s).expect("the state loads");
        let loaded_ms = t.elapsed().as_secs_f64() * 1e3;
        let mut c = console(market, Some(&image));
        c.load_state(&s).expect("the state loads");
        let (mut pictures, mut sound) = (0, 0);
        for _ in 0..span {
            m.advance();
            b.advance();
            pictures += usize::from(m.frame() != b.frame() || m.frame_info() != b.frame_info());
            let na = m.drain_audio(&mut chunk, usize::MAX);
            let heard = chunk[..2 * na].to_vec();
            let nb = b.drain_audio(&mut chunk, usize::MAX);
            sound += usize::from(heard[..] != chunk[..2 * nb]);
        }
        let (sa, _) = save(&m);
        let (sb, _) = save(&b);
        let fields = if sa == sb { Vec::new() } else { differing(&m.layout(), &sa, &sb) };
        let (sc, _) = save(&c);
        println!(
            "frame {p}: state {} bytes, saved in {saved_ms:.1} ms, loaded in {loaded_ms:.1} ms; save, load, save {}; over {span} frames pictures differing {pictures}, sound differing {sound}, state at frame {} {}; {:.0} s",
            s.len(),
            if sc == s { "equal" } else { "differing" },
            p + span,
            if fields.is_empty() { "equal".to_string() } else { format!("differing in {}", fields.join(", ")) },
            started.elapsed().as_secs_f64()
        );
    }
}
