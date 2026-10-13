//! The renderer's device: its buffers on the shared Vulkan device (`emusen-gpu`), the four programs, and the commands one
//! submission records from the steps the host planned (Diamond_Enhancements.md §16.5).

use emusen_gpu::{Commands, GpuBuffer, GpuDevice, GpuProgram, Where};

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
}

/// The device and what the renderer made on it, which `Drop` gives back before the device goes.
pub struct Device {
    pub gpu: GpuDevice,
    held: Option<Held>,
}

pub struct Held {
    pub walk: GpuProgram,
    pub scan: GpuProgram,
    pub clut: GpuProgram,
    pub copy: GpuProgram,
    pub vram: GpuBuffer,
    pub picture: GpuBuffer,
    pub cluts: GpuBuffer,
    pub scratch: GpuBuffer,
    /// The staged arrays on the host and their copies on the device.
    pub records: (GpuBuffer, GpuBuffer),
    pub entries: (GpuBuffer, GpuBuffer),
    pub tiles: (GpuBuffer, GpuBuffer),
    pub loads: (GpuBuffer, GpuBuffer),
    pub lines: (GpuBuffer, GpuBuffer),
    pub pixels: (GpuBuffer, GpuBuffer),
    /// What comes back, and what a seed goes up through.
    pub back: GpuBuffer,
}

fn pair(gpu: &GpuDevice, bytes: usize) -> Result<(GpuBuffer, GpuBuffer), String> {
    Ok((gpu.create_buffer(bytes as u64, Where::Host)?, gpu.create_buffer(bytes as u64, Where::Device)?))
}

impl Device {
    pub fn new(gpu: GpuDevice) -> Result<Device, String> {
        let words = |n: usize| (n * 4) as u64;
        let held = Held {
            walk: gpu.create_program(WALK, 6, 4)?,
            scan: gpu.create_program(SCAN, 3, 4)?,
            clut: gpu.create_program(CLUT, 3, 4)?,
            copy: gpu.create_program(COPY, 2, size_of::<CopyStep>())?,
            vram: gpu.create_buffer(words(VRAM_WORDS), Where::Device)?,
            picture: gpu.create_buffer(words(PICTURE_WORDS), Where::Device)?,
            cluts: gpu.create_buffer(words(SLOTS * 256), Where::Device)?,
            scratch: gpu.create_buffer(words(VRAM_WORDS), Where::Device)?,
            records: pair(&gpu, RECORDS * WORDS * 4)?,
            entries: pair(&gpu, ENTRIES * 4)?,
            tiles: pair(&gpu, TILE_RECORDS * 16)?,
            loads: pair(&gpu, LOADS * 16)?,
            lines: pair(&gpu, LINES * 32)?,
            pixels: pair(&gpu, PIXELS / 2 * 4)?,
            back: gpu.create_buffer(words(VRAM_WORDS), Where::Host)?,
        };
        Ok(Device { gpu, held: Some(held) })
    }

    pub fn held(&self) -> &Held {
        self.held.as_ref().expect("held until dropped")
    }

    /// Records `steps` into one submission, after copying what they use of the staged arrays to the device, and what
    /// is to come back; submits it and waits.
    pub fn submit(&self, steps: &[Step], used: Used, back: Back) {
        let held = self.held();
        self.gpu.submit(|c| held.commands(c, steps, used, back));
    }
}

impl Held {
    fn commands(&self, c: &mut Commands, steps: &[Step], used: Used, back: Back) {
        {
            let staged = [(&self.records, used.records * WORDS * 4), (&self.entries, used.entries * 4), (&self.tiles, used.tiles * 16), (&self.loads, used.loads * 16), (&self.lines, used.lines * 32), (&self.pixels, used.pixels.div_ceil(2) * 4)];
            for ((host, device), bytes) in staged {
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
            }
        }
    }

    fn record(&self, c: &mut Commands, step: &Step, first: &mut [bool; 4]) {
        let go = |c: &mut Commands, n: usize, program: &GpuProgram, buffers: &[&GpuBuffer], push: &u32, groups: u32, first: &mut [bool; 4]| {
            if std::mem::take(&mut first[n]) {
                c.dispatch(program, buffers, push, groups, 1, 1);
            } else {
                c.dispatch_again(program, 0, push, groups, 1, 1);
            }
        };
        match *step {
            Step::Stretch { clear, loads, lines, tiles } => {
                if clear {
                    c.fill(&self.picture, 0xFF00_0000);
                }
                if loads.1 > 0 {
                    go(c, 0, &self.clut, &[&self.vram, &self.cluts, &self.loads.1], &loads.0, loads.1, first);
                }
                if lines.1 > 0 {
                    go(c, 1, &self.scan, &[&self.vram, &self.picture, &self.lines.1], &lines.0, lines.1, first);
                }
                if tiles.1 > 0 {
                    go(c, 2, &self.walk, &[&self.vram, &self.records.1, &self.entries.1, &self.tiles.1, &self.cluts, &self.pixels.1], &tiles.0, tiles.1, first);
                }
            }
            Step::Copy(s) => {
                let groups = (s.width * s.rows).div_ceil(64);
                if std::mem::take(&mut first[3]) {
                    c.dispatch(&self.copy, &[&self.vram, &self.scratch], &s, groups, 1, 1);
                } else {
                    c.dispatch_again(&self.copy, 0, &s, groups, 1, 1);
                }
            }
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        let Some(h) = self.held.take() else { return };
        for p in [h.walk, h.scan, h.clut, h.copy] {
            self.gpu.destroy_program(p);
        }
        for b in [h.vram, h.picture, h.cluts, h.scratch, h.back] {
            self.gpu.destroy_buffer(b);
        }
        for (host, device) in [h.records, h.entries, h.tiles, h.loads, h.lines, h.pixels] {
            self.gpu.destroy_buffer(host);
            self.gpu.destroy_buffer(device);
        }
    }
}
