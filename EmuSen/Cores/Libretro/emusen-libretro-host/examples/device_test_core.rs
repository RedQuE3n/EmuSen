//! emusen-native's test core claiming DEVICE_PICTURE, painting a `PAINTED` image's picture (the libretro test core's)
//! on the device its host lends, so that the ring and the lent device are measured on one picture
//! (EmuSen_LibretroHost.md §10f). Built here, where ash already is, rather than in every crate that builds v1_test_core.

pub const TEST_ID: &str = "v1-device-core";
pub const TEST_NAME: &str = "V1DeviceCore";
pub const TEST_CAPABILITIES: u64 = emusen_native::core::caps::DEVICE_PICTURE;
pub const TEST_FAULTS: u64 = 0;
pub const PAINTER: Option<MakePainter> = Some(painter::make);

include!("../../../Shared/emusen-native/examples/common/test_core.rs");

emusen_native::core_exports!(TestCore; device_picture);

mod painter {
    use ash::vk::{self, Handle};
    use emusen_native::core::{DeviceFrame, DeviceLend, pixel, status};

    use super::Paint;

    const SLOTS: usize = 2;

    struct Slot {
        buffer: vk::Buffer,
        memory: vk::DeviceMemory,
        staging: vk::Buffer,
        staging_memory: vk::DeviceMemory,
        mapped: *mut u8,
        commands: vk::CommandBuffer,
        fence: vk::Fence,
        in_flight: bool,
        /// The last picture handed from this slot, which `done` must reach before it is written again.
        value: u64,
    }

    /// Two slots on the lent queue: the picture copied from host memory into a buffer both families may read.
    struct Painter {
        device: ash::Device,
        queue: vk::Queue,
        pool: vk::CommandPool,
        ready: vk::Semaphore,
        done: vk::Semaphore,
        slots: Vec<Slot>,
        count: u64,
    }

    pub fn make(lend: &DeviceLend) -> Option<Box<dyn Paint>> {
        let gipa = lend.get_instance_proc_addr?;
        if lend.api_version < vk::API_VERSION_1_2 {
            return None;
        }
        let gipa: vk::PFN_vkGetInstanceProcAddr = unsafe { std::mem::transmute(gipa) };
        let instance = unsafe { ash::Instance::load(&ash::StaticFn { get_instance_proc_addr: gipa }, vk::Instance::from_raw(lend.instance)) };
        let device = unsafe { ash::Device::load(instance.fp_v1_0(), vk::Device::from_raw(lend.device)) };
        let memory = unsafe { instance.get_physical_device_memory_properties(vk::PhysicalDevice::from_raw(lend.physical_device)) };
        let kind = |bits: u32, flags: vk::MemoryPropertyFlags| (0..memory.memory_type_count).find(|&k| bits & (1 << k) != 0 && memory.memory_types[k as usize].property_flags.contains(flags));
        let bytes = (super::PAINTED_WIDTH * super::PAINTED_HEIGHT * 4) as u64;
        let families = [lend.queue_family, lend.host_queue_family];
        unsafe {
            let queue = device.get_device_queue(lend.queue_family, lend.queue_index);
            let pool = device.create_command_pool(&vk::CommandPoolCreateInfo::default().queue_family_index(lend.queue_family).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER), None).ok()?;
            let commands = device.allocate_command_buffers(&vk::CommandBufferAllocateInfo::default().command_pool(pool).command_buffer_count(SLOTS as u32)).ok()?;
            let mut timeline = vk::SemaphoreTypeCreateInfo::default().semaphore_type(vk::SemaphoreType::TIMELINE).initial_value(0);
            let ready = device.create_semaphore(&vk::SemaphoreCreateInfo::default().push_next(&mut timeline), None).ok()?;
            let mut p = Painter { device: device.clone(), queue, pool, ready, done: vk::Semaphore::from_raw(lend.done), slots: Vec::new(), count: 0 };
            for commands in commands {
                let mut info = vk::BufferCreateInfo::default().size(bytes).usage(vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::TRANSFER_SRC);
                info = if families[0] == families[1] { info.sharing_mode(vk::SharingMode::EXCLUSIVE) } else { info.sharing_mode(vk::SharingMode::CONCURRENT).queue_family_indices(&families) };
                let buffer = device.create_buffer(&info, None).ok()?;
                let needs = device.get_buffer_memory_requirements(buffer);
                let memory = device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(needs.size).memory_type_index(kind(needs.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL)?), None).ok()?;
                device.bind_buffer_memory(buffer, memory, 0).ok()?;
                let staging = device.create_buffer(&vk::BufferCreateInfo::default().size(bytes).usage(vk::BufferUsageFlags::TRANSFER_SRC), None).ok()?;
                let needs = device.get_buffer_memory_requirements(staging);
                let staging_memory = device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(needs.size).memory_type_index(kind(needs.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)?), None).ok()?;
                device.bind_buffer_memory(staging, staging_memory, 0).ok()?;
                let mapped = device.map_memory(staging_memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()).ok()? as *mut u8;
                let fence = device.create_fence(&vk::FenceCreateInfo::default(), None).ok()?;
                p.slots.push(Slot { buffer, memory, staging, staging_memory, mapped, commands, fence, in_flight: false, value: 0 });
            }
            Some(Box::new(p))
        }
    }

    impl Paint for Painter {
        fn hand(&mut self, picture: &[u8], width: u32, height: u32) -> Result<DeviceFrame, i32> {
            let value = self.count + 1;
            let d = &self.device;
            let slot = &mut self.slots[value as usize % SLOTS];
            let bytes = (width * height * 4) as usize;
            unsafe {
                if slot.in_flight {
                    d.wait_for_fences(&[slot.fence], true, u64::MAX).map_err(|_| status::FAULT_BASE)?;
                    d.reset_fences(&[slot.fence]).map_err(|_| status::FAULT_BASE)?;
                    slot.in_flight = false;
                }
                std::ptr::copy_nonoverlapping(picture.as_ptr(), slot.mapped, bytes.min(picture.len()));
                let cmd = slot.commands;
                d.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty()).map_err(|_| status::FAULT_BASE)?;
                d.begin_command_buffer(cmd, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)).map_err(|_| status::FAULT_BASE)?;
                d.cmd_copy_buffer(cmd, slot.staging, slot.buffer, &[vk::BufferCopy { src_offset: 0, dst_offset: 0, size: bytes as u64 }]);
                d.end_command_buffer(cmd).map_err(|_| status::FAULT_BASE)?;
                // The slot's last picture given back by the host before its memory is written again (§6.29).
                let (waits, wait_values, stages) = ([self.done], [slot.value], [vk::PipelineStageFlags::TRANSFER]);
                let (signals, signal_values, buffers) = ([self.ready], [value], [cmd]);
                let mut values = vk::TimelineSemaphoreSubmitInfo::default().wait_semaphore_values(&wait_values).signal_semaphore_values(&signal_values);
                let submit = vk::SubmitInfo::default().command_buffers(&buffers).wait_semaphores(&waits).wait_dst_stage_mask(&stages).signal_semaphores(&signals).push_next(&mut values);
                d.queue_submit(self.queue, &[submit], slot.fence).map_err(|_| status::FAULT_BASE)?;
            }
            slot.in_flight = true;
            slot.value = value;
            self.count = value;
            Ok(DeviceFrame { format: pixel::RGBA8888, buffer: slot.buffer.as_raw(), offset: 0, width: width as i32, height: height as i32, stride: width as i32 * 4, reserved: 0, ready: self.ready.as_raw(), value, size: 0 })
        }
    }

    impl Drop for Painter {
        fn drop(&mut self) {
            unsafe {
                for s in &self.slots {
                    if s.in_flight {
                        let _ = self.device.wait_for_fences(&[s.fence], true, u64::MAX);
                    }
                    self.device.destroy_fence(s.fence, None);
                    self.device.destroy_buffer(s.buffer, None);
                    self.device.free_memory(s.memory, None);
                    self.device.destroy_buffer(s.staging, None);
                    self.device.free_memory(s.staging_memory, None);
                }
                self.device.destroy_command_pool(self.pool, None);
                self.device.destroy_semaphore(self.ready, None);
            }
        }
    }
}
