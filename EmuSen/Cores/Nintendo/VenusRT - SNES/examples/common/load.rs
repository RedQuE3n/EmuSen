//! The examples' machine: the ROM, with VenusRT's own SPC700 boot program (D-38) unless EMUSEN_VENUSRT_BOOT names the
//! player's 64-byte image, and a NEC DSP's firmware when EMUSEN_VENUSRT_DSP names one.
pub fn machine(path: &str) -> venusrt::machine::Machine {
    let image = std::fs::read(path).expect("the ROM");
    let mut m = venusrt::machine::Machine::load_rom(&image).expect("an image");
    if let Some(boot) = std::env::var_os("EMUSEN_VENUSRT_BOOT").and_then(|p| std::fs::read(p).ok()) {
        m.attach_boot(boot.try_into().expect("a boot image of 64 bytes"));
    }
    if let Some(fw) = std::env::var_os("EMUSEN_VENUSRT_DSP").and_then(|p| std::fs::read(p).ok()) {
        assert!(m.attach_dsp(&fw), "firmware of 8,192 or 53,248 bytes");
    }
    m
}
