//! Diamond on the core ABI v1 as the host sees it: the machine's `Core` passed on call by call, with the enhancements
//! the machine itself never reads. "Internal resolution", "True colour" and "Texture filtering" attach the renderer,
//! and "Precise geometry" pairs it with the precision layer, between frames; the frame given to the host is then the
//! renderer's picture (Diamond_Enhancements.md §24, §42), and after a load it settles within the bound machine info
//! states (§52).

use std::any::Any;

use diamond_renderer::{Filter, Options, Renderer};
use emusen_native::core::*;
use emusen_native::ffi::StateMachine;
use machine::layer::Layer;
use machine::machine::Machine;
use machine::state::StateError;
use machine::v1::{Enhancement, Values};
use precision::{Precise, Precision};

pub const PRECISION_KEY: &str = "precision";
pub const RESOLUTION_KEY: &str = "resolution";
pub const COLOUR_KEY: &str = "colour";
pub const FILTERING_KEY: &str = "filtering";

/// Diamond_Plan.md §11 Q14's setting, as Diamond_Enhancements.md §24.2 words it.
pub const PRECISE_GEOMETRY: Enhancement = Enhancement {
    key: PRECISION_KEY,
    label: "Precise geometry",
    help: "Draws the picture on the graphics card with each 3D corner where the game's geometry puts it, between the pixels, and textures in perspective, so that polygons stop wobbling and textures stop bending as they move. The game itself runs exactly as on a console; only the picture shown is drawn otherwise.",
    cost: "Needs a Vulkan graphics card, and takes some of its time and about half a millisecond of the processor's each frame; the picture is shown a frame later unless the frontend presents it on the same card, and just after a save state is loaded it takes a moment to regain its detail. Corners a game moves itself, as in most 2D menus and overlays, stay on whole pixels, and an outline's top and left edges stay on the console's line.",
    values: Values::Switch,
};

/// Mars's label and kind of choice for the same fact (MarsCore.cs' RenderScale), Diamond_Enhancements.md §39.
pub const INTERNAL_RESOLUTION: Enhancement = Enhancement {
    key: RESOLUTION_KEY,
    label: "Internal resolution",
    help: "Draws the picture on the graphics card at a multiple of the console's resolution, so that 3D edges and textures are finer on a large screen; sprites, text and 2D backgrounds stay on the console's pixels, enlarged. The game itself runs exactly as on a console.",
    cost: "Needs a Vulkan graphics card; each step costs its square in the card's work and memory (2x is four times the drawing, 4x sixteen), the picture is shown a frame later unless the frontend presents it on the same card, and just after a save state is loaded it takes a moment to regain its detail. A CRT filter draws the enlarged picture's lines rather than the console's, and effects a game makes by reading the screen back through the processor stay at the console's resolution.",
    values: Values::Choice(&[("1", "1x (the console's)"), ("2", "2x"), ("3", "3x"), ("4", "4x")]),
};

/// Diamond_Enhancements.md §40.
pub const TRUE_COLOUR: Enhancement = Enhancement {
    key: COLOUR_KEY,
    label: "True colour",
    help: "Draws shading, lighting and see-through effects with eight bits a colour where the console keeps five, and without the console's dither, so that gradients, fog and shadows are smooth instead of grainy.",
    cost: "Needs a Vulkan graphics card and a little more of its time and memory; the picture is shown a frame later unless the frontend presents it on the same card, and just after a save state is loaded smooth fog and glows can take a few seconds to settle. Games whose art relies on the console's dither, such as a grainy sky or fog, look smoother than they were made to.",
    values: Values::Switch,
};

/// Diamond_Enhancements.md §41.
pub const TEXTURE_FILTERING: Enhancement = Enhancement {
    key: FILTERING_KEY,
    label: "Texture filtering",
    help: "Smooths textures between their texels instead of showing each as a block: on 3D surfaces alone, leaving sprites, text and menus sharp, or on everything. Cut-out shapes keep the console's edges.",
    cost: "Needs a Vulkan graphics card and a little of its time; the picture is shown a frame later unless the frontend presents it on the same card. Pixel art on 3D surfaces softens; with Everything, sprites and text blur once the internal resolution is above 1x; where a game packs its textures tightly an edge can show a trace of its neighbour.",
    values: Values::Choice(&[("off", "Off (the console's)"), ("3d", "3D polygons"), ("all", "Everything")]),
};

/// The frames after a load whose pictures may differ from the saver's with any enhancement on, and with true colour on,
/// whose plane a feedback effect forgets slowly (Diamond_Enhancements.md §52.4).
pub const SETTLES: u32 = 60;
pub const SETTLES_TRUE_COLOUR: u32 = 240;

/// The enhancements this library offers, each its own setting (Diamond_Plan.md §6.2).
pub const ENHANCEMENTS: &[Enhancement] = &[PRECISE_GEOMETRY, INTERNAL_RESOLUTION, TRUE_COLOUR, TEXTURE_FILTERING];

/// The enhancements as set: precise geometry, and the renderer's options.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Chosen {
    precise: bool,
    options: Options,
}

impl Chosen {
    /// Whether the renderer draws the picture at all: any enhancement not at the hardware's value.
    fn drawn(&self) -> bool {
        *self != Chosen::default()
    }

    /// The keys of the enhancements not at the hardware's value, which a note about the renderer belongs to.
    fn keys(&self) -> Vec<&'static str> {
        let o = self.options;
        [(self.precise, PRECISION_KEY), (o.scale != 1, RESOLUTION_KEY), (o.colour, COLOUR_KEY), (o.filter != Filter::Off, FILTERING_KEY)].iter().filter(|k| k.0).map(|k| k.1).collect()
    }

    /// The settings' values over these, each key given; a value the schema checked.
    fn with(mut self, settings: &Settings) -> Chosen {
        if let Some(v) = settings.get(PRECISION_KEY) {
            self.precise = v == "true";
        }
        if let Some(v) = settings.get(RESOLUTION_KEY) {
            self.options.scale = v.parse().unwrap_or(1);
        }
        if let Some(v) = settings.get(COLOUR_KEY) {
            self.options.colour = v == "true";
        }
        if let Some(v) = settings.get(FILTERING_KEY) {
            self.options.filter = match v {
                "3d" => Filter::Polygons,
                "all" => Filter::All,
                _ => Filter::Off,
            };
        }
        self
    }
}

pub struct Diamond {
    machine: Machine,
    chosen: Chosen,
    /// Why the enhancements are set but not drawn, or drawn slowly, for `setting_notes`.
    note: Option<String>,
    /// The device the host lent at create, which the renderer draws on and hands its picture on (since 1.9,
    /// EmuSen_DevicePicture.md §7).
    lend: Option<DeviceLend>,
}

impl Diamond {
    /// The renderer attached, alone or paired with the precision layer.
    fn renderer(&mut self) -> Option<&mut Renderer> {
        let layer: &mut dyn Any = self.machine.layer()?;
        if layer.is::<Renderer>() {
            return layer.downcast_mut();
        }
        layer.downcast_mut::<Precise<Renderer>>().map(|p| &mut p.drawer)
    }

    fn renderer_ref(&self) -> Option<&Renderer> {
        let layer: &dyn Any = self.machine.layer_ref()?;
        if layer.is::<Renderer>() {
            return layer.downcast_ref();
        }
        layer.downcast_ref::<Precise<Renderer>>().map(|p| &p.drawer)
    }

    /// The enhancements set between frames (§42.1): with all at the hardware's values the machine's own picture; else
    /// the renderer, kept across the change and rebuilt for new options, attached alone or paired with the precision
    /// layer, which seeds it from the machine. With no usable Vulkan device the machine is left as off and the reason
    /// kept for the note (§24.2).
    fn choose(&mut self, chosen: Chosen) {
        self.choose_on(chosen, None);
    }

    /// `choose` on the device `device` names a part of, `EMUSEN_GPU_DEVICE`'s or the first usable without one.
    fn choose_on(&mut self, chosen: Chosen, device: Option<&str>) {
        if chosen == self.chosen && (self.renderer_ref().is_some() || self.note.is_some()) {
            return;
        }
        let kept = self.machine.detach().and_then(|layer| {
            let layer: Box<dyn Any> = layer;
            match layer.downcast::<Renderer>() {
                Ok(r) => Some(*r),
                Err(layer) => layer.downcast::<Precise<Renderer>>().ok().map(|p| p.drawer),
            }
        });
        self.chosen = chosen;
        self.note = None;
        if !chosen.drawn() {
            return;
        }
        let made = match kept {
            Some(mut r) => r.configure(chosen.options).map(|_| r),
            None => self.renderer_on_lend(chosen.options).unwrap_or_else(|| Renderer::with(device, chosen.options)),
        };
        match made {
            Ok(r) => {
                self.note = r.software().then(|| format!("Drawn by {}, a Vulkan the processor runs, since no graphics card was found: the picture is the same, but slow.", r.report()));
                let layer: Box<dyn Layer> = if chosen.precise { Box::new(Precise::new(Precision::default(), r)) } else { Box::new(r) };
                self.machine.attach(layer);
            }
            Err(why) => self.note = Some(format!("No usable Vulkan graphics card was found ({why}), so the console's own picture is shown.")),
        }
    }

    /// The frame's work sent to the device, and the picture of the frame before handed to the host (§38.1); also when
    /// rendering is skipped, since the renderer must keep step.
    fn frame_end(&mut self) {
        if let Some(r) = self.renderer() {
            r.frame_end();
        }
    }

    /// The renderer on the lent device, or `None` when nothing is lent or the lend cannot be drawn on, the renderer
    /// then making a device of its own.
    fn renderer_on_lend(&self, options: Options) -> Option<Result<Renderer, String>> {
        let l = self.lend?;
        // SAFETY: the lend's handles and loader are the host's, live until the machine is destroyed (EmuSen_CoreAPI.md §6.29).
        let lent = unsafe { diamond_renderer::Lent::from_raw(l.get_instance_proc_addr?, l.instance, l.physical_device, l.device, l.api_version, l.queue_family, l.queue_index, l.host_queue_family) };
        // SAFETY: as above.
        unsafe { Renderer::lent_or(None, options, Some((lent, l.done))) }.ok().map(Ok)
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
    const CAPABILITIES: u64 = <Machine as Core>::CAPABILITIES | caps::SETTINGS | caps::SETTING_NOTES | caps::DEVICE_PICTURE;

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
        let mut d = Diamond { machine: Machine::create(request)?, chosen: Chosen::default(), note: None, lend: request.device };
        d.choose(Chosen::default().with(&request.settings));
        Ok(d)
    }

    /// The machine's, with the largest picture the largest internal resolution makes, which a host may size its buffer
    /// from once (§39.4).
    fn machine_info(&self) -> MachineInfo {
        let mut info = self.machine.machine_info();
        info.video.max_width *= diamond_renderer::device::MAX_SCALE;
        info.video.max_height *= diamond_renderer::device::MAX_SCALE;
        info
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
        match self.renderer_ref() {
            Some(r) => {
                let (w, h) = r.handed_size();
                FrameInfo { aspect_num: 4, aspect_den: 3, ..FrameInfo::rgba(w as i32, h as i32) }
            }
            None => self.machine.frame_info(),
        }
    }

    fn frame(&self) -> &[u8] {
        match self.renderer_ref() {
            Some(r) => r.handed().0,
            None => self.machine.frame(),
        }
    }

    /// The renderer's picture where it lies on the lent device, this frame's; `None` with no renderer or no lend.
    fn frame_device(&mut self) -> Result<Option<DeviceFrame>, i32> {
        let Some(r) = self.renderer_ref() else { return Ok(None) };
        Ok(r.tell_handed().map(|(buffer, w, h, ready, value)| DeviceFrame {
            size: 0,
            format: pixel::RGBA8888,
            buffer: buffer.raw(),
            offset: 0,
            width: w as i32,
            height: h as i32,
            stride: w as i32 * 4,
            reserved: 0,
            ready: ready.raw(),
            value,
        }))
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

    /// Run-scope keys only reach here, checked against the schema: the machine's own are all read at create. A change
    /// of the bound of settling pictures is a change of machine info.
    fn set_settings(&mut self, settings: &Settings) -> Result<(), i32> {
        let settles = self.picture_settles();
        self.choose(self.chosen.with(settings));
        if self.picture_settles() != settles {
            emit(event::MACHINE_INFO, 0, 0);
        }
        Ok(())
    }

    /// 0 while the machine's own picture is shown; with the renderer, the frames §52.4 measured with a margin.
    fn picture_settles(&self) -> u32 {
        match self.renderer_ref() {
            None => 0,
            Some(r) if r.options().colour => SETTLES_TRUE_COLOUR,
            Some(_) => SETTLES,
        }
    }

    fn setting_notes(&self) -> Vec<(String, String)> {
        self.note.iter().flat_map(|n| self.chosen.keys().into_iter().map(move |k| (k.to_owned(), n.clone()))).collect()
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

emusen_native::core_exports!(Diamond; axes, debug, debug_stack, debug_registers, debug_disassemble, debug_reads, media, settings, setting_notes, device_picture);

#[cfg(test)]
mod tests;
