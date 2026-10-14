//! Diamond on the core ABI v1 as the host sees it: the machine's `Core` passed on call by call, with the enhancements
//! the machine itself never reads. "Precise geometry" attaches the precision layer paired with the renderer between
//! frames, and the frame given to the host is then the renderer's picture (Diamond_Enhancements.md §24).

use std::any::Any;

use diamond_renderer::Renderer;
use emusen_native::core::*;
use emusen_native::ffi::StateMachine;
use machine::machine::Machine;
use machine::state::StateError;
use machine::v1::{Enhancement, Values};
use precision::{Precise, Precision};

pub const PRECISION_KEY: &str = "precision";

/// Diamond_Plan.md §11 Q14's setting, as Diamond_Enhancements.md §24.2 words it.
pub const PRECISE_GEOMETRY: Enhancement = Enhancement {
    key: PRECISION_KEY,
    label: "Precise geometry",
    help: "Draws the picture on the graphics card with each 3D corner where the game's geometry puts it, between the pixels, and textures in perspective, so that polygons stop wobbling and textures stop bending as they move. The game itself runs exactly as on a console; only the picture shown is drawn otherwise.",
    cost: "Needs a Vulkan graphics card, and takes some of its time and about half a millisecond of the processor's each frame; save states are about twice the size. Corners a game moves itself, as in most 2D menus and overlays, stay on whole pixels, and an outline's top and left edges stay on the console's line.",
    values: Values::Switch,
};

/// The enhancements this library offers, each its own setting (Diamond_Plan.md §6.2).
pub const ENHANCEMENTS: &[Enhancement] = &[PRECISE_GEOMETRY];

pub struct Diamond {
    machine: Machine,
    /// Why "Precise geometry" is on but not drawn, or drawn slowly, for `setting_notes`.
    note: Option<String>,
}

impl Diamond {
    fn pair(&mut self) -> Option<&mut Precise<Renderer>> {
        let layer: &mut dyn Any = self.machine.layer()?;
        layer.downcast_mut()
    }

    fn pair_ref(&self) -> Option<&Precise<Renderer>> {
        let layer: &dyn Any = self.machine.layer_ref()?;
        layer.downcast_ref()
    }

    /// "Precise geometry" on or off, between frames: the pair attached and its first picture read, or detached; with no
    /// usable Vulkan device the machine is left as off and the reason kept for the note (§24.2).
    fn set_precise(&mut self, on: bool) {
        self.set_precise_on(on, None);
    }

    /// `set_precise` on the device `device` names a part of, `EMUSEN_GPU_DEVICE`'s or the first usable without one.
    fn set_precise_on(&mut self, on: bool, device: Option<&str>) {
        if !on {
            self.machine.detach();
            self.note = None;
            return;
        }
        if self.pair_ref().is_some() {
            return;
        }
        match Renderer::new(device) {
            Ok(r) => {
                self.note = r.software().then(|| format!("Drawn by {}, a Vulkan the processor runs, since no graphics card was found: the picture is the same, but slow.", r.report()));
                self.machine.attach(Box::new(Precise::new(Precision::default(), r.keeping())));
            }
            Err(why) => {
                self.machine.detach();
                self.note = Some(format!("No usable Vulkan graphics card was found ({why}), so the console's own picture is shown."));
            }
        }
    }

    /// The frame's work sent to the device, and the picture of the frame before handed to the host (§38.1); also when
    /// rendering is skipped, since the renderer must keep step.
    fn frame_end(&mut self) {
        if let Some(p) = self.pair() {
            p.drawer.frame_end();
        }
    }

    pub fn machine(&self) -> &Machine {
        &self.machine
    }
}

impl StateMachine for Diamond {
    type Error = StateError;

    fn load_state(&mut self, data: &[u8]) -> Result<(), StateError> {
        self.machine.load_state(data)?;
        Ok(())
    }

    fn state_size(&self) -> usize {
        self.machine.state_size()
    }

    fn save_state(&self, out: &mut [u8]) -> Result<usize, StateError> {
        self.machine.save_state(out)
    }

    fn layout(&self) -> String {
        self.machine.layout()
    }
}

impl Core for Diamond {
    const CAPABILITIES: u64 = <Machine as Core>::CAPABILITIES | caps::SETTINGS | caps::SETTING_NOTES;

    fn info() -> Info {
        Machine::info()
    }

    fn settings_schema() -> Vec<Setting> {
        let mut s = Machine::settings_schema();
        s.extend(ENHANCEMENTS.iter().map(Enhancement::setting));
        s
    }

    fn firmware_for(image: &[u8]) -> Vec<Firmware> {
        Machine::firmware_for(image)
    }

    fn status_text(code: i32) -> Option<String> {
        Machine::status_text(code)
    }

    fn create(request: &Create<'_>) -> Result<Self, i32> {
        let mut d = Diamond { machine: Machine::create(request)?, note: None };
        d.set_precise(request.settings.get(PRECISION_KEY) == Some("true"));
        Ok(d)
    }

    fn machine_info(&self) -> MachineInfo {
        self.machine.machine_info()
    }

    fn advance(&mut self, detail: &mut u64) -> Result<(), i32> {
        Core::advance(&mut self.machine, detail)?;
        self.frame_end();
        Ok(())
    }

    fn set_options(&mut self, flags: u32) {
        self.machine.set_options(flags);
    }

    fn frame_count(&self) -> i64 {
        self.machine.frame_count()
    }

    fn frame_info(&self) -> FrameInfo {
        match self.pair_ref() {
            Some(p) => {
                let (_, w, h) = p.drawer.handed();
                FrameInfo { aspect_num: 4, aspect_den: 3, ..FrameInfo::rgba(w as i32, h as i32) }
            }
            None => self.machine.frame_info(),
        }
    }

    fn frame(&self) -> &[u8] {
        match self.pair_ref() {
            Some(p) => p.drawer.handed().0,
            None => self.machine.frame(),
        }
    }

    fn audio_rate(&self) -> i32 {
        self.machine.audio_rate()
    }

    fn audio_buffered(&self) -> usize {
        self.machine.audio_buffered()
    }

    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        self.machine.drain_audio(out, max_frames)
    }

    fn set_audio_limit(&mut self, samples: usize) {
        self.machine.set_audio_limit(samples);
    }

    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        self.machine.set_buttons(port, mask, changed)
    }

    fn set_axis(&mut self, port: u32, axis: u32, value: f64) -> Result<(), i32> {
        self.machine.set_axis(port, axis, value)
    }

    fn settle(&mut self) {
        Core::settle(&mut self.machine);
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        self.machine.space_size(space)
    }

    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        self.machine.space_read(space, address, out)
    }

    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        self.machine.space_write(space, address, data)
    }

    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        self.machine.battery(which)
    }

    fn battery_saved(&mut self, which: u32) -> Result<(), i32> {
        self.machine.battery_saved(which)
    }

    /// Run-scope keys only reach here, checked against the schema: the machine's own are all read at create.
    fn set_settings(&mut self, settings: &Settings) -> Result<(), i32> {
        if let Some(v) = settings.get(PRECISION_KEY) {
            self.set_precise(v == "true");
        }
        Ok(())
    }

    fn setting_notes(&self) -> Vec<(String, String)> {
        self.note.iter().map(|n| (PRECISION_KEY.to_owned(), n.clone())).collect()
    }

    fn debug_hooks(&mut self) -> Option<&mut emusen_native::debug::Hooks> {
        self.machine.debug_hooks()
    }

    fn debug_breakpoints(&mut self, processor: u32, pairs: &[i32]) -> Result<(), i32> {
        self.machine.debug_breakpoints(processor, pairs)
    }

    fn debug_run_frame(&mut self, flags: u32, detail: &mut u64) -> Result<u32, i32> {
        let why = self.machine.debug_run_frame(flags, detail)?;
        self.frame_end();
        Ok(why)
    }

    fn debug_stopped(&self) -> u32 {
        self.machine.debug_stopped()
    }

    fn debug_pc(&self, processor: u32) -> Option<u64> {
        Core::debug_pc(&self.machine, processor)
    }

    fn debug_registers(&self, processor: u32) -> Result<Vec<i64>, i32> {
        Core::debug_registers(&self.machine, processor)
    }

    fn debug_disassemble(&mut self, processor: u32, space: u32, address: u32, count: u32) -> Result<Vec<Instruction>, i32> {
        Core::debug_disassemble(&mut self.machine, processor, space, address, count)
    }

    fn battery_files(system: &str) -> Option<Vec<Battery>> {
        Machine::battery_files(system)
    }

    fn drives(&self) -> Vec<Drive> {
        self.machine.drives()
    }

    fn media_lid(&mut self, drive: u32, open: bool) -> Result<(), i32> {
        self.machine.media_lid(drive, open)
    }

    fn media_insert(&mut self, drive: u32, insert: &Insert<'_>) -> Result<(), i32> {
        self.machine.media_insert(drive, insert)
    }
}

emusen_native::core_exports!(Diamond; axes, debug, debug_stack, debug_registers, debug_disassemble, debug_reads, media, settings, setting_notes);

#[cfg(test)]
mod tests;
