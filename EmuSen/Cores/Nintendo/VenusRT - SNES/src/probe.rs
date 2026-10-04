//! A processor's debugger seam, fitted only while a debugger's frame runs: its breakpoints, the addresses it stepped
//! through for coverage, its stores, and where it stopped. The S-CPU is observed by the machine's own loop; the
//! SPC700 and the cartridge's processor run inside catch-ups and carry one of these. See VenusRT_Native.md §36.

#[derive(Clone, Debug, Default)]
pub struct Probe {
    /// First and last address of each breakpoint, both included.
    pub breakpoints: Vec<(i32, i32)>,
    /// Every step's address kept in `steps`, for the coverage map.
    pub coverage: bool,
    pub steps: Vec<u32>,
    /// Stores as (address, value, the storing instruction's address), when a watch wants them.
    pub stores: Option<Vec<(u32, u8, u32)>>,
    /// The address of the step the processor stands in front of, stopped by a breakpoint.
    pub halted: Option<u32>,
    /// The next step is taken without its check: the one a halt was resumed from.
    pub resume: bool,
    /// The step in progress, for stamping its stores.
    pub at: u32,
}

impl Probe {
    /// Before the step at `pc`: true to stop in front of it.
    #[inline]
    pub fn before(&mut self, pc: u32) -> bool {
        if self.halted.is_some() {
            return true;
        }
        if std::mem::take(&mut self.resume) {
        } else if self.breakpoints.iter().any(|&(first, last)| pc as i32 >= first && pc as i32 <= last) {
            self.halted = Some(pc);
            return true;
        }
        if self.coverage {
            self.steps.push(pc);
        }
        self.at = pc;
        false
    }

    #[inline]
    pub fn store(&mut self, address: u32, value: u8) {
        if let Some(log) = self.stores.as_mut() {
            log.push((address, value, self.at));
        }
    }
}
