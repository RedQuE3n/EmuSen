//! The PlayStation 2's recogniser: a disc, CD or DVD, whose ISO 9660 root holds SYSTEM.CNF with a BOOT2 line, its
//! serial the program BOOT2 names (Sapphire_Plan.md §4.1, Sapphire_Native.md §3.2). A disc with BOOT and no BOOT2 is
//! the PlayStation's, which `psx::disc` claims and this refuses.

use crate::media::ascii;
use crate::probe::{Head, Need, ROOT_FILE_LIMIT, Recognition, root_file_through};
use emusen_dotnet::text::{equals_ignore_case, trim};

/// The bytes read first: sector 16, the primary volume descriptor, in an .iso's 2,048-byte sectors or a raw track's 2,352-byte ones.
pub const FIRST: usize = 40 * 1024;

/// The program SYSTEM.CNF's BOOT2 line names, as written after its `=`; none where no line's key is BOOT2.
pub fn boot2(config: Option<&[u8]>) -> Option<String> {
    let text = ascii(config?);
    text.split('\n').find_map(|line| {
        let eq = line.find('=')?;
        equals_ignore_case(trim(&line[..eq]), "BOOT2").then(|| trim(&line[eq + 1..]).to_string())
    })
}

/// The disc's claim with the serial BOOT2's program gives; none for no PlayStation 2 disc. The market is read at S29.
pub fn recognise(head: &Head) -> Result<Option<Recognition>, Need> {
    let config = root_file_through(head, FIRST, "SYSTEM.CNF", ROOT_FILE_LIMIT)?;
    let Some(program) = boot2(config.as_deref()) else { return Ok(None) };
    Ok(Some(Recognition { system: "ps2".into(), serial: crate::psx::disc::serial_of_file(&program), region: None }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot2_is_its_own_key_and_boot_is_not_it() {
        assert_eq!(boot2(Some(b"BOOT2 = cdrom0:\\SLUS_203.12;1\r\nVER = 1.00\r\nVMODE = NTSC\r\n")).as_deref(), Some("cdrom0:\\SLUS_203.12;1"));
        assert_eq!(boot2(Some(b"  boot2=cdrom0:\\SCES_500.00;1")).as_deref(), Some("cdrom0:\\SCES_500.00;1"));
        assert_eq!(boot2(Some(b"BOOT = cdrom:\\SLUS_007.70;1\r\n")), None);
        assert_eq!(boot2(Some(b"BOOT20 = x\nBOOT2 cdrom0:\\X\n")), None, "a longer key, and a line with no '=', are not BOOT2 lines");
        assert_eq!(boot2(None), None);
        assert_eq!(crate::psx::disc::serial_of_file("cdrom0:\\SLUS_203.12;1").as_deref(), Some("SLUS-20312"));
    }
}
