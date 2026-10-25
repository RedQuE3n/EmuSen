//! The console's EEPROM as the drive reads and writes it for the BIOS (ps2tek "CDVD S Commands": OpenConfig,
//! ReadConfig, CloseConfig; Sapphire_Plan.md §4.4): 16-byte blocks in areas, a session opened for reading or writing
//! a count of blocks. Data, not firmware: a blank one is all zeros (Sapphire_Native.md §25.5).

/// The EEPROM's bytes [C, 1,024, the size of the file another emulator keeps for it, S3's listing].
pub const BYTES: usize = 1024;
/// A block, the unit ReadConfig answers: "four 32bit words" [R ps2tek].
pub const BLOCK: usize = 16;
/// The blocks an area holds, and the areas [C, until S24 learns the BIOS's layout].
pub const AREA_BLOCKS: usize = 16;
pub const AREAS: usize = BYTES / BLOCK / AREA_BLOCKS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Session {
    write: bool,
    /// The next block, and the blocks left of the count opened.
    block: usize,
    left: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nvm {
    pub bytes: Box<[u8; BYTES]>,
    session: Option<Session>,
    /// Written since the frontend last saved the medium.
    pub dirty: bool,
}

impl Default for Nvm {
    fn default() -> Nvm {
        Nvm { bytes: Box::new([0; BYTES]), session: None, dirty: false }
    }
}

impl Nvm {
    /// OpenConfig with its parameters mode (0 read, 1 write), area and count, in that order [M PCSX2, §25.3]. An area
    /// past the last, or a count past the area's end, opens nothing beyond it.
    pub fn open(&mut self, p: [u8; 3]) {
        let (write, area, count) = (p[0] == 1, p[1] as usize, p[2] as usize);
        let left = if area < AREAS { count.min(AREA_BLOCKS) } else { 0 };
        self.session = Some(Session { write, block: area * AREA_BLOCKS, left });
    }

    /// ReadConfig: the next block of a session opened for reading, zeros past its count or with none open.
    pub fn read(&mut self) -> [u8; BLOCK] {
        let mut out = [0; BLOCK];
        if let Some(s) = self.session.as_mut().filter(|s| !s.write && s.left > 0) {
            out.copy_from_slice(&self.bytes[s.block * BLOCK..(s.block + 1) * BLOCK]);
            s.block += 1;
            s.left -= 1;
        }
        out
    }

    /// WriteConfig: the next block of a session opened for writing; ignored otherwise.
    pub fn write(&mut self, data: &[u8; BLOCK]) {
        if let Some(s) = self.session.as_mut().filter(|s| s.write && s.left > 0) {
            self.bytes[s.block * BLOCK..(s.block + 1) * BLOCK].copy_from_slice(data);
            s.block += 1;
            s.left -= 1;
            self.dirty = true;
        }
    }

    pub fn close(&mut self) {
        self.session = None;
    }
}

impl Nvm {
    /// The session open in `SPH2`: whether one is, for writing, its next block and the blocks left. The bytes are the
    /// console's medium, kept apart from every state (Sapphire_Native.md §38.3).
    pub fn session_saved(&self) -> ([bool; 2], [u32; 2]) {
        let s = self.session.unwrap_or(Session { write: false, block: 0, left: 0 });
        ([self.session.is_some(), s.write], [s.block as u32, s.left as u32])
    }

    pub fn restore_session(&mut self, flags: [bool; 2], v: [u32; 2]) {
        let block = (v[0] as usize).min(BYTES / BLOCK);
        let left = (v[1] as usize).min(BYTES / BLOCK - block);
        self.session = flags[0].then_some(Session { write: flags[1], block, left });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn areas_are_written_and_read_back_by_sessions() {
        let mut n = Nvm::default();
        let (p1, p2): ([u8; 16], [u8; 16]) = (std::array::from_fn(|i| 0x11 + i as u8), std::array::from_fn(|i| 0xA1 + i as u8));
        n.open([1, 0, 1]);
        n.write(&p1);
        n.write(&p2);
        n.close();
        n.open([1, 1, 1]);
        n.write(&p2);
        n.close();
        n.open([0, 0, 2]);
        assert_eq!((n.read(), n.read(), n.read()), (p1, [0; 16], [0; 16]), "the count's second block is blank, the third past it");
        n.open([0, 1, 1]);
        n.write(&p1);
        assert_eq!(n.read(), p2, "a write in a reading session is ignored");
        n.close();
        assert_eq!(n.read(), [0; 16], "nothing open");
        n.open([1, AREAS as u8, 1]);
        n.write(&p1);
        assert_eq!(n.bytes.iter().filter(|&&b| b != 0).count(), 32);
    }
}
