//! The access speeds measured three ways: synthetic ROMs whose loop counts in WRAM, run on the machine, against the
//! documents' cycle table, and against Mesen through the reference probe when it is there. See VenusRT_Native.md §12.4.

use crate::machine::Machine;

/// Where the loop's code runs, and the one long read each iteration makes.
#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub name: &'static str,
    pub code: Code,
    pub data: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    SlowRom,
    FastRom,
    Wram,
}

pub const VARIANTS: [Variant; 9] = [
    Variant { name: "slow ROM, ROM data", code: Code::SlowRom, data: 0x00_8000 },
    Variant { name: "slow ROM, WRAM data", code: Code::SlowRom, data: 0x7E_0010 },
    Variant { name: "slow ROM, $2000 data", code: Code::SlowRom, data: 0x00_2000 },
    Variant { name: "slow ROM, $4100 data", code: Code::SlowRom, data: 0x00_4100 },
    Variant { name: "slow ROM, $4300 data", code: Code::SlowRom, data: 0x00_4300 },
    Variant { name: "slow ROM, $6000 data", code: Code::SlowRom, data: 0x00_6000 },
    Variant { name: "fast ROM, fast ROM data", code: Code::FastRom, data: 0x80_8000 },
    Variant { name: "fast ROM, slow ROM data", code: Code::FastRom, data: 0x00_8000 },
    Variant { name: "WRAM, WRAM data", code: Code::Wram, data: 0x7E_0010 },
];

/// The loop: LDA long, then a 24-bit counter at $00:0000 counted with INC dp and BNE, native mode, 8-bit A.
fn the_loop(data: u32) -> Vec<u8> {
    vec![0xAF, data as u8, (data >> 8) as u8, (data >> 16) as u8, 0xE6, 0x00, 0xD0, 0xF8, 0xE6, 0x01, 0xD0, 0xF4, 0xE6, 0x02, 0x80, 0xF0]
}

/// A LoROM image: native mode, MEMSEL set for fast code, the counter cleared, the loop placed and jumped to.
pub fn image(v: Variant) -> Vec<u8> {
    let mut rom = vec![0xEAu8; 0x8000];
    let mut p = vec![0x78, 0x18, 0xFB];
    if v.code == Code::FastRom {
        p.extend([0xA9, 0x01, 0x8D, 0x0D, 0x42]);
    }
    p.extend([0x64, 0x00, 0x64, 0x01, 0x64, 0x02]);
    let body = the_loop(v.data);
    let target: u32 = match v.code {
        Code::SlowRom => 0x00_8100,
        Code::FastRom => 0x80_8100,
        Code::Wram => 0x7E_2000,
    };
    if v.code == Code::Wram {
        for (i, &b) in body.iter().enumerate() {
            let a = target + i as u32;
            p.extend([0xA9, b, 0x8F, a as u8, (a >> 8) as u8, (a >> 16) as u8]);
        }
    } else {
        rom[0x100..0x100 + body.len()].copy_from_slice(&body);
    }
    p.extend([0x5C, target as u8, (target >> 8) as u8, (target >> 16) as u8]);
    rom[..p.len()].copy_from_slice(&p);
    rom[0x7FC0..0x7FD5].copy_from_slice(b"VENUSRT SPEED TEST   ");
    rom[0x7FD5] = if v.code == Code::FastRom { 0x30 } else { 0x20 };
    rom[0x7FD7] = 0x08;
    rom[0x7FDC..0x7FE0].copy_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
    rom[0x7FFC] = 0x00;
    rom[0x7FFD] = 0x80;
    rom
}

/// Master clocks one iteration without a carry takes by fullsnes's table and the datasheet's cycles: eight code bytes
/// (LDA long's four, two for INC dp, two for BNE), the long read, INC's read, internal cycle and write in WRAM, and the
/// taken branch's internal cycle.
pub fn documented_iteration(v: Variant) -> u64 {
    let code = match v.code {
        Code::SlowRom | Code::Wram => 8,
        Code::FastRom => 6,
    };
    let data = match v.data {
        0x00_2000 | 0x00_4300 => 6,
        0x00_4100 => 12,
        0x80_8000 => 6,
        _ => 8,
    };
    8 * code + data + 8 + 6 + 8 + 6
}

pub fn counter(wram: &[u8]) -> u64 {
    wram[0] as u64 | (wram[1] as u64) << 8 | (wram[2] as u64) << 16
}

/// The counter's growth from frame `a` to frame `b` on the machine.
pub fn growth_on_machine(v: Variant, a: u64, b: u64) -> u64 {
    let mut m = Machine::load_rom(&image(v)).unwrap();
    while (m.total_frames() as u64) < a {
        m.run_frame();
    }
    let at_a = counter(&m.sys.wram);
    while (m.total_frames() as u64) < b {
        m.run_frame();
    }
    counter(&m.sys.wram) - at_a
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;

    /// Per frame, iterations the documents predict: the frame less its refresh, over one iteration, with the
    /// 256th iteration's carry spread in: the branch not taken, then INC $01 and its taken branch, 4 code bytes and 22 clocks.
    fn predicted_growth(v: Variant, frames: u64) -> f64 {
        let clocks = frames as f64 * (262.0 * 1364.0 - 2.0 - 262.0 * 40.0);
        let it = documented_iteration(v) as f64;
        let code = if v.code == Code::FastRom { 6.0 } else { 8.0 };
        clocks / (it + (4.0 * code + 22.0) / 256.0)
    }

    #[test]
    fn each_region_runs_at_the_documented_speed() {
        for v in VARIANTS {
            let got = growth_on_machine(v, 10, 70) as f64;
            let want = predicted_growth(v, 60);
            eprintln!("{:26} {} clocks an iteration: {got} iterations in 60 frames, {want:.1} predicted", v.name, documented_iteration(v));
            assert!((got - want).abs() / want < 0.002, "{}: {got} against {want}", v.name);
        }
    }

    fn probe() -> Option<(PathBuf, PathBuf)> {
        let probe = PathBuf::from(std::env::var_os("EMUSEN_MESEN_PROBE")?);
        let checkout = PathBuf::from(std::env::var_os("EMUSEN_MESEN_CHECKOUT")?);
        (probe.is_file() && checkout.is_dir()).then_some((probe, checkout))
    }

    // Mesen as a black box: the same images, the counter's growth over the same 300 frames.
    #[test]
    fn mesen_counts_as_the_machine_does() {
        let Some((probe, checkout)) = probe() else {
            eprintln!("EMUSEN_MESEN_PROBE or EMUSEN_MESEN_CHECKOUT unset, not run");
            return;
        };
        let dir = std::env::temp_dir().join(format!("venusrt-speed-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut report = String::new();
        let mut disagreements = 0;
        for (i, v) in VARIANTS.iter().enumerate() {
            let rom = dir.join(format!("speed-{i}.sfc"));
            std::fs::write(&rom, image(*v)).unwrap();
            let out = dir.join(format!("out-{i}"));
            std::fs::create_dir_all(&out).unwrap();
            let status = Command::new(&probe).current_dir(&checkout).arg(&rom).arg(&out).args(["300", "600", "300"]).output().unwrap();
            assert!(status.status.success(), "{}", String::from_utf8_lossy(&status.stderr));
            let wram = |f: u32| std::fs::read(out.join(format!("mesen_wram_f{f:05}.bin"))).unwrap();
            let mesen = counter(&wram(600)) - counter(&wram(300));
            let ours = growth_on_machine(*v, 300, 600);
            let diff = ours as i64 - mesen as i64;
            disagreements += (diff.unsigned_abs() > 2) as u32;
            report += &format!("{:26} VenusRT {ours:>8}  Mesen {mesen:>8}  difference {diff:+}\n", v.name);
        }
        eprint!("{report}");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(disagreements, 0, "{report}");
    }
}
