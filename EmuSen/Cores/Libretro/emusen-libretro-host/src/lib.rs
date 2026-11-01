//! EmuSen's libretro host: the one libretro core beside this library, offered as a core on the core ABI v1, so that
//! every frontend runs it as it runs the project's own. See EmuSen_LibretroHost.md.

pub mod audio;
pub mod files;
pub mod front;
pub mod input;
pub mod library;
pub mod machine;
pub mod memory;
pub mod options;
pub mod state;
pub mod video;
pub mod vulkan;

/// The space a console's RAM cheats write into, as the systems table names it.
pub fn cheat_space(system: &str) -> Option<String> {
    emusen_systems::find(system).and_then(|s| s.cheat_space.clone())
}

emusen_native::core_exports!(machine::Machine; reset, phases, axes, settings, setting_notes, media, presence, device_ring);
