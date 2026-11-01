//! `libretro_vulkan.h` (MIT): the HW-render Vulkan interface v5 and its context negotiation v2, by hand, with ash's
//! Vulkan types; see EmuSen_LibretroHost.md §5.1.

use std::ffi::{c_char, c_uint, c_void};

use ash::vk;

pub const HW_RENDER_INTERFACE_VULKAN_VERSION: c_uint = 5;
pub const HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_VULKAN_VERSION: c_uint = 2;
/// `RETRO_HW_RENDER_INTERFACE_VULKAN` and `RETRO_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_VULKAN`.
pub const INTERFACE_VULKAN: c_uint = 0;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct retro_vulkan_image {
    pub image_view: vk::ImageView,
    pub image_layout: vk::ImageLayout,
    pub create_info: vk::ImageViewCreateInfo<'static>,
}

pub type retro_vulkan_set_image_t = Option<unsafe extern "C" fn(handle: *mut c_void, image: *const retro_vulkan_image, num_semaphores: u32, semaphores: *const vk::Semaphore, src_queue_family: u32)>;
pub type retro_vulkan_get_sync_index_t = Option<unsafe extern "C" fn(handle: *mut c_void) -> u32>;
pub type retro_vulkan_set_command_buffers_t = Option<unsafe extern "C" fn(handle: *mut c_void, num_cmd: u32, cmd: *const vk::CommandBuffer)>;
pub type retro_vulkan_handle_t = Option<unsafe extern "C" fn(handle: *mut c_void)>;
pub type retro_vulkan_set_signal_semaphore_t = Option<unsafe extern "C" fn(handle: *mut c_void, semaphore: vk::Semaphore)>;
pub type retro_vulkan_get_application_info_t = Option<unsafe extern "C" fn() -> *const vk::ApplicationInfo<'static>>;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct retro_vulkan_context {
    pub gpu: vk::PhysicalDevice,
    pub device: vk::Device,
    pub queue: vk::Queue,
    pub queue_family_index: u32,
    pub presentation_queue: vk::Queue,
    pub presentation_queue_family_index: u32,
}

pub type retro_vulkan_create_device_t = Option<
    unsafe extern "C" fn(
        context: *mut retro_vulkan_context,
        instance: vk::Instance,
        gpu: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
        required_device_extensions: *const *const c_char,
        num_required_device_extensions: c_uint,
        required_device_layers: *const *const c_char,
        num_required_device_layers: c_uint,
        required_features: *const vk::PhysicalDeviceFeatures,
    ) -> bool,
>;
pub type retro_vulkan_destroy_device_t = Option<unsafe extern "C" fn()>;
pub type retro_vulkan_create_instance_wrapper_t = unsafe extern "C" fn(opaque: *mut c_void, create_info: *const vk::InstanceCreateInfo<'_>) -> vk::Instance;
pub type retro_vulkan_create_instance_t = Option<
    unsafe extern "C" fn(get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr, app: *const vk::ApplicationInfo<'_>, wrapper: retro_vulkan_create_instance_wrapper_t, opaque: *mut c_void) -> vk::Instance,
>;
pub type retro_vulkan_create_device_wrapper_t = unsafe extern "C" fn(gpu: vk::PhysicalDevice, opaque: *mut c_void, create_info: *const vk::DeviceCreateInfo<'_>) -> vk::Device;
pub type retro_vulkan_create_device2_t = Option<
    unsafe extern "C" fn(
        context: *mut retro_vulkan_context,
        instance: vk::Instance,
        gpu: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
        wrapper: retro_vulkan_create_device_wrapper_t,
        opaque: *mut c_void,
    ) -> bool,
>;

/// What a core gives by `SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE`; the v2 members read only at version 2 or above.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct retro_hw_render_context_negotiation_interface_vulkan {
    pub interface_type: c_uint,
    pub interface_version: c_uint,
    pub get_application_info: retro_vulkan_get_application_info_t,
    pub create_device: retro_vulkan_create_device_t,
    pub destroy_device: retro_vulkan_destroy_device_t,
    pub create_instance: retro_vulkan_create_instance_t,
    pub create_device2: retro_vulkan_create_device2_t,
}

/// What `GET_HW_RENDER_INTERFACE` hands a Vulkan core, interface version 5.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct retro_hw_render_interface_vulkan {
    pub interface_type: c_uint,
    pub interface_version: c_uint,
    pub handle: *mut c_void,
    pub instance: vk::Instance,
    pub gpu: vk::PhysicalDevice,
    pub device: vk::Device,
    pub get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    pub get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    pub queue: vk::Queue,
    pub queue_index: c_uint,
    pub set_image: retro_vulkan_set_image_t,
    pub get_sync_index: retro_vulkan_get_sync_index_t,
    pub get_sync_index_mask: retro_vulkan_get_sync_index_t,
    pub set_command_buffers: retro_vulkan_set_command_buffers_t,
    pub wait_sync_index: retro_vulkan_handle_t,
    pub lock_queue: retro_vulkan_handle_t,
    pub unlock_queue: retro_vulkan_handle_t,
    pub set_signal_semaphore: retro_vulkan_set_signal_semaphore_t,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{offset_of, size_of};

    /// Sizes and offsets gcc 16.2.1 gave for `libretro_vulkan.h` with the Khronos headers on x86-64, measured 2026-11-01.
    #[test]
    fn the_vulkan_interface_lays_out_as_the_header() {
        assert_eq!(size_of::<retro_vulkan_image>(), 96);
        assert_eq!(offset_of!(retro_vulkan_image, image_layout), 8);
        assert_eq!(offset_of!(retro_vulkan_image, create_info), 16);
        assert_eq!(size_of::<retro_vulkan_context>(), 48);
        assert_eq!(offset_of!(retro_vulkan_context, queue_family_index), 24);
        assert_eq!(offset_of!(retro_vulkan_context, presentation_queue_family_index), 40);
        assert_eq!(size_of::<retro_hw_render_context_negotiation_interface_vulkan>(), 48);
        assert_eq!(offset_of!(retro_hw_render_context_negotiation_interface_vulkan, get_application_info), 8);
        assert_eq!(offset_of!(retro_hw_render_context_negotiation_interface_vulkan, create_instance), 32);
        assert_eq!(offset_of!(retro_hw_render_context_negotiation_interface_vulkan, create_device2), 40);
        assert_eq!(size_of::<retro_hw_render_interface_vulkan>(), 136);
        assert_eq!(offset_of!(retro_hw_render_interface_vulkan, handle), 8);
        assert_eq!(offset_of!(retro_hw_render_interface_vulkan, queue), 56);
        assert_eq!(offset_of!(retro_hw_render_interface_vulkan, queue_index), 64);
        assert_eq!(offset_of!(retro_hw_render_interface_vulkan, set_image), 72);
        assert_eq!(offset_of!(retro_hw_render_interface_vulkan, set_signal_semaphore), 128);
    }
}
