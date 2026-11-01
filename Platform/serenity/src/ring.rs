//! The host's side of a ring (`DEVICE_RING`, core ABI 1.18): images and two timeline semaphores made on this device and
//! exported, which a hosted core's device imports; each picture taken from `VK_QUEUE_FAMILY_EXTERNAL` once `ready`
//! reaches its value and given back by `done`. See EmuSen_LibretroHost.md §5.4, §5.4a and §10f.

use ash::vk;

use crate::vulkan::{Checked, Device, color, libc_close};

/// The ring's images' format, the one the ABI names.
pub const FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
/// The layout a picture crosses in, both ways: the core releases a slot in it, the host gives it back in it.
pub const LAYOUT: vk::ImageLayout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;

/// One exported image as the ABI's `emusen_ring_image` carries it; the handle stays the ring's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RingImage {
    pub memory_type: u32,
    pub handle: i32,
    pub allocation_size: u64,
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub usage: u32,
}

struct Slot {
    image: vk::Image,
    memory: vk::DeviceMemory,
}

/// A ring and its descriptors, which it closes when dropped, after the device is idle.
pub struct Ring {
    device: Device,
    slots: Vec<Slot>,
    ready: vk::Semaphore,
    done: vk::Semaphore,
    pub images: Vec<RingImage>,
    pub ready_fd: i32,
    pub done_fd: i32,
    pub device_uuid: [u8; 16],
    pub driver_uuid: [u8; 16],
}

impl Drop for Ring {
    fn drop(&mut self) {
        self.device.wait_idle();
        unsafe {
            let d = self.device.vk();
            for s in &self.slots {
                d.destroy_image(s.image, None);
                d.free_memory(s.memory, None);
            }
            d.destroy_semaphore(self.ready, None);
            d.destroy_semaphore(self.done, None);
            for fd in self.images.iter().map(|i| i.handle).chain([self.ready_fd, self.done_fd]) {
                if fd >= 0 {
                    libc_close(fd);
                }
            }
        }
    }
}

impl Device {
    /// A ring of `count` images `width` by `height`, dedicated device memory exportable as `OPAQUE_FD`, with `ready`
    /// and `done` timeline semaphores, also exported.
    pub fn create_ring(&self, width: u32, height: u32, count: usize) -> Result<Ring, String> {
        if !self.rings() {
            return Err("the device cannot export images and timeline semaphores".into());
        }
        let (memory_fd, semaphore_fd) = self.export_loaders().unwrap();
        let d = self.vk();
        let usage = vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::SAMPLED;
        let mut ring = Ring { device: self.clone(), slots: Vec::new(), ready: vk::Semaphore::null(), done: vk::Semaphore::null(), images: Vec::new(), ready_fd: -1, done_fd: -1, device_uuid: self.uuid(), driver_uuid: self.driver_uuid() };
        for _ in 0..count {
            let mut external = vk::ExternalMemoryImageCreateInfo::default().handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
            let info = vk::ImageCreateInfo::default()
                .push_next(&mut external)
                .image_type(vk::ImageType::TYPE_2D)
                .format(FORMAT)
                .extent(vk::Extent3D { width, height, depth: 1 })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .usage(usage);
            let image = unsafe { d.create_image(&info, None) }.check("vkCreateImage")?;
            let needs = unsafe { d.get_image_memory_requirements(image) };
            let kind = match self.memory_type(needs.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL, vk::MemoryPropertyFlags::empty(), "DeviceLocalBit") {
                Ok(k) => k,
                Err(e) => {
                    unsafe { d.destroy_image(image, None) };
                    return Err(e);
                }
            };
            let mut export = vk::ExportMemoryAllocateInfo::default().handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
            let mut dedicated = vk::MemoryDedicatedAllocateInfo::default().image(image);
            let allocate = vk::MemoryAllocateInfo::default().allocation_size(needs.size).memory_type_index(kind).push_next(&mut export).push_next(&mut dedicated);
            let memory = match unsafe { d.allocate_memory(&allocate, None) }.check("vkAllocateMemory") {
                Ok(m) => m,
                Err(e) => {
                    unsafe { d.destroy_image(image, None) };
                    return Err(e);
                }
            };
            ring.slots.push(Slot { image, memory });
            unsafe { d.bind_image_memory(image, memory, 0) }.check("vkBindImageMemory")?;
            let fd = unsafe { memory_fd.get_memory_fd(&vk::MemoryGetFdInfoKHR::default().memory(memory).handle_type(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD)) }.check("vkGetMemoryFdKHR")?;
            ring.images.push(RingImage { memory_type: kind, handle: fd, allocation_size: needs.size, width, height, format: FORMAT.as_raw() as u32, usage: usage.as_raw() });
        }
        for at in [&mut ring.ready, &mut ring.done] {
            let mut kind = vk::SemaphoreTypeCreateInfo::default().semaphore_type(vk::SemaphoreType::TIMELINE).initial_value(0);
            let mut export = vk::ExportSemaphoreCreateInfo::default().handle_types(vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_FD);
            *at = unsafe { d.create_semaphore(&vk::SemaphoreCreateInfo::default().push_next(&mut kind).push_next(&mut export), None) }.check("vkCreateSemaphore")?;
        }
        let fd = |s: vk::Semaphore| unsafe { semaphore_fd.get_semaphore_fd(&vk::SemaphoreGetFdInfoKHR::default().semaphore(s).handle_type(vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_FD)) }.check("vkGetSemaphoreFdKHR");
        ring.ready_fd = fd(ring.ready)?;
        ring.done_fd = fd(ring.done)?;
        Ok(ring)
    }
}

fn layers() -> vk::ImageSubresourceLayers {
    vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 }
}

/// One picture of a ring as this device reads it: the slot's image, `ready`'s value to wait for and `done` to signal
/// with it once the picture is read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InRing {
    image: vk::Image,
    family: u32,
    pub ready: vk::Semaphore,
    pub done: vk::Semaphore,
    pub value: u64,
    pub width: u32,
    pub height: u32,
}

impl InRing {
    fn crossing(&self, take: bool) -> vk::ImageMemoryBarrier<'static> {
        let (from, to) = if take { (vk::QUEUE_FAMILY_EXTERNAL, self.family) } else { (self.family, vk::QUEUE_FAMILY_EXTERNAL) };
        vk::ImageMemoryBarrier::default()
            .image(self.image)
            .old_layout(LAYOUT)
            .new_layout(LAYOUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(if take { vk::AccessFlags::SHADER_READ } else { vk::AccessFlags::empty() })
            .src_queue_family_index(from)
            .dst_queue_family_index(to)
            .subresource_range(color(0, 1))
    }

    /// The slot taken, moved for a transfer read, read by `copy` given its image, moved back and given back, in a
    /// submission that waits for `ready` at the transfer stage and signals `done`.
    pub(crate) fn record(&self, device: &Device, cmd: vk::CommandBuffer, copy: impl FnOnce(vk::Image)) {
        let d = device.vk();
        let to_src = vk::ImageMemoryBarrier::default().image(self.image).old_layout(LAYOUT).new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL).src_access_mask(vk::AccessFlags::empty()).dst_access_mask(vk::AccessFlags::TRANSFER_READ).src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED).subresource_range(color(0, 1));
        let back = vk::ImageMemoryBarrier { old_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL, new_layout: LAYOUT, src_access_mask: vk::AccessFlags::TRANSFER_READ, dst_access_mask: vk::AccessFlags::empty(), ..to_src };
        // The crossings keep the layout, as the core's do; the transitions to and from the copy are barriers of their own.
        let take = vk::ImageMemoryBarrier { dst_access_mask: vk::AccessFlags::TRANSFER_READ, ..self.crossing(true) };
        let one = |from, to, b: vk::ImageMemoryBarrier<'_>| unsafe { d.cmd_pipeline_barrier(cmd, from, to, vk::DependencyFlags::empty(), &[], &[], &[b]) };
        one(vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::TRANSFER, take);
        one(vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::TRANSFER, to_src);
        copy(self.image);
        one(vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::TRANSFER, back);
        one(vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::BOTTOM_OF_PIPE, self.crossing(false));
    }
}

impl Ring {
    /// The ownership crossing of a slot on this device: taken from `VK_QUEUE_FAMILY_EXTERNAL` for reading, or given
    /// back to it, in `LAYOUT` both ways.
    pub fn crossing(&self, slot: usize, take: bool) -> vk::ImageMemoryBarrier<'static> {
        self.picture(slot, 0, 0, 0).crossing(take)
    }

    /// The picture `frame_ring` answered, `value` in `slot`, `width` by `height`, as a chain takes it: the ring outlives
    /// every frame a chain advanced from it and has not yet rendered (EmuSen_LibretroHost.md §10f).
    pub fn picture(&self, slot: usize, value: u64, width: u32, height: u32) -> InRing {
        InRing { image: self.slots[slot].image, family: self.device.family(), ready: self.ready, done: self.done, value, width, height }
    }

    /// The picture of `slot` once `ready` reaches `value`, `width` by `height`, copied into host memory as RGBA rows,
    /// and the slot given back by signalling `done` to `value`: the check a test or a details window makes, never the
    /// path a picture is presented by.
    pub fn read(&self, slot: usize, value: u64, width: u32, height: u32) -> Result<Vec<u8>, String> {
        let bytes = width as u64 * height as u64 * 4;
        let out = self.device.create_buffer(bytes, vk::BufferUsageFlags::TRANSFER_DST)?;
        let d = self.device.vk();
        let picture = self.picture(slot, value, width, height);
        self.device.run_timed(
            |cmd| {
                picture.record(&self.device, cmd, |image| unsafe {
                    let region = vk::BufferImageCopy { buffer_offset: 0, buffer_row_length: 0, buffer_image_height: 0, image_subresource: layers(), image_offset: vk::Offset3D::default(), image_extent: vk::Extent3D { width, height, depth: 1 } };
                    d.cmd_copy_image_to_buffer(cmd, image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, out.handle, &[region]);
                });
                self.device.host_read_barrier(cmd, &out);
            },
            Some((self.ready, value)),
            Some((self.done, value)),
        )?;
        Ok(out.with_bytes(bytes as usize, |b| b.to_vec()))
    }

    /// After a core's process ends: `ready` raised from the CPU to `value`, so no submission of this device waits on a
    /// picture that will never come, before the ring is destroyed (§5.4).
    pub fn release_after_crash(&self, value: u64) -> Result<(), String> {
        let current = unsafe { self.device.vk().get_semaphore_counter_value(self.ready) }.check("vkGetSemaphoreCounterValue")?;
        if current >= value {
            return Ok(());
        }
        unsafe { self.device.vk().signal_semaphore(&vk::SemaphoreSignalInfo::default().semaphore(self.ready).value(value)) }.check("vkSignalSemaphore")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    // A device that says it makes rings makes one: lavapipe said so and refused the timeline semaphore's export (EmuSen_DevicePicture.md §5.6).
    #[test]
    fn a_device_that_says_it_makes_rings_makes_one() {
        let Ok(device) = Device::create(std::env::var("EMUSEN_GPU_DEVICE").ok().as_deref(), false) else { return eprintln!("skipped: no Vulkan device") };
        if device.rings() {
            device.create_ring(64, 64, 3).map(|_| ()).unwrap_or_else(|why| panic!("{} says it makes rings and made none: {why}", device.name()));
        } else {
            assert!(device.create_ring(64, 64, 3).is_err());
        }
    }

    // §5.4: a presenter's submission waiting on a `ready` no dead core will signal ends once the host raises it.
    #[test]
    fn a_wait_on_ready_that_a_dead_core_never_signals_ends_when_ready_is_raised() {
        let Ok(device) = Device::create(std::env::var("EMUSEN_GPU_DEVICE").ok().as_deref(), false) else { return eprintln!("skipped: no Vulkan device") };
        if !device.rings() {
            return eprintln!("skipped: the device makes no ring");
        }
        let ring = std::sync::Arc::new(device.create_ring(64, 64, 3).unwrap());
        let reader = ring.clone();
        let reading = std::thread::spawn(move || reader.read(0, 1, 64, 64).map(|_| ()));
        std::thread::sleep(Duration::from_millis(200));
        assert!(!reading.is_finished(), "the read waits for ready to reach 1");
        ring.release_after_crash(1).unwrap();
        let start = Instant::now();
        while !reading.is_finished() {
            assert!(start.elapsed() < Duration::from_secs(5), "the read still waits after ready was raised");
            std::thread::sleep(Duration::from_millis(10));
        }
        reading.join().unwrap().unwrap();
        ring.release_after_crash(1).unwrap();
    }
}
