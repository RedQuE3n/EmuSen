//! The Master System's and the SG-1000's two code formats, from SMS Power!'s "Pro Action Replay" and "Game Genie" pages
//! (EmuSen_Cheats.md §11): Datel's Pro Action Replay, a RAM poke, and Galoob's Game Genie, a ROM-read intercept whose
//! cipher is the Game Boy's.

use crate::cheats::{Decoded, format_error, genie_cipher, hex, is_hex};

/// The space a Pro Action Replay code writes: the console's work RAM, as Endou names it.
pub const RAM: &str = "RAM";

/// The work RAM's 8 KiB, which a Pro Action Replay address from $C000 reaches through its mirrors.
const RAM_MASK: u32 = 0x1FFF;

/// The code's hex digits, upper case, where there are as many as the format takes and nothing else but dashes and
/// spaces: a database's `??` for a value the player picks is no digit to drop.
fn digits(code: &str, lengths: &[usize]) -> Option<Vec<char>> {
    if code.chars().any(|c| !(is_hex(c) || c == '-' || c == ' ')) {
        return None;
    }
    let hex: Vec<char> = code.chars().filter(|&c| is_hex(c)).map(|c| c.to_ascii_uppercase()).collect();
    lengths.contains(&hex.len()).then_some(hex)
}

/// `00AA-AADD`: eight digits whose middle four are an address in the work RAM, $C000 or above.
fn par_parts(code: &str) -> Option<(u32, u8)> {
    let d: String = digits(code, &[8])?.into_iter().collect();
    let address = hex(&d[2..6]);
    (address >= 0xC000).then_some((address, hex(&d[6..8]) as u8))
}

pub fn par_can_decode(code: &str) -> bool {
    par_parts(code).is_some()
}

/// The work RAM's offset and the value; the first two digits, which the page says have no effect, are not read.
pub fn par_decode(code: &str) -> Decoded<(i32, u8)> {
    match par_parts(code) {
        Some((address, value)) => Ok(((address & RAM_MASK) as i32, value)),
        None => format_error(format!("'{code}' isn't a valid Pro Action Replay code - expected 8 hex digits (00AA-AADD) with an address from $C000.")),
    }
}

fn genie_digits(code: &str) -> Decoded<Vec<char>> {
    match digits(code, &[6, 9]) {
        Some(d) => Ok(d),
        None => format_error(format!("'{code}' isn't a valid Game Genie code - expected 6 or 9 hex digits (DDA-AAA-RRR).")),
    }
}

pub fn genie_can_decode(code: &str) -> bool {
    digits(code, &[6, 9]).is_some()
}

pub fn genie_decode(code: &str) -> Decoded<(i32, u8)> {
    let (address, value, _) = genie_cipher(&genie_digits(code)?);
    Ok((address, value))
}

/// A nine-digit code's reference byte; none for six digits.
pub fn genie_compare(code: &str) -> Decoded<Option<u8>> {
    Ok(genie_cipher(&genie_digits(code)?).2)
}

#[cfg(test)]
mod tests {
    use super::*;

    // SMS Power!'s page: the address's last digit moved to the front and inverted, the reference the sixth and ninth
    // digits rotated right by two and XORed with $BA.
    #[test]
    fn a_game_genie_code_decodes_as_the_page_gives_it() {
        let code = 0x3A6_4AB_2A2u64;
        let address = ((code >> 16) & 0xFFF) | ((code & 0xF000) ^ 0xF000);
        let reference = (((code >> 2) & 0x03) | ((code >> 6) & 0x3C) | ((code << 6) & 0xC0)) as u8 ^ 0xBA;
        assert_eq!(genie_decode("3A6-4AB-2A2"), Ok((address as i32, 0x3A)));
        assert_eq!(address, 0x464A);
        assert_eq!(genie_compare("3A6-4AB-2A2"), Ok(Some(reference)));
        assert_eq!(genie_compare("3A6-4AB"), Ok(None));
        assert!(genie_can_decode("3a6 4ab") && !genie_can_decode("3A6-4AB-2A") && genie_decode("XYZ").is_err());
    }

    #[test]
    fn a_pro_action_replay_code_is_a_work_ram_poke_from_c000() {
        assert_eq!(par_decode("00C0-DE03"), Ok((0x00DE, 0x03)));
        assert_eq!(par_decode("20D5-BD07"), Ok((0x15BD, 0x07)), "the first two digits are not read");
        assert_eq!(par_decode("00FF-FC01"), Ok((0x1FFC, 0x01)), "the mirror above $E000");
        assert!(!par_can_decode("0080-0001") && par_decode("0080-0001").is_err(), "no RAM below $C000");
        assert!(!par_can_decode("3A6-4AB-2A2") && !genie_can_decode("00C0-DE03"), "the two formats by their digits");
        assert!(!genie_can_decode("00D5-BD??") && !par_can_decode("00D5-BD0X"), "a placeholder is not dropped to leave a shorter code");
    }
}
