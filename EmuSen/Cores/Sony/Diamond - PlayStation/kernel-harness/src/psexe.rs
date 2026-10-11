//! Test programs: MIPS source assembled by emusen-mips into a PS-EXE (psx-spx "CDROM File Playstation EXE"), and its
//! side-load as Exec would start it. The source may use `li reg, value`, written here as LUI and ORI.

use crate::machine::Machine;

/// `li` expanded; everything else is emusen-mips's.
fn expand(source: &str) -> String {
    let mut out = String::new();
    for line in source.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("li ") {
            let (reg, value) = rest.split_once(',').expect("li reg, value");
            let v = value.trim();
            let v = if let Some(h) = v.strip_prefix("0x") {
                u32::from_str_radix(h, 16).unwrap()
            } else {
                v.parse::<i64>().unwrap() as u32
            };
            out += &format!("lui {reg}, 0x{:x}\nori {reg}, {reg}, 0x{:x}\n", v >> 16, v & 0xFFFF);
        } else {
            out += line;
            out.push('\n');
        }
    }
    out
}

pub fn assemble(source: &str, origin: u32) -> Vec<u32> {
    emusen_mips::assemble(&expand(source), origin, &blackmoon_r3000::dialect(None)).unwrap_or_else(|e| panic!("{e}"))
}

pub struct Exe {
    pub pc: u32,
    pub gp: u32,
    pub address: u32,
    pub body: Vec<u8>,
    pub bss: (u32, u32),
    pub stack: (u32, u32),
}

impl Exe {
    pub fn program(source: &str, origin: u32) -> Exe {
        let body: Vec<u8> = assemble(source, origin).iter().flat_map(|w| w.to_le_bytes()).collect();
        Exe { pc: origin, gp: 0, address: origin, body, bss: (0, 0), stack: (0x801F_FFF0, 0) }
    }

    /// The 800h-byte header and the body padded to 800h bytes.
    pub fn bytes(&self) -> Vec<u8> {
        let mut size = self.body.len().div_ceil(0x800) * 0x800;
        if size == 0 {
            size = 0x800;
        }
        let mut f = vec![0u8; 0x800 + size];
        f[..8].copy_from_slice(b"PS-X EXE");
        let fields =
            [self.pc, self.gp, self.address, size as u32, 0, 0, self.bss.0, self.bss.1, self.stack.0, self.stack.1];
        for (i, v) in fields.iter().enumerate() {
            f[0x10 + 4 * i..0x14 + 4 * i].copy_from_slice(&v.to_le_bytes());
        }
        f[0x800..0x800 + self.body.len()].copy_from_slice(&self.body);
        f
    }

    /// The body at its address, the memfill region zeroed, PC, GP, SP and FP set as Exec sets them, a0=1 and a1=0.
    pub fn sideload(&self, m: &mut Machine) {
        m.rig.put_bytes(self.address, &self.body);
        for i in (0..self.bss.1).step_by(4) {
            m.rig.put(self.bss.0 + i, 0);
        }
        let r = &mut m.cpu.regs;
        r.pc = self.pc;
        r.r[28] = self.gp;
        if self.stack.0 != 0 {
            r.r[29] = self.stack.0 + self.stack.1;
            r.r[30] = self.stack.0 + self.stack.1;
        }
        r.r[4] = 1;
        r.r[5] = 0;
        r.branch = Default::default();
        r.load = Default::default();
    }
}
