//! One Vulkan compute device and the little of Vulkan the multiple's shading needs: C#'s `GpuDevice.cs` on `ash`, the loader found at
//! run time so a machine without one gets a report and never a link failure. See Mars_Gpu.md §1 and Mars_Native.md §6.4.

use std::ffi::CStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};

use ash::vk;

/// Names a device by a part of its name, for tests and for a machine with more than one.
pub const DEVICE_VARIABLE: &str = "EMUSEN_MARS_GPU_DEVICE";
const PORTABILITY_ENUMERATION: &CStr = c"VK_KHR_portability_enumeration";
const PORTABILITY_SUBSET: &CStr = c"VK_KHR_portability_subset";

/// Where a buffer's memory lives: the host's, mapped for good, or the device's own, reached only by a copy.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Where {
    Host,
    Device,
}

/// A storage buffer; a host one is mapped for its whole life and read or written through a slice.
pub struct GpuBuffer {
    pub handle: vk::Buffer,
    memory: vk::DeviceMemory,
    pub bytes: u64,
    mapped: *mut u8,
}

// SAFETY: the mapped pointer is host memory the device owns for the buffer's life; which thread touches it when is the rasteriser's rule.
unsafe impl Send for GpuBuffer {}
unsafe impl Sync for GpuBuffer {}

impl GpuBuffer {
    /// The mapped bytes as values, for a host buffer.
    ///
    /// # Panics
    /// On a device buffer, which is reached by a copy, not mapped.
    pub fn slice<T: Copy>(&self) -> &[T] {
        assert!(!self.mapped.is_null(), "device memory is reached by a copy, not mapped");
        // SAFETY: mapped for the buffer's life, `bytes` long.
        unsafe { std::slice::from_raw_parts(self.mapped as *const T, self.bytes as usize / size_of::<T>()) }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn slice_mut<T: Copy>(&self) -> &mut [T] {
        assert!(!self.mapped.is_null(), "device memory is reached by a copy, not mapped");
        // SAFETY: as `slice`; the caller has the buffer to itself while it writes.
        unsafe { std::slice::from_raw_parts_mut(self.mapped as *mut T, self.bytes as usize / size_of::<T>()) }
    }
}

/// A compute shader with its storage buffers at bindings 0.. of set 0 and one block of push constants.
pub struct GpuProgram {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    set_layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    set: vk::DescriptorSet,
    pub buffers: usize,
    pub push_bytes: usize,
}

/// The pending submission's fence and flag, shared with a thread that only waits (the presenter's), which never submits.
pub struct Pending {
    device: ash::Device,
    fence: vk::Fence,
    pending: AtomicBool,
    gone: AtomicBool,
}

impl Pending {
    /// `WaitForPending`: returns once the pending submission has finished; safe from a thread that does not submit, while nothing submits one.
    pub fn wait(&self) {
        if !self.pending.load(Acquire) || self.gone.load(Acquire) {
            return;
        }
        // SAFETY: the fence is the device's own until `gone`.
        unsafe { self.device.wait_for_fences(&[self.fence], true, u64::MAX) }.expect("vkWaitForFences");
        self.pending.store(false, Release);
    }
}

pub struct GpuDevice {
    _entry: ash::Entry,
    instance: ash::Instance,
    physical: vk::PhysicalDevice,
    device: ash::Device,
    queue: vk::Queue,
    pool: vk::CommandPool,
    commands: vk::CommandBuffer,
    pending_commands: vk::CommandBuffer,
    fence: vk::Fence,
    pending: Arc<Pending>,
    memory: vk::PhysicalDeviceMemoryProperties,
    pub name: String,
    pub software: bool,
    /// The most one storage buffer may bind, which the memory at a multiple is measured against (Mars_Gpu.md §5).
    pub max_buffer_bytes: u64,
}

struct Candidate {
    physical: vk::PhysicalDevice,
    name: String,
    software: bool,
    family: u32,
    api: u32,
    rank: i32,
}

/// The instance and what it found; it owns the instance until a device takes it over.
struct Enumeration {
    entry: ash::Entry,
    instance: ash::Instance,
    candidates: Vec<Candidate>,
}

fn has(extensions: &[vk::ExtensionProperties], name: &CStr) -> bool {
    extensions.iter().any(|e| e.extension_name_as_c_str().is_ok_and(|n| n == name))
}

impl Enumeration {
    fn new() -> Result<Enumeration, String> {
        // SAFETY: the loader is loaded once and outlives every handle made through it, which the device keeps it for.
        let entry = unsafe { ash::Entry::load() }.map_err(|e| e.to_string())?;
        let application = vk::ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 1, 0));
        let extensions = unsafe { entry.enumerate_instance_extension_properties(None) }.map_err(|e| format!("vkEnumerateInstanceExtensionProperties returned {e}"))?;
        let mut create = vk::InstanceCreateInfo::default().application_info(&application);
        // Without this a loader hides layered devices, which on macOS is every device there is (Mars_Gpu.md §4).
        let names = [PORTABILITY_ENUMERATION.as_ptr()];
        if has(&extensions, PORTABILITY_ENUMERATION) {
            create = create.enabled_extension_names(&names).flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR);
        }
        let instance = unsafe { entry.create_instance(&create, None) }.map_err(|e| format!("vkCreateInstance returned {e}"))?;

        let mut candidates = Vec::new();
        let devices = unsafe { instance.enumerate_physical_devices() }.unwrap_or_default();
        for physical in devices {
            let properties = unsafe { instance.get_physical_device_properties(physical) };
            let queues = unsafe { instance.get_physical_device_queue_family_properties(physical) };
            let Some(family) = queues.iter().position(|q| q.queue_flags.contains(vk::QueueFlags::COMPUTE)) else { continue };
            let rank = match properties.device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU => 0,
                vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
                _ => 3,
            };
            let name = properties.device_name_as_c_str().map_or_else(|_| "unnamed".to_string(), |n| n.to_string_lossy().into_owned());
            candidates.push(Candidate { physical, name, software: properties.device_type == vk::PhysicalDeviceType::CPU, family: family as u32, api: properties.api_version, rank });
        }
        // A stable sort, where C#'s `List.Sort` is not: equal ranks keep the loader's order.
        candidates.sort_by_key(|c| c.rank);
        Ok(Enumeration { entry, instance, candidates })
    }
}


impl GpuDevice {
    /// Every compute-capable device the loader offers, in the order it would be chosen; empty with no loader or no driver.
    pub fn device_names() -> Vec<String> {
        match Enumeration::new() {
            Ok(probe) => {
                let names = probe.candidates.iter().map(|c| c.name.clone()).collect();
                unsafe { probe.instance.destroy_instance(None) };
                names
            }
            Err(_) => Vec::new(),
        }
    }

    /// None, with the reason, when there is no usable device: the caller's answer to that is the CPU path.
    pub fn try_create(name_contains: Option<&str>) -> Result<GpuDevice, String> {
        let wanted = name_contains.map(str::to_string).or_else(|| std::env::var(DEVICE_VARIABLE).ok()).unwrap_or_default();
        let probe = match Enumeration::new() {
            Ok(probe) => probe,
            Err(e) => return Err(format!("Vulkan is not available: {e}")),
        };
        let chosen = probe.candidates.iter().position(|c| wanted.is_empty() || c.name.to_lowercase().contains(&wanted.to_lowercase()));
        let Some(index) = chosen else {
            let report = if probe.candidates.is_empty() { "no Vulkan device offers a compute queue".to_string() } else { format!("no Vulkan device is named like \"{wanted}\"") };
            unsafe { probe.instance.destroy_instance(None) };
            return Err(report);
        };
        let pick = &probe.candidates[index];
        match GpuDevice::open(&probe.entry, &probe.instance, pick) {
            Ok(device) => {
                let _ = pick.api;
                Ok(device)
            }
            Err(e) => {
                unsafe { probe.instance.destroy_instance(None) };
                Err(format!("Vulkan is not available: {e}"))
            }
        }
    }

    /// `"{name}, Vulkan {major}.{minor}"`, the report of a device that opened.
    pub fn report(&self) -> String {
        let api = unsafe { self.instance.get_physical_device_properties(self.physical_of()) }.api_version;
        format!("{}, Vulkan {}.{}", self.name, vk::api_version_major(api), vk::api_version_minor(api))
    }

    fn physical_of(&self) -> vk::PhysicalDevice {
        self.physical
    }

    fn open(entry: &ash::Entry, instance: &ash::Instance, pick: &Candidate) -> Result<GpuDevice, String> {
        let priority = [1.0f32];
        let queue_info = [vk::DeviceQueueCreateInfo::default().queue_family_index(pick.family).queue_priorities(&priority)];
        let extensions = unsafe { instance.enumerate_device_extension_properties(pick.physical) }.unwrap_or_default();
        let names = [PORTABILITY_SUBSET.as_ptr()];
        let mut device_info = vk::DeviceCreateInfo::default().queue_create_infos(&queue_info);
        // A layered implementation must be told its subset is accepted; this is what MoltenVK over Metal asks (Mars_Gpu.md §4).
        if has(&extensions, PORTABILITY_SUBSET) {
            device_info = device_info.enabled_extension_names(&names);
        }
        let device = unsafe { instance.create_device(pick.physical, &device_info, None) }.map_err(|e| format!("vkCreateDevice returned {e}"))?;
        let queue = unsafe { device.get_device_queue(pick.family, 0) };

        let pool_info = vk::CommandPoolCreateInfo::default().queue_family_index(pick.family).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let pool = unsafe { device.create_command_pool(&pool_info, None) }.map_err(|e| format!("vkCreateCommandPool returned {e}"))?;
        let allocate = vk::CommandBufferAllocateInfo::default().command_pool(pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(2);
        let buffers = unsafe { device.allocate_command_buffers(&allocate) }.map_err(|e| format!("vkAllocateCommandBuffers returned {e}"))?;
        let fence = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None) }.map_err(|e| format!("vkCreateFence returned {e}"))?;
        let pending_fence = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None) }.map_err(|e| format!("vkCreateFence returned {e}"))?;
        let memory = unsafe { instance.get_physical_device_memory_properties(pick.physical) };
        let properties = unsafe { instance.get_physical_device_properties(pick.physical) };

        Ok(GpuDevice {
            _entry: entry.clone(),
            instance: instance.clone(),
            physical: pick.physical,
            device: device.clone(),
            queue,
            pool,
            commands: buffers[0],
            pending_commands: buffers[1],
            fence,
            pending: Arc::new(Pending { device, fence: pending_fence, pending: AtomicBool::new(false), gone: AtomicBool::new(false) }),
            memory,
            name: pick.name.clone(),
            software: pick.software,
            max_buffer_bytes: properties.limits.max_storage_buffer_range as u64,
        })
    }

    pub fn pending(&self) -> Arc<Pending> {
        self.pending.clone()
    }

    pub fn wait_for_pending(&self) {
        self.pending.wait();
    }

    pub fn create_buffer(&self, bytes: u64, at: Where) -> Result<GpuBuffer, String> {
        let size = bytes.max(4);
        let info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let handle = unsafe { self.device.create_buffer(&info, None) }.map_err(|e| format!("vkCreateBuffer returned {e}"))?;
        let needs = unsafe { self.device.get_buffer_memory_requirements(handle) };
        let allocate = vk::MemoryAllocateInfo::default().allocation_size(needs.size).memory_type_index(self.memory_type(needs.memory_type_bits, at)?);
        let memory = match unsafe { self.device.allocate_memory(&allocate, None) } {
            Ok(memory) => memory,
            Err(e) => {
                unsafe { self.device.destroy_buffer(handle, None) };
                return Err(format!("vkAllocateMemory returned {e}"));
            }
        };
        if let Err(e) = unsafe { self.device.bind_buffer_memory(handle, memory, 0) } {
            unsafe {
                self.device.destroy_buffer(handle, None);
                self.device.free_memory(memory, None);
            }
            return Err(format!("vkBindBufferMemory returned {e}"));
        }
        let mapped = if at == Where::Host {
            unsafe { self.device.map_memory(memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()) }.map_err(|e| format!("vkMapMemory returned {e}"))? as *mut u8
        } else {
            std::ptr::null_mut()
        };
        Ok(GpuBuffer { handle, memory, bytes: size, mapped })
    }

    /// Host memory the CPU can read back quickly if there is any, since a readback from write-combined memory crawls.
    fn memory_type(&self, allowed: u32, at: Where) -> Result<u32, String> {
        let need = if at == Where::Host { vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT } else { vk::MemoryPropertyFlags::DEVICE_LOCAL };
        let want = if at == Where::Host { need | vk::MemoryPropertyFlags::HOST_CACHED } else { need };
        for flags in [want, need] {
            for i in 0..self.memory.memory_type_count as usize {
                if allowed & (1 << i) != 0 && self.memory.memory_types[i].property_flags.contains(flags) {
                    return Ok(i as u32);
                }
            }
        }
        Err(format!("no {} memory type suits the buffer", if at == Where::Host { "host" } else { "device" }))
    }

    pub fn destroy_buffer(&self, buffer: GpuBuffer) {
        unsafe {
            if !buffer.mapped.is_null() {
                self.device.unmap_memory(buffer.memory);
            }
            self.device.destroy_buffer(buffer.handle, None);
            self.device.free_memory(buffer.memory, None);
        }
    }

    /// A compute shader with its storage buffers at bindings 0..buffers-1 of set 0 and one block of push constants.
    pub fn create_program(&self, spirv: &[u8], buffers: usize, push_bytes: usize) -> Result<GpuProgram, String> {
        if spirv.is_empty() || !spirv.len().is_multiple_of(4) {
            return Err("SPIR-V is a whole number of words".into());
        }
        let words = ash::util::read_spv(&mut std::io::Cursor::new(spirv)).map_err(|e| e.to_string())?;
        let module = unsafe { self.device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&words), None) }.map_err(|e| format!("vkCreateShaderModule returned {e}"))?;

        let bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..buffers as u32)
            .map(|i| vk::DescriptorSetLayoutBinding::default().binding(i).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(1).stage_flags(vk::ShaderStageFlags::COMPUTE))
            .collect();
        let set_layout = unsafe { self.device.create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings), None) }.map_err(|e| format!("vkCreateDescriptorSetLayout returned {e}"))?;
        let range = [vk::PushConstantRange::default().stage_flags(vk::ShaderStageFlags::COMPUTE).offset(0).size(push_bytes as u32)];
        let set_layouts = [set_layout];
        let mut layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);
        if push_bytes > 0 {
            layout_info = layout_info.push_constant_ranges(&range);
        }
        let layout = unsafe { self.device.create_pipeline_layout(&layout_info, None) }.map_err(|e| format!("vkCreatePipelineLayout returned {e}"))?;

        let stage = vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::COMPUTE).module(module).name(c"main");
        let pipeline_info = vk::ComputePipelineCreateInfo::default().stage(stage).layout(layout);
        let made = unsafe { self.device.create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None) };
        unsafe { self.device.destroy_shader_module(module, None) };
        let pipeline = match made {
            Ok(pipelines) => pipelines[0],
            Err((_, e)) => return Err(format!("vkCreateComputePipelines returned {e}")),
        };

        let sizes = [vk::DescriptorPoolSize::default().ty(vk::DescriptorType::STORAGE_BUFFER).descriptor_count(buffers.max(1) as u32)];
        let pool = unsafe { self.device.create_descriptor_pool(&vk::DescriptorPoolCreateInfo::default().max_sets(1).pool_sizes(&sizes), None) }.map_err(|e| format!("vkCreateDescriptorPool returned {e}"))?;
        let sets = unsafe { self.device.allocate_descriptor_sets(&vk::DescriptorSetAllocateInfo::default().descriptor_pool(pool).set_layouts(&set_layouts)) }.map_err(|e| format!("vkAllocateDescriptorSets returned {e}"))?;
        Ok(GpuProgram { pipeline, layout, set_layout, pool, set: sets[0], buffers, push_bytes })
    }

    pub fn destroy_program(&self, program: GpuProgram) {
        unsafe {
            self.device.destroy_pipeline(program.pipeline, None);
            self.device.destroy_pipeline_layout(program.layout, None);
            self.device.destroy_descriptor_pool(program.pool, None);
            self.device.destroy_descriptor_set_layout(program.set_layout, None);
        }
    }

    /// `Submit`: records, submits and waits; nothing of this batch is in flight when this returns, though a pending one may be (Mars_Gpu.md §14).
    pub fn submit(&self, record: impl FnOnce(&mut Commands)) {
        self.record(self.commands, self.fence, record);
        unsafe {
            self.device.wait_for_fences(&[self.fence], true, u64::MAX).expect("vkWaitForFences");
            self.device.reset_fences(&[self.fence]).expect("vkResetFences");
        }
    }

    /// `SubmitPending`: records and submits without waiting; one is pending at a time, and the next waits for the last before it records.
    pub fn submit_pending(&self, record: impl FnOnce(&mut Commands)) {
        self.pending.wait();
        unsafe { self.device.reset_fences(&[self.pending.fence]) }.expect("vkResetFences");
        self.record(self.pending_commands, self.pending.fence, record);
        self.pending.pending.store(true, Release);
    }

    /// Every command already ends in a barrier that orders the submissions after it; this adds only the host's (Mars_Gpu.md §14).
    fn record(&self, buffer: vk::CommandBuffer, fence: vk::Fence, record: impl FnOnce(&mut Commands)) {
        unsafe {
            self.device.reset_command_buffer(buffer, vk::CommandBufferResetFlags::empty()).expect("vkResetCommandBuffer");
            self.device.begin_command_buffer(buffer, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)).expect("vkBeginCommandBuffer");
        }
        let mut commands = Commands { device: &self.device, commands: buffer };
        record(&mut commands);
        commands.barrier_to_host();
        unsafe {
            self.device.end_command_buffer(buffer).expect("vkEndCommandBuffer");
            let buffers = [buffer];
            self.device.queue_submit(self.queue, &[vk::SubmitInfo::default().command_buffers(&buffers)], fence).expect("vkQueueSubmit");
        }
    }
}

impl Drop for GpuDevice {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.pending.gone.store(true, Release);
            self.device.destroy_fence(self.fence, None);
            self.device.destroy_fence(self.pending.fence, None);
            self.device.destroy_command_pool(self.pool, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

/// What one submission may hold: copies and dispatches, each made to wait for the one before it.
pub struct Commands<'a> {
    device: &'a ash::Device,
    commands: vk::CommandBuffer,
}

impl Commands<'_> {
    pub fn fill(&mut self, buffer: &GpuBuffer, word: u32) {
        unsafe { self.device.cmd_fill_buffer(self.commands, buffer.handle, 0, vk::WHOLE_SIZE, word) };
        self.barrier();
    }

    /// `bytes` zero copies the shorter of the two.
    pub fn copy(&mut self, from: &GpuBuffer, to: &GpuBuffer, bytes: u64, from_offset: u64, to_offset: u64) {
        let region = [vk::BufferCopy::default().src_offset(from_offset).dst_offset(to_offset).size(if bytes == 0 { from.bytes.min(to.bytes) } else { bytes })];
        unsafe { self.device.cmd_copy_buffer(self.commands, from.handle, to.handle, &region) };
        self.barrier();
    }

    /// The descriptor set is written here, at record time, which is sound only because every record first waits for the pending submission.
    pub fn dispatch<T: Copy>(&mut self, program: &GpuProgram, buffers: &[&GpuBuffer], push: &T, x: u32, y: u32, z: u32) {
        assert!(buffers.len() == program.buffers, "the program binds {} buffers, not {}", program.buffers, buffers.len());
        assert!(size_of::<T>() == program.push_bytes, "the program takes {} bytes of push constants, not {}", program.push_bytes, size_of::<T>());
        let infos: Vec<[vk::DescriptorBufferInfo; 1]> = buffers.iter().map(|b| [vk::DescriptorBufferInfo::default().buffer(b.handle).offset(0).range(vk::WHOLE_SIZE)]).collect();
        let writes: Vec<vk::WriteDescriptorSet> = infos
            .iter()
            .enumerate()
            .map(|(i, info)| vk::WriteDescriptorSet::default().dst_set(program.set).dst_binding(i as u32).descriptor_type(vk::DescriptorType::STORAGE_BUFFER).buffer_info(info))
            .collect();
        // SAFETY: the push value is plain data of the program's size, read as bytes.
        let bytes = unsafe { std::slice::from_raw_parts((push as *const T).cast::<u8>(), size_of::<T>()) };
        unsafe {
            self.device.update_descriptor_sets(&writes, &[]);
            self.device.cmd_bind_pipeline(self.commands, vk::PipelineBindPoint::COMPUTE, program.pipeline);
            self.device.cmd_bind_descriptor_sets(self.commands, vk::PipelineBindPoint::COMPUTE, program.layout, 0, &[program.set], &[]);
            if !bytes.is_empty() {
                self.device.cmd_push_constants(self.commands, program.layout, vk::ShaderStageFlags::COMPUTE, 0, bytes);
            }
            self.device.cmd_dispatch(self.commands, x, y, z);
        }
        self.barrier();
    }

    /// Everything after waits for everything before, in this submission and the ones that follow it.
    fn barrier(&mut self) {
        let barrier = [vk::MemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::SHADER_WRITE | vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE | vk::AccessFlags::TRANSFER_READ | vk::AccessFlags::TRANSFER_WRITE)];
        let stages = vk::PipelineStageFlags::COMPUTE_SHADER | vk::PipelineStageFlags::TRANSFER;
        unsafe { self.device.cmd_pipeline_barrier(self.commands, stages, stages, vk::DependencyFlags::empty(), &barrier, &[], &[]) };
    }

    /// A fence makes only the device's own accesses complete; a host read of mapped memory needs this as well.
    fn barrier_to_host(&mut self) {
        let barrier = [vk::MemoryBarrier::default().src_access_mask(vk::AccessFlags::SHADER_WRITE | vk::AccessFlags::TRANSFER_WRITE).dst_access_mask(vk::AccessFlags::HOST_READ)];
        unsafe {
            self.device.cmd_pipeline_barrier(
                self.commands,
                vk::PipelineStageFlags::COMPUTE_SHADER | vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::HOST,
                vk::DependencyFlags::empty(),
                &barrier,
                &[],
                &[],
            )
        };
    }
}
