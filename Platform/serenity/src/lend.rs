//! The device lent to a core, and the host's taking of the pictures the core hands on it (EmuSen_CoreAPI.md §6.29).
//!
//! A lend is one core's: its own `done` timeline, which the core waits on before it writes a picture again, and a
//! ring of three device buffers the pictures are copied into on the host's intake queue. The emulation thread takes
//! each picture as the core answers it, waiting on the device for the core's `ready` and signalling `done` once the
//! copy is made, so the core is let go of at the device's pace whatever the window does; a chain then claims the newest
//! copy and reads it on the device, after the `taken` timeline has reached it. A buffer the chain has claimed is
//! never written, and is let go of when the chain claims another, its last submission having been waited for. See
//! EmuSen_DevicePicture.md §17.

use crate::vulkan::{Checked, Device};
use ash::vk::{self, Handle};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What a host passes a core at create: the device, the queue lent, the intake's family and `done`, as the core ABI's
/// `emusen_device_lend` holds them.
#[derive(Clone, Copy, Debug, Default)]
pub struct Lent {
    pub get_instance_proc_addr: usize,
    pub api: u32,
    pub instance: u64,
    pub physical: u64,
    pub device: u64,
    pub family: u32,
    pub queue: u32,
    pub host_family: u32,
    pub done: u64,
}

/// A copy of a picture, on the device, `row_pixels` to a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Taken {
    pub buffer: vk::Buffer,
    pub width: u32,
    pub height: u32,
    pub row_pixels: u32,
    /// The value `taken` reaches once the copy is made, and the semaphore.
    pub value: u64,
    pub semaphore: vk::Semaphore,
}

struct Slot {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    bytes: u64,
    commands: vk::CommandBuffer,
    fence: vk::Fence,
    in_flight: bool,
    picture: Option<Taken>,
}

struct Ring {
    slots: Vec<Slot>,
    newest: Option<usize>,
    claimed: Option<usize>,
    /// The last value of `taken`, and the last of the core's pictures taken.
    count: u64,
    last: u64,
}

struct Inner {
    device: Device,
    done: vk::Semaphore,
    taken: vk::Semaphore,
    pool: vk::CommandPool,
    ring: Mutex<Ring>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        // The core is gone and no chain holds the ring: what is left running is the intake's and a chain's last reads.
        self.device.wait_idle();
        let device = self.device.vk();
        unsafe {
            for slot in &locked(&self.ring).slots {
                device.destroy_fence(slot.fence, None);
                if slot.buffer != vk::Buffer::null() {
                    device.destroy_buffer(slot.buffer, None);
                    device.free_memory(slot.memory, None);
                }
            }
            device.destroy_command_pool(self.pool, None);
            device.destroy_semaphore(self.done, None);
            device.destroy_semaphore(self.taken, None);
        }
    }
}

/// One core's lend; a clone is the same lend, kept by a chain while it reads the ring.
#[derive(Clone)]
pub struct Lend {
    inner: Arc<Inner>,
}

pub const SLOTS: usize = 3;

impl Lend {
    /// A lend of `device`, or why it cannot lend: no timeline semaphores, or too few queues.
    pub fn new(device: &Device) -> Result<Lend, String> {
        let Some(lending) = device.lending() else { return Err("the device cannot lend a queue".to_string()) };
        let vk_device = device.vk();
        let timeline = || {
            let mut kind = vk::SemaphoreTypeCreateInfo::default().semaphore_type(vk::SemaphoreType::TIMELINE).initial_value(0);
            unsafe { vk_device.create_semaphore(&vk::SemaphoreCreateInfo::default().push_next(&mut kind), None) }.check("vkCreateSemaphore")
        };
        let done = timeline()?;
        let taken = match timeline() {
            Ok(taken) => taken,
            Err(words) => {
                unsafe { vk_device.destroy_semaphore(done, None) };
                return Err(words);
            }
        };
        let made = (|| unsafe {
            let pool_info = vk::CommandPoolCreateInfo::default().queue_family_index(lending.intake_family).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
            let pool = vk_device.create_command_pool(&pool_info, None).check("vkCreateCommandPool")?;
            let allocate = vk::CommandBufferAllocateInfo::default().command_pool(pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(SLOTS as u32);
            let buffers = match vk_device.allocate_command_buffers(&allocate).check("vkAllocateCommandBuffers") {
                Ok(buffers) => buffers,
                Err(words) => {
                    vk_device.destroy_command_pool(pool, None);
                    return Err(words);
                }
            };
            let mut slots = Vec::with_capacity(SLOTS);
            for commands in buffers {
                match vk_device.create_fence(&vk::FenceCreateInfo::default(), None).check("vkCreateFence") {
                    Ok(fence) => slots.push(Slot { buffer: vk::Buffer::null(), memory: vk::DeviceMemory::null(), bytes: 0, commands, fence, in_flight: false, picture: None }),
                    Err(words) => {
                        for slot in &slots {
                            vk_device.destroy_fence(slot.fence, None);
                        }
                        vk_device.destroy_command_pool(pool, None);
                        return Err(words);
                    }
                }
            }
            Ok((pool, slots))
        })();
        let (pool, slots) = match made {
            Ok(made) => made,
            Err(words) => {
                unsafe {
                    vk_device.destroy_semaphore(done, None);
                    vk_device.destroy_semaphore(taken, None);
                }
                return Err(words);
            }
        };
        let ring = Ring { slots, newest: None, claimed: None, count: 0, last: 0 };
        Ok(Lend { inner: Arc::new(Inner { device: device.clone(), done, taken, pool, ring: Mutex::new(ring) }) })
    }

    /// Whether two lends are one.
    pub fn same(&self, other: &Lend) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    pub fn device(&self) -> &Device {
        &self.inner.device
    }

    /// What the core is given at create.
    pub fn lent(&self) -> Lent {
        let device = &self.inner.device;
        let lending = device.lending().expect("a lend's device lends");
        Lent {
            get_instance_proc_addr: device.entry().static_fn().get_instance_proc_addr as usize,
            api: device.api(),
            instance: device.raw_instance(),
            physical: device.raw_physical(),
            device: device.raw_device(),
            family: lending.core.0,
            queue: lending.core.1,
            host_family: lending.intake_family,
            done: self.inner.done.as_raw(),
        }
    }

    /// The core's picture `value`, `height` rows of `stride` bytes from `offset` in its buffer, copied into a slot of the
    /// ring once its `ready` reaches the value, `done` then signalled with it: true when taken, false for a value not
    /// newer than the last taken (EmuSen_DevicePicture.md §17).
    #[allow(clippy::too_many_arguments)]
    pub fn take(&self, buffer: u64, offset: u64, width: u32, height: u32, stride: u32, ready: u64, value: u64) -> Result<bool, String> {
        let inner = &*self.inner;
        let device = &inner.device;
        let vk_device = device.vk();
        let mut ring = locked(&inner.ring);
        if value <= ring.last {
            return Ok(false);
        }
        if stride < width.saturating_mul(4) || stride % 4 != 0 || width == 0 || height == 0 {
            return Err(format!("a picture {width} by {height} with rows of {stride} bytes cannot be taken"));
        }
        let (newest, claimed) = (ring.newest, ring.claimed);
        let Some(i) = (0..SLOTS).find(|&i| Some(i) != newest && Some(i) != claimed) else { unreachable!("three slots, two held at most") };
        let bytes = stride as u64 * height as u64;
        let slot = &mut ring.slots[i];
        unsafe {
            if slot.in_flight {
                vk_device.wait_for_fences(&[slot.fence], true, u64::MAX).check("vkWaitForFences")?;
                vk_device.reset_fences(&[slot.fence]).check("vkResetFences")?;
                slot.in_flight = false;
            }
            if slot.bytes < bytes {
                if slot.buffer != vk::Buffer::null() {
                    vk_device.destroy_buffer(slot.buffer, None);
                    vk_device.free_memory(slot.memory, None);
                    (slot.buffer, slot.memory, slot.bytes) = (vk::Buffer::null(), vk::DeviceMemory::null(), 0);
                }
                let lending = device.lending().expect("a lend's device lends");
                let families = [lending.intake_family, device.family()];
                let mut info = vk::BufferCreateInfo::default().size(bytes).usage(vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST);
                info = if families[0] == families[1] { info.sharing_mode(vk::SharingMode::EXCLUSIVE) } else { info.sharing_mode(vk::SharingMode::CONCURRENT).queue_family_indices(&families) };
                let handle = vk_device.create_buffer(&info, None).check("vkCreateBuffer")?;
                let needs = vk_device.get_buffer_memory_requirements(handle);
                let made = device
                    .memory_type(needs.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL, vk::MemoryPropertyFlags::empty(), "DeviceLocalBit")
                    .and_then(|kind| vk_device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(needs.size).memory_type_index(kind), None).check("vkAllocateMemory"))
                    .and_then(|memory| match vk_device.bind_buffer_memory(handle, memory, 0).check("vkBindBufferMemory") {
                        Ok(()) => Ok(memory),
                        Err(words) => {
                            vk_device.free_memory(memory, None);
                            Err(words)
                        }
                    });
                match made {
                    Ok(memory) => (slot.buffer, slot.memory, slot.bytes) = (handle, memory, bytes),
                    Err(words) => {
                        vk_device.destroy_buffer(handle, None);
                        return Err(words);
                    }
                }
            }
            let commands = slot.commands;
            vk_device.reset_command_buffer(commands, vk::CommandBufferResetFlags::empty()).check("vkResetCommandBuffer")?;
            vk_device.begin_command_buffer(commands, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)).check("vkBeginCommandBuffer")?;
            vk_device.cmd_copy_buffer(commands, vk::Buffer::from_raw(buffer), slot.buffer, &[vk::BufferCopy { src_offset: offset, dst_offset: 0, size: bytes }]);
            vk_device.end_command_buffer(commands).check("vkEndCommandBuffer")?;
        }
        let count = ring.count + 1;
        let slot = &mut ring.slots[i];
        let commands = [slot.commands];
        let waits = [vk::Semaphore::from_raw(ready)];
        let wait_values = [value];
        let stages = [vk::PipelineStageFlags::TRANSFER];
        let signals = [inner.done, inner.taken];
        let signal_values = [value, count];
        let mut values = vk::TimelineSemaphoreSubmitInfo::default().wait_semaphore_values(&wait_values).signal_semaphore_values(&signal_values);
        let submit = vk::SubmitInfo::default().command_buffers(&commands).wait_semaphores(&waits).wait_dst_stage_mask(&stages).signal_semaphores(&signals).push_next(&mut values);
        {
            let lending = device.lending().expect("a lend's device lends");
            let queue = locked(&lending.intake);
            unsafe { vk_device.queue_submit(*queue, &[submit], slot.fence) }.check("vkQueueSubmit")?;
        }
        slot.in_flight = true;
        slot.picture = Some(Taken { buffer: slot.buffer, width, height, row_pixels: stride / 4, value: count, semaphore: inner.taken });
        ring.newest = Some(i);
        (ring.count, ring.last) = (count, value);
        Ok(true)
    }

    /// The newest copy newer than `after`, claimed for a chain whose last submission has been waited for, the one it
    /// claimed before let go of; `None` when there is none newer.
    pub fn claim(&self, after: u64) -> Option<Taken> {
        let mut ring = locked(&self.inner.ring);
        let newest = ring.newest?;
        let picture = ring.slots[newest].picture.filter(|p| p.value > after)?;
        ring.claimed = Some(newest);
        Some(picture)
    }

    /// Lets go of what the chain claimed, its reads waited for.
    pub fn release(&self) {
        locked(&self.inner.ring).claimed = None;
    }

    /// Waits for every copy the intake has made, so that nothing of the core's is read once this returns.
    pub fn settle(&self) -> Result<(), String> {
        let vk_device = self.inner.device.vk();
        let mut ring = locked(&self.inner.ring);
        for slot in &mut ring.slots {
            if slot.in_flight {
                unsafe {
                    vk_device.wait_for_fences(&[slot.fence], true, u64::MAX).check("vkWaitForFences")?;
                    vk_device.reset_fences(&[slot.fence]).check("vkResetFences")?;
                }
                slot.in_flight = false;
            }
        }
        Ok(())
    }

    /// The value `done` has reached, waiting for nothing.
    pub fn done_value(&self) -> u64 {
        unsafe { self.inner.device.vk().get_semaphore_counter_value(self.inner.done) }.unwrap_or(0)
    }
}
