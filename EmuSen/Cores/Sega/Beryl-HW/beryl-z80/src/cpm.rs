//! ZEXDOC and ZEXALL, whose CRCs were taken on real Z80s, under a CP/M shim: the program at $0100, BDOS's functions
//! 2 and 9 answered at $0005, the top of memory at $0006, and the warm boot at $0000 ending the run. The programs are
//! found through `EMUSEN_BERYL_ZEX` (a folder holding `zexdoc.com` and `zexall.com`); without it they pass unrun.
//! Beryl_Z80.md §6 is the protocol.

use crate::{Bus, Z80};

pub const ZEX_VARIABLE: &str = "EMUSEN_BERYL_ZEX";

struct Cpm {
    ram: Vec<u8>,
    t: u64,
}

impl Bus for Cpm {
    fn fetch(&mut self, a: u16, _: u16) -> u8 {
        self.t += 4;
        self.ram[a as usize]
    }
    fn read(&mut self, a: u16) -> u8 {
        self.t += 3;
        self.ram[a as usize]
    }
    fn write(&mut self, a: u16, v: u8) {
        self.t += 3;
        self.ram[a as usize] = v;
    }
    fn input(&mut self, _: u16) -> u8 {
        self.t += 4;
        0xFF
    }
    fn output(&mut self, _: u16, _: u8) {
        self.t += 4;
    }
    fn idle(&mut self, t: u32) {
        self.t += t as u64;
    }
    fn int_line(&mut self) -> bool {
        false
    }
    fn nmi_edge(&mut self) -> bool {
        false
    }
    fn acknowledge(&mut self) -> u8 {
        0xFF
    }
}

/// A CP/M program run to its warm boot: what it printed, and the T-states it took.
pub fn run(program: &[u8]) -> (String, u64) {
    let mut bus = Cpm { ram: vec![0; 0x10000], t: 0 };
    bus.ram[0x100..0x100 + program.len()].copy_from_slice(program);
    bus.ram[5] = 0xC9;
    bus.ram[7] = 0xF0;
    let mut cpu = Z80::new();
    cpu.regs.pc = 0x100;
    cpu.regs.sp = 0xF000;
    let mut out = String::new();
    loop {
        match cpu.regs.pc {
            0 => break,
            5 => {
                let g = cpu.regs;
                match g.bc as u8 {
                    2 => out.push(g.de as u8 as char),
                    9 => {
                        let mut a = g.de;
                        while bus.ram[a as usize] != b'$' {
                            out.push(bus.ram[a as usize] as char);
                            a = a.wrapping_add(1);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        cpu.step(&mut bus);
    }
    (out, bus.t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exerciser(name: &str) {
        let Some(dir) = std::env::var_os(ZEX_VARIABLE) else {
            eprintln!("{ZEX_VARIABLE} is not set: {name} not run");
            return;
        };
        let program = std::fs::read(std::path::Path::new(&dir).join(name)).expect("the program");
        let started = std::time::Instant::now();
        let (out, t) = run(&program);
        let text = out.replace('\r', "");
        eprintln!("{text}");
        eprintln!("{name}: {t} T-states in {:.1} s", started.elapsed().as_secs_f64());
        assert!(text.contains("Tests complete"), "{name} did not finish");
        assert!(!text.contains("ERROR"), "{name} found a CRC that differs");
    }

    #[test]
    fn zexdoc() {
        exerciser("zexdoc.com");
    }

    #[test]
    fn zexall() {
        exerciser("zexall.com");
    }
}
