//! How busy a cartridge's processor is and where the S-CPU runs meanwhile, instruction by instruction over some
//! frames: the S-CPU's instructions fetched from the cartridge (ROM or its RAM) and from WRAM while the SA-1 or the GSU
//! runs. `chip_activity <rom> [frames]`; a NEC DSP's firmware from EMUSEN_VENUSRT_DSP. See VenusRT_Native.md §27.4.

#[path = "common/load.rs"]
mod load;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let frames: u64 = a.get(2).and_then(|f| f.parse().ok()).unwrap_or(600);
    let mut m = load::machine(&a[1]);
    let (mut total, mut busy, mut busy_cart, mut busy_wram) = (0u64, 0u64, 0u64, 0u64);
    let start = m.sys.timing.frame;
    while m.sys.timing.frame < start + frames {
        let c = &m.sys.cart;
        let active = c.sa1.as_ref().is_some_and(|s| s.ccnt & 0x60 == 0 && !s.cpu.waiting && !s.cpu.stopped)
            || c.gsu.as_ref().is_some_and(|g| g.sfr & 0x20 != 0);
        let (bank, pc) = (m.cpu.pbr, m.cpu.pc);
        let in_wram = bank & 0xFE == 0x7E || (bank & 0x40 == 0 && pc < 0x2000);
        total += 1;
        if active {
            busy += 1;
            if in_wram {
                busy_wram += 1;
            } else {
                busy_cart += 1;
            }
        }
        m.step();
    }
    let pct = |n: u64| 100.0 * n as f64 / total.max(1) as f64;
    println!(
        "{}: {total} S-CPU instructions over {frames} frames; {busy} ({:.1}%) while the cartridge's processor runs, of them {busy_cart} ({:.1}%) fetched from the cartridge and {busy_wram} ({:.1}%) from WRAM",
        a[1].rsplit('/').next().unwrap(),
        pct(busy),
        pct(busy_cart),
        pct(busy_wram)
    );
}
