//! What the host is told beside the frame: the two processors' registers and the VI's as a unit for `DEBUG_REGISTERS`,
//! the setting notes for `SETTING_NOTES`, the IS-Viewer's lines and the device's report in the log, the frame rate
//! machine info gives, and since core ABI 1.13 the counters for `TELEMETRY`, the picture's lines and the frame that holds
//! cheats. See Mars_Native.md §9 and §10.

use emusen_native::core::outbox::{Level, log};
use emusen_native::core::{Counter, Unit, UNIT_FIRST_ID, event};

use super::Core;
use crate::machine::CYCLE_CAP;

/// The VR4300's general registers as `MarsOperands.Gpr` names them, which the debugger's commands take.
pub const GPR: [&str; 32] = [
    "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7", "s0", "s1", "s2", "s3", "s4", "s5", "s6",
    "s7", "t8", "t9", "k0", "k1", "gp", "sp", "s8", "ra",
];

/// The processor's clock, which the frame's cycles count.
const PROCESSOR_CLOCK: u64 = 93_750_000;

/// The log's category for the IS-Viewer's lines and for the device's report.
const IS_VIEWER: &str = "IS-Viewer";
const DEVICE: &str = "MarsRT";

/// Processor 0's registers, in `debug_registers`' order: PC, the 32 GPRs, HI, LO and the bus's cycles, all 64 bits.
pub fn cpu_registers() -> Vec<(String, u32)> {
    std::iter::once("PC").chain(GPR).chain(["HI", "LO", "Cycles"]).map(|n| (n.to_string(), 64)).collect()
}

/// Processor 1's: the RSP's PC (12 bits of IMEM, shown as 16), whether it is halted, and its 32 scalar registers.
pub fn rsp_registers() -> Vec<(String, u32)> {
    [("PC".to_string(), 16), ("Halted".to_string(), 1)].into_iter().chain(GPR.iter().map(|n| (n.to_string(), 32))).collect()
}

/// The VI, the one part that runs no code whose registers the debugger shows: a `video` unit, by libultra's names.
pub const VI_UNIT: u32 = UNIT_FIRST_ID;
pub const VI_REGISTERS: [&str; 14] =
    ["CONTROL", "ORIGIN", "WIDTH", "V_INTR", "V_CURRENT", "BURST", "V_SYNC", "H_SYNC", "LEAP", "H_START", "V_START", "V_BURST", "X_SCALE", "Y_SCALE"];

/// Machine info's units: the VI, its fourteen registers of 32 bits as stored.
pub fn units() -> Vec<Unit> {
    vec![Unit { id: VI_UNIT, name: "VI".into(), kind: "video".into(), registers: VI_REGISTERS.iter().map(|n| (n.to_string(), 32)).collect() }]
}

/// The counters `telemetry` gives, in order: the machine's seven, the display processor's thread's, the recompiler's.
pub const COUNTERS: usize = 7 + super::threads::COUNTERS + crate::cpu::blocks::COUNTERS;

/// The display processor's thread's sites by `dp_threads::site`'s numbers.
const SITES: [&str; crate::memory::dp_threads::site::COUNT] =
    ["bus_read", "bus_write", "load", "store", "fetch", "block", "si", "ai", "vi", "cheat", "take", "sp_dma"];

fn counter(name: &str, label: &str, unit: &str, kind: &str) -> Counter {
    Counter { name: name.into(), label: label.into(), unit: unit.into(), kind: kind.into() }
}

/// Machine info's `telemetry`, the same list for every machine, its parts that are off reading 0.
pub fn counters() -> Vec<Counter> {
    let mut list = vec![
        counter("cpu.cycles", "Processor cycles", "cycles", "total"),
        counter("frames", "Frames run", "count", "total"),
        counter("frame.cycles", "Cycles in the last frame", "cycles", "frame"),
        counter("cpu.instructions", "Instructions", "instructions", "total"),
        counter("cpu.idle.turns", "Idle loop turns passed whole", "count", "total"),
        counter("cpu.idle.instructions", "Instructions the idle loop passed", "instructions", "total"),
        counter("cpu.exceptions", "Exceptions entered", "count", "total"),
        counter("rdp.thread", "Display processor on its own thread", "count", "level"),
        counter("rdp.drain.words", "Words the display processor's thread drained", "count", "total"),
        counter("rdp.drain.starts", "Drains started", "count", "total"),
        counter("rdp.drain.ns", "Time draining", "ns", "total"),
        counter("rdp.wait.page", "Waits for a page", "count", "total"),
        counter("rdp.wait.page.ns", "Time waiting for a page", "ns", "total"),
        counter("rdp.wait.range", "Waits for a range", "count", "total"),
        counter("rdp.wait.range.ns", "Time waiting for a range", "ns", "total"),
        counter("rdp.join", "Joins with the drain", "count", "total"),
        counter("rdp.join.ns", "Time joining the drain", "ns", "total"),
        counter("rdp.bystanders", "Accesses past marks of other pages", "count", "total"),
        counter("rdp.reads.narrowed", "Reads narrowed to their bytes", "count", "total"),
        counter("rdp.reads.freed", "Reads freed of a wait", "count", "total"),
        counter("vi.repeated_scans", "Scans that repeated the last", "count", "total"),
    ];
    list.extend(SITES.iter().map(|s| counter(&format!("rdp.site.{s}.waits"), &format!("Waits at {s}"), "count", "total")));
    list.extend(SITES.iter().map(|s| counter(&format!("rdp.site.{s}.ns"), &format!("Time waiting at {s}"), "ns", "total")));
    list.extend([
        counter("blocks.live", "Compiled blocks alive", "count", "level"),
        counter("blocks.shaped", "Blocks shaped", "count", "total"),
        counter("blocks.discarded", "Blocks discarded", "count", "total"),
        counter("blocks.entries", "Block entries", "count", "total"),
        counter("blocks.instructions", "Instructions run in blocks", "instructions", "total"),
        counter("blocks.stepped", "Instructions the interpreter stepped", "instructions", "total"),
        counter("blocks.mapped", "Entries into mapped code", "count", "total"),
        counter("blocks.compiled", "Blocks compiled", "count", "total"),
        counter("blocks.compile.ns", "Time compiling", "ns", "total"),
        counter("blocks.code.bytes", "Bytes of compiled code", "bytes", "level"),
        counter("blocks.compiled.entries", "Entries into compiled blocks", "count", "total"),
        counter("blocks.compiled.instructions", "Instructions run compiled", "instructions", "total"),
        counter("blocks.refused", "Blocks the code generator refused", "count", "total"),
    ]);
    list
}

/// The antialiasing row's note as `N64Console.AntialiasingNote` wrote it: what the chosen level becomes at the chosen
/// resolution, when that is less (Mars_Performance.md §42.5).
pub fn antialiasing_note(scale: i32, antialiasing: i32) -> Option<String> {
    let scale = scale.clamp(1, 4);
    let wanted = antialiasing.clamp(1, 4);
    let effective = wanted.min(4 / scale).max(1);
    if effective >= wanted {
        return None;
    }
    Some(if effective == 1 {
        format!("Off at {scale}x resolution: the two together are held to four, so {wanted}x has no effect.")
    } else {
        format!("{effective}x at {scale}x resolution: the two together are held to four.")
    })
}

impl Core {
    /// `telemetry`'s values in `counters`' order into `out` as far as it holds them; how many there are. No allocation.
    pub fn telemetry_into(&self, out: &mut [i64]) -> usize {
        let m = &self.machine;
        let head = [m.bus.cycles, m.total_frames, m.last_frame_cycles, m.cpu.instructions, m.cpu.run.idle_turns_passed, m.cpu.run.idle_instructions, m.cpu.run.exceptions];
        let threads = self.thread_counters();
        let blocks = m.blocks.counters();
        for (o, v) in out.iter_mut().zip(head.iter().chain(threads.iter()).chain(blocks.iter())) {
            *o = *v;
        }
        COUNTERS
    }

    /// The picture's lines: its rows over the multiple its width is of the raster's, or none where it is no multiple.
    pub fn lines(width: u32, height: u32) -> i32 {
        let raster = crate::vi::scan::RASTER_WIDTH as u32;
        if width >= raster && width % raster == 0 { (height / (width / raster)) as i32 } else { 0 }
    }

    /// Whether the frame just run ended where Project64 holds its cheats, interrupts disabled (Mars_Cheats.md §5.1).
    pub fn cheats_held(&self) -> bool {
        self.machine.cpu.cop0[crate::cpu::cop0::STATUS] & 1 == 0
    }

    /// `debug_registers`' values for processor 0 or 1, as the shim's `CpuRegisters` and `RspRegisters` read them, and the
    /// VI's unit.
    pub fn registers(&self, processor: u32) -> Option<Vec<i64>> {
        match processor {
            VI_UNIT => Some((0..crate::vi::REGISTERS).map(|i| self.machine.bus.vi.read32(i * 4) as i64).collect()),
            0 => {
                let cpu = &self.machine.cpu;
                let mut values = Vec::with_capacity(36);
                values.push(cpu.pc as i64);
                values.extend(cpu.gpr.iter().map(|&r| r as i64));
                values.extend([cpu.hi as i64, cpu.lo as i64, self.machine.bus.cycles]);
                Some(values)
            }
            1 => {
                let rsp = &self.machine.bus.sp.processor;
                let mut values = Vec::with_capacity(34);
                values.extend([rsp.pc as i64, rsp.halted as i64]);
                values.extend(rsp.gpr.iter().map(|&r| r as i64));
                Some(values)
            }
            _ => None,
        }
    }

    /// The notes for the values in force: the antialiasing the resolution holds back, and the device setting when it got no
    /// device at a multiple, in the words the last attempt gave.
    pub fn notes(&self) -> Vec<(String, String)> {
        let mut notes = Vec::new();
        let (scale, antialiasing, gpu) = self.applied.map_or((1, 1, false), |v| (v.scale, v.antialiasing, v.gpu));
        if let Some(note) = antialiasing_note(scale, antialiasing) {
            notes.push((super::v1::ANTIALIASING.to_string(), note));
        }
        let drawing = &self.machine.bus.dp.multiple;
        if gpu && scale > 1 && drawing.gpu.is_none() && !matches!(drawing.gpu_report.as_str(), "off" | "off at one") {
            notes.push((super::v1::GPU.to_string(), drawing.gpu_report.clone()));
        }
        notes
    }

    /// The frame rate machine info gives: the field rate the VI's registers program, or before they do, a frame of the cap's
    /// length, which is what the frame then is.
    pub fn frame_rate(&self) -> (u64, u64) {
        self.machine.bus.vi.field_rate().unwrap_or((PROCESSOR_CLOCK, CYCLE_CAP as u64))
    }

    /// After a call that ran the machine or presented: the IS-Viewer's whole lines and a changed device report into the log,
    /// and `MACHINE_INFO` when the frame rate moved. A comparison or two when nothing changed; no allocation.
    pub fn tell_host(&mut self) {
        self.log_is_viewer();
        self.log_device();
        let rate = self.frame_rate();
        if rate != self.rate_told {
            self.rate_told = rate;
            emusen_native::core::emit(event::MACHINE_INFO, 0, 0);
        }
    }

    fn log_is_viewer(&mut self) {
        let text = &self.machine.bus.is_viewer.text;
        if self.is_viewer_logged > text.len() {
            self.is_viewer_logged = 0;
        }
        let mut from = self.is_viewer_logged;
        while let Some(end) = text[from..].iter().position(|&b| b == b'\n') {
            let line = String::from_utf8_lossy(&text[from..from + end]);
            log(Level::Info, IS_VIEWER, line.trim_end_matches('\r'));
            from += end + 1;
        }
        self.is_viewer_logged = from;
    }

    fn log_device(&mut self) {
        let drawing = &self.machine.bus.dp.multiple;
        if drawing.gpu_report == self.gpu_logged {
            return;
        }
        if drawing.gpu.is_some() {
            log(Level::Info, DEVICE, &format!("The multiple is drawn on {}.", drawing.gpu_report));
        } else if !matches!(drawing.gpu_report.as_str(), "off" | "off at one") {
            log(Level::Warn, DEVICE, &format!("No device draws the multiple: {}", drawing.gpu_report));
        }
        self.gpu_logged = drawing.gpu_report.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::v1::*;
    use emusen_native::core::exports::core_of_mut;
    use emusen_native::core::sys::{ABI_VERSION, CreateParams, Event, FileEntry, Machine as Handle};
    use std::ptr::null_mut;

    fn bytes(f: impl Fn(*mut u8, usize) -> i64) -> Vec<u8> {
        let n = f(null_mut(), 0);
        assert!(n >= 0, "status {n}");
        let mut out = vec![0u8; n as usize];
        assert_eq!(f(out.as_mut_ptr(), out.len()), n);
        out
    }

    /// A machine through the exports, on a cartridge of the magic word and an empty boot code, as the kit makes one.
    fn make() -> (*mut Handle, Vec<u8>) {
        let mut image = vec![0u8; 0x2000];
        image[..4].copy_from_slice(&[0x80, 0x37, 0x12, 0x40]);
        let settings = "ThreadedRdp=false\nDeferredPresentation=false\nRecompiler=false";
        let files: [FileEntry; 0] = [];
        let p = CreateParams {
            size: std::mem::size_of::<CreateParams>() as u32,
            host_abi_version: ABI_VERSION,
            image: image.as_ptr(),
            image_len: image.len(),
            settings: settings.as_ptr(),
            settings_len: settings.len(),
            files: files.as_ptr(),
            file_count: 0,
            file_size: std::mem::size_of::<FileEntry>(),
            pixel_formats: 1,
            error: null_mut(),
            error_len: 0,
            flags: 0,
            device: std::ptr::null(),
            system: std::ptr::null(),
        };
        let mut code = 1;
        let m = unsafe { emusen_core_create(&p, &mut code) };
        assert!(!m.is_null() && code == 0, "create refused: {code}");
        (m, image)
    }

    fn events(m: *mut Handle) -> Vec<u32> {
        let n = unsafe { emusen_core_events(m, null_mut(), 0, 0) } as usize;
        let mut out = vec![Event::default(); n];
        unsafe { emusen_core_events(m, out.as_mut_ptr(), n, std::mem::size_of::<Event>()) };
        out.iter().map(|e| e.kind).collect()
    }

    fn info(m: *mut Handle) -> String {
        String::from_utf8(bytes(|o, l| unsafe { emusen_core_machine_info(m, o, l) })).unwrap()
    }

    #[test]
    fn the_frame_rate_is_the_cap_s_until_the_vi_is_programmed_then_the_field_s_and_the_host_is_told() {
        let (m, _image) = make();
        assert!(info(m).contains(r#""frame_rate":{"num":93750000,"den":4100000}"#), "{}", info(m));
        let core = unsafe { core_of_mut::<Core>(m) }.unwrap();
        core.machine.bus.vi.registers[(crate::vi::VERTICAL_SYNC >> 2) as usize] = 625;
        core.machine.bus.vi.registers[(crate::vi::HORIZONTAL_SYNC >> 2) as usize] = 3177;
        events(m);
        assert_eq!(unsafe { emusen_core_advance(m, null_mut()) }, 0);
        assert!(events(m).contains(&emusen_native::core::event::MACHINE_INFO), "no MACHINE_INFO for the new rate");
        assert!(info(m).contains(r#""frame_rate":{"num":99313060,"den":1985625}"#), "{}", info(m));
        assert_eq!(unsafe { emusen_core_advance(m, null_mut()) }, 0);
        assert!(!events(m).contains(&emusen_native::core::event::MACHINE_INFO), "told again with nothing changed");
        unsafe { emusen_core_free(m) };
    }

    #[test]
    fn the_is_viewer_s_whole_lines_reach_the_log_and_a_part_line_waits() {
        let (m, _image) = make();
        unsafe { emusen_core_log_drain(m, null_mut(), 0) };
        bytes(|o, l| unsafe { emusen_core_log_drain(m, o, l) });
        let core = unsafe { core_of_mut::<Core>(m) }.unwrap();
        core.machine.bus.is_viewer.text.extend_from_slice(b"first line\r\nsecond");
        unsafe { emusen_core_advance(m, null_mut()) };
        let log = String::from_utf8_lossy(&bytes(|o, l| unsafe { emusen_core_log_drain(m, o, l) })).into_owned();
        assert!(log.contains("first line") && !log.contains("second"), "{log:?}");
        let core = unsafe { core_of_mut::<Core>(m) }.unwrap();
        core.machine.bus.is_viewer.text.extend_from_slice(b" half\n");
        unsafe { emusen_core_advance(m, null_mut()) };
        let log = String::from_utf8_lossy(&bytes(|o, l| unsafe { emusen_core_log_drain(m, o, l) })).into_owned();
        assert!(log.contains("second half") && !log.contains("first line"), "{log:?}");
        unsafe { emusen_core_free(m) };
    }

    #[test]
    fn the_vi_is_a_video_unit_whose_registers_debug_registers_reads() {
        let (m, _image) = make();
        let core = unsafe { core_of_mut::<Core>(m) }.unwrap();
        core.machine.bus.vi.registers[1] = 0x0011_2233;
        assert!(info(m).contains(r#""units":[{"id":256,"name":"VI","kind":"video","registers":[{"name":"CONTROL","bits":32}"#), "{}", info(m));
        let n = unsafe { emusen_core_debug_registers(m, VI_UNIT, null_mut(), 0) };
        let mut v = vec![0i64; n as usize];
        unsafe { emusen_core_debug_registers(m, VI_UNIT, v.as_mut_ptr(), v.len()) };
        assert_eq!((v.len(), v[1]), (14, 0x0011_2233));
        assert!(!info(m).contains(r#""name":"VI","read_only""#), "the VI is no space");
        unsafe { emusen_core_free(m) };
    }

    #[test]
    fn telemetry_gives_every_counter_machine_info_names_without_allocating_a_list() {
        let (m, _image) = make();
        assert_eq!(info(m).matches(r#""kind":"total""#).count() + info(m).matches(r#""kind":"level""#).count() + info(m).matches(r#""kind":"frame""#).count(), COUNTERS);
        assert_eq!(unsafe { emusen_core_advance(m, null_mut()) }, 0);
        let n = unsafe { emusen_core_telemetry(m, null_mut(), 0) };
        assert_eq!(n, COUNTERS as i64);
        let mut v = vec![0i64; COUNTERS];
        unsafe { emusen_core_telemetry(m, v.as_mut_ptr(), v.len()) };
        let core = unsafe { core_of_mut::<Core>(m) }.unwrap();
        assert_eq!((v[0], v[1], v[2], v[3]), (core.machine.bus.cycles, 1, core.machine.last_frame_cycles, core.machine.cpu.instructions));
        unsafe { emusen_core_free(m) };
    }

    #[test]
    fn a_frame_ended_with_interrupts_disabled_holds_cheats_and_the_lines_follow_the_multiple() {
        let (m, _image) = make();
        unsafe { emusen_core_advance(m, null_mut()) };
        let core = unsafe { core_of_mut::<Core>(m) }.unwrap();
        core.machine.cpu.cop0[crate::cpu::cop0::STATUS] &= !1;
        assert_ne!(<Core as emusen_native::core::Core>::frame_info(core).flags & emusen_native::core::flags::FRAME_CHEATS_HELD, 0);
        core.machine.cpu.cop0[crate::cpu::cop0::STATUS] |= 1;
        assert_eq!(<Core as emusen_native::core::Core>::frame_info(core).flags & emusen_native::core::flags::FRAME_CHEATS_HELD, 0);
        assert_eq!((Core::lines(640, 480), Core::lines(1280, 1152), Core::lines(2560, 960), Core::lines(320, 240)), (480, 576, 240, 0));
        unsafe { emusen_core_free(m) };
    }

    #[test]
    fn presence_and_translation_reach_the_machine() {
        let (m, _image) = make();
        assert_eq!(unsafe { emusen_core_set_present(m, 2, 1) }, 0);
        assert!(unsafe { core_of_mut::<Core>(m) }.unwrap().machine.bus.si.controllers[2].present);
        assert!(unsafe { emusen_core_set_present(m, 4, 1) } < 0);
        let mut bus = 0u32;
        assert_eq!(unsafe { emusen_core_debug_translate(m, 0, 0x8000_1234, &mut bus) }, 1);
        assert_eq!(bus, 0x1234);
        assert_eq!(unsafe { emusen_core_debug_translate(m, 0, 0x0000_1000, &mut bus) }, 0);
        unsafe { emusen_core_free(m) };
    }

    #[test]
    fn debug_registers_gives_each_processor_s_values_in_machine_info_s_order() {
        let (m, _image) = make();
        let core = unsafe { core_of_mut::<Core>(m) }.unwrap();
        core.machine.cpu.gpr[29] = 0xA400_1FF0;
        core.machine.cpu.hi = 7;
        core.machine.bus.sp.processor.gpr[3] = 0x1234;
        let read = |p: u32| {
            let n = unsafe { emusen_core_debug_registers(m, p, null_mut(), 0) };
            let mut v = vec![0i64; n as usize];
            unsafe { emusen_core_debug_registers(m, p, v.as_mut_ptr(), v.len()) };
            v
        };
        let cpu = read(0);
        assert_eq!((cpu.len(), cpu[30], cpu[33]), (36, 0xA400_1FF0, 7));
        let rsp = read(1);
        assert_eq!((rsp.len(), rsp[5]), (34, 0x1234));
        assert!(unsafe { emusen_core_debug_registers(m, 2, null_mut(), 0) } < 0);
        unsafe { emusen_core_free(m) };
    }

    #[test]
    fn the_notes_say_what_the_resolution_holds_the_antialiasing_to() {
        let (m, _image) = make();
        let text = "RenderScale=2\nAntialiasing=4x";
        assert_eq!(unsafe { emusen_core_set_settings(m, text.as_ptr(), text.len()) }, 0);
        let notes = String::from_utf8(bytes(|o, l| unsafe { emusen_core_setting_notes(m, o, l) })).unwrap();
        assert_eq!(notes, r#"{"Antialiasing":"2x at 2x resolution: the two together are held to four."}"#);
        unsafe { emusen_core_free(m) };
    }

    #[test]
    fn the_antialiasing_note_is_the_shim_s_words() {
        assert_eq!(antialiasing_note(1, 4), None);
        assert_eq!(antialiasing_note(2, 2), None);
        assert_eq!(antialiasing_note(2, 4).as_deref(), Some("2x at 2x resolution: the two together are held to four."));
        assert_eq!(antialiasing_note(3, 2).as_deref(), Some("Off at 3x resolution: the two together are held to four, so 2x has no effect."));
        assert_eq!(antialiasing_note(4, 4).as_deref(), Some("Off at 4x resolution: the two together are held to four, so 4x has no effect."));
    }

    #[test]
    fn the_register_lists_are_the_shim_s() {
        let cpu = cpu_registers();
        assert_eq!(cpu.len(), 36);
        assert_eq!((cpu[0].0.as_str(), cpu[1].0.as_str(), cpu[31].0.as_str(), cpu[32].0.as_str(), cpu[35].0.as_str()), ("PC", "zero", "s8", "ra", "Cycles"));
        let rsp = rsp_registers();
        assert_eq!(rsp.len(), 34);
        assert_eq!((rsp[0].clone(), rsp[1].clone(), rsp[2].0.as_str()), (("PC".to_string(), 16), ("Halted".to_string(), 1), "zero"));
    }
}
