//! The presenter's device lent to each machine, EmuSen_Backend.md §14.12 and §29.2: the trait the runtime calls to
//! make a machine's lend, take its pictures inside the frame, settle its copies before the machine is freed and free
//! it. The platform library implements it over Serenity's `emusen_serenity_lend_*`, so this crate links nothing of
//! Vulkan.

use emusen_native::core::sys::{DeviceFrame, DeviceLend};

pub trait Lender: Send + Sync {
    /// A new lend of the presenter's `device` for one machine: its handle and what the core is given; the words why
    /// where the device lends nothing.
    fn make(&self, device: usize) -> Result<(usize, DeviceLend), String>;

    /// The core's picture taken: whether it was newer than the last.
    fn take(&self, lend: usize, frame: &DeviceFrame) -> Result<bool, String>;

    /// Every copy taken waited for.
    fn settle(&self, lend: usize) -> Result<(), String>;

    fn free(&self, lend: usize);
}

/// One machine's lend, settled and freed with its machine.
pub struct MachineLend {
    pub lender: std::sync::Arc<dyn Lender>,
    pub handle: usize,
    pub device: DeviceLend,
}

impl MachineLend {
    /// Settled before the machine is freed: nothing of the core's may still be read on the device after.
    pub fn settle(&self) {
        if let Err(words) = self.lender.settle(self.handle) {
            crate::error_log::warning("picture", &format!("a lent device's copies could not be settled: {words}"));
        }
    }
}

impl Drop for MachineLend {
    fn drop(&mut self) {
        self.lender.free(self.handle);
    }
}
