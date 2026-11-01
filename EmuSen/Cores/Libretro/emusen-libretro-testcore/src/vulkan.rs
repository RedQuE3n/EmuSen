//! The test core's Vulkan path: the same picture as its software one, uploaded into an image of its own device and
//! handed by `set_image`, the device made through the negotiation interface v2, or v1 (EmuSen_LibretroHost.md §9, §10f, §10o).

use std::ffi::{c_char, c_uint, c_void};
use std::sync::Mutex;

use ash::vk;
use emusen_libretro_sys::vulkan::*;
use emusen_libretro_sys::{self as sys, env};

use crate::layout::{MAX_HEIGHT, MAX_WIDTH};

/// What the core holds on the host's device once its context is reset.
struct Vk {
    iface: *const retro_hw_render_interface_vulkan,
    device: ash::Device,
    image: vk::Image,
    memory: vk::DeviceMemory,
    format: vk::Format,
    staging: [(vk::Buffer, vk::DeviceMemory, *mut u8); 2],
    pool: vk::CommandPool,
    commands: [vk::CommandBuffer; 2],
}

unsafe impl Send for Vk {}

static VK: Mutex<Option<Vk>> = Mutex::new(None);
static FORMAT: Mutex<vk::Format> = Mutex::new(vk::Format::R8G8B8A8_UNORM);
static NEGOTIATION: retro_hw_render_context_negotiation_interface_vulkan = retro_hw_render_context_negotiation_interface_vulkan {
    interface_type: INTERFACE_VULKAN,
    interface_version: 2,
    get_application_info: None,
    create_device: None,
    destroy_device: None,
    create_instance: Some(create_instance),
    create_device2: Some(create_device2),
};

/// Negotiation v1 (`tc_hw=vulkan_v1`): the core makes its device itself, through the vkGetInstanceProcAddr it is given.
static NEGOTIATION_V1: retro_hw_render_context_negotiation_interface_vulkan = retro_hw_render_context_negotiation_interface_vulkan {
    interface_type: INTERFACE_VULKAN,
    interface_version: 1,
    get_application_info: None,
    create_device: Some(create_device),
    destroy_device: None,
    create_instance: None,
    create_device2: None,
};
/// `tc_hw=vulkan_v1_loader`: v1, its vkCreateDevice taken from the system's loader rather than the one it is given.
static OWN_LOADER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The system loader's vkGetInstanceProcAddr, the library the host has already loaded.
fn loader_gipa() -> Option<vk::PFN_vkGetInstanceProcAddr> {
    let lib = unsafe { libc::dlopen(c"libvulkan.so.1".as_ptr(), libc::RTLD_NOW | libc::RTLD_NOLOAD) };
    let f = if lib.is_null() { std::ptr::null_mut() } else { unsafe { libc::dlsym(lib, c"vkGetInstanceProcAddr".as_ptr()) } };
    (!f.is_null()).then(|| unsafe { std::mem::transmute::<*mut c_void, vk::PFN_vkGetInstanceProcAddr>(f) })
}

/// The first queue family that draws and computes, which both device paths take.
fn graphics_family(i: &ash::Instance, gpu: vk::PhysicalDevice) -> Option<usize> {
    let families = unsafe { i.get_physical_device_queue_family_properties(gpu) };
    families.iter().position(|q| q.queue_flags.contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE))
}

#[allow(clippy::too_many_arguments)]
unsafe extern "C" fn create_device(context: *mut retro_vulkan_context, instance: vk::Instance, gpu: vk::PhysicalDevice, _surface: vk::SurfaceKHR, gipa: vk::PFN_vkGetInstanceProcAddr, extensions: *const *const c_char, extension_count: c_uint, _layers: *const *const c_char, _layer_count: c_uint, _features: *const vk::PhysicalDeviceFeatures) -> bool {
    let gipa = if OWN_LOADER.load(std::sync::atomic::Ordering::Relaxed) { loader_gipa().unwrap_or(gipa) } else { gipa };
    let i = unsafe { ash::Instance::load(&ash::StaticFn { get_instance_proc_addr: gipa }, instance) };
    let Some(family) = graphics_family(&i, gpu) else { return false };
    let priority = [1.0f32];
    let queue = [vk::DeviceQueueCreateInfo::default().queue_family_index(family as u32).queue_priorities(&priority)];
    let names = if extension_count == 0 || extensions.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(extensions, extension_count as usize) } };
    let Ok(d) = (unsafe { i.create_device(gpu, &vk::DeviceCreateInfo::default().queue_create_infos(&queue).enabled_extension_names(names), None) }) else { return false };
    let q = unsafe { d.get_device_queue(family as u32, 0) };
    unsafe { *context = retro_vulkan_context { gpu, device: d.handle(), queue: q, queue_family_index: family as u32, presentation_queue: q, presentation_queue_family_index: family as u32 } };
    true
}

unsafe extern "C" fn create_instance(_gipa: vk::PFN_vkGetInstanceProcAddr, app: *const vk::ApplicationInfo<'_>, wrapper: retro_vulkan_create_instance_wrapper_t, opaque: *mut c_void) -> vk::Instance {
    let fallback = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_1);
    let info = vk::InstanceCreateInfo::default().application_info(unsafe { app.as_ref() }.unwrap_or(&fallback));
    unsafe { wrapper(opaque, &info) }
}

unsafe extern "C" fn create_device2(context: *mut retro_vulkan_context, instance: vk::Instance, gpu: vk::PhysicalDevice, _surface: vk::SurfaceKHR, gipa: vk::PFN_vkGetInstanceProcAddr, wrapper: retro_vulkan_create_device_wrapper_t, opaque: *mut c_void) -> bool {
    let i = unsafe { ash::Instance::load(&ash::StaticFn { get_instance_proc_addr: gipa }, instance) };
    let Some(family) = graphics_family(&i, gpu) else { return false };
    let priority = [1.0f32];
    let queue = [vk::DeviceQueueCreateInfo::default().queue_family_index(family as u32).queue_priorities(&priority)];
    let device = unsafe { wrapper(gpu, opaque, &vk::DeviceCreateInfo::default().queue_create_infos(&queue)) };
    if device == vk::Device::null() {
        return false;
    }
    let d = unsafe { ash::Device::load(i.fp_v1_0(), device) };
    let q = unsafe { d.get_device_queue(family as u32, 0) };
    unsafe { *context = retro_vulkan_context { gpu, device, queue: q, queue_family_index: family as u32, presentation_queue: q, presentation_queue_family_index: family as u32 } };
    true
}

unsafe extern "C" fn context_reset() {
    let mut iface: *const retro_hw_render_interface_vulkan = std::ptr::null();
    if !crate::environment(env::GET_HW_RENDER_INTERFACE, &mut iface as *mut _ as *mut c_void) || iface.is_null() {
        crate::log(sys::LOG_ERROR, "no Vulkan interface after context_reset");
        return;
    }
    let f = unsafe { &*iface };
    let instance = unsafe { ash::Instance::load(&ash::StaticFn { get_instance_proc_addr: f.get_instance_proc_addr }, f.instance) };
    let device = unsafe { ash::Device::load(instance.fp_v1_0(), f.device) };
    let memory = unsafe { instance.get_physical_device_memory_properties(f.gpu) };
    let kind = |bits: u32, flags: vk::MemoryPropertyFlags| (0..memory.memory_type_count).find(|&k| bits & (1 << k) != 0 && memory.memory_types[k as usize].property_flags.contains(flags)).unwrap();
    let format = *FORMAT.lock().unwrap();
    unsafe {
        let info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D { width: MAX_WIDTH, height: MAX_HEIGHT, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::SAMPLED)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let image = device.create_image(&info, None).unwrap();
        let needs = device.get_image_memory_requirements(image);
        let mem = device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(needs.size).memory_type_index(kind(needs.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL)), None).unwrap();
        device.bind_image_memory(image, mem, 0).unwrap();
        let bytes = (MAX_WIDTH * MAX_HEIGHT * 4) as u64;
        let staging = [0, 1].map(|_| {
            let b = device.create_buffer(&vk::BufferCreateInfo::default().size(bytes).usage(vk::BufferUsageFlags::TRANSFER_SRC), None).unwrap();
            let needs = device.get_buffer_memory_requirements(b);
            let m = device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(needs.size).memory_type_index(kind(needs.memory_type_bits, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)), None).unwrap();
            device.bind_buffer_memory(b, m, 0).unwrap();
            let p = device.map_memory(m, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()).unwrap() as *mut u8;
            (b, m, p)
        });
        let families = instance.get_physical_device_queue_family_properties(f.gpu);
        let family = families.iter().position(|q| q.queue_flags.contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)).unwrap() as u32;
        let pool = device.create_command_pool(&vk::CommandPoolCreateInfo::default().queue_family_index(family).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER), None).unwrap();
        let list = device.allocate_command_buffers(&vk::CommandBufferAllocateInfo::default().command_pool(pool).command_buffer_count(2)).unwrap();
        *VK.lock().unwrap() = Some(Vk { iface, device, image, memory: mem, format, staging, pool, commands: [list[0], list[1]] });
    }
}

unsafe extern "C" fn context_destroy() {
    if let Some(v) = VK.lock().unwrap().take() {
        unsafe {
            let _ = v.device.device_wait_idle();
            v.device.destroy_command_pool(v.pool, None);
            for (b, m, _) in v.staging {
                v.device.destroy_buffer(b, None);
                v.device.free_memory(m, None);
            }
            v.device.destroy_image(v.image, None);
            v.device.free_memory(v.memory, None);
        }
    }
}

/// At load: the negotiation interface and a Vulkan 1.1 context asked for; false where the host refuses either.
pub fn ask(hw: &str) -> bool {
    *FORMAT.lock().unwrap() = if hw == "vulkan_bgra" { vk::Format::B8G8R8A8_UNORM } else { vk::Format::R8G8B8A8_UNORM };
    let v1 = hw.starts_with("vulkan_v1");
    OWN_LOADER.store(hw == "vulkan_v1_loader", std::sync::atomic::Ordering::Relaxed);
    let mut support = [INTERFACE_VULKAN, 0u32];
    crate::environment(env::GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT, support.as_mut_ptr() as *mut c_void);
    let mut preferred: c_uint = 0;
    crate::environment(env::GET_PREFERRED_HW_RENDER, &mut preferred as *mut c_uint as *mut c_void);
    let negotiation = if v1 { &NEGOTIATION_V1 } else { &NEGOTIATION };
    if support[1] >= negotiation.interface_version && !crate::environment(env::SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE, negotiation as *const _ as *mut c_void) {
        return false;
    }
    let mut hw = sys::retro_hw_render_callback {
        context_type: sys::HW_CONTEXT_VULKAN,
        context_reset: Some(context_reset),
        get_current_framebuffer: None,
        get_proc_address: None,
        depth: false,
        stencil: false,
        bottom_left_origin: false,
        version_major: 1,
        version_minor: 1,
        cache_context: false,
        context_destroy: Some(context_destroy),
        debug_context: false,
    };
    crate::environment(env::SET_HW_RENDER, &mut hw as *mut _ as *mut c_void)
}

/// One frame: the software picture (XRGB8888 rows of `pitch`) uploaded into the image, handed with `set_image`, then
/// `video_refresh(RETRO_HW_FRAME_BUFFER_VALID)`.
pub fn hand(picture: &[u8], pitch: usize, width: u32, height: u32, video: unsafe extern "C" fn(*const c_void, c_uint, c_uint, usize)) {
    let guard = VK.lock().unwrap();
    let Some(v) = guard.as_ref() else { return };
    let f = unsafe { &*v.iface };
    let index = unsafe { f.get_sync_index.unwrap()(f.handle) } as usize & 1;
    unsafe { f.wait_sync_index.unwrap()(f.handle) };
    let (buffer, _, mapped) = v.staging[index];
    let out = unsafe { std::slice::from_raw_parts_mut(mapped, (width * height * 4) as usize) };
    let bgra = v.format == vk::Format::B8G8R8A8_UNORM;
    for y in 0..height as usize {
        let src = &picture[y * pitch..y * pitch + width as usize * 4];
        for (o, p) in out[y * width as usize * 4..(y + 1) * width as usize * 4].chunks_exact_mut(4).zip(src.chunks_exact(4)) {
            o.copy_from_slice(&if bgra { [p[0], p[1], p[2], 0xFF] } else { [p[2], p[1], p[0], 0xFF] });
        }
    }
    let cmd = v.commands[index];
    let d = &v.device;
    let range = vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 };
    let to = |old, new, src, dst| vk::ImageMemoryBarrier::default().image(v.image).old_layout(old).new_layout(new).src_access_mask(src).dst_access_mask(dst).src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED).subresource_range(range);
    unsafe {
        d.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty()).unwrap();
        d.begin_command_buffer(cmd, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)).unwrap();
        d.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::ALL_COMMANDS, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[], &[], &[to(vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE)]);
        let layers = vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 };
        let region = vk::BufferImageCopy { buffer_offset: 0, buffer_row_length: 0, buffer_image_height: 0, image_subresource: layers, image_offset: vk::Offset3D::default(), image_extent: vk::Extent3D { width, height, depth: 1 } };
        d.cmd_copy_buffer_to_image(cmd, buffer, v.image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]);
        d.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::ALL_COMMANDS, vk::DependencyFlags::empty(), &[], &[], &[to(vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ)]);
        d.end_command_buffer(cmd).unwrap();
        f.set_command_buffers.unwrap()(f.handle, 1, &cmd);
        let mut info = vk::ImageViewCreateInfo::default().image(v.image).format(v.format).view_type(vk::ImageViewType::TYPE_2D).subresource_range(range);
        info.p_next = std::ptr::null();
        let handed = retro_vulkan_image { image_view: vk::ImageView::null(), image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, create_info: info };
        f.set_image.unwrap()(f.handle, &handed, 0, std::ptr::null(), vk::QUEUE_FAMILY_IGNORED);
    }
    drop(guard);
    unsafe { video(sys::HW_FRAME_BUFFER_VALID, width, height, 0) };
}
