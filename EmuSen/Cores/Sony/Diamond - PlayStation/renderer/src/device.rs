//! The renderer's device: its buffers on the shared Vulkan device (`emusen-gpu`), the four programs, and the commands one
//! submission records from the steps the host planned (Diamond_Enhancements.md §16.5), waited for or left pending with
//! the host's staging in the other of two sets (§38.1).

use std::cell::Cell;

use emusen_gpu::{Commands, GpuBuffer, GpuDevice, GpuProgram, Handed, PictureOut, Where};

use crate::record::WORDS;

const WALK: &[u8] = include_bytes!("../shaders/walk.spv");
const SCAN: &[u8] = include_bytes!("../shaders/scan.spv");
const CLUT: &[u8] = include_bytes!("../shaders/clut.spv");
const COPY: &[u8] = include_bytes!("../shaders/copy.spv");

/// What one submission may hold before it must go: records, tile entries, tiles, palette loads, display lines and
/// transfer pixels; and the palette slots, kept across submissions.
pub const RECORDS: usize = 32 * 1024;
pub const ENTRIES: usize = 1 << 20;
pub const TILE_RECORDS: usize = 1 << 17;
pub const LOADS: usize = 4096;
pub const LINES: usize = 8192;
pub const PIXELS: usize = 1 << 21;
pub const SLOTS: usize = 4096;

pub const VRAM_WORDS: usize = 1024 * 512;
pub const PICTURE_WORDS: usize = 640 * 576;

/// The largest scale offered (Diamond_Enhancements.md §39.5).
pub const MAX_SCALE: u32 = 4;

/// A VRAM-to-VRAM copy's step, as `copy.comp` takes it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CopyStep {
    pub write: u32,
    pub sx: u32,
    pub sy: u32,
    pub dx: u32,
    pub dy: u32,
    pub width: u32,
    pub row: u32,
    pub rows: u32,
    pub mask: u32,
    pub scale: u32,
    pub options: u32,
}

/// One planned step of a submission: a stretch's palette loads, display lines and tiles (their ranges in the staged
/// arrays), after the picture cleared when it was resized; or a copy's steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Stretch { clear: bool, loads: (u32, u32), lines: (u32, u32), tiles: (u32, u32) },
    Copy(CopyStep),
}

/// How much of each staged array a submission uses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Used {
    pub records: usize,
    pub entries: usize,
    pub tiles: usize,
    pub loads: usize,
    pub lines: usize,
    pub pixels: usize,
}

/// What a submission reads back when it has run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Back {
    Nothing,
    Picture(usize),
    Vram,
    /// True colour's plane.
    Fine,
    /// The picture's first words into one of the two slots the host is handed pictures from (§38.1).
    Slot(usize, usize),
}

/// The device and what the renderer made on it, which `Drop` gives back before the device goes; and which of the two
/// sets of staged arrays the host is filling, the other being a pending submission's until it finishes (§38.1).
pub struct Device {
    pub gpu: GpuDevice,
    held: Option<Held>,
    set: usize,
    /// The lender's `done`, when the device is lent and the host takes the picture on it (EmuSen_DevicePicture.md §7).
    done: Option<u64>,
}

/// The arrays the host stages a submission in, each a host buffer the submission copies to the device's own.
pub struct Staged {
    pub records: GpuBuffer,
    pub entries: GpuBuffer,
    pub tiles: GpuBuffer,
    pub loads: GpuBuffer,
    pub lines: GpuBuffer,
    pub pixels: GpuBuffer,
}

impl Staged {
    fn new(gpu: &GpuDevice, at: Where) -> Result<Staged, String> {
        let make = |bytes: usize| gpu.create_buffer(bytes as u64, at);
        Ok(Staged { records: make(RECORDS * WORDS * 4)?, entries: make(ENTRIES * 4)?, tiles: make(TILE_RECORDS * 16)?, loads: make(LOADS * 16)?, lines: make(LINES * 32)?, pixels: make(PIXELS / 2 * 4)? })
    }

    fn all(self) -> [GpuBuffer; 6] {
        [self.records, self.entries, self.tiles, self.loads, self.lines, self.pixels]
    }
}

/// The walk's push constants: its first tile, the scale and the options' bits.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WalkPush {
    pub first_tile: u32,
    pub scale: u32,
    pub options: u32,
}

/// The palette loads' and the display lines' push constants: the first of them, the scale and the options' bits.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct RangePush {
    first: u32,
    scale: u32,
    options: u32,
}

/// The options' bits the programs read: true colour (Diamond_Enhancements.md §40).
pub const COLOUR: u32 = 1;

pub struct Held {
    /// The scale the buffers are made for: the mirror, the scratch and the picture hold its square of samples a pixel; and
    /// the options' bits, true colour's plane being made only with it.
    pub scale: u32,
    pub options: u32,
    pub fine: GpuBuffer,
    pub walk: GpuProgram,
    pub scan: GpuProgram,
    pub clut: GpuProgram,
    pub copy: GpuProgram,
    pub vram: GpuBuffer,
    pub picture: GpuBuffer,
    pub cluts: GpuBuffer,
    pub scratch: GpuBuffer,
    /// The staged arrays on the host, two sets, and their one copy on the device.
    pub staged: [Staged; 2],
    pub onboard: Staged,
    /// What comes back, and what a seed goes up through.
    pub back: GpuBuffer,
    /// The two pictures the host is handed: in host memory a frame apart, or on a lent device the frame's own.
    pub out: PictureOut,
    /// What the last submission that handed a picture handed.
    pub handed: Cell<Handed>,
}

impl Held {
    fn new(gpu: &GpuDevice, scale: u32, options: u32, done: Option<u64>) -> Result<Held, String> {
        let words = |n: usize| (n * 4) as u64;
        let square = (scale * scale) as usize;
        Ok(Held {
            scale,
            options,
            fine: gpu.create_buffer(if options & COLOUR != 0 { words(VRAM_WORDS * square) } else { 4 }, Where::Device)?,
            walk: gpu.create_program(WALK, 7, size_of::<WalkPush>())?,
            scan: gpu.create_program(SCAN, 4, size_of::<RangePush>())?,
            clut: gpu.create_program(CLUT, 3, size_of::<RangePush>())?,
            copy: gpu.create_program(COPY, 3, size_of::<CopyStep>())?,
            vram: gpu.create_buffer(words(VRAM_WORDS * square), Where::Device)?,
            picture: gpu.create_buffer(words(PICTURE_WORDS * square), Where::Device)?,
            cluts: gpu.create_buffer(words(SLOTS * 256), Where::Device)?,
            scratch: gpu.create_buffer(words(VRAM_WORDS * square), Where::Device)?,
            staged: [Staged::new(gpu, Where::Host)?, Staged::new(gpu, Where::Host)?],
            onboard: Staged::new(gpu, Where::Device)?,
            back: gpu.create_buffer(words(VRAM_WORDS * square), Where::Host)?,
            out: PictureOut::new(gpu, words(PICTURE_WORDS * square), done)?,
            handed: Cell::new(Handed::default()),
        })
    }

    fn release(self, gpu: &GpuDevice) {
        for p in [self.walk, self.scan, self.clut, self.copy] {
            gpu.destroy_program(p);
        }
        let [s0, s1] = self.staged;
        self.out.destroy(gpu);
        for b in [self.vram, self.picture, self.cluts, self.scratch, self.back, self.fine].into_iter().chain(s0.all()).chain(s1.all()).chain(self.onboard.all()) {
            gpu.destroy_buffer(b);
        }
    }
}

impl Device {
    pub fn new(gpu: GpuDevice, scale: u32, options: u32) -> Result<Device, String> {
        Device::handing(gpu, scale, options, None)
    }

    /// A device whose pictures are handed on it to the lender, whose `done` is given, where the device is lent.
    pub fn handing(gpu: GpuDevice, scale: u32, options: u32, done: Option<u64>) -> Result<Device, String> {
        let held = Held::new(&gpu, scale, options, done)?;
        Ok(Device { gpu, held: Some(held), set: 0, done })
    }

    /// Whether the host is handed the picture on the device.
    pub fn on_device(&self) -> bool {
        self.held().out.on_device()
    }

    /// The buffers made again for another scale or other options, once nothing is in flight; what they held is gone.
    pub fn rebuild(&mut self, scale: u32, options: u32) -> Result<(), String> {
        self.gpu.wait_for_pending();
        if let Some(h) = self.held.take() {
            h.release(&self.gpu);
        }
        self.held = Some(Held::new(&self.gpu, scale, options, self.done)?);
        self.set = 0;
        Ok(())
    }

    pub fn held(&self) -> &Held {
        self.held.as_ref().expect("held until dropped")
    }

    /// The staged arrays the host fills now.
    pub fn staged(&self) -> &Staged {
        &self.held().staged[self.set]
    }

    /// Records `steps` into one submission, after copying what they use of the staged arrays to the device, and what
    /// is to come back; submits it and waits, after the pending submission if there is one.
    pub fn submit(&self, steps: &[Step], used: Used, back: Back) {
        self.gpu.wait_for_pending();
        let (held, set) = (self.held(), self.set);
        self.gpu.submit(|c| held.commands(c, set, steps, used, back));
    }

    /// The same submission left pending, once the one pending before it has finished; the host then fills the other set
    /// of staged arrays while this one is read (Diamond_Enhancements.md §38.1).
    pub fn submit_pending(&mut self, steps: &[Step], used: Used, back: Back) {
        let (held, set) = (self.held.as_ref().expect("held until dropped"), self.set);
        self.gpu.submit_pending(|c| held.commands(c, set, steps, used, back));
        self.set ^= 1;
    }
}

impl Held {
    fn commands(&self, c: &mut Commands, set: usize, steps: &[Step], used: Used, back: Back) {
        {
            let (host, device) = (&self.staged[set], &self.onboard);
            let staged = [
                (&host.records, &device.records, used.records * WORDS * 4),
                (&host.entries, &device.entries, used.entries * 4),
                (&host.tiles, &device.tiles, used.tiles * 16),
                (&host.loads, &device.loads, used.loads * 16),
                (&host.lines, &device.lines, used.lines * 32),
                (&host.pixels, &device.pixels, used.pixels.div_ceil(2) * 4),
            ];
            for (host, device, bytes) in staged {
                if bytes > 0 {
                    c.copy(host, device, bytes as u64, 0, 0);
                }
            }
            let mut first = [true; 4];
            for step in steps {
                self.record(c, step, &mut first);
            }
            match back {
                Back::Nothing => {}
                Back::Picture(words) if words > 0 => c.copy(&self.picture, &self.back, (words * 4) as u64, 0, 0),
                Back::Picture(_) => {}
                Back::Vram => c.copy(&self.vram, &self.back, 0, 0, 0),
                Back::Fine => c.copy(&self.fine, &self.back, 0, 0, 0),
                Back::Slot(slot, words) if words > 0 => self.handed.set(self.out.hand(c, &self.picture, (words * 4) as u64, slot)),
                Back::Slot(..) => {}
            }
        }
    }

    fn record(&self, c: &mut Commands, step: &Step, first: &mut [bool; 4]) {
        fn go<T: Copy>(c: &mut Commands, n: usize, program: &GpuProgram, buffers: &[&GpuBuffer], push: &T, groups: (u32, u32), first: &mut [bool; 4]) {
            if std::mem::take(&mut first[n]) {
                c.dispatch(program, buffers, push, groups.0, groups.1, 1);
            } else {
                c.dispatch_again(program, 0, push, groups.0, groups.1, 1);
            }
        }
        let (scale, options) = (self.scale, self.options);
        match *step {
            Step::Stretch { clear, loads, lines, tiles } => {
                if clear {
                    c.fill(&self.picture, 0xFF00_0000);
                }
                if loads.1 > 0 {
                    go(c, 0, &self.clut, &[&self.vram, &self.cluts, &self.onboard.loads], &RangePush { first: loads.0, scale, options }, (loads.1, 1), first);
                }
                if lines.1 > 0 {
                    go(c, 1, &self.scan, &[&self.vram, &self.picture, &self.onboard.lines, &self.fine], &RangePush { first: lines.0, scale, options }, (lines.1, scale), first);
                }
                if tiles.1 > 0 {
                    let push = WalkPush { first_tile: tiles.0, scale, options };
                    go(c, 2, &self.walk, &[&self.vram, &self.onboard.records, &self.onboard.entries, &self.onboard.tiles, &self.cluts, &self.onboard.pixels, &self.fine], &push, (tiles.1, scale * scale), first);
                }
            }
            Step::Copy(s) => {
                let s = CopyStep { scale, options, ..s };
                go(c, 3, &self.copy, &[&self.vram, &self.scratch, &self.fine], &s, ((s.width * scale).div_ceil(64), s.rows * scale), first);
            }
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        // The pending submission reads and writes what is about to go.
        self.gpu.wait_for_pending();
        if let Some(h) = self.held.take() {
            h.release(&self.gpu);
        }
    }
}
