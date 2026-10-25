//! The IOP's console on the player's BIOS, caught at the IOP's module interface (Sapphire_Plan.md's decision of
//! 2026-10-24; Sapphire_Native.md §36.3): IOMAN's `write` (export 7, ps2tek "BIOS IOP IOMAN") found at run time through
//! the export tables LOADCORE links by (ps2tek "BIOS IOP Module Linking"), never at an address in the image, and the
//! bytes a call writes to standard output kept as the text of the `tty` space. The bus notes where a store completes an
//! export table's magic word in RAM (`IopBus::export_tables`), since the word does not stay once the table is
//! registered (§36.3); the table is taken by its name and its entry, and nothing else of the IOP's memory is read.

/// An export table's magic word, its name's place and length, and its function pointers' place (ps2tek "BIOS IOP
/// Module Linking": magic, next, version, mode, name[8], export[]).
pub const EXPORT_MAGIC: u32 = 0x41C0_0000;
const NAME_AT: usize = 12;
const NAME_BYTES: usize = 8;
const EXPORTS_AT: usize = 20;

/// The places of export tables the bus keeps, the newest last.
pub const TABLES: usize = 256;

/// The library and the export: IOMAN's `int write(int fd, void *ptr, size_t size)`, number 7.
pub const LIBRARY: &[u8; NAME_BYTES] = b"ioman\0\0\0";
pub const WRITE: usize = 7;

/// Standard output's descriptor.
pub const STDOUT: u32 = 1;

/// The text kept, as Diamond's TTY keeps its own (64 KiB there; the longest expected text of ps2autotests is 93,677
/// bytes).
pub const TTY_BYTES: usize = 256 << 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tty {
    pub text: Vec<u8>,
    pub len: usize,
    /// The export table's place in RAM and the function's physical address, while the table is there.
    hook: Option<(usize, u32)>,
    /// The calls caught, by descriptor 0, 1, 2 and any other: the capture's own report.
    pub calls: [u64; 4],
    /// The arguments of a call met at the boundary, kept until its first instruction has run (an interrupt taken there
    /// re-enters it).
    pending: Option<(u32, u32, u32)>,
    /// The console has a TTY attached, as a development console or ps2link gives one: a write to standard output is
    /// answered here with its bytes kept and its size returned, where the retail BIOS's TTY driver is a stub
    /// (Sapphire_Native.md §36.3; ps2tek "BIOS IOP IOMAN"). Off, the call runs on and its bytes are only kept.
    pub device: bool,
}

impl Default for Tty {
    fn default() -> Tty {
        Tty { text: vec![0; TTY_BYTES], len: 0, hook: None, calls: [0; 4], pending: None, device: false }
    }
}

fn word(ram: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([ram[at], ram[at + 1], ram[at + 2], ram[at + 3]])
}

impl Tty {
    /// The function's physical address while it is resolved.
    pub fn hook(&self) -> Option<u32> {
        self.hook.map(|h| h.1)
    }

    /// Keeps the hook while its table still names `ioman` with the same entry, else takes the newest table among
    /// `tables` that does: a rebooted IOP's modules land anew.
    pub fn resolve(&mut self, ram: &[u8], tables: &[u32]) {
        if let Some((at, f)) = self.hook {
            if Self::table(ram, at) == Some(f) {
                return;
            }
        }
        self.hook = tables.iter().rev().find_map(|&at| Self::table(ram, at as usize).map(|f| (at as usize, f)));
    }

    /// `write`'s physical address if the table at `at` is `ioman`'s and its entry points into RAM.
    fn table(ram: &[u8], at: usize) -> Option<u32> {
        if at + EXPORTS_AT + 4 * WRITE + 4 > ram.len() || &ram[at + NAME_AT..at + NAME_AT + NAME_BYTES] != LIBRARY {
            return None;
        }
        let f = word(ram, at + EXPORTS_AT + 4 * WRITE) & 0x1FFF_FFFF;
        (f != 0 && f < super::bus::RAM_WINDOW).then_some(f)
    }

    /// At an instruction boundary at `pc` with a0 to a2 `a`: with the device attached, a call writing to standard output
    /// is answered, its bytes kept and its size the value to return, the caller resumed at its return address.
    pub fn answer(&mut self, pc: u32, a: [u32; 3], ram: &[u8]) -> Option<u32> {
        if !self.device || a[0] != STDOUT || self.hook.map(|h| h.1) != Some(pc & 0x1FFF_FFFF) {
            return None;
        }
        self.calls[1] += 1;
        self.keep(a[1], a[2], ram);
        Some(a[2])
    }

    fn keep(&mut self, buf: u32, size: u32, ram: &[u8]) {
        for i in 0..size {
            let b = ram[(buf.wrapping_add(i) as usize) & (ram.len() - 1)];
            if self.len < TTY_BYTES && b != 0 {
                self.text[self.len] = b;
                self.len += 1;
            }
        }
    }

    /// At an instruction boundary at `pc`: the call's descriptor, buffer and size (a0 to a2) are kept when `pc` is the
    /// hook.
    pub fn at(&mut self, pc: u32, a: [u32; 3]) {
        self.pending = (self.hook.map(|h| h.1) == Some(pc & 0x1FFF_FFFF)).then_some((a[0], a[1], a[2]));
    }

    /// After the step: a call whose first instruction ran is taken, its bytes to standard output kept.
    pub fn stepped(&mut self, ran: bool, ram: &[u8]) {
        let Some((fd, buf, size)) = self.pending.take() else { return };
        if !ran {
            return;
        }
        self.calls[(fd as usize).min(3)] += 1;
        if fd == STDOUT {
            self.keep(buf, size, ram);
        }
    }

    pub fn armed(&self) -> bool {
        self.hook.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(ram: &mut [u8], at: usize, name: &[u8; 8], write: u32) {
        ram[at..at + 4].copy_from_slice(&EXPORT_MAGIC.to_le_bytes());
        ram[at + NAME_AT..at + NAME_AT + 8].copy_from_slice(name);
        let w = at + EXPORTS_AT + 4 * WRITE;
        ram[w..w + 4].copy_from_slice(&write.to_le_bytes());
    }

    #[test]
    fn the_hook_is_iomans_write_found_by_its_table_and_follows_the_table() {
        let mut t = Tty::default();
        let mut ram = vec![0u8; super::super::bus::RAM];
        table(&mut ram, 0x4_0000, b"sysmem\0\0", 0x1234);
        t.resolve(&ram, &[0x4_0000]);
        assert!(!t.armed(), "another library's table is not taken");
        table(&mut ram, 0x5_1230, LIBRARY, 0x8000_9AB0);
        ram[0x5_1230] = 0x77;
        t.resolve(&ram, &[0x4_0000, 0x5_1230]);
        assert_eq!(t.hook(), Some(0x9AB0), "write's entry, physical, the magic overwritten once registered");
        ram[0x5_1230 + NAME_AT] = 0;
        t.resolve(&ram, &[0x4_0000, 0x5_1230]);
        assert!(!t.armed(), "a table overwritten is let go");
    }

    #[test]
    fn a_call_to_standard_output_is_kept_whole_once_its_first_instruction_ran() {
        let mut t = Tty::default();
        let mut ram = vec![0u8; super::super::bus::RAM];
        table(&mut ram, 0x1000, LIBRARY, 0x2000);
        ram[0x3000..0x3006].copy_from_slice(b"Hello\n");
        t.resolve(&ram, &[0x1000]);
        t.at(0x2000, [STDOUT, 0x3000, 6]);
        t.stepped(false, &ram);
        assert_eq!(t.len, 0, "an interrupt at the boundary: the call is entered again later");
        t.at(0x8000_2000, [STDOUT, 0x3000, 6]);
        t.stepped(true, &ram);
        t.at(0x2000, [2, 0x3000, 6]);
        t.stepped(true, &ram);
        t.at(0x2004, [STDOUT, 0x3000, 6]);
        t.stepped(true, &ram);
        assert_eq!(&t.text[..t.len], b"Hello\n");
        assert_eq!(t.calls, [0, 1, 1, 0]);
    }

    /// With the device attached a write to standard output is answered with its size and its bytes kept; another
    /// descriptor, or the device absent, is left to the call (D-364).
    #[test]
    fn a_call_to_standard_output_is_answered_by_the_device_with_its_size() {
        let mut t = Tty::default();
        let mut ram = vec![0u8; super::super::bus::RAM];
        table(&mut ram, 0x1000, LIBRARY, 0x2000);
        ram[0x3000..0x3004].copy_from_slice(b"abc\n");
        t.resolve(&ram, &[0x1000]);
        assert_eq!(t.answer(0x2000, [STDOUT, 0x3000, 4], &ram), None, "no device: the call runs");
        t.device = true;
        assert_eq!(t.answer(0x2000, [2, 0x3000, 4], &ram), None, "another descriptor runs");
        assert_eq!(t.answer(0x2004, [STDOUT, 0x3000, 4], &ram), None, "not the hook");
        assert_eq!(t.answer(0x8000_2000, [STDOUT, 0x3000, 4], &ram), Some(4));
        assert_eq!(&t.text[..t.len], b"abc\n");
    }
}
