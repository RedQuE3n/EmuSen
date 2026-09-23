//! The multiple, antialiasing and the device over the C ABI: C#'s `RenderScale`, `Antialiasing` and `Gpu`, applied between frames as
//! `MarsCore.ApplyMultiple` applies them. See Mars_Native.md §6.4.

use super::Core;

impl Core {
    /// `MarsCore.ApplyMultiple`: the drawing is the resolution times the averaging, held to four, the averaging giving way first
    /// (`EffectiveAntialiasing`); a walk still out is joined first, since it reads the shadow where the processor left it.
    pub fn set_multiple(&mut self, render_scale: i32, antialiasing: i32, gpu: bool) {
        let render_scale = render_scale.clamp(1, 4);
        let antialiasing = antialiasing.clamp(1, 4);
        let effective = antialiasing.min(4 / render_scale).max(1);
        // Sent again unchanged every frame by the shim until §6.4.8; a no-op must leave the deferred walk out.
        if self.multiple_applied && (render_scale, antialiasing, gpu) == (self.render_scale, self.antialiasing, self.gpu) {
            return;
        }
        self.multiple_applied = true;
        if self.machine.join_presentation(&mut self.scanout) {
            self.frame_serial += 1;
        }
        self.render_scale = render_scale;
        self.antialiasing = antialiasing;
        self.gpu = gpu;
        self.machine.set_scale(render_scale * effective);
        self.machine.set_gpu(gpu);
        self.scanout.average = effective;
    }

    /// `MarsCore.GpuReport`: what the device setting got, which is a sentence when it got nothing.
    pub fn gpu_report(&self) -> String {
        self.machine.bus.dp.gpu_report().to_string()
    }
}

/// The resolution multiple one to four, the antialiasing one to four, and whether the device is asked for.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_set_multiple(core: *mut Core, render_scale: i32, antialiasing: i32, gpu: u32) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.set_multiple(render_scale, antialiasing, gpu != 0);
    }
}

/// The device report as UTF-8, copied up to `len` bytes; returns its whole length.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_gpu_report(core: *const Core, out: *mut u8, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return super::STATUS_NULL as i64 };
    let report = c.gpu_report();
    if !out.is_null() {
        unsafe { std::ptr::copy_nonoverlapping(report.as_ptr(), out, report.len().min(len)) };
    }
    report.len() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::Machine;
    use crate::memory::bus::RDRAM_SIZE;

    /// `Antialiasing_multiplies_the_drawing_and_is_held_to_four_with_the_resolution`'s cases, as C#'s settings test holds them.
    #[test]
    fn the_drawing_is_the_resolution_times_the_averaging_held_to_four() {
        let mut c = Core::new(Machine::new(RDRAM_SIZE).unwrap());
        for (scale, level, drawn, averaged) in [(1, 1, 1, 1), (1, 4, 4, 4), (2, 2, 4, 2), (2, 4, 4, 2), (3, 2, 3, 1), (4, 3, 4, 1), (9, 8, 4, 1)] {
            c.set_multiple(scale, level, false);
            assert_eq!((c.machine.options.scale, c.machine.bus.dp.scale(), c.scanout.average), (drawn, drawn, averaged), "{scale} {level}");
        }
        assert_eq!(c.gpu_report(), "off");
        c.set_multiple(1, 1, true);
        assert_eq!(c.gpu_report(), "off at one");
        assert!(c.machine.bus.dp.multiple.processor.is_none());
        c.set_multiple(1, 1, false);
        assert_eq!(c.gpu_report(), "off");
    }
}
