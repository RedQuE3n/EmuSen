// The Mesen backend - see EmuSen_Debugging_Tools_Reference_v5.md §3.52.
//
// Thin by design. Everything that needs a C++ compiler stayed in Mesen's own
// tree (Core/Shared/EmuSenProbeApi.cpp, added by probe-c-api.patch): the
// IKeyManager subclass whose vtable no other language can lay out, and the
// register decode that reads four by-value state structs. What is left here is
// marshalling, and marshalling is all this file should ever be - a rule worth
// stating because the temptation is to creep emulator knowledge back across the
// boundary one convenience at a time.
//
// Unlike the libretro backend there is no dlopen: the probe links MesenCore.so
// directly, so this file holds no handle and no lifetime. The emulator is a
// process-global singleton on the far side, exactly as it was for the C++
// backend, which is sound because a probe drives one machine by construction.
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

use crate::backend::{
    InputSchedule, MemorySpace, ProbeBackend, ProbeButton, ProbeIdentity, ProbeOptions,
    ScreenFormat, ScreenView, TraceKind,
};
use crate::mesen_sys as sys;

// Checked before the first call rather than trusted, because the two sides of
// this ABI live in different repositories: a checkout carrying an older
// probe-c-api.patch would otherwise be a wrong dump instead of an error.
pub fn check_abi() -> bool {
    let found = unsafe { sys::probe_abi_version() };
    if found == sys::PROBE_ABI_VERSION {
        return true;
    }
    println!(
        "[ERROR] Mesen probe ABI is version {found}, this probe speaks {}",
        sys::PROBE_ABI_VERSION
    );
    println!("[ERROR] re-run ./build-probe.sh mesen <checkout> to reapply patches/mesen/probe-c-api.patch");
    false
}

#[derive(Default)]
pub struct MesenBackend {
    loaded: bool,
    // Kept only so name()/system() can hand out a &str; the C side owns the
    // authoritative answer and is asked again on every call that matters.
    system: &'static str,
}

impl MesenBackend {
    pub fn new() -> MesenBackend {
        MesenBackend { loaded: false, system: "snes" }
    }
}

fn trace_kind(kind: TraceKind) -> c_int {
    match kind {
        TraceKind::Cpu => sys::TRACE_CPU,
        TraceKind::Gsu => sys::TRACE_GSU,
        TraceKind::ApuWrites => sys::TRACE_APU_WRITES,
    }
}

fn screen_format(value: c_int) -> ScreenFormat {
    match value {
        sys::FORMAT_PALETTE_INDEX16 => ScreenFormat::PaletteIndex16,
        sys::FORMAT_BGR555 => ScreenFormat::Bgr555,
        sys::FORMAT_RGB565 => ScreenFormat::Rgb565,
        sys::FORMAT_XRGB8888 => ScreenFormat::Xrgb8888,
        _ => ScreenFormat::None,
    }
}

// Empty rather than a panic: every string crossing this boundary is either a
// literal or a std::string the C++ side is holding alive, so a null here means
// the ABI is wrong, and the version check is where that is reported.
fn to_string(raw: *const c_char) -> String {
    if raw.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(raw) }.to_string_lossy().into_owned()
}

impl ProbeBackend for MesenBackend {
    fn name(&self) -> &str {
        "mesen"
    }

    fn system(&self) -> &str {
        self.system
    }

    fn anchor_space(&self) -> &str {
        if self.system == "nes" { "ram" } else { "gsuram" }
    }

    fn load(&mut self, rom_path: &str, options: &ProbeOptions) -> bool {
        let rom = CString::new(rom_path).unwrap_or_default();
        let ram = CString::new(options.ram_state.as_str()).unwrap_or_default();
        let wav = CString::new(options.wav_path.as_str()).unwrap_or_default();
        let home = CString::new(options.home_folder.as_str()).unwrap_or_default();

        let ok = unsafe { sys::probe_load(rom.as_ptr(), ram.as_ptr(), wav.as_ptr(), home.as_ptr()) } != 0;
        if !ok {
            return false;
        }

        // Asked once, after load: before it, the console has not been built and
        // the answer would be "snes" for every machine.
        self.system = if to_string(unsafe { sys::probe_system() }) == "nes" { "nes" } else { "snes" };
        self.loaded = true;
        true
    }

    fn shutdown(&mut self) {
        if !self.loaded {
            return;
        }
        self.loaded = false;
        unsafe { sys::probe_shutdown() };
    }

    fn run_until(&mut self, frame: u32) {
        unsafe { sys::probe_run_until(frame) };
    }

    fn frame_count(&self) -> u32 {
        unsafe { sys::probe_frame_count() }
    }

    // The slices alias the emulator's own memory rather than copying it, which
    // is the point: a debugger-mediated read would perturb the timing being
    // measured. Their lifetime is this borrow of self, which is what makes the
    // C++ probe's dangling-anchor bug unwritable here - see §3.50.
    fn spaces(&self) -> Vec<MemorySpace<'_>> {
        let count = unsafe { sys::probe_space_count() };
        let mut out = Vec::with_capacity(count as usize);
        for index in 0..count {
            let data = unsafe { sys::probe_space_data(index) };
            let size = unsafe { sys::probe_space_size(index) } as usize;
            if data.is_null() || size == 0 {
                continue;
            }
            out.push(MemorySpace {
                name: to_string(unsafe { sys::probe_space_name(index) }),
                data: unsafe { std::slice::from_raw_parts(data, size) },
            });
        }
        out
    }

    fn set_schedule(&mut self, schedule: InputSchedule) {
        let mut flat: Vec<u32> = Vec::with_capacity(schedule.entries.len() * 3);
        for entry in &schedule.entries {
            flat.push(entry.start_frame);
            flat.push(entry.end_frame);
            flat.push(entry.button.index() as u32);
        }
        // Copied on the far side, so the vector may die at the semicolon.
        unsafe { sys::probe_set_schedule(flat.as_ptr(), schedule.entries.len() as u32) };
    }

    fn set_button(&mut self, button: ProbeButton, held: bool) {
        unsafe { sys::probe_set_button(button.index() as c_int, held as c_int) };
    }

    fn screen(&self) -> Option<ScreenView<'_>> {
        let mut data: *const c_void = std::ptr::null();
        let (mut width, mut height, mut bytes) = (0u32, 0u32, 0u32);
        let mut format: c_int = sys::FORMAT_NONE;
        let ok = unsafe {
            sys::probe_screen(&mut data, &mut width, &mut height, &mut bytes, &mut format)
        };
        if ok == 0 || data.is_null() || bytes == 0 {
            return None;
        }
        Some(ScreenView {
            data: unsafe { std::slice::from_raw_parts(data as *const u8, bytes as usize) },
            width,
            height,
            format: screen_format(format),
        })
    }

    fn identity(&self) -> ProbeIdentity {
        let mut board: *const c_char = std::ptr::null();
        let mut region: *const c_char = std::ptr::null();
        let (mut prg, mut chr) = (0u64, 0u64);
        let mut save: c_int = 0;
        unsafe { sys::probe_identity(&mut board, &mut region, &mut prg, &mut chr, &mut save) };
        ProbeIdentity {
            board: to_string(board),
            region: to_string(region),
            // Deliberately empty: this reference has no notion of how much of a
            // header it believed, so the gate must read it as "cannot say".
            header_trust: String::new(),
            prg_bytes: prg,
            chr_bytes: chr,
            save_loaded: save != 0,
        }
    }

    fn state_line(&self) -> String {
        to_string(unsafe { sys::probe_state_line() })
    }

    fn begin_trace(&mut self, kind: TraceKind) -> bool {
        unsafe { sys::probe_trace_begin(trace_kind(kind)) != 0 }
    }

    fn end_trace(&mut self, kind: TraceKind) -> Option<Vec<u8>> {
        let mut data: *const u8 = std::ptr::null();
        let mut size: u32 = 0;
        if unsafe { sys::probe_trace_end(trace_kind(kind), &mut data, &mut size) } == 0 {
            return None;
        }
        if data.is_null() || size == 0 {
            return Some(Vec::new());
        }
        // Copied here rather than borrowed: the far side keeps the buffer only
        // until the next probe_trace_end, and a run may end two traces.
        Some(unsafe { std::slice::from_raw_parts(data, size as usize) }.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The C ABI's enum values are a contract, not an implementation detail: the
    // C++ side numbers them by declaration order and cannot see this enum. A
    // reordering of backend::ProbeButton would otherwise silently remap the pad.
    #[test]
    fn button_indices_match_the_abi() {
        assert_eq!(ProbeButton::ALL.len() as u32, sys::BUTTON_COUNT);
        assert_eq!(ProbeButton::A.index(), 0);
        assert_eq!(ProbeButton::B.index(), 1);
        assert_eq!(ProbeButton::X.index(), 2);
        assert_eq!(ProbeButton::Y.index(), 3);
        assert_eq!(ProbeButton::L.index(), 4);
        assert_eq!(ProbeButton::R.index(), 5);
        assert_eq!(ProbeButton::Up.index(), 6);
        assert_eq!(ProbeButton::Down.index(), 7);
        assert_eq!(ProbeButton::Left.index(), 8);
        assert_eq!(ProbeButton::Right.index(), 9);
        assert_eq!(ProbeButton::Start.index(), 10);
        assert_eq!(ProbeButton::Select.index(), 11);
    }

    #[test]
    fn trace_kinds_match_the_abi() {
        assert_eq!(trace_kind(TraceKind::Cpu), 0);
        assert_eq!(trace_kind(TraceKind::Gsu), 1);
        assert_eq!(trace_kind(TraceKind::ApuWrites), 2);
    }

    #[test]
    fn screen_formats_match_the_abi() {
        assert_eq!(screen_format(1), ScreenFormat::PaletteIndex16);
        assert_eq!(screen_format(2), ScreenFormat::Bgr555);
        assert_eq!(screen_format(3), ScreenFormat::Rgb565);
        assert_eq!(screen_format(4), ScreenFormat::Xrgb8888);
        // An unknown value is None rather than a panic: a newer patch may name a
        // format this probe has never heard of, and the manifest should say so.
        assert_eq!(screen_format(0), ScreenFormat::None);
        assert_eq!(screen_format(99), ScreenFormat::None);
    }
}
