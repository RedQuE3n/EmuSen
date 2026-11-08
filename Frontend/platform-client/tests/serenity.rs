//! Serenity's device for an adopting host, and a chain drawn in place (EmuSen_Qt.md §1.8.1, interface 42). On the RX 6800.

use emusen_client_fixture::{World, library};
use emusen_platform_client::Platform;
use emusen_platform_client::serenity::{Chain, Device, builtin_preset};
use std::sync::Arc;

/// Vulkan's `VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL` and `VK_FORMAT_R8G8B8A8_UNORM`.
const SHADER_READ_ONLY: u32 = 5;
const RGBA8: u32 = 37;

#[test]
fn a_host_s_extensions_are_enabled_or_refused_by_name_and_a_chain_draws_in_place() {
    let world = World::new("serenity");
    let p = Platform::load(library()).unwrap();
    let name = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let refused =
        Device::new_for(&p, name.as_deref(), false, &["VK_KHR_surface"], &["VK_EMUSEN_no_such_extension"]).err().expect("refused");
    assert!(refused.to_string().contains("VK_EMUSEN_no_such_extension"), "{refused}");

    let device = Arc::new(Device::new_for(&p, name.as_deref(), true, &["VK_KHR_surface"], &["VK_KHR_swapchain"]).expect("a device"));
    let raw = device.raw();
    assert!(raw.instance != 0 && raw.physical != 0 && raw.device != 0 && raw.queue != 0, "{raw:?}");

    let preset = builtin_preset(&p, "effect.None.nearest", None, world.dir("presets").to_str().unwrap()).unwrap();
    let mut chain = Chain::new(&device, &preset).unwrap();
    assert!(chain.draw(32, 32).is_err(), "no frame yet");
    chain.advance(&[255u8; 16 * 8 * 4], 16, 8, 1).unwrap();
    let first = chain.draw(32, 16).unwrap();
    assert!(first.image != 0 && first.view != 0, "{first:?}");
    assert_eq!((first.width, first.height, first.format, first.layout), (32, 16, RGBA8, SHADER_READ_ONLY));
    assert_eq!(chain.draw(32, 16).unwrap().image, first.image, "the same size draws in the same image");
    let resized = chain.draw(64, 32).unwrap();
    assert_eq!((resized.width, resized.height), (64, 32));
    drop(chain);
    assert_eq!(device.findings().unwrap(), Vec::<String>::new());
}
