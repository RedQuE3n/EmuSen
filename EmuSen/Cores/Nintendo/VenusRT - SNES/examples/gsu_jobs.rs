//! The GSU's jobs frame by frame: how many times GO falls (a STOP or the S-CPU's abort) in each SNES frame, and the
//! master clocks it ran; with a directory, WRAM after each frame as well, for finding a game's own frame counter.
//! `gsu_jobs <rom> <frames> [dir]`. See VenusRT_Native.md §27.4.

#[path = "common/load.rs"]
mod load;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let frames: u64 = a[2].parse().unwrap();
    let dir = a.get(3);
    let mut m = load::machine(&a[1]);
    let go = |m: &venusrt::machine::Machine| m.sys.cart.gsu.as_ref().is_some_and(|g| g.sfr & 0x20 != 0);
    for f in 1..=frames {
        let (mut jobs, mut busy, mut was) = (0u32, 0u64, go(&m));
        let frame = m.sys.timing.frame;
        while m.sys.timing.frame == frame {
            let clock = m.sys.timing.clock;
            m.step();
            let now = go(&m);
            if was {
                busy += m.sys.timing.clock - clock;
            }
            if was && !now {
                jobs += 1;
            }
            was = now;
        }
        println!("{f} {jobs} {busy}");
        if let Some(d) = dir {
            std::fs::write(format!("{d}/venusrt_wram_f{f:05}.bin"), &m.sys.wram).unwrap();
        }
    }
}
