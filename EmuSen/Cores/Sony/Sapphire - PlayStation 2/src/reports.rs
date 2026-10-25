//! The emulated hardware's own reports while firmware runs (Sapphire_Native.md §35.4): each access to a register no
//! unit answers, or to an address no device holds, by processor, address and direction, with its count, the first and
//! last cycle and the last value written. Nothing else is kept: no program counter, no instruction and no memory
//! contents, as Sapphire_Plan.md §1.4 allows a player's-BIOS stage to observe.

use std::collections::BTreeMap;

/// Which processor's bus met the access.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    Ee,
    Iop,
}

/// What the access was.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Access {
    Read,
    Write,
    /// An instruction fetch from an address with no memory behind it.
    Fetch,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub count: u64,
    /// EE cycles.
    pub first: u64,
    pub last: u64,
    pub value: Option<u32>,
}

/// The EE's console output register, KPUTCHAR [R ps2tek "Misc registers"]: what is written there is the firmware's
/// text, so its values are never kept.
pub const KPUTCHAR: u32 = 0x1000_F180;

/// The reports by side, physical address and access, in address order; with `all` set, every register access the
/// units do answer is counted too, in `registers`, for finding which device a processor waits on.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reports {
    pub entries: BTreeMap<(Side, u32, Access), Report>,
    pub all: bool,
    pub registers: BTreeMap<(Side, u32, Access), Report>,
}

fn add(map: &mut BTreeMap<(Side, u32, Access), Report>, side: Side, paddr: u32, access: Access, value: Option<u32>, at: u64) {
    let value = value.filter(|_| !(side == Side::Ee && paddr & !3 == KPUTCHAR));
    let r = map.entry((side, paddr, access)).or_insert(Report { first: at, ..Report::default() });
    r.count += 1;
    r.last = at;
    if value.is_some() {
        r.value = value;
    }
}

impl Reports {
    /// An access a unit answers, counted when `all` is set.
    #[inline]
    pub fn touch(&mut self, side: Side, paddr: u32, access: Access, value: Option<u32>, at: u64) {
        if self.all {
            add(&mut self.registers, side, paddr, access, value, at);
        }
    }

    pub fn note(&mut self, side: Side, paddr: u32, access: Access, value: Option<u32>, at: u64) {
        add(&mut self.entries, side, paddr, access, value, at);
    }

    /// The registers' counts, one line each, as `lines` writes the reports.
    pub fn register_lines(&self) -> Vec<String> {
        Self::write(&self.registers)
    }

    /// One line a report: side, address, access, count, first and last cycle, the last value written.
    pub fn lines(&self) -> Vec<String> {
        Self::write(&self.entries)
    }

    fn write(map: &BTreeMap<(Side, u32, Access), Report>) -> Vec<String> {
        map.iter()
            .map(|(&(side, a, acc), r)| {
                let v = r.value.map(|v| format!(" last {v:08X}h")).unwrap_or_default();
                format!("{side:?} {a:08X}h {acc:?} x{} from {} to {}{v}", r.count, r.first, r.last)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kputchars_values_are_never_kept_and_registers_count_only_when_asked() {
        let mut r = Reports::default();
        r.note(Side::Ee, KPUTCHAR, Access::Write, Some(0x41), 5);
        r.note(Side::Ee, KPUTCHAR, Access::Write, Some(0x42), 9);
        r.touch(Side::Iop, 0x1F80_1070, Access::Read, None, 9);
        assert_eq!(r.lines(), ["Ee 1000F180h Write x2 from 5 to 9"]);
        assert!(r.registers.is_empty());
        r.all = true;
        r.touch(Side::Iop, 0x1F80_1070, Access::Write, Some(3), 12);
        assert_eq!(r.register_lines(), ["Iop 1F801070h Write x1 from 12 to 12 last 00000003h"]);
    }
}
