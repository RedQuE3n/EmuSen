//! The stable core ABI, version 1: the trait a Rust core implements and `core_exports!`, which generates every
//! `emusen_core_*` export of `include/emusen_core.h` from it. See EmuSen_CoreAPI.md §4-§6.
//!
//! The pre-stable `abi` module stays beside this one until MoonRT, MercuryRT and VenusRT have moved (§13.5).

pub mod desc;
#[doc(hidden)]
pub mod exports;
pub mod jsonw;
pub mod outbox;
pub mod schema;
pub mod sys;

#[cfg(test)]
mod tests;

pub use crate::abi::{File, Region, Settings};
pub use crate::debug::Hooks;
pub use crate::ffi::StateMachine;
pub use desc::*;
pub use outbox::{Level, detail, emit, log};
pub use sys::{FrameInfo, caps, event, flags, kind, pixel, status};

/// What `create` receives: the image, the settings resolved against the schema with every key present, the files,
/// and the pixel formats the host accepts (bit 0, RGBA8888, always set).
pub struct Create<'a> {
    pub image: &'a [u8],
    pub settings: Settings,
    pub files: Vec<File<'a>>,
    pub pixel_formats: u64,
    pub host_abi_version: u32,
}

/// What a Rust core implements; `core_exports!` turns it into the C ABI. Every `Err` is a status (§6.15), and
/// [`detail`] gives it words. Defaults answer as a core without the capability must.
pub trait Core: Sized + StateMachine {
    /// The bits claimed; the optional groups named to `core_exports!` must match them.
    const CAPABILITIES: u64;

    fn info() -> Info;

    fn settings_schema() -> Vec<Setting> {
        Vec::new()
    }

    /// What `image` needs, from its bytes alone.
    fn firmware_for(_image: &[u8]) -> Vec<Firmware> {
        Vec::new()
    }

    /// Words for a code of the core's own band; the crate knows the shared ones.
    fn status_text(_status: i32) -> Option<String> {
        None
    }

    fn create(request: &Create<'_>) -> Result<Self, i32>;

    fn machine_info(&self) -> MachineInfo;

    /// `RESET`.
    fn reset(&mut self) -> Result<(), i32> {
        Err(status::NOT_SUPPORTED)
    }

    /// The machine to the frame's end; `detail` is the core's word for a failure.
    fn advance(&mut self, detail: &mut u64) -> Result<(), i32>;

    /// `PRESENT`: called by the host after `advance` when rendering is not skipped.
    fn present(&mut self) -> Result<(), i32> {
        Err(status::NOT_SUPPORTED)
    }

    /// Bit 0 skips rendering; the crate clears every reserved bit before the call.
    fn set_options(&mut self, flags: u32);

    fn frame_count(&self) -> i64;

    /// `PHASES`: nanoseconds per phase, in machine info's order.
    fn phases(&self) -> Vec<i64> {
        Vec::new()
    }

    /// The picture's shape; without `FRAME_SERIAL` the crate writes the frame count as the serial, without
    /// `ROW_REPEAT` a row repeat of 1.
    fn frame_info(&self) -> FrameInfo;
    fn frame(&self) -> &[u8];

    /// The rate of the next sample a drain returns.
    fn audio_rate(&self) -> i32;
    fn audio_buffered(&self) -> usize;
    /// Whole stereo frames, at most `max_frames`, never across a change of `audio_rate`; the samples written.
    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize;
    fn set_audio_limit(&mut self, samples: usize);

    /// `AUDIO_PEEK`: the buffered samples copied without draining; how many are buffered.
    fn peek_audio(&self, _out: &mut [i16]) -> usize {
        0
    }

    /// `MUTES`: bit n mutes machine info's channel n.
    fn set_mutes(&mut self, _mask: u32) -> Result<(), i32> {
        Err(status::NOT_SUPPORTED)
    }

    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32>;

    /// `AXES`: sticks -1 to 1, right and down positive, triggers 0 to 1; the core scales to its console.
    fn set_axis(&mut self, _port: u32, _axis: u32, _value: f64) -> Result<(), i32> {
        Err(status::NOT_SUPPORTED)
    }

    /// Before a state is written: the core's threads brought to rest.
    fn settle(&mut self) {}

    /// `SNAPSHOT`: kind 1, which `state_load` also reads.
    fn snapshot_size(&self) -> usize {
        0
    }

    fn save_snapshot(&mut self, _out: &mut [u8]) -> Result<usize, i32> {
        Err(status::NOT_SUPPORTED)
    }

    fn snapshot_layout(&self) -> String {
        String::new()
    }

    fn space_size(&self, space: u32) -> Result<i64, i32>;
    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32>;
    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32>;

    /// A battery file's bytes and its `flags::BATTERY_*`; empty for a cartridge without one.
    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32>;

    fn battery_saved(&mut self, _which: u32) -> Result<(), i32> {
        Ok(())
    }

    /// `ROM_PATCHES`: (address, value, compare) triples.
    fn set_rom_patches(&mut self, _triples: &[u32]) -> Result<(), i32> {
        Err(status::NOT_SUPPORTED)
    }

    /// `CHEAT_POKES`: (space, address, value, compare) quads, applied by the core at its frame's end.
    fn set_cheat_pokes(&mut self, _quads: &[u32]) -> Result<(), i32> {
        Err(status::NOT_SUPPORTED)
    }

    /// `SETTINGS`: run-scope keys the crate has checked against the schema, applied together.
    fn set_settings(&mut self, _settings: &Settings) -> Result<(), i32> {
        Err(status::NOT_SUPPORTED)
    }

    /// `SETTING_NOTES`: (key, sentence) for values the core will not use as chosen.
    fn setting_notes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// `DEBUG`: the hooks the host's tables are pushed into and its logs drained from.
    fn debug_hooks(&mut self) -> Option<&mut Hooks> {
        None
    }

    /// `DEBUG`: a processor's breakpoints as (first, last) pairs; processor 0's live in the hooks.
    fn debug_breakpoints(&mut self, processor: u32, pairs: &[i32]) -> Result<(), i32> {
        match (processor, self.debug_hooks()) {
            (0, Some(h)) => {
                h.set_breakpoints(pairs);
                Ok(())
            }
            _ => Err(status::NOT_SUPPORTED),
        }
    }

    /// `DEBUG`: the frame through the observed loop; the stop reasons, zero at the frame's end.
    fn debug_run_frame(&mut self, _flags: u32, _detail: &mut u64) -> Result<u32, i32> {
        Err(status::NOT_SUPPORTED)
    }

    /// `DEBUG`: the processor the last stop was on.
    fn debug_stopped(&self) -> u32 {
        0
    }

    /// `DEBUG`: the address of the step `processor` stands in front of.
    fn debug_pc(&self, _processor: u32) -> Option<u64> {
        None
    }

    /// `DEBUG_REGISTERS`: in machine info's register order.
    fn debug_registers(&self, _processor: u32) -> Result<Vec<i64>, i32> {
        Err(status::NOT_SUPPORTED)
    }

    /// `DEBUG_DISASSEMBLE`.
    fn debug_disassemble(&mut self, _processor: u32, _space: u32, _address: u32, _count: u32) -> Result<Vec<Instruction>, i32> {
        Err(status::NOT_SUPPORTED)
    }

    /// Clears whatever the core records between calls, after every call that changes the machine.
    fn end_call() {}
}

/// The capability bits whose groups have exports; the others are behaviour only.
pub const EXPORTING: u64 = caps::RESET
    | caps::PRESENT
    | caps::PHASES
    | caps::AUDIO_PEEK
    | caps::MUTES
    | caps::AXES
    | caps::SETTINGS
    | caps::SETTING_NOTES
    | caps::ROM_PATCHES
    | caps::CHEAT_POKES
    | caps::DEBUG
    | caps::DEBUG_STACK
    | caps::DEBUG_REGISTERS
    | caps::DEBUG_DISASSEMBLE;

/// `core_exports!`'s compile-time check: the groups named are exactly the claimed bits that have exports.
#[doc(hidden)]
pub const fn check_groups<T: Core>(named: u64) {
    assert!(T::CAPABILITIES & EXPORTING == named, "the optional groups named and CAPABILITIES disagree");
    assert!(T::CAPABILITIES & caps::UNASSIGNED == 0, "CAPABILITIES claims a bit no minor has assigned");
}

/// The capability bit of an optional group.
#[macro_export]
#[doc(hidden)]
macro_rules! __core_capability {
    (reset) => { $crate::core::caps::RESET };
    (present) => { $crate::core::caps::PRESENT };
    (phases) => { $crate::core::caps::PHASES };
    (audio_peek) => { $crate::core::caps::AUDIO_PEEK };
    (mutes) => { $crate::core::caps::MUTES };
    (axes) => { $crate::core::caps::AXES };
    (settings) => { $crate::core::caps::SETTINGS };
    (setting_notes) => { $crate::core::caps::SETTING_NOTES };
    (rom_patches) => { $crate::core::caps::ROM_PATCHES };
    (cheat_pokes) => { $crate::core::caps::CHEAT_POKES };
    (debug) => { $crate::core::caps::DEBUG };
    (debug_stack) => { $crate::core::caps::DEBUG_STACK };
    (debug_registers) => { $crate::core::caps::DEBUG_REGISTERS };
    (debug_disassemble) => { $crate::core::caps::DEBUG_DISASSEMBLE };
}

/// One export: the C signature, forwarding to the generic body of the same name in `core::exports`.
#[macro_export]
#[doc(hidden)]
macro_rules! __core_export {
    ($t:ty, $name:ident => $body:ident ( $($arg:ident : $ty:ty),* ) -> $ret:ty) => {
        /// See `emusen_core.h`.
        ///
        /// # Safety
        /// The pointers as the header requires: a live machine or null, buffers valid for their lengths or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name($($arg: $ty),*) -> $ret {
            unsafe { $crate::core::exports::$body::<$t>($($arg),*) }
        }
    };
}

/// One optional group's exports.
#[macro_export]
#[doc(hidden)]
macro_rules! __core_optional {
    ($t:ty, reset) => {
        $crate::__core_export!($t, emusen_core_reset => reset(machine: *mut $crate::core::sys::Machine) -> i32);
    };
    ($t:ty, present) => {
        $crate::__core_export!($t, emusen_core_present => present(machine: *mut $crate::core::sys::Machine) -> i32);
    };
    ($t:ty, phases) => {
        $crate::__core_export!($t, emusen_core_phases => phases(machine: *const $crate::core::sys::Machine, out: *mut i64, len: usize) -> i64);
    };
    ($t:ty, audio_peek) => {
        $crate::__core_export!($t, emusen_core_audio_peek => audio_peek(machine: *const $crate::core::sys::Machine, out: *mut i16, len: usize) -> i64);
    };
    ($t:ty, mutes) => {
        $crate::__core_export!($t, emusen_core_set_mutes => set_mutes(machine: *mut $crate::core::sys::Machine, mask: u32) -> i32);
    };
    ($t:ty, axes) => {
        $crate::__core_export!($t, emusen_core_set_axis => set_axis(machine: *mut $crate::core::sys::Machine, port: u32, axis: u32, value: f64) -> i32);
    };
    ($t:ty, settings) => {
        $crate::__core_export!($t, emusen_core_set_settings => set_settings(machine: *mut $crate::core::sys::Machine, text: *const u8, len: usize) -> i32);
    };
    ($t:ty, setting_notes) => {
        $crate::__core_export!($t, emusen_core_setting_notes => setting_notes(machine: *const $crate::core::sys::Machine, out: *mut u8, len: usize) -> i64);
    };
    ($t:ty, rom_patches) => {
        $crate::__core_export!($t, emusen_core_set_rom_patches => set_rom_patches(machine: *mut $crate::core::sys::Machine, triples: *const u32, count: usize) -> i64);
    };
    ($t:ty, cheat_pokes) => {
        $crate::__core_export!($t, emusen_core_set_cheat_pokes => set_cheat_pokes(machine: *mut $crate::core::sys::Machine, quads: *const u32, count: usize) -> i64);
    };
    ($t:ty, debug_stack) => {
        $crate::__core_export!($t, emusen_core_debug_set_stack => debug_set_stack(machine: *mut $crate::core::sys::Machine, pairs: *const u32, count: usize) -> i32);
    };
    ($t:ty, debug_registers) => {
        $crate::__core_export!($t, emusen_core_debug_registers => debug_registers(machine: *const $crate::core::sys::Machine, processor: u32, out: *mut i64, len: usize) -> i64);
    };
    ($t:ty, debug_disassemble) => {
        $crate::__core_export!($t, emusen_core_debug_disassemble => debug_disassemble(machine: *mut $crate::core::sys::Machine, processor: u32, space: u32, address: u32, count: u32, out: *mut u8, len: usize) -> i64);
    };
    ($t:ty, debug) => {
        $crate::__core_export!($t, emusen_core_debug_set => debug_set(machine: *mut $crate::core::sys::Machine, flags: u32, depth_target: i32, depth_guard: i32) -> i32);
        $crate::__core_export!($t, emusen_core_debug_set_breakpoints => debug_set_breakpoints(machine: *mut $crate::core::sys::Machine, processor: u32, pairs: *const i32, count: usize) -> i32);
        $crate::__core_export!($t, emusen_core_debug_set_ranges => debug_set_ranges(machine: *mut $crate::core::sys::Machine, kind: u32, triples: *const u32, count: usize) -> i32);
        $crate::__core_export!($t, emusen_core_debug_run_frame => debug_run_frame(machine: *mut $crate::core::sys::Machine, flags: u32, processor: *mut u32, pc: *mut u64, detail: *mut u64) -> i32);
        $crate::__core_export!($t, emusen_core_debug_writes => debug_writes(machine: *mut $crate::core::sys::Machine, out: *mut u32, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_debug_calls => debug_calls(machine: *mut $crate::core::sys::Machine, out: *mut u32, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_debug_profile => debug_profile(machine: *mut $crate::core::sys::Machine, out: *mut i64, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_debug_coverage => debug_coverage(machine: *mut $crate::core::sys::Machine, processor: u32, out: *mut u8, len: usize, recorded: *mut i64) -> i64);
        $crate::__core_export!($t, emusen_core_debug_counters => debug_counters(machine: *mut $crate::core::sys::Machine, out: *mut i64, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_debug_pc => debug_pc(machine: *const $crate::core::sys::Machine, processor: u32, pc: *mut u64) -> i32);
    };
}

/// Every required export over a [`Core`], and the optional groups named, which must be exactly the claimed bits
/// that have exports: `core_exports!(Machine; reset, settings, debug);`.
#[macro_export]
macro_rules! core_exports {
    ($t:ty; $($opt:ident),* $(,)?) => {
        const _: () = $crate::core::check_groups::<$t>(0 $(| $crate::__core_capability!($opt))*);
        $( $crate::__core_optional!($t, $opt); )*

        $crate::__core_export!($t, emusen_core_abi_version => abi_version() -> u32);
        $crate::__core_export!($t, emusen_core_capabilities => capabilities() -> u64);
        $crate::__core_export!($t, emusen_core_info => info(out: *mut u8, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_settings_schema => settings_schema(out: *mut u8, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_firmware_for => firmware_for(image: *const u8, image_len: usize, out: *mut u8, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_status_text => status_text(status: i32, out: *mut u8, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_set_crash_log => set_crash_log(path: *const std::ffi::c_char) -> i32);
        $crate::__core_export!($t, emusen_core_log_drain => log_drain(machine: *mut $crate::core::sys::Machine, out: *mut u8, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_create => create(params: *const $crate::core::sys::CreateParams, status: *mut i32) -> *mut $crate::core::sys::Machine);
        $crate::__core_export!($t, emusen_core_free => free(machine: *mut $crate::core::sys::Machine) -> i32);
        $crate::__core_export!($t, emusen_core_machine_info => machine_info(machine: *const $crate::core::sys::Machine, out: *mut u8, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_last_error => last_error(machine: *const $crate::core::sys::Machine, out: *mut u8, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_advance => advance(machine: *mut $crate::core::sys::Machine, detail: *mut u64) -> i32);
        $crate::__core_export!($t, emusen_core_set_options => set_options(machine: *mut $crate::core::sys::Machine, flags: u32) -> i32);
        $crate::__core_export!($t, emusen_core_frame_count => frame_count(machine: *const $crate::core::sys::Machine) -> i64);
        $crate::__core_export!($t, emusen_core_events => events(machine: *mut $crate::core::sys::Machine, out: *mut $crate::core::sys::Event, count: usize, event_size: usize) -> i64);
        $crate::__core_export!($t, emusen_core_frame_info => frame_info(machine: *const $crate::core::sys::Machine, out: *mut $crate::core::sys::FrameInfo) -> i32);
        $crate::__core_export!($t, emusen_core_frame_copy => frame_copy(machine: *const $crate::core::sys::Machine, out: *mut u8, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_audio_rate => audio_rate(machine: *const $crate::core::sys::Machine) -> i32);
        $crate::__core_export!($t, emusen_core_audio_buffered => audio_buffered(machine: *const $crate::core::sys::Machine) -> i64);
        $crate::__core_export!($t, emusen_core_audio_drain => audio_drain(machine: *mut $crate::core::sys::Machine, out: *mut i16, len: usize, max_frames: i64, rate: *mut i32) -> i64);
        $crate::__core_export!($t, emusen_core_set_audio_limit => set_audio_limit(machine: *mut $crate::core::sys::Machine, samples: u64) -> i32);
        $crate::__core_export!($t, emusen_core_set_buttons => set_buttons(machine: *mut $crate::core::sys::Machine, port: u32, mask: u32, changed: u32) -> i32);
        $crate::__core_export!($t, emusen_core_state_size => state_size(machine: *const $crate::core::sys::Machine, kind: u32) -> i64);
        $crate::__core_export!($t, emusen_core_state_save => state_save(machine: *mut $crate::core::sys::Machine, kind: u32, out: *mut u8, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_state_load => state_load(machine: *mut $crate::core::sys::Machine, data: *const u8, len: usize) -> i32);
        $crate::__core_export!($t, emusen_core_state_layout => state_layout(machine: *const $crate::core::sys::Machine, kind: u32, out: *mut u8, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_space_size => space_size(machine: *const $crate::core::sys::Machine, space: u32) -> i64);
        $crate::__core_export!($t, emusen_core_space_read => space_read(machine: *mut $crate::core::sys::Machine, space: u32, address: u32, out: *mut u8, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_space_write => space_write(machine: *mut $crate::core::sys::Machine, space: u32, address: u32, data: *const u8, len: usize) -> i64);
        $crate::__core_export!($t, emusen_core_battery => battery(machine: *const $crate::core::sys::Machine, which: u32, out: *mut u8, len: usize, flags: *mut u32) -> i64);
        $crate::__core_export!($t, emusen_core_battery_saved => battery_saved(machine: *mut $crate::core::sys::Machine, which: u32) -> i32);
    };
}
