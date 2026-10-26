//! The PlayStation 2's recogniser: a disc, CD or DVD, whose ISO 9660 root holds SYSTEM.CNF with a BOOT2 line, its
//! serial the program BOOT2 names (Sapphire_Plan.md §4.1, Sapphire_Native.md §3.2). A disc with BOOT and no BOOT2 is
//! the PlayStation's, which `psx::disc` claims and this refuses.

use crate::media::ascii;
use crate::probe::{Head, Need, ROOT_FILE_LIMIT, Recognition, root_file_through};
use emusen_dotnet::text::{equals_ignore_case, trim};

/// The bytes read first: sector 16, the primary volume descriptor, in an .iso's 2,048-byte sectors or a raw track's 2,352-byte ones.
pub const FIRST: usize = 40 * 1024;

/// The value of SYSTEM.CNF's line whose key is `key`, as written after its `=`; none where no line has that key.
pub fn value(config: Option<&[u8]>, key: &str) -> Option<String> {
    let text = ascii(config?);
    text.split('\n').find_map(|line| {
        let eq = line.find('=')?;
        equals_ignore_case(trim(&line[..eq]), key).then(|| trim(&line[eq + 1..]).to_string())
    })
}

/// The program SYSTEM.CNF's BOOT2 line names; none where no line's key is BOOT2.
pub fn boot2(config: Option<&[u8]>) -> Option<String> {
    value(config, "BOOT2")
}

/// The market the disc was made for, in the PlayStation's recogniser's words: its serial's third letter, Sony's region
/// letter (U the Americas, E Europe, P, A, K and C Japan and the markets its consoles serve), else Europe where VMODE
/// says PAL; none otherwise (Sapphire_Native.md §49.2).
pub fn region(serial: Option<&str>, config: Option<&[u8]>) -> Option<String> {
    let market = match serial.and_then(|s| s.as_bytes().get(2)).map(u8::to_ascii_uppercase) {
        Some(b'U') => "Americas",
        Some(b'E') => "Europe",
        Some(b'P' | b'A' | b'K' | b'C') => "Japan",
        _ if value(config, "VMODE").is_some_and(|v| equals_ignore_case(&v, "PAL")) => "Europe",
        _ => return None,
    };
    Some(market.to_string())
}

/// The disc's claim with the serial BOOT2's program gives and the market of `region`; none for no PlayStation 2 disc.
pub fn recognise(head: &Head) -> Result<Option<Recognition>, Need> {
    let config = root_file_through(head, FIRST, "SYSTEM.CNF", ROOT_FILE_LIMIT)?;
    let Some(program) = boot2(config.as_deref()) else { return Ok(None) };
    let serial = crate::psx::disc::serial_of_file(&program);
    let region = region(serial.as_deref(), config.as_deref());
    Ok(Some(Recognition { system: "ps2".into(), serial, region }))
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

    #[test]
    fn the_market_is_the_serials_region_letter_else_vmodes_pal() {
        let ntsc: &[u8] = b"VMODE = NTSC\r\n";
        let pal: &[u8] = b"vmode=PAL\r\n";
        for (serial, config, market) in [
            (Some("SLUS-20312"), ntsc, Some("Americas")),
            (Some("SCUS-97124"), ntsc, Some("Americas")),
            (Some("SLES-51234"), pal, Some("Europe")),
            (Some("SCED-50001"), ntsc, Some("Europe")),
            (Some("SLPM-65001"), ntsc, Some("Japan")),
            (Some("SCAJ-20001"), ntsc, Some("Japan")),
            (Some("SLKA-25001"), ntsc, Some("Japan")),
            (Some("EMUS-00002"), ntsc, Some("Americas")),
            (None, pal, Some("Europe")),
            (Some("GAMEELF"), ntsc, None),
            (None, ntsc, None),
        ] {
            assert_eq!(region(serial, Some(config)).as_deref(), market, "{serial:?}");
        }
    }
}
