//! What a console's debugger views read and give: the memory spaces an engine publishes, read through `Memory`, and the
//! address an instruction names. A view never holds an engine: a reader hands it bytes, the C# facade's by rounds over
//! the C layer (`Tape`), Crystal's own when it is in Rust (B7) (EmuSen_Backend.md §13.4, §13.5).

use emusen_dotnet::text::equals_ignore_case;

/// What an instruction does with the address it names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reference {
    Call,
    Read,
    Write,
}

impl Reference {
    pub fn name(self) -> &'static str {
        match self {
            Reference::Call => "Call",
            Reference::Read => "Read",
            Reference::Write => "Write",
        }
    }
}

/// A memory space as an engine publishes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Space {
    pub name: String,
    pub size: i32,
    /// Whether a read disturbs live state.
    pub side_effects: bool,
}

/// The views' one way to memory: the spaces, a byte read, a batch of reads whose addresses the view knew before reading
/// any of them, and a whole run from a space's start.
pub trait Memory {
    fn spaces(&self) -> &[Space];

    fn read(&mut self, space: &str, address: i32) -> u8;

    /// The bytes at `addresses`, read in that order.
    fn batch(&mut self, space: &str, addresses: &[i32]) -> Vec<u8> {
        addresses.iter().map(|&a| self.read(space, a)).collect()
    }

    /// `length` bytes from the space's start, read in order.
    fn run(&mut self, space: &str, length: usize) -> Vec<u8> {
        let addresses: Vec<i32> = (0..length as i32).collect();
        self.batch(space, &addresses)
    }
}

/// A space by its name, ignoring case, as `SystemDebugSpaces.Find` takes it.
pub fn find<'a>(memory: &'a dyn Memory, name: &str) -> Option<&'a Space> {
    memory.spaces().iter().find(|s| equals_ignore_case(&s.name, name))
}

/// A register by its name, ignoring case, as `SystemDebugSpaces.Register` takes it.
pub fn register(registers: &[(String, u64)], name: &str) -> Option<u64> {
    registers.iter().find(|(n, _)| equals_ignore_case(n, name)).map(|&(_, v)| v)
}

/// A read the tape had no byte for yet: the space, the first address and how many bytes from it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Want {
    pub space: String,
    pub address: i32,
    pub length: usize,
}

/// A reader over the bytes a host has read so far, in the order the view asked for them. The first read past them is
/// recorded, with the rest of its batch, for the host to read and the view to be run again; every read after it answers
/// 0 and is not recorded, since its address may hang on a byte not yet read. So the host reads exactly the bytes, in
/// exactly the order, the view reads with the real bytes, a batch to a round.
pub struct Tape<'a> {
    spaces: Vec<Space>,
    bytes: &'a [u8],
    at: usize,
    stopped: bool,
    pub wants: Vec<Want>,
}

impl<'a> Tape<'a> {
    pub fn new(spaces: Vec<Space>, bytes: &'a [u8]) -> Tape<'a> {
        Tape { spaces, bytes, at: 0, stopped: false, wants: Vec::new() }
    }

    fn want(&mut self, space: &str, address: i32) {
        match self.wants.last_mut() {
            Some(w) if w.space == space && w.address.wrapping_add(w.length as i32) == address => w.length += 1,
            _ => self.wants.push(Want { space: space.to_string(), address, length: 1 }),
        }
    }
}

impl Memory for Tape<'_> {
    fn spaces(&self) -> &[Space] {
        &self.spaces
    }

    fn read(&mut self, space: &str, address: i32) -> u8 {
        self.batch(space, &[address])[0]
    }

    fn batch(&mut self, space: &str, addresses: &[i32]) -> Vec<u8> {
        let mut out = Vec::with_capacity(addresses.len());
        for &a in addresses {
            if let Some(&b) = self.bytes.get(self.at).filter(|_| !self.stopped) {
                self.at += 1;
                out.push(b);
            } else {
                out.push(0);
                if !self.stopped {
                    self.want(space, a);
                }
            }
        }
        if !self.wants.is_empty() {
            self.stopped = true;
        }
        out
    }
}

/// A part that runs no code and holds registers, as machine info's units name it with this frame's values
/// (`DebugUnit` in C#; EmuSen_CoreAPI.md §31.7).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Unit {
    pub name: String,
    pub kind: String,
    pub registers: Vec<(String, u64)>,
}

/// A unit by its name, ignoring case.
pub fn unit<'a>(units: &'a [Unit], name: &str) -> Option<&'a Unit> {
    units.iter().find(|u| equals_ignore_case(&u.name, name))
}
