//! What the host is told beside the frame: the two processors' registers for `DEBUG_REGISTERS`, the setting notes for
//! `SETTING_NOTES`, the IS-Viewer's lines and the device's report in the log, and the frame rate machine info gives, each
//! through a surface the core ABI v1 already has. See Mars_Native.md §9.

use emusen_native::core::outbox::{Level, log};
use emusen_native::core::event;

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
    /// `debug_registers`' values for processor 0 or 1, as the shim's `CpuRegisters` and `RspRegisters` read them.
    pub fn registers(&self, processor: u32) -> Option<Vec<i64>> {
        match processor {
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
    use crate::ffi::space;
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
    fn the_vi_space_is_its_registers_big_endian_and_takes_no_write() {
        let (m, _image) = make();
        let core = unsafe { core_of_mut::<Core>(m) }.unwrap();
        core.machine.bus.vi.registers[1] = 0x0011_2233;
        assert_eq!(unsafe { emusen_core_space_size(m, space::VI) }, 56);
        let mut out = [0xEEu8; 8];
        assert_eq!(unsafe { emusen_core_space_read(m, space::VI, 4, out.as_mut_ptr(), 8) }, 8);
        assert_eq!(out, [0x00, 0x11, 0x22, 0x33, 0, 0, 0, 0]);
        let mut past = [0xEEu8; 2];
        unsafe { emusen_core_space_read(m, space::VI, 55, past.as_mut_ptr(), 2) };
        assert_eq!(past, [0, 0]);
        assert!(unsafe { emusen_core_space_write(m, space::VI, 4, [1u8].as_ptr(), 1) } < 0);
        assert!(info(m).contains(r#"{"id":6,"name":"VI","#), "{}", info(m));
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
