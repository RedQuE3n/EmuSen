//! A GS stream (Sapphire_Redesign.md §7.3): the GS's state at a start, then everything the machine did to it in order,
//! the general registers' writes, the host interface's reads, the privileged registers' writes and the V-syncs, so that
//! a game's drawing replays headless into a GS made fresh, field by field, without the rest of the machine. The format
//! is the project's own: `GSS1`, the state's length and its bytes as `emusen_native::State` writes them, then events of
//! one tag byte each. Sapphire_Gs.md §3.

use super::{Gs, HostInterface};

const MAGIC: &[u8; 4] = b"GSS1";
const WRITE: u8 = 0;
const READ: u8 = 1;
const PRIVILEGED: u8 = 2;
const VSYNC: u8 = 3;

/// The events kept so far, from the state at the start.
#[derive(Clone, Debug, Default)]
pub struct Recorder {
    pub bytes: Vec<u8>,
    pub fields: u64,
}

impl Recorder {
    /// A recording starting from `gs` as it stands.
    pub fn start(gs: &Gs) -> Recorder {
        let mut count = emusen_native::StateWriter::counter();
        emusen_native::State::write_state(gs, &mut count);
        let mut state = vec![0u8; count.len()];
        emusen_native::State::write_state(gs, &mut emusen_native::StateWriter::new(&mut state));
        let mut bytes = Vec::with_capacity(state.len() + (64 << 20));
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&(state.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&state);
        Recorder { bytes, fields: 0 }
    }

    pub(super) fn write(&mut self, reg: u8, data: u64) {
        self.bytes.push(WRITE);
        self.bytes.push(reg);
        self.bytes.extend_from_slice(&data.to_le_bytes());
    }

    pub(super) fn read(&mut self, data: Option<u64>) {
        self.bytes.push(READ);
        self.bytes.extend_from_slice(&data.unwrap_or(0).to_le_bytes());
    }

    pub(super) fn privileged(&mut self, paddr: u32, bytes: usize, value: u64) {
        self.bytes.push(PRIVILEGED);
        self.bytes.extend_from_slice(&paddr.to_le_bytes());
        self.bytes.push(bytes as u8);
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn vsync(&mut self, odd: bool) {
        self.bytes.push(VSYNC);
        self.bytes.push(u8::from(odd));
        self.fields += 1;
    }
}

/// One event of a stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Write(u8, u64),
    /// A local-to-host read and the value the machine was given.
    Read(u64),
    Privileged(u32, usize, u64),
    Vsync(bool),
}

/// A stream read back: its start state and its events.
pub struct Replay<'a> {
    state: &'a [u8],
    events: &'a [u8],
    at: usize,
}

impl<'a> Replay<'a> {
    pub fn open(bytes: &'a [u8]) -> Result<Replay<'a>, String> {
        if bytes.len() < 8 || &bytes[..4] != MAGIC {
            return Err("not a GS stream".into());
        }
        let n = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let state = bytes.get(8..8 + n).ok_or("the stream's state is cut short")?;
        Ok(Replay { state, events: &bytes[8 + n..], at: 0 })
    }

    /// The GS at the stream's start written into `gs`.
    pub fn restore(&self, gs: &mut Gs) -> Result<(), String> {
        emusen_native::State::read_state(gs, &mut emusen_native::StateReader::new(self.state)).map_err(|e| format!("the stream's state: {e:?}"))
    }

    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let b = self.events.get(self.at..self.at + N)?.try_into().ok()?;
        self.at += N;
        Some(b)
    }

    pub fn next_event(&mut self) -> Option<Event> {
        let [tag] = self.take::<1>()?;
        Some(match tag {
            WRITE => {
                let [r] = self.take::<1>()?;
                Event::Write(r, u64::from_le_bytes(self.take::<8>()?))
            }
            READ => Event::Read(u64::from_le_bytes(self.take::<8>()?)),
            PRIVILEGED => {
                let p = u32::from_le_bytes(self.take::<4>()?);
                let [b] = self.take::<1>()?;
                Event::Privileged(p, b as usize, u64::from_le_bytes(self.take::<8>()?))
            }
            VSYNC => Event::Vsync(self.take::<1>()?[0] != 0),
            _ => return None,
        })
    }

    /// An event applied to `gs`; a read answers whether the GS gave the value recorded.
    pub fn apply(gs: &mut Gs, e: Event) -> bool {
        match e {
            Event::Write(r, d) => gs.write(r, d),
            Event::Read(v) => return gs.read().unwrap_or(0) == v,
            Event::Privileged(p, b, v) => gs.write_privileged(p, b, v),
            Event::Vsync(odd) => gs.vsync(odd),
        }
        true
    }
}
