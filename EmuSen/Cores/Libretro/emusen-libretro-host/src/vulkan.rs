//! A Vulkan core's device, made through its negotiation interface with the external-memory and timeline extensions
//! added; its frames copied on the device into the host's ring, or read back where no ring is lent. See
//! EmuSen_LibretroHost.md §5.3 to §5.5 and §10f.

use std::ffi::{CStr, c_char, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

use ash::vk;
use emusen_libretro_sys::vulkan::*;
use emusen_native::core::{Ring, RingFrame, status};

/// The frames in flight the interface offers: `get_sync_index` counts 0 and 1.
const IN_FLIGHT: usize = 2;
const SLOT_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
const WANTED: [&CStr; 3] = [ash::khr::external_memory_fd::NAME, ash::khr::external_semaphore_fd::NAME, ash::khr::timeline_semaphore::NAME];

/// What the core asked for at load: its context's callbacks and the Vulkan version, and its negotiation interface.
#[derive(Clone, Copy)]
pub struct Ask {
    pub reset: emusen_libretro_sys::retro_hw_context_reset_t,
    pub destroy: emusen_libretro_sys::retro_hw_context_reset_t,
    pub version: (u32, u32),
}

/// The image `set_image` handed for the coming frame.
#[derive(Clone, Copy)]
struct Handed {
    image: vk::Image,
    layout: vk::ImageLayout,
    format: vk::Format,
}

/// One imported image of the host's ring, and the `done` value its last picture must reach before it is written again.
struct Slot {
    image: vk::Image,
    memory: vk::DeviceMemory,
    width: u32,
    height: u32,
    used: bool,
    released_at: u64,
}

struct Imported {
    slots: Vec<Slot>,
    ready: vk::Semaphore,
    done: vk::Semaphore,
    /// Pictures handed on this ring, so the next one's `ready` value is `count + 1`.
    count: u64,
}

/// A frame larger than the ring's images, kept on the device until a ring that holds it is lent (§5.4).
struct Held {
    image: vk::Image,
    memory: vk::DeviceMemory,
    width: u32,
    height: u32,
    pending: bool,
}

/// A host-visible buffer the picture is read back into where no ring is lent (§5.5).
struct Readback {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    bytes: u64,
    mapped: *mut u8,
}

/// Where one copy goes.
#[derive(Clone, Copy)]
enum Target {
    Slot(usize),
    Held,
    Buffer,
}

/// What a frame left for the host to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    /// The picture is in the ring, as the record says.
    Ring,
    /// The picture was read back, `read` hands it: its size and whether it is BGRA.
    Bytes(u32, u32, bool),
    /// The picture is larger than the ring's images: held, and read back as `Bytes` is.
    Larger(u32, u32, bool),
    Nothing,
}

/// The objects a create builds up, reached by the wrappers' opaque pointer.
struct Building {
    entry: ash::Entry,
    instance: Option<ash::Instance>,
    device: Option<ash::Device>,
    api: u32,
    timeline: bool,
    why: String,
}

/// The core's device and everything the host made on it.
pub struct Gpu {
    entry: ash::Entry,
    instance: ash::Instance,
    physical: vk::PhysicalDevice,
    device: ash::Device,
    queue: vk::Queue,
    family: u32,
    memory: vk::PhysicalDeviceMemoryProperties,
    exports: Option<(ash::khr::external_memory_fd::Device, ash::khr::external_semaphore_fd::Device, ash::khr::timeline_semaphore::Device)>,
    uuids: ([u8; 16], [u8; 16]),
    interface: retro_hw_render_interface_vulkan,
    lock: AtomicBool,
    pool: vk::CommandPool,
    commands: [vk::CommandBuffer; IN_FLIGHT],
    fences: [vk::Fence; IN_FLIGHT],
    submitted: [bool; IN_FLIGHT],
    index: usize,
    handed: Option<Handed>,
    waits: Vec<vk::Semaphore>,
    core_commands: Vec<vk::CommandBuffer>,
    signal: vk::Semaphore,
    ring: Option<Imported>,
    record: Option<RingFrame>,
    /// The last frame put in the ring, not yet read back: the core's image, which stays its until the next frame.
    unread: Option<(Handed, u32, u32)>,
    held: Option<Held>,
    readback: Option<Readback>,
    swizzled: Vec<u8>,
    pub log: Vec<String>,
}

fn check<T>(r: Result<T, vk::Result>, what: &str) -> Result<T, String> {
    r.map_err(|e| format!("{what} returned {e}"))
}

unsafe extern "C" fn create_instance_wrapper(opaque: *mut c_void, info: *const vk::InstanceCreateInfo<'_>) -> vk::Instance {
    let b = unsafe { &mut *(opaque as *mut Building) };
    let Some(given) = (unsafe { info.as_ref() }) else { return vk::Instance::null() };
    let mut info = *given;
    // 1.1 at least, where the external-memory and semaphore capabilities are core; a newer version asked is kept.
    let mut app = unsafe { given.p_application_info.as_ref() }.copied().unwrap_or_default();
    app.api_version = app.api_version.max(b.api);
    info.p_application_info = &app;
    match unsafe { b.entry.create_instance(&info, None) } {
        Ok(i) => {
            let h = i.handle();
            b.instance = Some(i);
            h
        }
        Err(e) => {
            b.why = format!("vkCreateInstance returned {e}");
            vk::Instance::null()
        }
    }
}

unsafe extern "C" fn create_device_wrapper(gpu: vk::PhysicalDevice, opaque: *mut c_void, info: *const vk::DeviceCreateInfo<'_>) -> vk::Device {
    let b = unsafe { &mut *(opaque as *mut Building) };
    let (Some(instance), Some(given)) = (b.instance.as_ref(), unsafe { info.as_ref() }) else { return vk::Device::null() };
    let mut info = *given;
    let offered = unsafe { instance.enumerate_device_extension_properties(gpu) }.unwrap_or_default();
    let has = |n: &CStr| offered.iter().any(|e| e.extension_name_as_c_str().is_ok_and(|x| x == n));
    let mut names: Vec<*const c_char> = if given.enabled_extension_count == 0 { Vec::new() } else { unsafe { std::slice::from_raw_parts(given.pp_enabled_extension_names, given.enabled_extension_count as usize) }.to_vec() };
    let named = |names: &[*const c_char], n: &CStr| names.iter().any(|&p| !p.is_null() && unsafe { CStr::from_ptr(p) } == n);
    b.timeline = WANTED.iter().all(|&n| has(n));
    if b.timeline {
        for n in WANTED {
            if !named(&names, n) {
                names.push(n.as_ptr());
            }
        }
    }
    // Timeline semaphores turned on where the core's chain already has their member, else by a struct put in front;
    // nothing the core asked for is changed (§5.3).
    let mut timeline = vk::PhysicalDeviceTimelineSemaphoreFeatures::default().timeline_semaphore(true);
    if b.timeline {
        let mut at = given.p_next as *mut vk::BaseOutStructure<'_>;
        let mut found = false;
        while let Some(s) = unsafe { at.as_mut() } {
            match s.s_type {
                vk::StructureType::PHYSICAL_DEVICE_VULKAN_1_2_FEATURES => {
                    unsafe { (*(at as *mut vk::PhysicalDeviceVulkan12Features<'_>)).timeline_semaphore = vk::TRUE };
                    found = true;
                }
                vk::StructureType::PHYSICAL_DEVICE_TIMELINE_SEMAPHORE_FEATURES => {
                    unsafe { (*(at as *mut vk::PhysicalDeviceTimelineSemaphoreFeatures<'_>)).timeline_semaphore = vk::TRUE };
                    found = true;
                }
                _ => {}
            }
            at = s.p_next;
        }
        if !found {
            timeline.p_next = given.p_next as *mut c_void;
            info.p_next = &timeline as *const _ as *const c_void;
        }
    }
    info.enabled_extension_count = names.len() as u32;
    info.pp_enabled_extension_names = names.as_ptr();
    match unsafe { instance.create_device(gpu, &info, None) } {
        Ok(d) => {
            let h = d.handle();
            b.device = Some(d);
            h
        }
        Err(e) => {
            b.why = format!("vkCreateDevice returned {e}");
            vk::Device::null()
        }
    }
}

// ---- The interface the core calls ------------------------------------------------------------------------------

unsafe fn gpu<'a>(handle: *mut c_void) -> &'a mut Gpu {
    unsafe { &mut *(handle as *mut Gpu) }
}

unsafe extern "C" fn set_image(handle: *mut c_void, image: *const retro_vulkan_image, count: u32, semaphores: *const vk::Semaphore, src_queue_family: u32) {
    let g = unsafe { gpu(handle) };
    g.handed = unsafe { image.as_ref() }.map(|i| Handed { image: i.create_info.image, layout: i.image_layout, format: i.create_info.format });
    g.waits.clear();
    if count > 0 && !semaphores.is_null() {
        g.waits.extend_from_slice(unsafe { std::slice::from_raw_parts(semaphores, count as usize) });
    }
    if src_queue_family != vk::QUEUE_FAMILY_IGNORED && src_queue_family != g.family && g.log.len() < 8 {
        g.log.push(format!("the core's image came from queue family {src_queue_family}, not the one it was given"));
    }
}

unsafe extern "C" fn get_sync_index(handle: *mut c_void) -> u32 {
    unsafe { gpu(handle) }.index as u32
}

unsafe extern "C" fn get_sync_index_mask(_handle: *mut c_void) -> u32 {
    (1 << IN_FLIGHT) - 1
}

unsafe extern "C" fn set_command_buffers(handle: *mut c_void, count: u32, cmd: *const vk::CommandBuffer) {
    let g = unsafe { gpu(handle) };
    g.core_commands.clear();
    if count > 0 && !cmd.is_null() {
        g.core_commands.extend_from_slice(unsafe { std::slice::from_raw_parts(cmd, count as usize) });
    }
}

unsafe extern "C" fn wait_sync_index(handle: *mut c_void) {
    let g = unsafe { gpu(handle) };
    g.wait_slot(g.index);
}

unsafe extern "C" fn lock_queue(handle: *mut c_void) {
    let g = unsafe { gpu(handle) };
    while g.lock.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
        std::thread::yield_now();
    }
}

unsafe extern "C" fn unlock_queue(handle: *mut c_void) {
    unsafe { gpu(handle) }.lock.store(false, Ordering::Release);
}

unsafe extern "C" fn set_signal_semaphore(handle: *mut c_void, semaphore: vk::Semaphore) {
    unsafe { gpu(handle) }.signal = semaphore;
}

fn subresource() -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 }
}

fn layers() -> vk::ImageSubresourceLayers {
    vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 }
}

fn barrier(image: vk::Image, old: vk::ImageLayout, new: vk::ImageLayout, src: vk::AccessFlags, dst: vk::AccessFlags, from: u32, to: u32) -> vk::ImageMemoryBarrier<'static> {
    vk::ImageMemoryBarrier::default().image(image).old_layout(old).new_layout(new).src_access_mask(src).dst_access_mask(dst).src_queue_family_index(from).dst_queue_family_index(to).subresource_range(subresource())
}

/// The bytes of a pixel of a format the read-back takes, and whether it is BGRA; `None` for any other.
fn readable(format: vk::Format) -> Option<bool> {
    match format {
        vk::Format::R8G8B8A8_UNORM | vk::Format::R8G8B8A8_SRGB => Some(false),
        vk::Format::B8G8R8A8_UNORM | vk::Format::B8G8R8A8_SRGB => Some(true),
        _ => None,
    }
}

impl Gpu {
    /// The core's device: through its `create_instance` and `create_device2` where it offers them, through v1's
    /// `create_device`, or made here with one graphics-and-compute queue; then the interface it will be handed.
    pub fn create(ask: &Ask, negotiation: Option<&retro_hw_render_context_negotiation_interface_vulkan>) -> Result<Box<Gpu>, String> {
        // SAFETY: the loader is loaded once per device and outlives every handle made through it.
        let entry = unsafe { ash::Entry::load() }.map_err(|e| format!("Vulkan is not available: {e}"))?;
        let asked = vk::make_api_version(0, ask.version.0, ask.version.1, 0);
        let mut b = Box::new(Building { entry: entry.clone(), instance: None, device: None, api: asked.max(vk::API_VERSION_1_1), timeline: false, why: String::new() });
        let opaque = &mut *b as *mut Building as *mut c_void;
        let v2 = negotiation.filter(|n| n.interface_type == INTERFACE_VULKAN && n.interface_version >= 2);
        let gipa = entry.static_fn().get_instance_proc_addr;
        let app_of_core = negotiation.and_then(|n| n.get_application_info).map(|f| unsafe { f() }).filter(|p| !p.is_null());
        match v2.and_then(|n| n.create_instance) {
            Some(create) => {
                let made = unsafe { create(gipa, app_of_core.unwrap_or(std::ptr::null()), create_instance_wrapper, opaque) };
                if made == vk::Instance::null() || b.instance.is_none() {
                    return Err(format!("the core's create_instance failed: {}", b.why));
                }
            }
            None => {
                let app = vk::ApplicationInfo::default().api_version(b.api);
                let info = vk::InstanceCreateInfo::default().application_info(unsafe { app_of_core.and_then(|p| p.as_ref()) }.unwrap_or(&app));
                if unsafe { create_instance_wrapper(opaque, &info) } == vk::Instance::null() {
                    return Err(b.why.clone());
                }
            }
        }
        let instance = b.instance.clone().unwrap();
        let destroy_instance = |why: String| {
            unsafe { instance.destroy_instance(None) };
            Err(why)
        };
        let Some(chosen) = emusen_gpu::choose_device(&instance) else { return destroy_instance("no Vulkan device offers a compute queue".into()) };
        let mut context = retro_vulkan_context::default();
        let made = if let Some(create2) = v2.and_then(|n| n.create_device2) {
            unsafe { create2(&mut context, instance.handle(), chosen, vk::SurfaceKHR::null(), gipa, create_device_wrapper, opaque) }
        } else if let Some(create1) = negotiation.and_then(|n| n.create_device) {
            let names: Vec<*const c_char> = WANTED.iter().map(|n| n.as_ptr()).collect();
            let ok = unsafe { create1(&mut context, instance.handle(), chosen, vk::SurfaceKHR::null(), gipa, names.as_ptr(), names.len() as u32, std::ptr::null(), 0, std::ptr::null()) };
            if ok {
                b.device = Some(unsafe { ash::Device::load(instance.fp_v1_0(), context.device) });
            }
            ok
        } else {
            let queues = unsafe { instance.get_physical_device_queue_family_properties(chosen) };
            let Some(family) = queues.iter().position(|q| q.queue_flags.contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)) else { return destroy_instance("the device has no graphics-and-compute queue".into()) };
            let priority = [1.0f32];
            let queue = [vk::DeviceQueueCreateInfo::default().queue_family_index(family as u32).queue_priorities(&priority)];
            let info = vk::DeviceCreateInfo::default().queue_create_infos(&queue);
            let made = unsafe { create_device_wrapper(chosen, opaque, &info) } != vk::Device::null();
            if made {
                let d = b.device.as_ref().unwrap();
                context = retro_vulkan_context { gpu: chosen, device: d.handle(), queue: unsafe { d.get_device_queue(family as u32, 0) }, queue_family_index: family as u32, ..Default::default() };
            }
            made
        };
        let device = match (made, b.device.clone()) {
            (true, Some(d)) if d.handle() == context.device => d,
            (true, _) if context.device != vk::Device::null() => unsafe { ash::Device::load(instance.fp_v1_0(), context.device) },
            _ => {
                if let Some(d) = b.device.take() {
                    unsafe { d.destroy_device(None) };
                }
                return destroy_instance(format!("the core's device could not be made: {}", if b.why.is_empty() { "the core refused" } else { &b.why }));
            }
        };
        let physical = if context.gpu == vk::PhysicalDevice::null() { chosen } else { context.gpu };
        let mut id = vk::PhysicalDeviceIDProperties::default();
        let mut props = vk::PhysicalDeviceProperties2::default().push_next(&mut id);
        unsafe { instance.get_physical_device_properties2(physical, &mut props) };
        let uuids = (id.device_uuid, id.driver_uuid);
        let exports = b.timeline.then(|| {
            (ash::khr::external_memory_fd::Device::new(&instance, &device), ash::khr::external_semaphore_fd::Device::new(&instance, &device), ash::khr::timeline_semaphore::Device::new(&instance, &device))
        });
        let family = context.queue_family_index;
        let pool = check(unsafe { device.create_command_pool(&vk::CommandPoolCreateInfo::default().queue_family_index(family).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER), None) }, "vkCreateCommandPool")?;
        let list = check(unsafe { device.allocate_command_buffers(&vk::CommandBufferAllocateInfo::default().command_pool(pool).command_buffer_count(IN_FLIGHT as u32)) }, "vkAllocateCommandBuffers")?;
        let mut fences = [vk::Fence::null(); IN_FLIGHT];
        for f in &mut fences {
            *f = check(unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None) }, "vkCreateFence")?;
        }
        let memory = unsafe { instance.get_physical_device_memory_properties(physical) };
        let interface = retro_hw_render_interface_vulkan {
            interface_type: INTERFACE_VULKAN,
            interface_version: HW_RENDER_INTERFACE_VULKAN_VERSION,
            handle: std::ptr::null_mut(),
            instance: instance.handle(),
            gpu: physical,
            device: device.handle(),
            get_device_proc_addr: instance.fp_v1_0().get_device_proc_addr,
            get_instance_proc_addr: gipa,
            queue: context.queue,
            queue_index: 0,
            set_image: Some(set_image),
            get_sync_index: Some(get_sync_index),
            get_sync_index_mask: Some(get_sync_index_mask),
            set_command_buffers: Some(set_command_buffers),
            wait_sync_index: Some(wait_sync_index),
            lock_queue: Some(lock_queue),
            unlock_queue: Some(unlock_queue),
            set_signal_semaphore: Some(set_signal_semaphore),
        };
        let mut g = Box::new(Gpu {
            entry,
            instance,
            physical,
            device,
            queue: context.queue,
            family,
            memory,
            exports,
            uuids,
            interface,
            lock: AtomicBool::new(false),
            pool,
            commands: [list[0], list[1]],
            fences,
            submitted: [false; IN_FLIGHT],
            index: 0,
            handed: None,
            waits: Vec::new(),
            core_commands: Vec::new(),
            signal: vk::Semaphore::null(),
            ring: None,
            record: None,
            unread: None,
            held: None,
            readback: None,
            swizzled: Vec::new(),
            log: Vec::new(),
        });
        g.interface.handle = &mut *g as *mut Gpu as *mut c_void;
        if g.exports.is_none() {
            g.log.push("the card lacks external memory, semaphores or timeline semaphores: pictures are read back".into());
        }
        Ok(g)
    }

    /// The interface `GET_HW_RENDER_INTERFACE` hands, valid while this lives.
    pub fn interface(&self) -> *const retro_hw_render_interface_vulkan {
        &self.interface
    }

    /// A frame begins: last frame's record is gone.
    pub fn begin_frame(&mut self) {
        self.record = None;
    }

    fn wait_slot(&mut self, i: usize) {
        if self.submitted[i] {
            unsafe {
                let _ = self.device.wait_for_fences(&[self.fences[i]], true, u64::MAX);
                let _ = self.device.reset_fences(&[self.fences[i]]);
            }
            self.submitted[i] = false;
        }
    }

    fn memory_type(&self, bits: u32, flags: vk::MemoryPropertyFlags) -> Option<u32> {
        (0..self.memory.memory_type_count).find(|&i| bits & (1 << i) != 0 && self.memory.memory_types[i as usize].property_flags.contains(flags))
    }

    // ---- The ring --------------------------------------------------------------------------------------------

    /// The host's ring imported, replacing the last once its pictures are released; `None` withdraws it. A ring on
    /// another card, or a card that cannot import, is `NOT_SUPPORTED` (EmuSen_CoreAPI.md §6.43).
    pub fn lend(&mut self, ring: Option<&Ring>) -> Result<(), i32> {
        self.drop_ring();
        let Some(r) = ring else { return Ok(()) };
        if self.exports.is_none() || (r.device_uuid, r.driver_uuid) != self.uuids {
            return Err(status::NOT_SUPPORTED);
        }
        if r.images.is_empty() || r.images.iter().any(|i| i.format != SLOT_FORMAT.as_raw() as u32 || i.memory_type >= self.memory.memory_type_count) {
            return Err(status::BAD_STRUCT);
        }
        let mut imported = Imported { slots: Vec::new(), ready: vk::Semaphore::null(), done: vk::Semaphore::null(), count: 0 };
        let made = self.import(r, &mut imported);
        if let Err(why) = made {
            self.log.push(format!("the host's ring could not be imported: {why}"));
            self.free_imported(imported);
            return Err(status::NOT_SUPPORTED);
        }
        self.ring = Some(imported);
        Ok(())
    }

    fn import(&self, r: &Ring, into: &mut Imported) -> Result<(), String> {
        let (memory_fd, semaphore_fd, _) = self.exports.as_ref().unwrap();
        let _ = memory_fd;
        let dup = |h: u64| -> Result<i32, String> {
            let fd = unsafe { libc::dup(h as i32) };
            if fd < 0 { Err(format!("handle {h} does not duplicate")) } else { Ok(fd) }
        };
        for i in &r.images {
            let mut external = vk::ExternalMemoryImageCreateInfo::default().handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
            let info = vk::ImageCreateInfo::default()
                .push_next(&mut external)
                .image_type(vk::ImageType::TYPE_2D)
                .format(SLOT_FORMAT)
                .extent(vk::Extent3D { width: i.width, height: i.height, depth: 1 })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::from_raw(i.usage))
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED);
            let image = check(unsafe { self.device.create_image(&info, None) }, "vkCreateImage")?;
            let fd = match dup(i.handle) {
                Ok(fd) => fd,
                Err(e) => {
                    unsafe { self.device.destroy_image(image, None) };
                    return Err(e);
                }
            };
            let mut import = vk::ImportMemoryFdInfoKHR::default().handle_type(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD).fd(fd);
            let mut dedicated = vk::MemoryDedicatedAllocateInfo::default().image(image);
            let allocate = vk::MemoryAllocateInfo::default().allocation_size(i.allocation_size).memory_type_index(i.memory_type).push_next(&mut import).push_next(&mut dedicated);
            let memory = match unsafe { self.device.allocate_memory(&allocate, None) } {
                Ok(m) => m,
                Err(e) => {
                    unsafe {
                        libc::close(fd);
                        self.device.destroy_image(image, None);
                    }
                    return Err(format!("vkAllocateMemory (import) returned {e}"));
                }
            };
            into.slots.push(Slot { image, memory, width: i.width, height: i.height, used: false, released_at: 0 });
            check(unsafe { self.device.bind_image_memory(image, memory, 0) }, "vkBindImageMemory")?;
        }
        for (h, at) in [(r.ready, &mut into.ready), (r.done, &mut into.done)] {
            let mut kind = vk::SemaphoreTypeCreateInfo::default().semaphore_type(vk::SemaphoreType::TIMELINE).initial_value(0);
            *at = check(unsafe { self.device.create_semaphore(&vk::SemaphoreCreateInfo::default().push_next(&mut kind), None) }, "vkCreateSemaphore")?;
            let fd = dup(h)?;
            if let Err(e) = unsafe { semaphore_fd.import_semaphore_fd(&vk::ImportSemaphoreFdInfoKHR::default().semaphore(*at).handle_type(vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_FD).fd(fd)) } {
                unsafe { libc::close(fd) };
                return Err(format!("vkImportSemaphoreFdKHR returned {e}"));
            }
        }
        Ok(())
    }

    fn free_imported(&self, r: Imported) {
        unsafe {
            for s in r.slots {
                self.device.destroy_image(s.image, None);
                self.device.free_memory(s.memory, None);
            }
            self.device.destroy_semaphore(r.ready, None);
            self.device.destroy_semaphore(r.done, None);
        }
    }

    /// The ring let go once the host has released every picture handed on it, and this device is idle.
    fn drop_ring(&mut self) {
        let Some(r) = self.ring.take() else { return };
        let last = r.slots.iter().map(|s| s.released_at).max().unwrap_or(0);
        if let Some((_, _, timeline)) = &self.exports
            && last > 0
        {
            let values = [last];
            let semaphores = [r.done];
            // A host that never releases would stall here; a second is the bound, after which the ring goes anyway.
            let _ = unsafe { timeline.wait_semaphores(&vk::SemaphoreWaitInfo::default().semaphores(&semaphores).values(&values), 1_000_000_000) };
        }
        unsafe {
            let _ = self.device.queue_wait_idle(self.queue);
        }
        self.submitted = [false; IN_FLIGHT];
        unsafe {
            let _ = self.device.reset_fences(&self.fences);
        }
        self.free_imported(r);
    }

    /// The largest picture the lent ring holds.
    fn ring_holds(&self, w: u32, h: u32) -> Option<bool> {
        self.ring.as_ref().map(|r| r.slots.iter().all(|s| s.width >= w && s.height >= h))
    }

    // ---- A frame -----------------------------------------------------------------------------------------------

    /// The frame `video_refresh(RETRO_HW_FRAME_BUFFER_VALID)` handed, copied into the ring, held, or read back.
    pub fn frame(&mut self, w: u32, h: u32) -> Shown {
        self.unread = None;
        let Some(handed) = self.handed else {
            if self.log.len() < 8 {
                self.log.push("the core handed a hardware frame without setting an image".into());
            }
            return Shown::Nothing;
        };
        let shown = match self.ring_holds(w, h) {
            Some(true) => {
                let slot = (self.ring.as_ref().unwrap().count % self.ring.as_ref().unwrap().slots.len() as u64) as usize;
                match self.submit(handed, w, h, Target::Slot(slot), true) {
                    Ok(()) => {
                        self.unread = Some((handed, w, h));
                        return Shown::Ring;
                    }
                    Err(why) => {
                        self.log.push(why);
                        return Shown::Nothing;
                    }
                }
            }
            Some(false) => {
                if let Err(why) = self.hold(handed, w, h) {
                    self.log.push(why);
                    return Shown::Nothing;
                }
                true
            }
            None => false,
        };
        let src = if shown { Handed { image: self.held.as_ref().unwrap().image, layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL, format: SLOT_FORMAT } } else { handed };
        let Some(bgra) = readable(src.format) else {
            if self.log.len() < 8 {
                self.log.push(format!("a picture of format {:?} cannot be read back", src.format));
            }
            return Shown::Nothing;
        };
        if let Err(why) = self.read_back(src, w, h, !shown) {
            self.log.push(why);
            return Shown::Nothing;
        }
        if shown { Shown::Larger(w, h, bgra) } else { Shown::Bytes(w, h, bgra) }
    }

    /// The size of the ring's last frame while it has not been read back.
    pub fn unread(&self) -> Option<(u32, u32)> {
        self.unread.map(|(_, w, h)| (w, h))
    }

    /// The ring's last frame read back on demand, for a screenshot or a state's picture: its size and whether it is
    /// BGRA, `read` handing the bytes; nothing is read back on the ring's own path (EmuSen_LibretroHost.md §10h).
    pub fn read_unread(&mut self) -> Option<(u32, u32, bool)> {
        let (src, w, h) = self.unread.take()?;
        let bgra = readable(src.format)?;
        match self.read_back(src, w, h, false) {
            Ok(()) => Some((w, h, bgra)),
            Err(why) => {
                self.log.push(why);
                None
            }
        }
    }

    /// The frame's record for `frame_ring`; a held frame is copied into a ring that now holds it first.
    pub fn frame_ring(&mut self) -> Option<RingFrame> {
        if let Some(held) = self.held.as_ref().filter(|h| h.pending).map(|h| (h.image, h.width, h.height))
            && self.ring_holds(held.1, held.2) == Some(true)
        {
            let r = self.ring.as_ref().unwrap();
            let slot = (r.count % r.slots.len() as u64) as usize;
            let src = Handed { image: held.0, layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL, format: SLOT_FORMAT };
            match self.submit(src, held.1, held.2, Target::Slot(slot), false) {
                Ok(()) => self.held.as_mut().unwrap().pending = false,
                Err(why) => self.log.push(why),
            }
        }
        self.record
    }

    fn hold(&mut self, handed: Handed, w: u32, h: u32) -> Result<(), String> {
        if self.held.as_ref().is_none_or(|x| x.width != w || x.height != h) {
            self.free_held();
            let info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(SLOT_FORMAT)
                .extent(vk::Extent3D { width: w, height: h, depth: 1 })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC)
                .initial_layout(vk::ImageLayout::UNDEFINED);
            let image = check(unsafe { self.device.create_image(&info, None) }, "vkCreateImage")?;
            let needs = unsafe { self.device.get_image_memory_requirements(image) };
            let kind = self.memory_type(needs.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL).ok_or("no device memory for a held frame")?;
            let memory = check(unsafe { self.device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(needs.size).memory_type_index(kind), None) }, "vkAllocateMemory")?;
            check(unsafe { self.device.bind_image_memory(image, memory, 0) }, "vkBindImageMemory")?;
            self.held = Some(Held { image, memory, width: w, height: h, pending: false });
        }
        self.submit(handed, w, h, Target::Held, true)?;
        self.held.as_mut().unwrap().pending = true;
        Ok(())
    }

    fn free_held(&mut self) {
        if let Some(h) = self.held.take() {
            unsafe {
                let _ = self.device.queue_wait_idle(self.queue);
                self.device.destroy_image(h.image, None);
                self.device.free_memory(h.memory, None);
            }
        }
    }

    fn read_back(&mut self, src: Handed, w: u32, h: u32, core_frame: bool) -> Result<(), String> {
        let bytes = w as u64 * h as u64 * 4;
        if self.readback.as_ref().is_none_or(|r| r.bytes < bytes) {
            if let Some(r) = self.readback.take() {
                unsafe {
                    self.device.destroy_buffer(r.buffer, None);
                    self.device.free_memory(r.memory, None);
                }
            }
            let buffer = check(unsafe { self.device.create_buffer(&vk::BufferCreateInfo::default().size(bytes).usage(vk::BufferUsageFlags::TRANSFER_DST), None) }, "vkCreateBuffer")?;
            let needs = unsafe { self.device.get_buffer_memory_requirements(buffer) };
            // Cached where offered: reading uncached memory cost 17.9 ms a 320x240 frame on the RX 6800 (measured 2026-11-01).
            let visible = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
            let kind = self.memory_type(needs.memory_type_bits, visible | vk::MemoryPropertyFlags::HOST_CACHED).or_else(|| self.memory_type(needs.memory_type_bits, visible)).ok_or("no host-visible memory")?;
            let memory = check(unsafe { self.device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(needs.size).memory_type_index(kind), None) }, "vkAllocateMemory")?;
            check(unsafe { self.device.bind_buffer_memory(buffer, memory, 0) }, "vkBindBufferMemory")?;
            let mapped = check(unsafe { self.device.map_memory(memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()) }, "vkMapMemory")? as *mut u8;
            self.readback = Some(Readback { buffer, memory, bytes, mapped });
        }
        let i = self.index;
        self.submit(src, w, h, Target::Buffer, core_frame)?;
        self.wait_slot(i);
        Ok(())
    }

    /// One submission: the core's command buffers, then the copy of `src` into `target`, ordered by the core's
    /// semaphores and, for a slot, by `done` and `ready` with the ownership crossing `VK_QUEUE_FAMILY_EXTERNAL`.
    fn submit(&mut self, src: Handed, w: u32, h: u32, target: Target, core_frame: bool) -> Result<(), String> {
        let i = self.index;
        self.wait_slot(i);
        let cmd = self.commands[i];
        let d = &self.device;
        let ext = vk::QUEUE_FAMILY_EXTERNAL;
        let ignored = vk::QUEUE_FAMILY_IGNORED;
        let (dst_image, dst_old, dst_from) = match target {
            Target::Slot(s) => {
                let slot = &self.ring.as_ref().unwrap().slots[s];
                (slot.image, if slot.used { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL } else { vk::ImageLayout::UNDEFINED }, ext)
            }
            Target::Held => (self.held.as_ref().unwrap().image, vk::ImageLayout::UNDEFINED, ignored),
            Target::Buffer => (vk::Image::null(), vk::ImageLayout::UNDEFINED, ignored),
        };
        unsafe {
            check(d.begin_command_buffer(cmd, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)), "vkBeginCommandBuffer")?;
            // A slot crosses from VK_QUEUE_FAMILY_EXTERNAL in its shared layout; the transition to the copy is a barrier of its own.
            if dst_from == ext && dst_old != vk::ImageLayout::UNDEFINED {
                let take = barrier(dst_image, dst_old, dst_old, vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE, ext, self.family);
                d.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[], &[], &[take]);
            }
            let mut before = vec![barrier(src.image, src.layout, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, vk::AccessFlags::MEMORY_WRITE, vk::AccessFlags::TRANSFER_READ, ignored, ignored)];
            if dst_image != vk::Image::null() {
                before.push(barrier(dst_image, dst_old, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE, ignored, ignored));
            }
            d.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::ALL_COMMANDS, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[], &[], &before);
            let extent = vk::Extent3D { width: w, height: h, depth: 1 };
            if let Target::Buffer = target {
                let region = vk::BufferImageCopy { buffer_offset: 0, buffer_row_length: 0, buffer_image_height: 0, image_subresource: layers(), image_offset: vk::Offset3D::default(), image_extent: extent };
                d.cmd_copy_image_to_buffer(cmd, src.image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, self.readback.as_ref().unwrap().buffer, &[region]);
            } else if src.format == SLOT_FORMAT {
                let region = vk::ImageCopy { src_subresource: layers(), src_offset: vk::Offset3D::default(), dst_subresource: layers(), dst_offset: vk::Offset3D::default(), extent };
                d.cmd_copy_image(cmd, src.image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, dst_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]);
            } else {
                let corner = vk::Offset3D { x: w as i32, y: h as i32, z: 1 };
                let region = vk::ImageBlit { src_subresource: layers(), src_offsets: [vk::Offset3D::default(), corner], dst_subresource: layers(), dst_offsets: [vk::Offset3D::default(), corner] };
                d.cmd_blit_image(cmd, src.image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, dst_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region], vk::Filter::NEAREST);
            }
            let mut after = vec![barrier(src.image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, src.layout, vk::AccessFlags::TRANSFER_READ, vk::AccessFlags::empty(), ignored, ignored)];
            match target {
                Target::Slot(_) => after.push(barrier(dst_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::empty(), ignored, ignored)),
                Target::Held => after.push(barrier(dst_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::TRANSFER_READ, ignored, ignored)),
                Target::Buffer => {}
            }
            d.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::ALL_COMMANDS | vk::PipelineStageFlags::HOST, vk::DependencyFlags::empty(), &[], &[], &after);
            if let Target::Slot(_) = target {
                let give = barrier(dst_image, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::AccessFlags::empty(), vk::AccessFlags::empty(), self.family, ext);
                d.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::ALL_COMMANDS, vk::PipelineStageFlags::BOTTOM_OF_PIPE, vk::DependencyFlags::empty(), &[], &[], &[give]);
            }
            if let Target::Buffer = target {
                let host = vk::MemoryBarrier::default().src_access_mask(vk::AccessFlags::TRANSFER_WRITE).dst_access_mask(vk::AccessFlags::HOST_READ);
                d.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::HOST, vk::DependencyFlags::empty(), &[host], &[], &[]);
            }
            check(d.end_command_buffer(cmd), "vkEndCommandBuffer")?;
        }
        let mut waits: Vec<vk::Semaphore> = if core_frame { self.waits.clone() } else { Vec::new() };
        let mut wait_values = vec![0u64; waits.len()];
        let mut stages = vec![vk::PipelineStageFlags::ALL_COMMANDS; waits.len()];
        let mut signals = Vec::new();
        let mut signal_values = Vec::new();
        let mut buffers: Vec<vk::CommandBuffer> = if core_frame { std::mem::take(&mut self.core_commands) } else { Vec::new() };
        buffers.push(cmd);
        let mut value = 0;
        if let Target::Slot(s) = target {
            let r = self.ring.as_ref().unwrap();
            if r.slots[s].released_at > 0 {
                waits.push(r.done);
                wait_values.push(r.slots[s].released_at);
                stages.push(vk::PipelineStageFlags::TRANSFER);
            }
            value = r.count + 1;
            signals.push(r.ready);
            signal_values.push(value);
        }
        if core_frame && self.signal != vk::Semaphore::null() {
            signals.push(std::mem::replace(&mut self.signal, vk::Semaphore::null()));
            signal_values.push(0);
        }
        let mut values = vk::TimelineSemaphoreSubmitInfo::default().wait_semaphore_values(&wait_values).signal_semaphore_values(&signal_values);
        let mut info = vk::SubmitInfo::default().wait_semaphores(&waits).wait_dst_stage_mask(&stages).command_buffers(&buffers).signal_semaphores(&signals);
        if self.exports.is_some() {
            info = info.push_next(&mut values);
        }
        unsafe { lock_queue(self.interface.handle) };
        let sent = unsafe { self.device.queue_submit(self.queue, &[info], self.fences[i]) };
        unsafe { unlock_queue(self.interface.handle) };
        check(sent, "vkQueueSubmit")?;
        self.submitted[i] = true;
        self.index = (i + 1) % IN_FLIGHT;
        if let Target::Slot(s) = target {
            let r = self.ring.as_mut().unwrap();
            r.count = value;
            r.slots[s].used = true;
            r.slots[s].released_at = value;
            self.record = Some(RingFrame { size: size_of::<RingFrame>() as u32, slot: s as u32, width: w as i32, height: h as i32, value, ..RingFrame::default() });
        }
        Ok(())
    }

    /// A read-back picture in the bytes `Picture::take` reads as XRGB8888: BGRA as it is, RGBA swapped once.
    pub fn read(&mut self, w: u32, h: u32, bgra: bool) -> &[u8] {
        let Some(rb) = self.readback.as_ref() else { return &[] };
        let bytes = unsafe { std::slice::from_raw_parts(rb.mapped, (w as usize * h as usize * 4).min(rb.bytes as usize)) };
        if bgra {
            return bytes;
        }
        self.swizzled.resize(bytes.len(), 0);
        for (o, p) in self.swizzled.chunks_exact_mut(4).zip(bytes.chunks_exact(4)) {
            o.copy_from_slice(&[p[2], p[1], p[0], 0xFF]);
        }
        &self.swizzled
    }
}

impl Drop for Gpu {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
        }
        self.drop_ring();
        self.free_held();
        unsafe {
            if let Some(r) = self.readback.take() {
                self.device.destroy_buffer(r.buffer, None);
                self.device.free_memory(r.memory, None);
            }
            for f in self.fences {
                self.device.destroy_fence(f, None);
            }
            self.device.destroy_command_pool(self.pool, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
        let _ = &self.entry;
        let _ = self.physical;
    }
}

/// The picture's field structure for the console's screen filters, from its height against the console's modes: 0 a
/// progressive frame, 1 one field, 2 two fields woven (EmuSen_LibretroHost.md §0.3, §5.4a).
pub fn fields_of(system: &str, height: u32) -> (u32, u32) {
    match (system, height) {
        ("ps2", 224 | 256) => (1, 0),
        ("ps2", 448 | 512) => (2, 0),
        _ => (0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ps2_picture_is_one_field_at_224_or_256_rows_and_two_woven_at_448_or_512() {
        assert_eq!([224, 256, 448, 512, 480].map(|h| fields_of("ps2", h)), [(1, 0), (1, 0), (2, 0), (2, 0), (0, 0)]);
        assert_eq!(fields_of("dreamcast", 480), (0, 0));
        assert_eq!(fields_of("", 240), (0, 0));
    }
}
