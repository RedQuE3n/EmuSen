//! A machine as the debugger reads and writes it: `CoreEngine.ReadSpace` with the cheats brought up to date first,
//! `WriteSpace`, a space's size and a processor's or unit's registers by `CoreMachine`'s rules, and the views' memory,
//! `CoreDebugTarget.ViewSpaces` with `ViewSpaceCopy` (EmuSen_Backend.md §14.5, §18.10).

use crate::info::Info;
use emusen_cheats::registry::{Applied, CheatRegistry, Send};
use emusen_core_host::Machine;
use emusen_dotnet::text::equals_ignore_case;
use emusen_native::core::caps;
use emusen_systems::debug::{Memory, Space};

/// The machine with what a read needs beside it.
pub struct Machined<'a> {
    pub machine: &'a mut Machine,
    pub info: &'a Info,
    pub cheats: &'a CheatRegistry,
    pub applied: &'a mut Applied,
}

/// `RefreshCheats`: the patch table and the pokes sent when the registry's version has moved.
pub fn refresh_cheats(machine: &mut Machine, info: &Info, cheats: &CheatRegistry, applied: &mut Applied) {
    let pokes = machine.library().claims(caps::CHEAT_POKES);
    let space_of = |name: &str| info.space_named(name).map(|s| s.id);
    if let Send::Tables { triples, quads } = applied.refresh(cheats, info.patches, pokes, &space_of) {
        if let Some(t) = triples {
            let _ = machine.set_rom_patches(t);
        }
        if let Some(q) = quads {
            let _ = machine.set_cheat_pokes(q);
        }
    }
}

/// `CoreMachine.Registers(id, span)`: how many registers the processor or unit has, `into` written only when they fit;
/// none without DEBUG_REGISTERS.
pub fn registers(machine: &Machine, id: u32, into: &mut [i64]) -> usize {
    if !machine.library().claims(caps::DEBUG_REGISTERS) {
        return 0;
    }
    let n = match machine.registers(id, &mut []) {
        Ok(n) if n > 0 => n,
        _ => return 0,
    };
    if n > into.len() {
        return n;
    }
    let _ = machine.registers(id, &mut into[..n]);
    n
}

/// `CoreMachine.Registers(id)`: every register, read live.
pub fn registers_all(machine: &Machine, id: u32) -> Vec<i64> {
    if !machine.library().claims(caps::DEBUG_REGISTERS) {
        return Vec::new();
    }
    let n = match machine.registers(id, &mut []) {
        Ok(n) if n > 0 => n,
        _ => return Vec::new(),
    };
    let mut values = vec![0i64; n];
    let _ = machine.registers(id, &mut values);
    values
}

impl Machined<'_> {
    /// `SpaceSize`, at most `i32::MAX`.
    pub fn space_size(&self, index: usize) -> i32 {
        let id = self.info.spaces[index].id;
        self.machine.space_size(id).unwrap_or(0).min(i32::MAX as u64) as i32
    }

    /// `ReadSpace`: the cheats brought up to date, then one byte; 0 for a space the machine does not name.
    pub fn read_space(&mut self, name: &str, address: i32) -> u8 {
        let Some(id) = self.info.space_named(name).map(|s| s.id) else { return 0 };
        refresh_cheats(self.machine, self.info, self.cheats, self.applied);
        let mut one = [0u8; 1];
        let _ = self.machine.space_read(id, address as u32, &mut one);
        one[0]
    }

    pub fn write_space(&mut self, name: &str, address: i32, value: u8) {
        let Some(id) = self.info.space_named(name).map(|s| s.id) else { return };
        let _ = self.machine.space_write(id, address as u32, &[value]);
    }

    /// A byte of a space as `GetMemorySpaces`' space reads it: wrapped to its size unless it is a view.
    pub fn read_listed(&mut self, index: usize, address: i32) -> u8 {
        let s = &self.info.spaces[index];
        let at = if s.view { address } else { wrap(address, self.space_size(index)) };
        let name = s.name.clone();
        self.read_space(&name, at)
    }

    pub fn write_listed(&mut self, index: usize, address: i32, value: u8) {
        let s = &self.info.spaces[index];
        if s.read_only {
            return;
        }
        let at = if s.view { address } else { wrap(address, self.space_size(index)) };
        let name = s.name.clone();
        self.write_space(&name, at, value);
    }
}

/// `DelegateDebugMemorySpace.Wrap`: modulo the size, negatives included; a space of size zero sends address zero.
pub fn wrap(address: i32, size: i32) -> i32 {
    if size <= 0 { 0 } else { address.rem_euclid(size) }
}

/// Where a view's space reads from.
#[derive(Clone, Debug)]
enum Back {
    /// The machine's space, wrapped unless a view, through `ReadSpace`.
    Live { index: usize },
    /// A plain memory, copied whole by one read when first touched.
    Lazy { index: usize, copy: Option<Vec<u8>> },
    /// A learnt space's bytes in a frame's copy, from `offset`.
    Copied { offset: usize },
}

/// The spaces a view reads, `ViewSpaces()` or `ViewSpaceCopy.Over`'s, as `emusen-systems`' `Memory`.
pub struct ViewMemory<'m, 'a> {
    pub at: &'m mut Machined<'a>,
    spaces: Vec<Space>,
    backs: Vec<Back>,
    copy: &'m [u8],
    /// Which spaces a read reached, for learning.
    pub touched: Vec<bool>,
}

impl<'m, 'a> ViewMemory<'m, 'a> {
    /// `ViewSpaces()`: each plain memory copied when first touched, a view or a space with side effects read live.
    pub fn live(at: &'m mut Machined<'a>) -> ViewMemory<'m, 'a> {
        let n = at.info.spaces.len();
        let mut spaces = Vec::with_capacity(n);
        let mut backs = Vec::with_capacity(n);
        for i in 0..n {
            let size = at.space_size(i);
            let s = &at.info.spaces[i];
            spaces.push(Space { name: s.name.clone(), size, side_effects: s.side_effects });
            backs.push(if s.side_effects || s.view || size == 0 { Back::Live { index: i } } else { Back::Lazy { index: i, copy: None } });
        }
        ViewMemory { at, spaces, backs, copy: &[], touched: vec![false; n] }
    }

    /// `ViewSpaceCopy.Over`: the learnt spaces read from a frame's copy, the others as `live`.
    pub fn over(at: &'m mut Machined<'a>, learnt: &[(usize, usize, usize)], copy: &'m [u8]) -> ViewMemory<'m, 'a> {
        let mut m = ViewMemory::live(at);
        m.copy = copy;
        for &(index, offset, size) in learnt {
            if index < m.backs.len() && offset + size <= copy.len() {
                m.backs[index] = Back::Copied { offset };
                m.spaces[index].size = size as i32;
            }
        }
        m
    }

    pub fn spaces_ref(&self) -> &[Space] {
        &self.spaces
    }

    fn index(&self, name: &str) -> Option<usize> {
        self.spaces.iter().position(|s| s.name == name).or_else(|| self.spaces.iter().position(|s| equals_ignore_case(&s.name, name)))
    }

    /// One byte of the space at `index`, by that space's own rule.
    pub fn byte(&mut self, index: usize, address: i32) -> u8 {
        self.touched[index] = true;
        let size = self.spaces[index].size;
        match &mut self.backs[index] {
            Back::Live { index: i } => {
                let i = *i;
                self.at.read_listed(i, address)
            }
            Back::Lazy { index: i, copy } => {
                let i = *i;
                if copy.is_none() {
                    let mut bytes = vec![0u8; size as usize];
                    let id = self.at.info.spaces[i].id;
                    let _ = self.at.machine.space_read(id, 0, &mut bytes);
                    *copy = Some(bytes);
                }
                copy.as_ref().map_or(0, |c| c[wrap(address, size) as usize])
            }
            Back::Copied { offset } => self.copy[*offset + wrap(address, size) as usize],
        }
    }
}

impl Memory for ViewMemory<'_, '_> {
    fn spaces(&self) -> &[Space] {
        &self.spaces
    }

    fn read(&mut self, space: &str, address: i32) -> u8 {
        match self.index(space) {
            Some(i) => self.byte(i, address),
            None => 0,
        }
    }
}
