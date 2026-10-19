//! The open firmware's own font: 47 glyphs of five dots by seven, drawn for Nephrite's open Sega CD BIOS and shared by
//! Endou's open boot program, so that one crate owns them (Beryl_Font.md). Each core's `build.rs` makes its console's
//! tiles from the glyphs; nothing here knows a console.

/// The glyphs as drawn, one a line: the character (`space` for a space) and seven rows of five dots.
pub const SOURCE: &str = include_str!("font.txt");

/// A glyph: its character and its seven rows, each row's five dots in bits 4-0, the leftmost in bit 4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Glyph {
    pub char: u8,
    pub rows: [u8; 7],
}

/// The glyphs in the source's order.
pub fn glyphs() -> impl Iterator<Item = Glyph> {
    SOURCE.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()).map(|line| {
        let mut f = line.split_whitespace();
        let name = f.next().expect("a character");
        let char = if name == "space" { b' ' } else { name.as_bytes()[0] };
        let mut rows = [0u8; 7];
        for (row, hex) in rows.iter_mut().zip(f) {
            *row = u8::from_str_radix(hex, 16).expect("a row of dots");
        }
        Glyph { char, rows }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_is_printable_ascii_once_with_seven_rows_of_five_dots() {
        let all: Vec<Glyph> = glyphs().collect();
        assert_eq!(all.len(), 47);
        let mut seen = [false; 128];
        for g in &all {
            assert!((0x20..0x60).contains(&g.char), "{}", g.char as char);
            assert!(!std::mem::replace(&mut seen[g.char as usize], true), "{} twice", g.char as char);
            assert!(g.rows.iter().all(|&r| r < 0x20), "{}", g.char as char);
        }
        for c in (b'A'..=b'Z').chain(b'0'..=b'9').chain(*b" .,-:!?/()'") {
            assert!(seen[c as usize], "{} missing", c as char);
        }
        assert_eq!(all.iter().find(|g| g.char == b'1').unwrap().rows, [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E]);
    }
}
