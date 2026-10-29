//! The MIPS front end of the tailored recompiler (EmuSen_Recompiler.md §3.1). See EmuSen_Recompiler.md §14.

use std::fmt;

/// The `Plan`'s and `Isa`'s version: (major, minor).
pub const VERSION: (u16, u16) = (1, 0);

/// What a block does with an instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Compiled, and the next instruction follows.
    Plain,
    /// A load or store: compiled, leaving its block before it at run time where its access is not one a block makes.
    Memory,
    /// A branch or jump: compiled with its delay slot, which must be plain or memory; the block ends after the slot.
    Branch,
    /// Not compiled: the block ends before it.
    Ender,
}

/// A load's or store's address and size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Access {
    pub base: u8,
    pub disp: i16,
    pub bytes: u8,
    pub store: bool,
    /// The mask the address is aligned down by before the access (all ones for a naturally aligned one).
    pub align: u32,
}

/// One instruction as the guest's table describes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Info {
    pub class: Class,
    /// The general registers it reads and writes, a bit a register; register 0 is neither.
    pub reads: u32,
    pub writes: u32,
    pub access: Option<Access>,
    /// A likely branch, whose slot is annulled when it is not taken.
    pub likely: bool,
}

/// A guest's instruction table, over its own decode.
pub trait Isa {
    /// The most instructions a block holds.
    const MAX: usize;
    fn info(&self, word: u32) -> Info;
}

/// Where a register's value comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Def {
    /// From the block's entry.
    Entry,
    /// Written by the block's instruction at this index.
    At(usize),
}

/// What a load or store may reuse of an earlier access in the block (§5). See EmuSen_Recompiler.md §14.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reuse {
    pub page: Option<(usize, bool)>,
    pub line: Option<usize>,
}

/// One instruction of a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub pc: u32,
    pub word: u32,
    pub info: Info,
    /// A delay slot.
    pub slot: bool,
    /// The definition its base register reaches, for a load or store.
    pub base_def: Option<Def>,
    pub reuse: Reuse,
    /// The registers written before it in the block.
    pub dirty: u32,
}

/// A block's plan: plain data, printable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub start: u32,
    pub steps: Vec<Step>,
    /// The registers some instruction reads before the block writes them.
    pub exposed: u32,
    /// Every register the block writes.
    pub written: u32,
}

/// The most earlier accesses a block keeps for reuse, the latest for each base (as the forms keep, `REACHES`).
pub const REACHES: usize = 4;

/// The words from the block's start that make a block. See EmuSen_Recompiler.md §14.
pub fn shape<I: Isa>(isa: &I, words: &[u32]) -> Option<Vec<u32>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < words.len() && out.len() < I::MAX {
        match isa.info(words[i]).class {
            Class::Plain | Class::Memory => out.push(words[i]),
            Class::Branch => {
                if i + 1 < words.len() && out.len() + 2 <= I::MAX && matches!(isa.info(words[i + 1]).class, Class::Plain | Class::Memory) {
                    out.extend_from_slice(&words[i..i + 2]);
                }
                break;
            }
            Class::Ender => break,
        }
        i += 1;
    }
    (!out.is_empty()).then_some(out)
}

/// The plan of a shaped block starting at `start`; an error where the words are not a block's.
pub fn plan<I: Isa>(isa: &I, words: &[u32], start: u32) -> Result<Plan, String> {
    if words.is_empty() || words.len() > I::MAX {
        return Err(format!("{} words", words.len()));
    }
    let mut steps = Vec::with_capacity(words.len());
    let mut defs = [Def::Entry; 32];
    let (mut exposed, mut written) = (0u32, 0u32);
    // (base register, its definition, the access's index), oldest first.
    let mut reached: Vec<(u8, Def, usize)> = Vec::new();
    for (k, &word) in words.iter().enumerate() {
        let info = isa.info(word);
        let slot = k > 0 && steps.last().is_some_and(|s: &Step| s.info.class == Class::Branch);
        match info.class {
            Class::Ender => return Err(format!("the word at {k} ends a block")),
            Class::Branch if k + 2 != words.len() => return Err(format!("the branch at {k} is not the block's last but one")),
            Class::Branch if slot => return Err(format!("the branch at {k} is in a delay slot")),
            _ => {}
        }
        if !slot && k + 1 == words.len() && info.class == Class::Branch {
            return Err("a branch without its slot".into());
        }
        exposed |= info.reads & !written & !1;
        let base_def = info.access.map(|a| defs[a.base as usize]);
        let mut reuse = Reuse { page: None, line: None };
        if let Some(a) = info.access {
            let d = defs[a.base as usize];
            if a.base != 0 {
                if let Some(&(_, _, j)) = reached.iter().rev().find(|&&(b, e, _)| b == a.base && e == d) {
                    let earlier = steps[j].info.access.expect("a reached step accesses memory");
                    let same = (earlier.disp, earlier.align) == (a.disp, a.align);
                    reuse.page = Some((j, same));
                    if (earlier.store || !a.store) && (i32::from(earlier.disp) - i32::from(a.disp)).abs() < 64 {
                        reuse.line = Some(j);
                    }
                }
                reached.retain(|&(b, e, _)| (b, e) != (a.base, d));
                if reached.len() == REACHES {
                    reached.remove(0);
                }
                reached.push((a.base, d, k));
            }
        }
        steps.push(Step { pc: start.wrapping_add(4 * k as u32), word, info, slot, base_def, reuse, dirty: written });
        let w = info.writes & !1;
        written |= w;
        for r in 0..32 {
            if w >> r & 1 == 1 {
                defs[r] = Def::At(k);
            }
        }
    }
    Ok(Plan { start, steps, exposed, written })
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "block {:08x} steps {} exposed {:08x} written {:08x}", self.start, self.steps.len(), self.exposed, self.written)?;
        for (k, s) in self.steps.iter().enumerate() {
            write!(f, "{k:2} {:08x} {:08x} {:?}{} r{:08x} w{:08x} d{:08x}", s.pc, s.word, s.info.class, if s.slot { " slot" } else { "" }, s.info.reads, s.info.writes, s.dirty)?;
            if let Some(a) = s.info.access {
                write!(f, " [r{}{:+}] {}{}", a.base, a.disp, a.bytes, if a.store { " store" } else { "" })?;
                match s.base_def {
                    Some(Def::At(j)) => write!(f, " base@{j}")?,
                    _ => write!(f, " base@entry")?,
                }
                if let Some((j, same)) = s.reuse.page {
                    write!(f, " page@{j}{}", if same { " same" } else { "" })?;
                }
                if let Some(j) = s.reuse.line {
                    write!(f, " line@{j}")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A toy table: opcode 1 plain (writes rt, reads rs), 2 load (rt from [rs+imm]), 3 store, 4 branch, 5 likely, 0 ender.
    struct Toy;
    impl Isa for Toy {
        const MAX: usize = 8;
        fn info(&self, w: u32) -> Info {
            let (rs, rt) = (w >> 21 & 31, w >> 16 & 31);
            let bit = |r: u32| if r == 0 { 0 } else { 1u32 << r };
            let access = |store| Some(Access { base: rs as u8, disp: w as u16 as i16, bytes: 4, store, align: !0 });
            match w >> 26 {
                1 => Info { class: Class::Plain, reads: bit(rs), writes: bit(rt), access: None, likely: false },
                2 => Info { class: Class::Memory, reads: bit(rs), writes: bit(rt), access: access(false), likely: false },
                3 => Info { class: Class::Memory, reads: bit(rs) | bit(rt), writes: 0, access: access(true), likely: false },
                4 | 5 => Info { class: Class::Branch, reads: bit(rs), writes: 0, access: None, likely: w >> 26 == 5 },
                _ => Info { class: Class::Ender, reads: 0, writes: 0, access: None, likely: false },
            }
        }
    }

    fn op(o: u32, rs: u32, rt: u32, imm: i16) -> u32 {
        o << 26 | rs << 21 | rt << 16 | imm as u16 as u32
    }

    #[test]
    fn a_plan_reads_its_shape_definitions_and_reuse_keys() {
        let words = [op(2, 29, 8, 0), op(2, 29, 9, 4), op(3, 29, 9, 8), op(1, 29, 29, 0), op(2, 29, 10, 0), op(3, 29, 10, 100), op(4, 31, 0, 0), op(1, 8, 11, 0)];
        let p = plan(&Toy, &words, 0x1000).expect("a block");
        let s = &p.steps;
        assert_eq!(s[1].reuse, Reuse { page: Some((0, false)), line: Some(0) });
        assert_eq!(s[2].reuse, Reuse { page: Some((1, false)), line: None }, "a store does not reuse a load's line");
        assert_eq!(s[4].base_def, Some(Def::At(3)), "sp written: a new base");
        assert_eq!(s[4].reuse, Reuse { page: None, line: None });
        assert_eq!(s[5].reuse, Reuse { page: Some((4, false)), line: None }, "100 bytes away: the page, not the line");
        assert!(s[7].slot);
        assert_eq!(p.exposed, 1 << 29 | 1 << 31);
        assert_eq!(s[5].dirty, 1 << 8 | 1 << 9 | 1 << 29 | 1 << 10);
        let text = p.to_string();
        assert!(text.contains("page@0") && text.contains("line@0") && text.contains("slot"), "{text}");
    }

    #[test]
    fn the_same_address_needs_no_guard_and_register_zero_never_reuses() {
        let words = [op(2, 4, 8, 16), op(2, 4, 9, 16), op(2, 0, 10, 16), op(2, 0, 11, 16)];
        let p = plan(&Toy, &words, 0).expect("a block");
        assert_eq!(p.steps[1].reuse.page, Some((0, true)));
        assert_eq!(p.steps[3].reuse, Reuse { page: None, line: None });
    }

    #[test]
    fn at_most_four_earlier_bases_are_kept() {
        let words: Vec<u32> = (1..=6).map(|b| op(2, b, 20, 0)).chain([op(2, 1, 21, 0), op(2, 6, 22, 0)]).collect();
        let p = plan(&Toy, &words, 0).expect("a block");
        assert_eq!(p.steps[6].reuse.page, None, "r1's access was pushed out by four later bases");
        let stale = [op(2, 1, 20, 0), op(1, 0, 1, 0), op(2, 1, 21, 0), op(2, 2, 22, 0), op(2, 3, 23, 0), op(2, 1, 24, 0)];
        let p2 = plan(&Toy, &stale, 0).expect("a block");
        assert_eq!(p2.steps[5].reuse.page, Some((2, true)), "r1's new value reused; its old value's entry keeps a place");
        assert_eq!(p.steps[7].reuse.page, Some((5, true)));
    }

    #[test]
    fn shape_and_plan_refuse_what_is_not_a_block() {
        assert_eq!(shape(&Toy, &[op(1, 1, 2, 0), op(0, 0, 0, 0), op(1, 1, 2, 0)]), Some(vec![op(1, 1, 2, 0)]));
        assert_eq!(shape(&Toy, &[op(4, 1, 0, 0), op(4, 1, 0, 0)]), None, "a branch in a slot is refused with its branch");
        assert_eq!(shape(&Toy, &[op(0, 0, 0, 0)]), None);
        assert!(plan(&Toy, &[op(4, 1, 0, 0)], 0).is_err());
        assert!(plan(&Toy, &[op(4, 1, 0, 0), op(1, 1, 2, 0), op(1, 1, 2, 0)], 0).is_err());
        assert!(plan(&Toy, &[op(1, 1, 2, 0), op(0, 0, 0, 0)], 0).is_err());
    }
}
