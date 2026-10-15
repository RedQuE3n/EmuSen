//! One Vulkan device with a graphics queue, and the few resources a slang chain needs: the C# `SlangVulkan`.
//!
//! A device records, submits and waits on one command buffer behind its own lock, so a chain may be built on one
//! thread while another draws. It lives until the last buffer or image made on it is gone, which is what lets a
//! readback outlive its chain while a picture still holds it. With validation asked for, the Khronos layer is loaded
//! with synchronisation validation and every finding waits to be taken: the library never calls its host. A failing
//! Vulkan call is said in the C#'s words, with the result named as the C# binding names it. See EmuSen_Serenity.md
//! §7.4 and §10, and EmuSen_RustPlatform.md §17.

use crate::preset::Wrap;
use ash::prelude::VkResult;
use ash::vk::{self, Handle};
use emusen_galaxia::dotnet_text::contains_ignore_case;
use std::cell::Cell;
use std::ffi::{CStr, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

/// The variable that names the device when the caller does not; the C# reads it, and a Rust host may.
pub const DEVICE_VARIABLE: &str = "EMUSEN_SLANG_GPU_DEVICE";

/// The system's Vulkan loader, by the name each platform's loader of libraries knows it by.
#[cfg(windows)]
const LOADER: &str = "vulkan-1.dll";
#[cfg(target_os = "macos")]
const LOADER: &str = "libvulkan.dylib";
#[cfg(all(unix, not(target_os = "macos")))]
const LOADER: &str = "libvulkan.so.1";

const LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";
static ON: u32 = 1;
static OFF: u32 = 0;

/// `VkResult` as Silk.NET.Vulkan 2.23.0 prints it, taken on 2026-10-09: a failure's words carry the name.
const RESULTS: [(i32, &str); 52] = [
    (-1000483000, "ErrorNotEnoughSpaceKhr"), (-1000338000, "ErrorCompressionExhaustedExt"), (-1000299000, "ErrorInvalidVideoStdParametersKhr"),
    (-1000298001, "ErrorNoPipelineMatch"), (-1000298000, "ErrorInvalidPipelineCacheData"), (-1000257000, "ErrorInvalidOpaqueCaptureAddress"),
    (-1000255000, "ErrorFullScreenExclusiveModeLostExt"), (-1000208000, "ErrorPresentTimingQueueFullExt"), (-1000174001, "ErrorNotPermitted"),
    (-1000161000, "ErrorFragmentationExt"), (-1000158000, "ErrorInvalidDrmFormatModifierPlaneLayoutExt"), (-1000072003, "ErrorInvalidExternalHandle"),
    (-1000069000, "ErrorOutOfPoolMemory"), (-1000023005, "ErrorVideoStdVersionNotSupportedKhr"),
    (-1000023004, "ErrorVideoProfileCodecNotSupportedKhr"), (-1000023003, "ErrorVideoProfileFormatNotSupportedKhr"),
    (-1000023002, "ErrorVideoProfileOperationNotSupportedKhr"), (-1000023001, "ErrorVideoPictureLayoutNotSupportedKhr"),
    (-1000023000, "ErrorImageUsageNotSupportedKhr"), (-1000012000, "ErrorInvalidShaderNV"), (-1000011001, "ErrorValidationFailed"),
    (-1000003001, "ErrorIncompatibleDisplayKhr"), (-1000001004, "ErrorOutOfDateKhr"), (-1000000001, "ErrorNativeWindowInUseKhr"),
    (-1000000000, "ErrorSurfaceLostKhr"), (-13, "ErrorUnknown"), (-12, "ErrorFragmentedPool"), (-11, "ErrorFormatNotSupported"),
    (-10, "ErrorTooManyObjects"), (-9, "ErrorIncompatibleDriver"), (-8, "ErrorFeatureNotPresent"), (-7, "ErrorExtensionNotPresent"),
    (-6, "ErrorLayerNotPresent"), (-5, "ErrorMemoryMapFailed"), (-4, "ErrorDeviceLost"), (-3, "ErrorInitializationFailed"),
    (-2, "ErrorOutOfDeviceMemory"), (-1, "ErrorOutOfHostMemory"), (0, "Success"), (1, "NotReady"), (2, "Timeout"), (3, "EventSet"), (4, "EventReset"),
    (5, "Incomplete"), (1000001003, "SuboptimalKhr"), (1000268000, "ThreadIdleKhr"), (1000268001, "ThreadDoneKhr"),
    (1000268002, "OperationDeferredKhr"), (1000268003, "OperationNotDeferredKhr"), (1000297000, "ErrorPipelineCompileRequiredExt"),
    (1000482000, "ErrorIncompatibleShaderBinaryExt"), (1000483000, "PipelineBinaryMissingKhr"),
];

/// A result's name as the C# prints it, or its number for one the binding does not name.
pub fn result_name(result: vk::Result) -> String {
    match RESULTS.iter().find(|(value, _)| *value == result.as_raw()) {
        Some((_, name)) => name.to_string(),
        None => result.as_raw().to_string(),
    }
}

fn words(call: &str, result: vk::Result) -> String {
    format!("{call} returned {}", result_name(result))
}

/// A Vulkan call's answer, or the C#'s words for its failure.
pub(crate) trait Checked<T> {
    fn check(self, call: &str) -> Result<T, String>;
}

impl<T> Checked<T> for VkResult<T> {
    fn check(self, call: &str) -> Result<T, String> {
        self.map_err(|result| words(call, result))
    }
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the validation layer has said and nobody has taken yet.
#[derive(Default)]
pub struct Findings {
    waiting: Mutex<Vec<String>>,
}

impl Findings {
    /// Every finding since the last taking, in the order the layer made them.
    pub fn take(&self) -> Vec<String> {
        std::mem::take(&mut *locked(&self.waiting))
    }
}

/// The layer's callback, on whichever thread made the call it speaks of: the finding waits, and the call goes on.
unsafe extern "system" fn heard(_severity: vk::DebugUtilsMessageSeverityFlagsEXT, _kinds: vk::DebugUtilsMessageTypeFlagsEXT, data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>, user: *mut c_void) -> vk::Bool32 {
    // SAFETY: `user` is the `Findings` the messenger was made with, which the instance's owner keeps while the messenger lives.
    if let (Some(findings), Some(data)) = (unsafe { user.cast::<Findings>().as_ref() }, unsafe { data.as_ref() }) {
        let message = if data.p_message.is_null() { String::new() } else { unsafe { CStr::from_ptr(data.p_message) }.to_string_lossy().into_owned() };
        locked(&findings.waiting).push(message);
    }
    vk::FALSE
}

/// The points a bench hears of, as the C# numbers them (`SlangProbe`).
pub mod point {
    pub const SUBMISSION_BEGIN: i32 = -1;
    pub const SUBMISSION_END: i32 = -2;
    pub const SUBMITTED: i32 = 1;
    pub const WAITED: i32 = 2;
    pub const ADVANCE_BEGIN: i32 = 10;
    pub const ADVANCE_END: i32 = 11;
    pub const RENDER_BEGIN: i32 = 20;
    pub const BOUND: i32 = 21;
    pub const RAN: i32 = 22;
    pub const COPIED: i32 = 23;
}

/// A moment on the CPU: `value` is nanoseconds since the probe was turned on.
pub const MARK_PHASE: u32 = 0;
/// A moment on the device: `value` is nanoseconds since its submission began.
pub const MARK_GPU: u32 = 1;

/// One thing a probe saw, in the order it saw them; a submission's device marks come before its `WAITED`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub kind: u32,
    pub point: i32,
    pub value: f64,
}

/// Timestamps for one submission at a time, read as soon as it has been waited for.
const PROBE_SLOTS: u32 = 64;

struct Probe {
    pool: vk::QueryPool,
    epoch: Instant,
    marks: Vec<Mark>,
    written: Vec<(u32, i32)>,
}

/// The loader, the instance and the layer's listener, destroyed after everything made from them.
struct Root {
    entry: ash::Entry,
    instance: ash::Instance,
    listener: Option<(ash::ext::debug_utils::Instance, vk::DebugUtilsMessengerEXT)>,
    findings: Arc<Findings>,
    // Last, so that the loader is let go of only once nothing made through it is left.
    _loader: libloading::Library,
}

impl Drop for Root {
    fn drop(&mut self) {
        unsafe {
            if let Some((loader, listener)) = &self.listener {
                loader.destroy_debug_utils_messenger(*listener, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

struct Inner {
    device: ash::Device,
    queue: vk::Queue,
    family: u32,
    pool: vk::CommandPool,
    commands: vk::CommandBuffer,
    fence: vk::Fence,
    memory: vk::PhysicalDeviceMemoryProperties,
    samplers: Mutex<[vk::Sampler; 16]>,
    /// One command buffer, and a chain may be built on one thread while another draws; true while a submission that
    /// was not waited for may still be running (EmuSen_DevicePicture.md §4.2).
    submit: Mutex<bool>,
    /// The physical device's UUID, and the loaders of the two calls that export memory and semaphores, where the
    /// device has the extensions (EmuSen_DevicePicture.md §2).
    uuid: [u8; 16],
    exports: Option<(ash::khr::external_memory_fd::Device, ash::khr::external_semaphore_fd::Device)>,
    probing: AtomicBool,
    probe: Mutex<Option<Probe>>,
    timestamp_period: f64,
    name: String,
    // Last, so that the device is destroyed before the instance it came from.
    root: Root,
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            for sampler in locked(&self.samplers).iter() {
                self.device.destroy_sampler(*sampler, None);
            }
            if let Some(probe) = locked(&self.probe).take() {
                self.device.destroy_query_pool(probe.pool, None);
            }
            self.device.destroy_fence(self.fence, None);
            self.device.destroy_command_pool(self.pool, None);
            self.device.destroy_device(None);
        }
    }
}

/// A device, shared by everything made on it.
#[derive(Clone)]
pub struct Device {
    inner: Arc<Inner>,
}

/// A buffer and its memory, mapped for its whole life.
pub struct Buffer {
    device: Device,
    pub(crate) handle: vk::Buffer,
    memory: vk::DeviceMemory,
    size: u64,
    mapped: *mut u8,
}

// SAFETY: the mapping belongs to the buffer alone, and whoever holds the buffer is the one thread that writes through it.
unsafe impl Send for Buffer {}

impl Buffer {
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Where the buffer's memory is mapped, valid for `size` bytes until the buffer is dropped.
    pub fn mapped(&self) -> *mut u8 {
        self.mapped
    }

    /// The buffer's first bytes handed to the host to write. Nothing the device is running reads them: every
    /// submission is waited for before `Device::run` returns, and one thread holds the buffer.
    pub(crate) fn with_bytes<R>(&self, length: usize, write: impl FnOnce(&mut [u8]) -> R) -> R {
        // SAFETY: the mapping holds `size` bytes and is this buffer's alone.
        write(unsafe { std::slice::from_raw_parts_mut(self.mapped, length.min(self.size as usize)) })
    }

    fn write(&self, bytes: &[u8]) {
        let length = bytes.len().min(self.size as usize);
        // SAFETY: the mapping holds `size` bytes, and no submission is running while the host writes its staging.
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.mapped, length) };
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let device = &self.device.inner.device;
        unsafe {
            device.unmap_memory(self.memory);
            device.destroy_buffer(self.handle, None);
            device.free_memory(self.memory, None);
        }
    }
}

/// A 2D image with its mip chain, a view of it all to sample and of level 0 to draw into, and the layout it was last left in.
pub struct Image {
    device: Device,
    pub(crate) handle: vk::Image,
    memory: vk::DeviceMemory,
    pub(crate) view: vk::ImageView,
    pub(crate) target: vk::ImageView,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) levels: u32,
    pub(crate) layout: Cell<vk::ImageLayout>,
}

impl Image {
    pub(crate) fn device(&self) -> &Device {
        &self.device
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        let device = &self.device.inner.device;
        unsafe {
            if self.target != self.view {
                device.destroy_image_view(self.target, None);
            }
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.handle, None);
            device.free_memory(self.memory, None);
        }
    }
}

struct Candidate {
    physical: vk::PhysicalDevice,
    family: u32,
    name: String,
    rank: u32,
    timestamp_period: f32,
}

fn color(base: u32, levels: u32) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: base, level_count: levels, base_array_layer: 0, layer_count: 1 }
}

fn level(level: u32) -> vk::ImageSubresourceLayers {
    vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: level, base_array_layer: 0, layer_count: 1 }
}

/// How many levels a mip chain of this size has: one for each halving down to a single texel.
pub fn levels_for(width: u32, height: u32) -> u32 {
    (32 - width.max(height).leading_zeros()).max(1)
}

impl Device {
    /// The device whose name holds `name_contains` whatever its case, a discrete one before an integrated one before
    /// a virtual one, or the reason there is none, in the C#'s words. With `validate`, the Khronos layer is loaded
    /// with synchronisation validation and its findings wait in `findings`.
    pub fn create(name_contains: Option<&str>, validate: bool) -> Result<Device, String> {
        let unavailable = |why: String| format!("Vulkan is not available: {why}");
        // SAFETY: the system's loader, whose one entry point has the type Vulkan declares for it; the device keeps
        // the loader for as long as any handle made through it lives.
        let loader = unsafe { libloading::Library::new(LOADER) }.map_err(|e| unavailable(e.to_string()))?;
        let entry = match unsafe { loader.get::<vk::PFN_vkGetInstanceProcAddr>(b"vkGetInstanceProcAddr\0") } {
            Ok(get_instance_proc_addr) => unsafe { ash::Entry::from_static_fn(ash::StaticFn { get_instance_proc_addr: *get_instance_proc_addr }) },
            Err(e) => return Err(unavailable(e.to_string())),
        };
        let root = Self::root(entry, loader, validate).map_err(unavailable)?;
        let instance = &root.instance;

        let mut candidates = Vec::new();
        for physical in unsafe { instance.enumerate_physical_devices() }.unwrap_or_default() {
            let properties = unsafe { instance.get_physical_device_properties(physical) };
            let queues = unsafe { instance.get_physical_device_queue_family_properties(physical) };
            let Some(family) = queues.iter().position(|q| q.queue_flags.contains(vk::QueueFlags::GRAPHICS)) else { continue };
            let name = properties.device_name_as_c_str().map_or_else(|_| "unnamed".to_string(), |n| n.to_string_lossy().into_owned());
            let rank = match properties.device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU => 0,
                vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
                _ => 3,
            };
            candidates.push(Candidate { physical, family: family as u32, name, rank, timestamp_period: properties.limits.timestamp_period });
        }

        let wanted = name_contains.unwrap_or_default();
        // The first of the lowest rank, as the C#'s stable ordering takes it.
        let Some(chosen) = candidates.iter().filter(|c| wanted.is_empty() || contains_ignore_case(&c.name, wanted)).min_by_key(|c| c.rank) else {
            return Err(if candidates.is_empty() { "no Vulkan device offers a graphics queue".to_string() } else { format!("no Vulkan device is named like \"{wanted}\"") });
        };
        let (physical, family, name, timestamp_period) = (chosen.physical, chosen.family, chosen.name.clone(), chosen.timestamp_period as f64);
        Self::open(root, physical, family, name, timestamp_period).map_err(unavailable)
    }

    /// Whether the device offers both extensions an image and its semaphores are exported with.
    fn offers_exports(instance: &ash::Instance, physical: vk::PhysicalDevice) -> bool {
        let extensions = unsafe { instance.enumerate_device_extension_properties(physical) }.unwrap_or_default();
        let has = |name: &CStr| extensions.iter().any(|e| e.extension_name_as_c_str().is_ok_and(|n| n == name));
        has(ash::khr::external_memory_fd::NAME) && has(ash::khr::external_semaphore_fd::NAME)
    }

    fn root(entry: ash::Entry, loader: libloading::Library, validate: bool) -> Result<Root, String> {
        let application = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_1);
        let mut create = vk::InstanceCreateInfo::default().application_info(&application);
        let setting = |name: &CStr, value: &'static u32| vk::LayerSettingEXT {
            p_layer_name: LAYER.as_ptr(),
            p_setting_name: name.as_ptr(),
            ty: vk::LayerSettingTypeEXT::BOOL32,
            value_count: 1,
            p_values: std::ptr::from_ref(value).cast(),
            ..Default::default()
        };
        let settings = [setting(c"validate_sync", &ON), setting(c"syncval_shader_accesses_heuristic", &ON), setting(c"enable_message_limit", &OFF)];
        let mut layer_settings = vk::LayerSettingsCreateInfoEXT::default().settings(&settings);
        let layers = [LAYER.as_ptr()];
        let extensions = [ash::ext::debug_utils::NAME.as_ptr(), ash::ext::layer_settings::NAME.as_ptr()];
        if validate {
            create = create.push_next(&mut layer_settings).enabled_layer_names(&layers).enabled_extension_names(&extensions);
        }
        let instance = unsafe { entry.create_instance(&create, None) }.check("vkCreateInstance")?;
        let findings = Arc::new(Findings::default());
        let mut root = Root { entry, instance, listener: None, findings, _loader: loader };
        if validate {
            let messenger = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
                .message_type(vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
                .pfn_user_callback(Some(heard))
                .user_data(Arc::as_ptr(&root.findings).cast_mut().cast());
            let loader = ash::ext::debug_utils::Instance::new(&root.entry, &root.instance);
            let listener = unsafe { loader.create_debug_utils_messenger(&messenger, None) }.check("vkCreateDebugUtilsMessengerEXT")?;
            root.listener = Some((loader, listener));
        }
        Ok(root)
    }

    fn open(root: Root, physical: vk::PhysicalDevice, family: u32, name: String, timestamp_period: f64) -> Result<Device, String> {
        let priority = [1.0f32];
        let queue_info = [vk::DeviceQueueCreateInfo::default().queue_family_index(family).queue_priorities(&priority)];
        let exporting = Self::offers_exports(&root.instance, physical);
        let wanted = [ash::khr::external_memory_fd::NAME.as_ptr(), ash::khr::external_semaphore_fd::NAME.as_ptr()];
        let mut device_info = vk::DeviceCreateInfo::default().queue_create_infos(&queue_info);
        if exporting {
            device_info = device_info.enabled_extension_names(&wanted);
        }
        let device = unsafe { root.instance.create_device(physical, &device_info, None) }.check("vkCreateDevice")?;
        let mut id = vk::PhysicalDeviceIDProperties::default();
        let mut properties = vk::PhysicalDeviceProperties2::default().push_next(&mut id);
        unsafe { root.instance.get_physical_device_properties2(physical, &mut properties) };
        let uuid = id.device_uuid;
        let exports = exporting.then(|| (ash::khr::external_memory_fd::Device::new(&root.instance, &device), ash::khr::external_semaphore_fd::Device::new(&root.instance, &device)));
        let made = (|| unsafe {
            let queue = device.get_device_queue(family, 0);
            let pool_info = vk::CommandPoolCreateInfo::default().queue_family_index(family).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
            let pool = device.create_command_pool(&pool_info, None).check("vkCreateCommandPool")?;
            let allocate = vk::CommandBufferAllocateInfo::default().command_pool(pool).level(vk::CommandBufferLevel::PRIMARY).command_buffer_count(1);
            let commands = match device.allocate_command_buffers(&allocate).check("vkAllocateCommandBuffers") {
                Ok(commands) => commands[0],
                Err(words) => {
                    device.destroy_command_pool(pool, None);
                    return Err(words);
                }
            };
            let fence = match device.create_fence(&vk::FenceCreateInfo::default(), None).check("vkCreateFence") {
                Ok(fence) => fence,
                Err(words) => {
                    device.destroy_command_pool(pool, None);
                    return Err(words);
                }
            };
            Ok((queue, pool, commands, fence))
        })();
        let (queue, pool, commands, fence) = match made {
            Ok(made) => made,
            Err(words) => {
                unsafe { device.destroy_device(None) };
                return Err(words);
            }
        };
        let memory = unsafe { root.instance.get_physical_device_memory_properties(physical) };
        Ok(Device {
            inner: Arc::new(Inner {
                device,
                queue,
                family,
                pool,
                commands,
                fence,
                memory,
                samplers: Mutex::new([vk::Sampler::null(); 16]),
                submit: Mutex::new(false),
                uuid,
                exports,
                probing: AtomicBool::new(false),
                probe: Mutex::new(None),
                timestamp_period,
                name,
                root,
            }),
        })
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// What the layer says of this device's calls, which outlives the device so that what its end says can be taken.
    pub fn findings(&self) -> Arc<Findings> {
        self.inner.root.findings.clone()
    }

    /// The instance's and the device's own handles, for a caller that makes a Vulkan call of its own on them. The
    /// queue is the device's: nothing else may submit to it.
    pub fn raw_instance(&self) -> u64 {
        self.inner.root.instance.handle().as_raw()
    }

    pub fn raw_device(&self) -> u64 {
        self.inner.device.handle().as_raw()
    }

    /// The physical device's UUID, which a compositor's own must equal for an exported image to be imported there.
    pub fn uuid(&self) -> [u8; 16] {
        self.inner.uuid
    }

    /// Whether images and semaphores made here can be exported as opaque file descriptors.
    pub fn exports(&self) -> bool {
        self.inner.exports.is_some()
    }

    pub(crate) fn vk(&self) -> &ash::Device {
        &self.inner.device
    }

    fn memory_type(&self, allowed: u32, need: vk::MemoryPropertyFlags, want: vk::MemoryPropertyFlags, need_name: &str) -> Result<u32, String> {
        let memory = &self.inner.memory;
        for flags in [need | want, need] {
            for i in 0..memory.memory_type_count as usize {
                if allowed & (1 << i) != 0 && memory.memory_types[i].property_flags & flags == flags {
                    return Ok(i as u32);
                }
            }
        }
        Err(format!("no memory type with {need_name}"))
    }

    pub(crate) fn create_buffer(&self, bytes: u64, usage: vk::BufferUsageFlags) -> Result<Buffer, String> {
        let device = &self.inner.device;
        let size = bytes.max(16);
        let info = vk::BufferCreateInfo::default().size(size).usage(usage).sharing_mode(vk::SharingMode::EXCLUSIVE);
        let handle = unsafe { device.create_buffer(&info, None) }.check("vkCreateBuffer")?;
        let needs = unsafe { device.get_buffer_memory_requirements(handle) };
        let made = (|| unsafe {
            let host = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
            let kind = self.memory_type(needs.memory_type_bits, host, vk::MemoryPropertyFlags::HOST_CACHED, "HostVisibleBit, HostCoherentBit")?;
            let allocate = vk::MemoryAllocateInfo::default().allocation_size(needs.size).memory_type_index(kind);
            let memory = device.allocate_memory(&allocate, None).check("vkAllocateMemory")?;
            let mapped = device.bind_buffer_memory(handle, memory, 0).check("vkBindBufferMemory").and_then(|()| device.map_memory(memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()).check("vkMapMemory"));
            match mapped {
                Ok(mapped) => Ok((memory, mapped.cast::<u8>())),
                Err(words) => {
                    device.free_memory(memory, None);
                    Err(words)
                }
            }
        })();
        match made {
            Ok((memory, mapped)) => Ok(Buffer { device: self.clone(), handle, memory, size, mapped }),
            Err(words) => {
                unsafe { device.destroy_buffer(handle, None) };
                Err(words)
            }
        }
    }

    pub(crate) fn create_image(&self, width: u32, height: u32, format: vk::Format, mipmapped: bool) -> Result<Image, String> {
        let device = &self.inner.device;
        let levels = if mipmapped { levels_for(width, height) } else { 1 };
        let info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(levels)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST);
        let handle = unsafe { device.create_image(&info, None) }.check("vkCreateImage")?;
        let needs = unsafe { device.get_image_memory_requirements(handle) };
        let made = (|| unsafe {
            let kind = self.memory_type(needs.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL, vk::MemoryPropertyFlags::empty(), "DeviceLocalBit")?;
            let allocate = vk::MemoryAllocateInfo::default().allocation_size(needs.size).memory_type_index(kind);
            let memory = device.allocate_memory(&allocate, None).check("vkAllocateMemory")?;
            let views = (|| {
                device.bind_image_memory(handle, memory, 0).check("vkBindImageMemory")?;
                let view_info = vk::ImageViewCreateInfo::default().image(handle).view_type(vk::ImageViewType::TYPE_2D).format(format).subresource_range(color(0, levels));
                let view = device.create_image_view(&view_info, None).check("vkCreateImageView")?;
                if levels == 1 {
                    return Ok((view, view));
                }
                match device.create_image_view(&view_info.subresource_range(color(0, 1)), None).check("vkCreateImageView") {
                    Ok(target) => Ok((view, target)),
                    Err(words) => {
                        device.destroy_image_view(view, None);
                        Err(words)
                    }
                }
            })();
            match views {
                Ok((view, target)) => Ok((memory, view, target)),
                Err(words) => {
                    device.free_memory(memory, None);
                    Err(words)
                }
            }
        })();
        match made {
            Ok((memory, view, target)) => Ok(Image { device: self.clone(), handle, memory, view, target, width, height, levels, layout: Cell::new(vk::ImageLayout::UNDEFINED) }),
            Err(words) => {
                unsafe { device.destroy_image(handle, None) };
                Err(words)
            }
        }
    }

    /// The sampler for a way of sampling, made once and kept for the device's life.
    pub(crate) fn sampler_for(&self, linear: bool, wrap: Wrap, mipmap: bool) -> Result<vk::Sampler, String> {
        let (mode, which) = match wrap {
            Wrap::ClampToBorder => (vk::SamplerAddressMode::CLAMP_TO_BORDER, 0),
            Wrap::ClampToEdge => (vk::SamplerAddressMode::CLAMP_TO_EDGE, 1),
            Wrap::Repeat => (vk::SamplerAddressMode::REPEAT, 2),
            Wrap::MirroredRepeat => (vk::SamplerAddressMode::MIRRORED_REPEAT, 3),
        };
        let mut samplers = locked(&self.inner.samplers);
        let kept = &mut samplers[linear as usize | which << 1 | (mipmap as usize) << 3];
        if *kept != vk::Sampler::null() {
            return Ok(*kept);
        }
        let filter = if linear { vk::Filter::LINEAR } else { vk::Filter::NEAREST };
        let info = vk::SamplerCreateInfo::default()
            .mag_filter(filter)
            .min_filter(filter)
            .mipmap_mode(if linear { vk::SamplerMipmapMode::LINEAR } else { vk::SamplerMipmapMode::NEAREST })
            .address_mode_u(mode)
            .address_mode_v(mode)
            .address_mode_w(mode)
            .min_lod(0.0)
            .max_lod(if mipmap { 1000.0 } else { 0.0 })
            .border_color(vk::BorderColor::FLOAT_TRANSPARENT_BLACK);
        *kept = unsafe { self.inner.device.create_sampler(&info, None) }.check("vkCreateSampler")?;
        Ok(*kept)
    }

    /// Waits for a submission left running by `run_unwaited`, if there is one, under the queue's lock: what the host
    /// writes next (a staging buffer, a pass's uniforms and descriptors) is then no longer read.
    pub(crate) fn settle(&self) -> Result<(), String> {
        let mut pending = locked(&self.inner.submit);
        self.settle_held(&mut pending)
    }

    fn settle_held(&self, pending: &mut bool) -> Result<(), String> {
        if !*pending {
            return Ok(());
        }
        let fence = self.inner.fence;
        unsafe {
            self.inner.device.wait_for_fences(&[fence], true, u64::MAX).check("vkWaitForFences")?;
            self.inner.device.reset_fences(&[fence]).check("vkResetFences")?;
        }
        *pending = false;
        self.read_marks();
        self.phase(point::WAITED);
        Ok(())
    }

    /// Records and submits, waiting on the device for `wait` and signalling `signal`, and returns at once; the next
    /// call that submits or settles waits for it (EmuSen_DevicePicture.md §4.2).
    pub(crate) fn run_unwaited(&self, record: impl FnOnce(vk::CommandBuffer), wait: Option<vk::Semaphore>, signal: vk::Semaphore) -> Result<(), String> {
        let mut pending = locked(&self.inner.submit);
        self.settle_held(&mut pending)?;
        let (device, commands, fence) = (&self.inner.device, self.inner.commands, self.inner.fence);
        unsafe {
            device.reset_command_buffer(commands, vk::CommandBufferResetFlags::empty()).check("vkResetCommandBuffer")?;
            let begin = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device.begin_command_buffer(commands, &begin).check("vkBeginCommandBuffer")?;
            self.mark(commands, point::SUBMISSION_BEGIN);
            record(commands);
            self.mark(commands, point::SUBMISSION_END);
            device.end_command_buffer(commands).check("vkEndCommandBuffer")?;
            let buffers = [commands];
            let waits: Vec<vk::Semaphore> = wait.into_iter().collect();
            let stages = [vk::PipelineStageFlags::TRANSFER];
            let signals = [signal];
            let submit = [vk::SubmitInfo::default().command_buffers(&buffers).wait_semaphores(&waits).wait_dst_stage_mask(&stages[..waits.len()]).signal_semaphores(&signals)];
            device.queue_submit(self.inner.queue, &submit, fence).check("vkQueueSubmit")?;
            self.phase(point::SUBMITTED);
        }
        *pending = true;
        Ok(())
    }

    /// Records, submits and waits; nothing recorded here is still running when it returns.
    pub(crate) fn run(&self, record: impl FnOnce(vk::CommandBuffer)) -> Result<(), String> {
        let mut pending = locked(&self.inner.submit);
        self.settle_held(&mut pending)?;
        let (device, commands, fence) = (&self.inner.device, self.inner.commands, self.inner.fence);
        unsafe {
            device.reset_command_buffer(commands, vk::CommandBufferResetFlags::empty()).check("vkResetCommandBuffer")?;
            let begin = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device.begin_command_buffer(commands, &begin).check("vkBeginCommandBuffer")?;
            self.mark(commands, point::SUBMISSION_BEGIN);
            record(commands);
            self.mark(commands, point::SUBMISSION_END);
            device.end_command_buffer(commands).check("vkEndCommandBuffer")?;
            let buffers = [commands];
            let submit = [vk::SubmitInfo::default().command_buffers(&buffers)];
            device.queue_submit(self.inner.queue, &submit, fence).check("vkQueueSubmit")?;
            self.phase(point::SUBMITTED);
            device.wait_for_fences(&[fence], true, u64::MAX).check("vkWaitForFences")?;
            device.reset_fences(&[fence]).check("vkResetFences")?;
            self.read_marks();
            self.phase(point::WAITED);
        }
        Ok(())
    }

    /// Waits for everything the device has been given, with the queue held as Vulkan asks.
    pub(crate) fn wait_idle(&self) {
        let mut pending = locked(&self.inner.submit);
        let _ = self.settle_held(&mut pending);
        let _ = unsafe { self.inner.device.device_wait_idle() };
    }

    /// Levels of an image moved to a layout, with the widest barrier, since a chain's passes run one after another
    /// anyway: every level from `base` when `levels` is none, from the image's own layout when `from` is none.
    pub(crate) fn transition(&self, commands: vk::CommandBuffer, image: &Image, to: vk::ImageLayout, base: u32, levels: Option<u32>, from: Option<vk::ImageLayout>) {
        let count = levels.unwrap_or(image.levels.wrapping_sub(base));
        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(from.unwrap_or(image.layout.get()))
            .new_layout(to)
            .image(image.handle)
            .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(color(base, count));
        unsafe { self.inner.device.cmd_pipeline_barrier(commands, vk::PipelineStageFlags::ALL_COMMANDS, vk::PipelineStageFlags::ALL_COMMANDS, vk::DependencyFlags::empty(), &[], &[], &[barrier]) };
        if base == 0 && count == image.levels {
            image.layout.set(to);
        }
    }

    /// Level 0 blitted down the chain, every level left ready to sample.
    pub(crate) fn generate_mipmaps(&self, commands: vk::CommandBuffer, image: &Image) {
        const READ: vk::ImageLayout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        const SOURCE: vk::ImageLayout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        const DESTINATION: vk::ImageLayout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        if image.levels <= 1 {
            self.transition(commands, image, READ, 0, None, None);
            return;
        }
        self.transition(commands, image, DESTINATION, 0, None, None);
        let (mut w, mut h) = (image.width as i32, image.height as i32);
        for to in 1..image.levels {
            self.transition(commands, image, SOURCE, to - 1, Some(1), Some(DESTINATION));
            let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
            let blit = vk::ImageBlit {
                src_subresource: level(to - 1),
                src_offsets: [vk::Offset3D::default(), vk::Offset3D { x: w, y: h, z: 1 }],
                dst_subresource: level(to),
                dst_offsets: [vk::Offset3D::default(), vk::Offset3D { x: nw, y: nh, z: 1 }],
            };
            unsafe { self.inner.device.cmd_blit_image(commands, image.handle, SOURCE, image.handle, DESTINATION, &[blit], vk::Filter::LINEAR) };
            self.transition(commands, image, READ, to - 1, Some(1), Some(SOURCE));
            (w, h) = (nw, nh);
        }
        self.transition(commands, image, READ, image.levels - 1, Some(1), Some(DESTINATION));
        image.layout.set(READ);
    }

    fn copy_in(&self, commands: vk::CommandBuffer, staging: &Buffer, image: &Image) {
        let region = vk::BufferImageCopy::default().image_subresource(level(0)).image_extent(vk::Extent3D { width: image.width, height: image.height, depth: 1 });
        unsafe { self.inner.device.cmd_copy_buffer_to_image(commands, staging.handle, image.handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]) };
    }

    fn finish(&self, commands: vk::CommandBuffer, image: &Image) {
        if image.levels > 1 {
            self.generate_mipmaps(commands, image);
        } else {
            self.transition(commands, image, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, 0, None, None);
        }
    }

    /// Pixels in RGBA8 from the host into level 0, and the chain built if the image has one.
    pub(crate) fn upload(&self, image: &Image, rgba: &[u8], staging: &Buffer) -> Result<(), String> {
        self.settle()?;
        staging.write(rgba);
        self.run(|commands| self.record_upload(commands, image, staging))
    }

    /// The commands of an upload whose pixels are already in `staging`, for a submission that carries more.
    pub(crate) fn record_upload(&self, commands: vk::CommandBuffer, image: &Image, staging: &Buffer) {
        self.transition(commands, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, 0, None, None);
        self.copy_in(commands, staging, image);
        self.finish(commands, image);
    }

    /// Each row uploaded once into `rows`, then blitted to `repeat` times its height by nearest filtering, which
    /// repeats each row exactly (EmuSen_Serenity.md §9.2): the commands, the pixels already in `staging`.
    pub(crate) fn record_upload_repeated(&self, commands: vk::CommandBuffer, image: &Image, rows: &Image, repeat: u32, staging: &Buffer) {
        self.transition(commands, rows, vk::ImageLayout::TRANSFER_DST_OPTIMAL, 0, None, Some(vk::ImageLayout::UNDEFINED));
        self.copy_in(commands, staging, rows);
        self.transition(commands, rows, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, 0, None, None);
        self.transition(commands, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, 0, None, None);
        let blit = vk::ImageBlit {
            src_subresource: level(0),
            src_offsets: [vk::Offset3D::default(), vk::Offset3D { x: rows.width as i32, y: rows.height as i32, z: 1 }],
            dst_subresource: level(0),
            dst_offsets: [vk::Offset3D::default(), vk::Offset3D { x: rows.width as i32, y: rows.height.wrapping_mul(repeat) as i32, z: 1 }],
        };
        unsafe { self.inner.device.cmd_blit_image(commands, rows.handle, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, image.handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[blit], vk::Filter::NEAREST) };
        self.finish(commands, image);
    }

    /// Bytes written into a host buffer that no submission still running reads, the caller having settled.
    pub(crate) fn write_staging(&self, staging: &Buffer, bytes: &[u8]) {
        staging.write(bytes);
    }

    /// A transfer's writes to a buffer made visible to the host, which a fence alone does not do.
    pub(crate) fn host_read_barrier(&self, commands: vk::CommandBuffer, buffer: &Buffer) {
        let barrier = vk::BufferMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::HOST_READ)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .buffer(buffer.handle)
            .offset(0)
            .size(vk::WHOLE_SIZE);
        unsafe { self.inner.device.cmd_pipeline_barrier(commands, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::HOST, vk::DependencyFlags::empty(), &[], &[barrier], &[]) };
    }

    /// Turns the probe on, with its clock started now, or off. While it is on, every phase and every submission's
    /// timestamps wait to be taken, which is what a bench reads a frame's cost from (EmuSen_Serenity.md §8.1).
    pub fn probe(&self, on: bool) -> Result<(), String> {
        // No submission is out while the pool it writes to is made or destroyed.
        let mut pending = locked(&self.inner.submit);
        self.settle_held(&mut pending)?;
        let mut probe = locked(&self.inner.probe);
        if let Some(old) = probe.take() {
            unsafe { self.inner.device.destroy_query_pool(old.pool, None) };
        }
        self.inner.probing.store(false, Ordering::Release);
        if on {
            let info = vk::QueryPoolCreateInfo::default().query_type(vk::QueryType::TIMESTAMP).query_count(PROBE_SLOTS);
            let pool = unsafe { self.inner.device.create_query_pool(&info, None) }.check("vkCreateQueryPool")?;
            *probe = Some(Probe { pool, epoch: Instant::now(), marks: Vec::new(), written: Vec::new() });
            self.inner.probing.store(true, Ordering::Release);
        }
        Ok(())
    }

    /// Everything the probe saw since the last taking.
    pub fn take_marks(&self) -> Vec<Mark> {
        locked(&self.inner.probe).as_mut().map(|probe| std::mem::take(&mut probe.marks)).unwrap_or_default()
    }

    /// A moment on the CPU, kept when a probe is listening.
    pub(crate) fn phase(&self, point: i32) {
        if !self.inner.probing.load(Ordering::Acquire) {
            return;
        }
        if let Some(probe) = locked(&self.inner.probe).as_mut() {
            let value = probe.epoch.elapsed().as_nanos() as f64;
            probe.marks.push(Mark { kind: MARK_PHASE, point, value });
        }
    }

    /// A timestamp written into the submission being recorded, when a probe is listening: the submission's two ends,
    /// and the end of each pass that fits between them.
    pub(crate) fn mark(&self, commands: vk::CommandBuffer, point: i32) {
        if !self.inner.probing.load(Ordering::Acquire) {
            return;
        }
        let mut probe = locked(&self.inner.probe);
        let Some(probe) = probe.as_mut() else { return };
        let device = &self.inner.device;
        if point == point::SUBMISSION_BEGIN {
            unsafe {
                device.cmd_reset_query_pool(commands, probe.pool, 0, PROBE_SLOTS);
                device.cmd_write_timestamp(commands, vk::PipelineStageFlags::TOP_OF_PIPE, probe.pool, 0);
            }
            probe.written.push((0, point));
            return;
        }
        if point >= PROBE_SLOTS as i32 - 2 {
            return;
        }
        let slot = if point == point::SUBMISSION_END { PROBE_SLOTS - 1 } else { point as u32 + 1 };
        unsafe { device.cmd_write_timestamp(commands, vk::PipelineStageFlags::BOTTOM_OF_PIPE, probe.pool, slot) };
        probe.written.push((slot, point));
    }

    fn read_marks(&self) {
        if !self.inner.probing.load(Ordering::Acquire) {
            return;
        }
        let mut probe = locked(&self.inner.probe);
        let Some(probe) = probe.as_mut() else { return };
        let mut values = Vec::with_capacity(probe.written.len());
        let mut begin = 0u64;
        for (slot, point) in probe.written.drain(..) {
            let mut value = [0u64];
            let _ = unsafe { self.inner.device.get_query_pool_results(probe.pool, slot, &mut value, vk::QueryResultFlags::TYPE_64) };
            if point == point::SUBMISSION_BEGIN {
                begin = value[0];
            }
            values.push((point, value[0]));
        }
        for (point, value) in values {
            probe.marks.push(Mark { kind: MARK_GPU, point, value: value.wrapping_sub(begin) as f64 * self.inner.timestamp_period });
        }
    }
}

/// An image a picture is presented in, exported to a compositor with the two semaphores that order it there: `ready`,
/// signalled when a picture has been copied in, and `free`, signalled by the compositor when it has taken its copy
/// (EmuSen_DevicePicture.md §4.2).
pub struct Shown {
    pub(crate) image: Image,
    pub(crate) ready: vk::Semaphore,
    pub(crate) free: vk::Semaphore,
    pub memory_bytes: u64,
}

impl Drop for Shown {
    fn drop(&mut self) {
        let device = self.image.device();
        device.wait_idle();
        unsafe {
            device.vk().destroy_semaphore(self.ready, None);
            device.vk().destroy_semaphore(self.free, None);
        }
    }
}

/// A target's memory and semaphores as file descriptors, each the caller's to hand on or close.
pub struct Exported {
    pub memory: i32,
    pub ready: i32,
    pub free: i32,
}

impl Device {
    /// A target `width` by `height`, RGBA8, its memory not dedicated, as Avalonia's GL import takes it, laid out
    /// linearly or optimally (which one a compositor reads as written is found by trying, EmuSen_DevicePicture.md
    /// §13), and its descriptors; none on a device that cannot export.
    pub fn create_target(&self, width: u32, height: u32, linear: bool) -> Result<(Shown, Exported), String> {
        let Some((memory_fd, semaphore_fd)) = &self.inner.exports else { return Err("the device cannot export images".to_string()) };
        let device = &self.inner.device;
        let format = vk::Format::R8G8B8A8_UNORM;
        let mut external = vk::ExternalMemoryImageCreateInfo::default().handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
        let info = vk::ImageCreateInfo::default()
            .push_next(&mut external)
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(if linear { vk::ImageTiling::LINEAR } else { vk::ImageTiling::OPTIMAL })
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC);
        let handle = unsafe { device.create_image(&info, None) }.check("vkCreateImage")?;
        let needs = unsafe { device.get_image_memory_requirements(handle) };
        let made = (|| unsafe {
            let kind = self.memory_type(needs.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL, vk::MemoryPropertyFlags::empty(), "DeviceLocalBit")?;
            let mut export = vk::ExportMemoryAllocateInfo::default().handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
            let allocate = vk::MemoryAllocateInfo::default().push_next(&mut export).allocation_size(needs.size).memory_type_index(kind);
            let memory = device.allocate_memory(&allocate, None).check("vkAllocateMemory")?;
            // Only copied into and out of, so the image has no view.
            match device.bind_image_memory(handle, memory, 0).check("vkBindImageMemory") {
                Ok(()) => Ok((memory, vk::ImageView::null())),
                Err(words) => {
                    device.free_memory(memory, None);
                    Err(words)
                }
            }
        })();
        let (memory, view) = match made {
            Ok(made) => made,
            Err(words) => {
                unsafe { device.destroy_image(handle, None) };
                return Err(words);
            }
        };
        let image = Image { device: self.clone(), handle, memory, view, target: view, width, height, levels: 1, layout: Cell::new(vk::ImageLayout::UNDEFINED) };
        let semaphore = || {
            let mut export = vk::ExportSemaphoreCreateInfo::default().handle_types(vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_FD);
            unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default().push_next(&mut export), None) }.check("vkCreateSemaphore")
        };
        let ready = semaphore()?;
        let free = match semaphore() {
            Ok(free) => free,
            Err(words) => {
                unsafe { device.destroy_semaphore(ready, None) };
                return Err(words);
            }
        };
        let target = Shown { image, ready, free, memory_bytes: needs.size };
        let memory_out = unsafe { memory_fd.get_memory_fd(&vk::MemoryGetFdInfoKHR::default().memory(target.image.memory).handle_type(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD)) }.check("vkGetMemoryFdKHR")?;
        let fd = |semaphore: vk::Semaphore| unsafe { semaphore_fd.get_semaphore_fd(&vk::SemaphoreGetFdInfoKHR::default().semaphore(semaphore).handle_type(vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_FD)) }.check("vkGetSemaphoreFdKHR");
        let ready_out = match fd(target.ready) {
            Ok(fd) => fd,
            Err(words) => {
                unsafe { libc_close(memory_out) };
                return Err(words);
            }
        };
        let free_out = match fd(target.free) {
            Ok(fd) => fd,
            Err(words) => {
                unsafe {
                    libc_close(memory_out);
                    libc_close(ready_out);
                }
                return Err(words);
            }
        };
        Ok((target, Exported { memory: memory_out, ready: ready_out, free: free_out }))
    }

    /// An image's ownership given to or taken back from the compositor, with its layout: taken back with its
    /// contents discarded, given with the layout the compositor copies from.
    pub(crate) fn external(&self, commands: vk::CommandBuffer, image: &Image, give: bool) {
        let (from, to, old, new) = if give {
            (self.inner.family, vk::QUEUE_FAMILY_EXTERNAL, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        } else {
            (vk::QUEUE_FAMILY_EXTERNAL, self.inner.family, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        };
        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(old)
            .new_layout(new)
            .image(image.handle)
            .src_access_mask(if give { vk::AccessFlags::TRANSFER_WRITE } else { vk::AccessFlags::empty() })
            .dst_access_mask(if give { vk::AccessFlags::empty() } else { vk::AccessFlags::TRANSFER_WRITE })
            .src_queue_family_index(from)
            .dst_queue_family_index(to)
            .subresource_range(color(0, 1));
        let (source, destination) = if give { (vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::BOTTOM_OF_PIPE) } else { (vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER) };
        unsafe { self.inner.device.cmd_pipeline_barrier(commands, source, destination, vk::DependencyFlags::empty(), &[], &[], &[barrier]) };
        image.layout.set(new);
    }
}

/// Closes a descriptor the library made and could not hand over.
unsafe fn libc_close(fd: i32) {
    unsafe extern "C" {
        fn close(fd: i32) -> i32;
    }
    unsafe { close(fd) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_result_is_named_as_the_csharp_binding_names_it() {
        assert_eq!(result_name(vk::Result::ERROR_OUT_OF_DEVICE_MEMORY), "ErrorOutOfDeviceMemory");
        assert_eq!(result_name(vk::Result::ERROR_LAYER_NOT_PRESENT), "ErrorLayerNotPresent");
        assert_eq!(result_name(vk::Result::ERROR_INITIALIZATION_FAILED), "ErrorInitializationFailed");
        assert_eq!(result_name(vk::Result::TIMEOUT), "Timeout");
        assert_eq!(result_name(vk::Result::from_raw(-123456)), "-123456");
        assert_eq!(result_name(vk::Result::from_raw(77)), "77");
        assert_eq!(words("vkCreateImage", vk::Result::ERROR_FORMAT_NOT_SUPPORTED), "vkCreateImage returned ErrorFormatNotSupported");
        assert!(RESULTS.windows(2).all(|pair| pair[0].0 < pair[1].0));
    }

    #[test]
    fn a_mip_chain_has_a_level_for_each_halving() {
        for (size, levels) in [(0, 1), (1, 1), (2, 2), (3, 2), (4, 3), (7, 3), (8, 4), (255, 8), (256, 9), (257, 9), (65535, 16), (65536, 17), (2147483647, 31), (2147483648, 32), (u32::MAX, 32)] {
            assert_eq!(levels_for(size, 0), levels, "{size}");
            assert_eq!(levels_for(1, size), levels, "{size}");
        }
    }

    #[test]
    fn a_mark_is_sixteen_bytes_for_the_header_s_struct() {
        assert_eq!(std::mem::size_of::<Mark>(), 16);
        assert_eq!(std::mem::align_of::<Mark>(), 8);
    }
}
