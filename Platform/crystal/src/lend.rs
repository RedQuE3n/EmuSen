//! The presenter's device lent to each machine, EmuSen_Backend.md §14.12 and §29.2: the trait the runtime calls to
//! make a machine's lend, take its pictures inside the frame, settle its copies before the machine is freed and free
//! it. The platform library implements it over Serenity's `emusen_serenity_lend_*`, so this crate links nothing of
//! Vulkan.

use emusen_native::core::sys::{DeviceFrame, DeviceLend, RingImage, RingLend};

pub trait Lender: Send + Sync {
    /// A new lend of the presenter's `device` for one machine: its handle and what the core is given; the words why
    /// where the device lends nothing.
    fn make(&self, device: usize) -> Result<(usize, DeviceLend), String>;

    /// The core's picture taken: whether it was newer than the last.
    fn take(&self, lend: usize, frame: &DeviceFrame) -> Result<bool, String>;

    /// Every copy taken waited for.
    fn settle(&self, lend: usize) -> Result<(), String>;

    fn free(&self, lend: usize);

    /// Since the API's 1.4: a ring of `count` images `width` by `height` made on `device` (core ABI 1.18's
    /// `DEVICE_RING`), its handle and what the core is lent; the words why where the device makes none.
    fn make_ring(&self, _device: usize, _width: u32, _height: u32, _count: u32) -> Result<MachineRing, String> {
        Err("this lender makes no ring".to_string())
    }

    /// The ring's `ready` raised to `value` if it has not reached it, then the ring freed once the device is idle.
    fn free_ring(&self, _ring: usize, _value: u64) {}
}

/// One machine's ring, EmuSen_Backend.md §33.2: the images and semaphores the core is lent, freed after the machine.
pub struct MachineRing {
    pub lender: Option<std::sync::Arc<dyn Lender>>,
    pub handle: usize,
    pub images: Vec<RingImage>,
    pub ready: u64,
    pub done: u64,
    pub device_uuid: [u8; 16],
    pub driver_uuid: [u8; 16],
    pub width: u32,
    pub height: u32,
    /// The last value a frame was handed at, which `ready` is raised to before the ring is freed.
    pub last_value: u64,
}

impl MachineRing {
    /// What the core is lent; borrows the images, so it lives no longer than the ring.
    pub fn lend(&self) -> RingLend {
        RingLend {
            size: std::mem::size_of::<RingLend>() as u32,
            count: self.images.len() as u32,
            images: self.images.as_ptr(),
            image_size: std::mem::size_of::<RingImage>(),
            ready: self.ready,
            done: self.done,
            device_uuid: self.device_uuid,
            driver_uuid: self.driver_uuid,
        }
    }
}

impl Drop for MachineRing {
    fn drop(&mut self) {
        if let Some(l) = &self.lender {
            l.free_ring(self.handle, self.last_value);
        }
    }
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
