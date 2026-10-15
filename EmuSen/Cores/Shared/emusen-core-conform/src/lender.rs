//! The kit's own lender for C20: a Vulkan 1.2 device with timeline semaphores, a queue the kit submits to and one lent
//! to the core, and the host's reading of a picture handed on it (EmuSen_CoreAPI.md §6.29).

use ash::vk::{self, Handle};
use emusen_native::core::sys::{DeviceFrame, DeviceLend};

pub struct Lender {
    _entry: ash::Entry,
    instance: ash::Instance,
    physical: vk::PhysicalDevice,
    device: ash::Device,
    api: u32,
    host_family: u32,
    host_queue: vk::Queue,
    lent_family: u32,
    lent_index: u32,
    pool: vk::CommandPool,
    commands: vk::CommandBuffer,
    fence: vk::Fence,
    done: vk::Semaphore,
    released: u64,
    readback: Option<(vk::Buffer, vk::DeviceMemory, *mut u8, u64)>,
    memory: vk::PhysicalDeviceMemoryProperties,
}

impl Lender {
    /// A device that can lend a queue, the first discrete one by preference, or why there is none.
    pub fn new() -> Result<Lender, String> {
        let entry = unsafe { ash::Entry::load() }.map_err(|e| format!("no Vulkan loader: {e}"))?;
        let application = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_2);
        let instance = unsafe { entry.create_instance(&vk::InstanceCreateInfo::default().application_info(&application), None) }.map_err(|e| format!("vkCreateInstance returned {e}"))?;
        let mut found = None;
        let mut devices = unsafe { instance.enumerate_physical_devices() }.unwrap_or_default();
        devices.sort_by_key(|&p| unsafe { instance.get_physical_device_properties(p) }.device_type != vk::PhysicalDeviceType::DISCRETE_GPU);
        for physical in devices {
            let properties = unsafe { instance.get_physical_device_properties(physical) };
            if vk::api_version_minor(properties.api_version) < 2 && vk::api_version_major(properties.api_version) == 1 {
                continue;
            }
            let queues = unsafe { instance.get_physical_device_queue_family_properties(physical) };
            let Some(host) = queues.iter().position(|q| q.queue_flags.contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)) else { continue };
            let lent = match queues.iter().enumerate().position(|(i, q)| i != host && q.queue_flags.contains(vk::QueueFlags::COMPUTE)) {
                Some(family) => (family as u32, 0),
                None if queues[host].queue_count > 1 => (host as u32, 1),
                None => continue,
            };
            found = Some((physical, properties.api_version, host as u32, lent));
            break;
        }
        let Some((physical, api, host_family, (lent_family, lent_index))) = found else {
            unsafe { instance.destroy_instance(None) };
            return Err("no Vulkan 1.2 device has a queue to lend".to_string());
        };
        let priorities = [1.0f32, 1.0];
        let mut infos = vec![vk::DeviceQueueCreateInfo::default().queue_family_index(host_family).queue_priorities(&priorities[..if lent_family == host_family { 2 } else { 1 }])];
        if lent_family != host_family {
            infos.push(vk::DeviceQueueCreateInfo::default().queue_family_index(lent_family).queue_priorities(&priorities[..1]));
        }
        let mut features = vk::PhysicalDeviceVulkan12Features::default().timeline_semaphore(true);
        let device = match unsafe { instance.create_device(physical, &vk::DeviceCreateInfo::default().queue_create_infos(&infos).push_next(&mut features), None) } {
            Ok(device) => device,
            Err(e) => {
                unsafe { instance.destroy_instance(None) };
                return Err(format!("vkCreateDevice returned {e}"));
            }
        };
        let host_queue = unsafe { device.get_device_queue(host_family, 0) };
        let pool = unsafe { device.create_command_pool(&vk::CommandPoolCreateInfo::default().queue_family_index(host_family).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER), None) }.map_err(|e| e.to_string())?;
        let commands = unsafe { device.allocate_command_buffers(&vk::CommandBufferAllocateInfo::default().command_pool(pool).command_buffer_count(1)) }.map_err(|e| e.to_string())?[0];
        let fence = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None) }.map_err(|e| e.to_string())?;
        let mut kind = vk::SemaphoreTypeCreateInfo::default().semaphore_type(vk::SemaphoreType::TIMELINE).initial_value(0);
        let done = unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default().push_next(&mut kind), None) }.map_err(|e| e.to_string())?;
        let memory = unsafe { instance.get_physical_device_memory_properties(physical) };
        Ok(Lender { _entry: entry, instance, physical, device, api, host_family, host_queue, lent_family, lent_index, pool, commands, fence, done, released: 0, readback: None, memory })
    }

    /// The lend a host of 1.9 passes at create.
    pub fn lend(&self) -> DeviceLend {
        let gipa = self._entry.static_fn().get_instance_proc_addr;
        DeviceLend {
            size: std::mem::size_of::<DeviceLend>() as u32,
            api_version: self.api,
            // SAFETY: a function pointer as the header's `void (*)(void)`, which the core turns back into Vulkan's type.
            get_instance_proc_addr: Some(unsafe { std::mem::transmute::<vk::PFN_vkGetInstanceProcAddr, unsafe extern "C" fn()>(gipa) }),
            instance: self.instance.handle().as_raw(),
            physical_device: self.physical.as_raw(),
            device: self.device.handle().as_raw(),
            queue_family: self.lent_family,
            queue_index: self.lent_index,
            host_queue_family: self.host_family,
            reserved: 0,
            done: self.done.as_raw(),
        }
    }

    fn readback(&mut self, bytes: u64) -> Result<(vk::Buffer, *mut u8), String> {
        if let Some((buffer, _, mapped, size)) = self.readback.filter(|r| r.3 >= bytes) {
            let _ = size;
            return Ok((buffer, mapped));
        }
        if let Some((buffer, memory, _, _)) = self.readback.take() {
            unsafe {
                self.device.destroy_buffer(buffer, None);
                self.device.free_memory(memory, None);
            }
        }
        let buffer = unsafe { self.device.create_buffer(&vk::BufferCreateInfo::default().size(bytes).usage(vk::BufferUsageFlags::TRANSFER_DST), None) }.map_err(|e| e.to_string())?;
        let needs = unsafe { self.device.get_buffer_memory_requirements(buffer) };
        let want = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let kind = (0..self.memory.memory_type_count).find(|&i| needs.memory_type_bits & (1 << i) != 0 && self.memory.memory_types[i as usize].property_flags.contains(want)).ok_or("no host memory")?;
        let memory = unsafe { self.device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(needs.size).memory_type_index(kind), None) }.map_err(|e| e.to_string())?;
        unsafe { self.device.bind_buffer_memory(buffer, memory, 0) }.map_err(|e| e.to_string())?;
        let mapped = unsafe { self.device.map_memory(memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()) }.map_err(|e| e.to_string())? as *mut u8;
        self.readback = Some((buffer, memory, mapped, bytes));
        Ok((buffer, mapped))
    }

    /// The picture `frame` names, read on the kit's queue once `ready` has reached its value, a row of `width` pixels
    /// at a time; with `release`, `done` is signalled with its value as the reading ends.
    pub fn read(&mut self, frame: &DeviceFrame, release: bool) -> Result<Vec<u8>, String> {
        let (rows, stride, width) = (frame.height.max(0) as u64, frame.stride.max(0) as u64, frame.width.max(0) as u64 * 4);
        let bytes = rows * stride;
        let (buffer, mapped) = self.readback(bytes.max(4))?;
        let device = &self.device;
        unsafe {
            device.reset_command_buffer(self.commands, vk::CommandBufferResetFlags::empty()).map_err(|e| e.to_string())?;
            device.begin_command_buffer(self.commands, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)).map_err(|e| e.to_string())?;
            let region = [vk::BufferCopy::default().src_offset(frame.offset).dst_offset(0).size(bytes.max(4))];
            device.cmd_copy_buffer(self.commands, vk::Buffer::from_raw(frame.buffer), buffer, &region);
            let barrier = [vk::MemoryBarrier::default().src_access_mask(vk::AccessFlags::TRANSFER_WRITE).dst_access_mask(vk::AccessFlags::HOST_READ)];
            device.cmd_pipeline_barrier(self.commands, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::HOST, vk::DependencyFlags::empty(), &barrier, &[], &[]);
            device.end_command_buffer(self.commands).map_err(|e| e.to_string())?;
            let buffers = [self.commands];
            let waits = [vk::Semaphore::from_raw(frame.ready)];
            let wait_values = [frame.value];
            let stages = [vk::PipelineStageFlags::TRANSFER];
            let signals = [self.done];
            let released = frame.value.max(self.released);
            let signal_values = [released];
            let mut values = vk::TimelineSemaphoreSubmitInfo::default().wait_semaphore_values(&wait_values).signal_semaphore_values(if release { &signal_values } else { &[] });
            let submit = vk::SubmitInfo::default().command_buffers(&buffers).wait_semaphores(&waits).wait_dst_stage_mask(&stages).signal_semaphores(if release { &signals } else { &[] }).push_next(&mut values);
            device.queue_submit(self.host_queue, &[submit], self.fence).map_err(|e| e.to_string())?;
            device.wait_for_fences(&[self.fence], true, 10_000_000_000).map_err(|e| format!("the picture's ready value never came: {e}"))?;
            device.reset_fences(&[self.fence]).map_err(|e| e.to_string())?;
            if release {
                self.released = released;
            }
        }
        let mut out = Vec::with_capacity((rows * width) as usize);
        for y in 0..rows {
            // SAFETY: the readback holds `rows * stride` bytes, written and made visible above.
            out.extend_from_slice(unsafe { std::slice::from_raw_parts(mapped.add((y * stride) as usize), width as usize) });
        }
        Ok(out)
    }

    /// `done` signalled with `value`, from the host, letting go of every picture up to it.
    pub fn release(&mut self, value: u64) -> Result<(), String> {
        if value <= self.released {
            return Ok(());
        }
        unsafe { self.device.signal_semaphore(&vk::SemaphoreSignalInfo::default().semaphore(self.done).value(value)) }.map_err(|e| e.to_string())?;
        self.released = value;
        Ok(())
    }
}

impl Drop for Lender {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            if let Some((buffer, memory, _, _)) = self.readback.take() {
                self.device.destroy_buffer(buffer, None);
                self.device.free_memory(memory, None);
            }
            self.device.destroy_semaphore(self.done, None);
            self.device.destroy_fence(self.fence, None);
            self.device.destroy_command_pool(self.pool, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
