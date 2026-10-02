//! The examples' machine: the ROM with the SPC700 boot ROM from EMUSEN_VENUSRT_IPL or the corpus's firmware folder,
//! a local file never shipped (VenusRT_Native.md §21); without one the sound unit does not run.
pub fn machine(path: &str) -> venusrt::machine::Machine {
    let image = std::fs::read(path).expect("the ROM");
    let ipl = std::env::var_os("EMUSEN_VENUSRT_IPL")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".cache/emusen/probe/venusrt/firmware/spc700.rom")))
        .and_then(|p| std::fs::read(p).ok())
        .and_then(|b| <[u8; 64]>::try_from(b.as_slice()).ok());
    match ipl {
        Some(ipl) => venusrt::machine::Machine::with_ipl(&image, ipl).expect("an image"),
        None => venusrt::machine::Machine::load_rom(&image).expect("an image"),
    }
}
