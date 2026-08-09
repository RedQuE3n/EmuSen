// The contract every reference emulator implements - see EmuSen_Debugging_Tools_Reference_v5.md §3.50.
//
// Nothing in this module names an emulator. A backend is the only place that
// knows what "Mesen" or "libretro" means; the policy layer above (main.rs)
// parses arguments, schedules presses, anchors on state and writes dumps
// without ever naming one.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProbeButton {
    A,
    B,
    X,
    Y,
    L,
    R,
    Up,
    Down,
    Left,
    Right,
    Start,
    Select,
}

impl ProbeButton {
    pub const ALL: [ProbeButton; 12] = [
        ProbeButton::A,
        ProbeButton::B,
        ProbeButton::X,
        ProbeButton::Y,
        ProbeButton::L,
        ProbeButton::R,
        ProbeButton::Up,
        ProbeButton::Down,
        ProbeButton::Left,
        ProbeButton::Right,
        ProbeButton::Start,
        ProbeButton::Select,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ProbeButton::A => "A",
            ProbeButton::B => "B",
            ProbeButton::X => "X",
            ProbeButton::Y => "Y",
            ProbeButton::L => "L",
            ProbeButton::R => "R",
            ProbeButton::Up => "Up",
            ProbeButton::Down => "Down",
            ProbeButton::Left => "Left",
            ProbeButton::Right => "Right",
            ProbeButton::Start => "Start",
            ProbeButton::Select => "Select",
        }
    }

    // Case-sensitive, matching the C++ probe: a script that says "start" is a
    // typo worth reporting, not a spelling to guess at.
    pub fn from_name(name: &str) -> Option<ProbeButton> {
        ProbeButton::ALL.into_iter().find(|b| b.name() == name)
    }

    pub fn index(self) -> usize {
        ProbeButton::ALL.iter().position(|&b| b == self).unwrap()
    }
}

// How to read the bytes a backend hands back for the screen. A dump is always
// raw - what the emulator actually had - and the manifest records which of
// these it is, so a consumer never has to guess from the file size.
// PaletteIndex16 is emitted only by a native backend, so a libretro-only build
// never constructs it; the format still has to be nameable because a consumer
// reads these names back out of a manifest this build did not write.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[allow(dead_code)]
pub enum ScreenFormat {
    #[default]
    None,
    PaletteIndex16,
    Bgr555,
    Rgb565,
    Xrgb8888,
}

impl ScreenFormat {
    pub fn name(self) -> &'static str {
        match self {
            ScreenFormat::PaletteIndex16 => "PaletteIndex16",
            ScreenFormat::Bgr555 => "Bgr555",
            ScreenFormat::Rgb565 => "Rgb565",
            ScreenFormat::Xrgb8888 => "Xrgb8888",
            ScreenFormat::None => "None",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TraceKind {
    Cpu,
    Gsu,
    ApuWrites,
}

// One named block of emulator memory. Name is lowercase and becomes the middle
// field of <backend>_<name>_f<frame>.bin, which is why it is a string rather
// than an enum: a backend may expose spaces this tool has never heard of.
//
// The slice borrows the backend, which is the one place this port is not a
// transcription: the C++ returned a vector of raw pointers by value, so
// `Find(backend->Spaces(), name)` handed back a pointer into a temporary that
// died at the semicolon. See §3.50's note on the anchor lifetime.
pub struct MemorySpace<'a> {
    pub name: String,
    pub data: &'a [u8],
}

impl MemorySpace<'_> {
    pub fn size(&self) -> u32 {
        self.data.len() as u32
    }
}

pub struct ScreenView<'a> {
    pub data: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub format: ScreenFormat,
}

impl ScreenView<'_> {
    pub fn bytes(&self) -> u32 {
        self.data.len() as u32
    }
}

// What the backend decided the machine *is*, as opposed to what it did. A
// differential run compares this before it compares a single pixel, because two
// emulators can resolve the same file to different boards and neither will say
// so - see EmuSen_Debugging_Tools_Reference_v5.md §3.48.
//
// Every field may be left empty. "Unknown" is a real answer here and a useful
// one: a libretro core cannot report its mapper, and a gate that treats silence
// as agreement is worse than one that reports reduced confidence.
#[derive(Clone, Default, Debug)]
pub struct ProbeIdentity {
    pub board: String,
    pub region: String,
    pub header_trust: String,
    pub prg_bytes: u64,
    pub chr_bytes: u64,
    pub save_loaded: bool,
}

#[derive(Clone, Debug)]
pub struct ProbeOptions {
    pub ram_state: String,
    pub wav_path: String,
    // Kept beside the dumps, so a probe run never touches a real emulator profile
    // and its .srm never silently changes what the ROM boots into.
    pub home_folder: String,
}

impl Default for ProbeOptions {
    fn default() -> Self {
        ProbeOptions {
            ram_state: "zeros".to_string(),
            wav_path: String::new(),
            home_folder: String::new(),
        }
    }
}

// A --press script, in frames and buttons and nothing else. Parsing it is policy;
// *sampling* it has to be the backend's job, because only the backend knows when
// its emulator polls the pad - and a probe that free-runs between reports cannot
// stop precisely enough to set the pad itself without dropping presses.
#[derive(Clone, Default, Debug)]
pub struct InputSchedule {
    pub entries: Vec<ScheduleEntry>,
}

#[derive(Clone, Copy, Debug)]
pub struct ScheduleEntry {
    pub start_frame: u32,
    pub end_frame: u32,
    pub button: ProbeButton,
}

impl InputSchedule {
    pub fn add(&mut self, start: u32, duration: u32, button: ProbeButton) {
        self.entries.push(ScheduleEntry {
            start_frame: start,
            end_frame: start + duration,
            button,
        });
    }

    // Sampled by a backend that resolves the schedule itself. The Mesen backend
    // hands the entries to C++ instead, because only a vtable Mesen can call
    // gets to see the moment the pad is polled - so a mesen-only build never
    // reaches this, and that absence is correct rather than dead.
    #[allow(dead_code)]
    pub fn held_at(&self, frame: u32, button: ProbeButton) -> bool {
        self.entries
            .iter()
            .any(|e| e.button == button && frame >= e.start_frame && frame < e.end_frame)
    }
}

pub trait ProbeBackend {
    // Becomes the dump prefix, so an existing Mesen dump set keeps its filenames.
    fn name(&self) -> &str;
    // Which machine the loaded ROM turned out to be: "nes", "snes", ...
    fn system(&self) -> &str;

    // What this backend resolved the cartridge to be. Empty fields are honest.
    fn identity(&self) -> ProbeIdentity {
        ProbeIdentity::default()
    }

    fn load(&mut self, rom_path: &str, options: &ProbeOptions) -> bool;
    fn shutdown(&mut self) {}

    // Advances to at least `frame`, then stops on a frame boundary. Every other
    // call here is only valid while stopped, and that is the whole point: the
    // old probe raced a free-running emulator, which is why --pressuntil needed
    // an atomic live key and why §3.39 warns about cached frame counters.
    fn run_until(&mut self, frame: u32);
    fn frame_count(&self) -> u32;

    fn spaces(&self) -> Vec<MemorySpace<'_>>;
    // Which space --pressuntil watches when none is named: GSU RAM on a SNES,
    // internal RAM on an NES. Empty means this backend cannot be anchored.
    fn anchor_space(&self) -> &str;

    // The frame-indexed script, fixed before boot. Sampled by the backend.
    fn set_schedule(&mut self, _schedule: InputSchedule) {}
    // The live override --pressuntil decides at runtime, on top of the schedule.
    fn set_button(&mut self, button: ProbeButton, held: bool);

    fn screen(&self) -> Option<ScreenView<'_>> {
        None
    }

    // The emulator-internal register decode for one report. Anything derivable
    // from spaces() belongs to the policy layer instead, not here.
    fn state_line(&self) -> String {
        String::new()
    }

    // Optional by nature: a per-instruction trace cannot be had from an
    // arbitrary emulator without patching it, so a backend that has no such
    // hook says so instead of silently producing an empty file. Arming is
    // valid before load, which is what the CPU and APU traces need.
    fn begin_trace(&mut self, _kind: TraceKind) -> bool {
        false
    }

    fn end_trace(&mut self, _kind: TraceKind) -> Option<Vec<u8>> {
        None
    }
}

pub fn find_space<'s, 'a>(spaces: &'s [MemorySpace<'a>], name: &str) -> Option<&'s MemorySpace<'a>> {
    spaces.iter().find(|s| s.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_names_round_trip() {
        for button in ProbeButton::ALL {
            assert_eq!(ProbeButton::from_name(button.name()), Some(button));
        }
        assert_eq!(ProbeButton::from_name("start"), None);
        assert_eq!(ProbeButton::from_name("Nope"), None);
    }

    #[test]
    fn button_index_matches_declaration_order() {
        assert_eq!(ProbeButton::A.index(), 0);
        assert_eq!(ProbeButton::Select.index(), 11);
    }

    #[test]
    fn schedule_is_half_open() {
        let mut schedule = InputSchedule::default();
        schedule.add(10, 4, ProbeButton::Start);
        assert!(!schedule.held_at(9, ProbeButton::Start));
        assert!(schedule.held_at(10, ProbeButton::Start));
        assert!(schedule.held_at(13, ProbeButton::Start));
        assert!(!schedule.held_at(14, ProbeButton::Start));
        assert!(!schedule.held_at(11, ProbeButton::A));
    }
}
