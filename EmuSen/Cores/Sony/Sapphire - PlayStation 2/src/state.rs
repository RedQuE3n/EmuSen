//! Sapphire's own state format, "SPH2" (Sapphire_Plan.md §4.7, §5.10; Sapphire_Native.md §38): the magic, the
//! version, the frame count, the market, the BIOS entry by its declared identity, the image's identity and the EE's
//! stop; then the picture, the one block whose length varies, so that a load knows the whole state's length before it
//! changes anything; then every unit's block in the plan's order (`board::System::write_state`). The layout listing of
//! each version is pinned by a test against its file in `tests/sph2/`. Neither the BIOS image nor the disc is ever in a
//! state, nor the cards' images and the EEPROM's bytes, which are the player's media.

use std::collections::VecDeque;

use emusen_native::ffi::{StateMachine, Status, status};
use emusen_native::{StateReader, StateWriter, Truncated};

use crate::machine::{Machine, Market, Stop};
use crate::pcrtc::Picture;

pub const STATE_MAGIC: u32 = u32::from_le_bytes(*b"SPH2");
/// Version 4, of S28: version 3's (S10: the fast boot's entry, the EE's BC1 slot and the IOP's console capture,
/// Sapphire_Native.md §36.10) with the drive's register 38h (§39). Versions 3, 2 (S30, every unit's state) and 1 (S24,
/// the memories alone) were never a player's, the console being offered to none, so none is read (§38.4).
pub const STATE_VERSION: i32 = 4;
/// The oldest version a state may have and still load.
pub const OLDEST_STATE_VERSION: i32 = 4;

/// A state made on a console with another BIOS entry, or with one where this console has none, or the reverse.
pub const STATUS_OTHER_BIOS: i32 = -14;
/// A queue of the machine holds more entries than its slots in the state at this instant.
pub const STATUS_QUEUE_PAST_STATE: i32 = -15;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    Foreign(u32),
    Version(i32),
    /// A state of a console of another market, by its byte.
    Market(u8),
    /// A state made with another BIOS entry than this console's, by the two file numbers (0 for none).
    Bios { state: u32, here: u32 },
    /// A state made over another image, by the two identities.
    WrongMedia { state: u64, here: u64 },
    /// A state longer than its picture and blocks, by the length it should have had.
    Length { wanted: usize },
    /// A queue past its slots: the machine cannot be saved at this instant.
    Queue,
    BufferTooSmall { needed: usize },
}

impl Status for StateError {
    fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => status::TRUNCATED,
            StateError::Foreign(_) | StateError::Market(_) | StateError::Length { .. } => status::FOREIGN,
            StateError::Version(_) => status::VERSION,
            StateError::Bios { .. } => STATUS_OTHER_BIOS,
            StateError::WrongMedia { .. } => emusen_native::core::status::WRONG_MEDIA,
            StateError::Queue => STATUS_QUEUE_PAST_STATE,
            StateError::BufferTooSmall { .. } => status::BUFFER_TOO_SMALL,
        }
    }
}

impl From<Truncated> for StateError {
    fn from(t: Truncated) -> Self {
        StateError::Truncated { at: t.at, wanted: t.wanted }
    }
}

fn market_byte(m: Market) -> u8 {
    match m {
        Market::Japan => 0,
        Market::Americas => 1,
        Market::Europe => 2,
    }
}

/// An optional value as a flag and the value, 0 when absent, so that its place in the state never moves.
pub(crate) fn opt_u64(w: &mut StateWriter, name: &str, v: Option<u64>) {
    w.group(name, |w| {
        w.bool("Some", v.is_some());
        w.u64("Value", v.unwrap_or(0));
    });
}

pub(crate) fn read_opt_u64(r: &mut StateReader) -> Result<Option<u64>, Truncated> {
    let some = r.bool()?;
    let v = r.u64()?;
    Ok(some.then_some(v))
}

pub(crate) fn opt_u32(w: &mut StateWriter, name: &str, v: Option<u32>) {
    w.group(name, |w| {
        w.bool("Some", v.is_some());
        w.u32("Value", v.unwrap_or(0));
    });
}

pub(crate) fn read_opt_u32(r: &mut StateReader) -> Result<Option<u32>, Truncated> {
    let some = r.bool()?;
    let v = r.u32()?;
    Ok(some.then_some(v))
}

/// A queue in `N` fixed slots, the first its front and the slots past its length default, so that every state of a
/// machine has one length (EmuSen_Rewind_And_FastForward.md §1.8); each unit's `fits` says whether its queues are within
/// them.
pub(crate) fn slots<T: Copy + Default, const N: usize>(q: &VecDeque<T>) -> [T; N] {
    let mut a = [T::default(); N];
    for (d, s) in a.iter_mut().zip(q) {
        *d = *s;
    }
    a
}

/// A queue refilled from its slots: its first `len` entries, at most `N`.
pub(crate) fn refill<T: Copy, const N: usize>(q: &mut VecDeque<T>, len: u32, slots: &[T; N]) {
    q.clear();
    q.extend(slots.iter().take((len as usize).min(N)).copied());
}

impl Machine {
    /// The BIOS entry the console runs by its file number, the entry's declared identity; 0 for none.
    pub fn bios_entry(&self) -> u32 {
        if self.booted { crate::v1::bios(self.market).which } else { 0 }
    }

    fn write_header(&self, w: &mut StateWriter) {
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.i64("Frames", self.frames);
        w.u8("Market", market_byte(self.market));
        w.u32("Bios", self.bios_entry());
        w.u64("Image", self.media_identity());
        w.u8("Stop", u8::from(self.stop.is_some()));
        opt_u64(w, "Entered", self.entered.map(|f| f as u64));
    }

    fn write(&self, w: &mut StateWriter) {
        self.write_header(w);
        let p = &self.sys.board.pcrtc.picture;
        w.group("Picture", |w| {
            p.write_shape(w);
            w.bytes("Rgba", &p.rgba);
        });
        self.sys.write_state(w);
    }

    /// The length of everything after the picture's raster, the same for every state of this version.
    fn blocks_len(&self) -> usize {
        let mut w = StateWriter::counter();
        self.sys.write_state(&mut w);
        w.len()
    }
}

impl StateMachine for Machine {
    type Error = StateError;

    fn load_state(&mut self, data: &[u8]) -> Result<(), StateError> {
        let mut r = StateReader::new(data);
        let magic = r.u32().map_err(|_| StateError::Foreign(0))?;
        if magic != STATE_MAGIC {
            return Err(StateError::Foreign(magic));
        }
        let version = r.i32()?;
        if !(OLDEST_STATE_VERSION..=STATE_VERSION).contains(&version) {
            return Err(StateError::Version(version));
        }
        r.set_version(version);
        let frames = r.i64()?;
        let market = r.u8()?;
        if market != market_byte(self.market) {
            return Err(StateError::Market(market));
        }
        let bios = r.u32()?;
        if bios != self.bios_entry() {
            return Err(StateError::Bios { state: bios, here: self.bios_entry() });
        }
        let image = r.u64()?;
        if image != self.media_identity() {
            return Err(StateError::WrongMedia { state: image, here: self.media_identity() });
        }
        let stop = r.u8()? != 0;
        let entered = read_opt_u64(&mut r)?.map(|f| f as i64);
        let (geometry, aspect, odd, raster) = Picture::read_shape(&mut r)?;
        let wanted = r.position() + raster + self.blocks_len();
        if data.len() < wanted {
            return Err(StateError::Truncated { at: data.len(), wanted: wanted - data.len() });
        }
        if data.len() > wanted {
            return Err(StateError::Length { wanted });
        }
        // Nothing past here can fail: the state's length is the one its picture and this version's blocks make.
        let p = &mut self.sys.board.pcrtc.picture;
        (p.geometry, p.aspect, p.odd) = (geometry, aspect, odd);
        p.rgba.resize(raster, 0);
        r.bytes(&mut p.rgba)?;
        self.sys.read_state(&mut r)?;
        debug_assert_eq!(r.position(), data.len(), "a state read to its end");
        self.frames = frames;
        self.stop = stop.then_some(Stop::Unimplemented);
        self.entered = entered;
        self.sys.board.iop.bus.spu2.out.clear();
        self.forget_stop();
        self.idle.forget();
        Ok(())
    }

    fn state_size(&self) -> usize {
        let mut w = StateWriter::counter();
        self.write(&mut w);
        w.len()
    }

    fn save_state(&self, out: &mut [u8]) -> Result<usize, StateError> {
        let needed = self.state_size();
        if out.len() < needed {
            return Err(StateError::BufferTooSmall { needed });
        }
        if !self.sys.fits() {
            return Err(StateError::Queue);
        }
        let mut w = StateWriter::new(out);
        self.write(&mut w);
        Ok(w.len())
    }

    fn layout(&self) -> String {
        let mut w = StateWriter::layout();
        self.write(&mut w);
        w.into_layout()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{BIOS_BYTES, RAM_ID};
    use crate::media::Media;
    use crate::media::sample::{dvd, elf};

    fn machine(market: Market) -> Machine {
        Machine::new(Media::read(&elf()).unwrap(), market, [Some("ps2.dualshock2"), None])
    }

    fn saved(m: &Machine) -> Vec<u8> {
        let mut s = vec![0; m.state_size()];
        assert_eq!(m.save_state(&mut s), Ok(s.len()));
        s
    }

    /// The listing of version 4 is the file beside the tests, line for line: a change to any block is a new version.
    #[test]
    fn the_version_4_layout_is_pinned() {
        let m = machine(Market::Americas);
        let pinned = include_str!("../tests/sph2/version-4.layout");
        let layout = m.layout();
        for (n, (got, want)) in layout.lines().zip(pinned.lines()).enumerate() {
            assert_eq!(got, want, "line {}", n + 1);
        }
        assert_eq!(layout.lines().count(), pinned.lines().count());
        assert_eq!(&layout.lines().take(7).collect::<Vec<_>>(), &["0 4 u32 Magic", "4 4 i32 Version", "8 8 i64 Frames", "16 1 u8 Market", "17 4 u32 Bios", "21 8 u64 Image", "29 1 u8 Stop"]);
    }

    #[test]
    fn a_state_round_trips_byte_for_byte_and_a_refused_load_changes_nothing() {
        let mut a = machine(Market::Americas);
        for _ in 0..3 {
            a.advance();
        }
        a.set_space_byte(RAM_ID, (32 << 20) - 1, 0x5A);
        a.set_space_byte(9, 3, 0xA5);
        a.sys.ee.gpr[7] = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210;
        let b = &mut a.sys.board;
        for w in 1..=5 {
            b.iop.bus.sif.fifo[1].push(w * 0x1111);
        }
        (b.iop.bus.sif.mscom, b.intc.mask, b.timers[2].count, b.dmac.ch[3].madr) = (0xA5A5, 0x7FFF, 77, 0x1230);
        (b.vpu.units.vu1.acc, b.iop.bus.spu2.regs[5]) = ([1, 2, 3, 4], 0x55);
        let s = saved(&a);
        let mut b = machine(Market::Americas);
        b.load_state(&s).unwrap();
        assert_eq!(saved(&b), s, "save, load, save");
        assert_eq!((b.frames, b.space_byte(RAM_ID, (32 << 20) - 1), b.space_byte(9, 3), b.sys.ee.gpr[7]), (3, Some(0x5A), Some(0xA5), a.sys.ee.gpr[7]));
        let mut c = machine(Market::Americas);
        let before = saved(&c);
        let mut long = s.clone();
        long.push(0);
        let mut v1 = s.clone();
        v1[4] = 1;
        let refusals: [(&[u8], i32); 6] =
            [(&s[..s.len() - 1], status::TRUNCATED), (&s[..40], status::TRUNCATED), (b"DMND0000", status::FOREIGN), (&[], status::FOREIGN), (&long, status::FOREIGN), (&v1, status::VERSION)];
        for (bad, code) in refusals {
            assert_eq!(c.load_state(bad).unwrap_err().status(), code);
            assert_eq!(saved(&c), before, "a refused load changes nothing");
        }
        assert_eq!(machine(Market::Europe).load_state(&s).unwrap_err(), StateError::Market(1));
        let mut other = Machine::new(Media::read(&dvd()).unwrap(), Market::Americas, [Some("ps2.dualshock2"), None]);
        assert!(matches!(other.load_state(&s), Err(StateError::WrongMedia { .. })));
        let mut d = machine(Market::Americas);
        d.insert_bios(&vec![0; BIOS_BYTES]);
        let e = d.load_state(&s).unwrap_err();
        assert_eq!((e.clone(), e.status()), (StateError::Bios { state: 0, here: crate::v1::BIOS_AMERICAS }, STATUS_OTHER_BIOS), "a state of a console with no BIOS");
    }

    /// A console running a boot ROM loads a state made at any instruction and continues as the machine that made it.
    #[test]
    fn a_running_console_loads_mid_frame_and_continues_as_the_one_that_saved() {
        let make = || {
            let mut m = machine(Market::Americas);
            m.insert_bios(&vec![0; BIOS_BYTES]);
            m
        };
        let mut a = make();
        a.advance();
        for _ in 0..12_345 {
            a.sys.step().unwrap();
        }
        let s = saved(&a);
        let mut b = make();
        b.load_state(&s).unwrap();
        for _ in 0..2 {
            a.advance();
            b.advance();
        }
        assert_eq!(saved(&b), saved(&a), "the loaded console continues as the one that saved it");
        assert_eq!(b.sys.board.pcrtc.picture.rgba, a.sys.board.pcrtc.picture.rgba);
    }

    /// The fast boot's entry is state (version 3): a state made after it loads into a console still waiting and does not
    /// enter the program again, one made before it waits again on a console that has entered; the console's capture,
    /// its device and its hook, come with it (Sapphire_Native.md §36.10).
    #[test]
    fn the_fast_boots_entry_and_the_consoles_capture_are_state() {
        let make = || {
            let mut m = machine(Market::Americas);
            m.insert_bios(&vec![0; BIOS_BYTES]);
            m.program = Some(elf());
            m
        };
        let before = saved(&make());
        let mut a = make();
        a.sys.board.iop.bus.sif.smflg = crate::machine::IOP_BOOTED;
        a.sys.ee.cop0[crate::ee::cop0::STATUS] = 0x10;
        a.sys.board.iop.bus.export_tables = vec![0x1000, 0x2000];
        a.advance();
        assert!(a.entered.is_some() && a.sys.board.iop.tty.device);
        let after = saved(&a);
        let mut b = make();
        b.load_state(&after).unwrap();
        assert!(!b.waiting() && b.entered == a.entered && b.sys.board.iop.tty.device, "entered once, the device attached");
        assert_eq!(b.sys.board.iop.bus.export_tables, [0x1000, 0x2000]);
        assert_eq!(saved(&b), after);
        b.load_state(&before).unwrap();
        assert!(b.waiting() && !b.sys.board.iop.tty.device, "a state from before the entry waits for it again");
    }

    #[test]
    fn a_queue_past_its_slots_is_refused_with_words() {
        let mut m = machine(Market::Japan);
        for w in 0..70 {
            m.sys.board.iop.bus.sif.fifo[0].push(w);
        }
        let mut s = vec![0; m.state_size()];
        assert_eq!(m.save_state(&mut s), Err(StateError::Queue));
    }
}
