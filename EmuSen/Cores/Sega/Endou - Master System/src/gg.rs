//! Jadeite, the Game Gear's module (Jadeite_Native.md §2-§3): its picture on its LCD. In its own mode the VDP's fixed
//! window of 160 by 144 in 12-bit colours; in a Master System cartridge's mode the whole picture shrunk onto the panel's
//! 160 by 146, three columns into two by their colour elements and three lines into two with their neighbours
//! (Endou_Disputes.md D-22). The VDP's picture is CRAM entries; the colours are `beryl-sms-colours`'.

use beryl_sms_colours::{SMS_COLOURS, gg_colour};
use beryl_sms_vdp::{GG_WINDOW, gg_window_top};

/// The panel's columns, and its rows: GG mode lights the central 144, a Master System cartridge's mode all 146 [R SMS
/// Power!'s "Game Gear LCD", "Pixel Aspect Ratio"].
pub const WIDTH: u32 = 160;
pub const WINDOW_HEIGHT: u32 = 144;
pub const PANEL_HEIGHT: u32 = 146;

/// The VDP's picture lines a Master System cartridge's mode shows: 218, the 224-line display less three at each end,
/// the 192-line display with 13 of border at each end (D-22).
const LINES: i32 = 218;

/// The columns of the VDP's 256 shown in that mode: the middle 240 [R MacDonald's `msvdp` §14].
const FIRST_COLUMN: usize = 8;

/// GG mode: the window of the VDP's 256-wide `picture` of `height` lines of 12-bit entries, into `out` as RGBA, 160 by
/// 144; a 224-line display's from line 40 (D-21).
pub fn window(picture: &[u16], height: u32, out: &mut [u8]) {
    let (x0, _, w, h) = GG_WINDOW;
    let rows = picture.chunks_exact(256).skip(gg_window_top(height) as usize).take(h as usize);
    for (row, line) in out.chunks_exact_mut(w as usize * 4).zip(rows) {
        for (px, &e) in row.chunks_exact_mut(4).zip(&line[x0 as usize..(x0 + w) as usize]) {
            px.copy_from_slice(&gg_colour(e));
        }
    }
}

/// A Master System cartridge's mode: each line of the picture shrunk to 160 columns once, then the panel's rows made
/// from them. The rows are kept so that a frame allocates nothing.
#[derive(Clone, Debug)]
pub struct Shrink {
    rows: Vec<[u8; 3 * WIDTH as usize]>,
    edge: [u8; 3 * WIDTH as usize],
}

impl Default for Shrink {
    fn default() -> Shrink {
        Shrink { rows: vec![[0; 3 * WIDTH as usize]; 240], edge: [0; 3 * WIDTH as usize] }
    }
}

/// Three pixels into two by their colour elements: red and green of the first and blue of the second, then red of the
/// second and green and blue of the third [R SMS Power!'s "Game Gear LCD"; Maxim, SMS Power! forum topic 9562].
fn columns(line: &[u16], out: &mut [u8; 3 * WIDTH as usize]) {
    let rgb = |e: u16| SMS_COLOURS[e as usize & 0x3F];
    for (pair, three) in out.chunks_exact_mut(6).zip(line[FIRST_COLUMN..FIRST_COLUMN + 240].chunks_exact(3)) {
        let (a, b, c) = (rgb(three[0]), rgb(three[1]), rgb(three[2]));
        pair.copy_from_slice(&[a[0], a[1], b[2], b[0], c[1], c[2]]);
    }
}

impl Shrink {
    /// The picture, `height` lines of 256 `--BBGGRR` entries, onto the panel into `out` as RGBA, 160 by 146; a line
    /// outside the display is the `border` entry. Of each three lines from the window's first, the panel's even row
    /// takes the line before and the first two, its odd row the three and the line after, weighted 1, 2, 2, 1 in sixths
    /// (D-22).
    pub fn draw(&mut self, picture: &[u16], height: u32, border: u16, out: &mut [u8]) {
        for (row, line) in self.rows.iter_mut().zip(picture.chunks_exact(256)) {
            columns(line, row);
        }
        let b = SMS_COLOURS[border as usize & 0x3F];
        for px in self.edge.chunks_exact_mut(3) {
            px.copy_from_slice(&b[..3]);
        }
        let first = (height as i32 - LINES) / 2;
        let line = |n: i32| -> &[u8] { if (0..height as i32).contains(&n) { &self.rows[n as usize] } else { &self.edge } };
        for (y, row) in out.chunks_exact_mut(4 * WIDTH as usize).take(PANEL_HEIGHT as usize).enumerate() {
            let s = first + 3 * (y as i32 / 2) + if y % 2 == 0 { -1 } else { 0 };
            let taps = [line(s), line(s + 1), line(s + 2), line(s + 3)];
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                for (k, out) in px[..3].iter_mut().enumerate() {
                    let i = 3 * x + k;
                    let sum = taps[0][i] as u32 + 2 * taps[1][i] as u32 + 2 * taps[2][i] as u32 + taps[3][i] as u32;
                    *out = ((sum + 3) / 6) as u8;
                }
                px[3] = 255;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::Machine;
    use crate::media::{Console, Market, Media};
    use crate::model::Model;
    use crate::programs::images::{self, Program};

    /// A test program on the Game Gear, as the cartridge `console` it is: a Game Gear cartridge in its own mode, a Master
    /// System cartridge in the Master System's.
    fn game_gear(p: &Program, console: Console, frames: usize) -> Machine {
        let rom = images::image(p);
        let media = Media { console, skipped: 0, header: None };
        let board = crate::board::identify(&rom, console);
        let mut m = Machine::with_board(media, Market::Americas, Model::GameGear, board, rom);
        for _ in 0..frames {
            m.advance();
        }
        m
    }

    fn digest(m: &Machine) -> u64 {
        m.frame_bytes().iter().fold(0xCBF2_9CE4_8422_2325, |h, &b| (h ^ b as u64).wrapping_mul(0x100_0000_01B3))
    }

    fn at(m: &Machine, x: usize, y: usize) -> [u8; 3] {
        let i = (y * m.width as usize + x) * 4;
        [m.picture[i], m.picture[i + 1], m.picture[i + 2]]
    }

    // Equal to Genesis Plus GX's, SMS Plus GX's, PicoDrive's and BlastEm's up to a colour map but for the line the line
    // interrupt's write lands on, measured 2026-10-20 (Jadeite_Native.md §4).
    #[test]
    fn a_game_gear_program_shows_its_window_in_twelve_bits_and_moves_it_for_224_lines() {
        let (yellow, blue, green, magenta, white) = ([255, 255, 0], [0, 0, 255], [0, 255, 0], [255, 0, 255], [255, 255, 255]);
        let m = game_gear(&images::GG, Console::Gg, 10);
        assert_eq!((m.width, m.height, m.cram_len(), digest(&m)), (160, 144, 64, 0xD866_9150_8E48_9726));
        assert_eq!((at(&m, 0, 0), at(&m, 159, 143), at(&m, 8, 8)), (yellow, yellow, blue), "the edge cells, the latch's entry 2, and inside");
        assert_eq!((at(&m, 3, 76), at(&m, 4, 76), at(&m, 8, 76), at(&m, 8, 75)), (green, yellow, magenta, blue), "the sprite's last four columns; line 100's colour");
        assert!(m.frame_bytes().chunks(4).all(|p| p[..3] != white), "nothing outside the window shows");
        assert_eq!(m.bus.ram[0], 0xC1, "the frame interrupt on line $C1");
        let m = game_gear(&images::GG, Console::Gg, 40);
        assert_eq!((m.height, m.bus.vdp.height, m.bus.ram[0], digest(&m)), (144, 224, 0xE1, 0x978D_F17F_8DB5_5E0D));
        assert_eq!((at(&m, 0, 15), at(&m, 0, 16)), (white, yellow), "lines 40-55 of the taller layout, outside the old window (D-21)");
    }

    // Equal to Genesis Plus GX's, SMS Plus GX's, PicoDrive's and BlastEm's up to a colour map, measured 2026-10-20.
    #[test]
    fn the_tms9918_s_codes_show_as_cram_entries_16_to_31_in_the_game_gear_s_mode() {
        let m = game_gear(&images::GG_TMS, Console::Gg, 10);
        assert_eq!((at(&m, 0, 0), at(&m, 4, 0), at(&m, 159, 143), digest(&m)), ([51, 34, 17], [102, 85, 68], [102, 85, 68], 0x3CAC_C94D_7C2C_3525));
    }

    // The VDP's picture equals Gearsystem's and SMS Plus GX's in their Game Gear's Master System mode, and a second
    // writing of D-22 on their pictures gives this picture, measured 2026-10-20 (Jadeite_Native.md §4).
    #[test]
    fn a_master_system_cartridge_is_shrunk_onto_the_panel_in_both_heights() {
        let m = game_gear(&images::GG_SMS, Console::Sms, 10);
        assert_eq!((m.width, m.height, m.cram_len(), m.bus.vdp.height, digest(&m)), (160, 146, 32, 192, 0x3911_2A78_3903_8EB5));
        assert!((0..8).chain(138..146).all(|y| at(&m, 37, y) == [0, 0, 255]), "the border's eight rows at each end");
        let m = game_gear(&images::GG_SMS, Console::Sms, 40);
        assert_eq!((m.height, m.bus.vdp.height, digest(&m)), (146, 224, 0x93BD_DDD5_76DB_1715));
        assert!((0..146).all(|y| at(&m, 37, y) != [0, 0, 255]), "224 lines fill the panel");
    }

    // The state carries the mode in the VDP's block; a state of the other cartridge's is the other console's, and refused.
    #[test]
    fn a_game_gear_state_round_trips_in_either_mode() {
        use emusen_native::ffi::StateMachine;
        let mut states = Vec::new();
        for (p, console) in [(&images::GG, Console::Gg), (&images::GG_SMS, Console::Sms)] {
            let mut a = game_gear(p, console, 20);
            let mut s = vec![0; a.state_size()];
            a.save_state(&mut s).unwrap();
            let mut b = game_gear(p, console, 0);
            b.load_state(&s).unwrap();
            for _ in 0..25 {
                a.advance();
                b.advance();
            }
            assert_eq!((digest(&a), b.bus.vdp.gg_mode), (digest(&b), console == Console::Gg));
            states.push(s);
        }
        assert!(game_gear(&images::GG, Console::Gg, 0).load_state(&states[1]).is_err());
    }

    /// A 256-wide picture of `height` lines of entry `bg`, with line `lit` (if any) of entry `fg`.
    fn picture(height: u32, bg: u16, lit: Option<usize>, fg: u16) -> Vec<u16> {
        let mut p = vec![bg; 256 * height as usize];
        if let Some(l) = lit {
            p[l * 256..(l + 1) * 256].fill(fg);
        }
        p
    }

    fn column(out: &[u8], y: usize) -> u8 {
        out[y * 4 * WIDTH as usize + 4 * 80]
    }

    #[test]
    fn gg_mode_shows_the_central_160_by_144_in_twelve_bits() {
        let mut p = vec![0u16; 256 * 192];
        p[24 * 256 + 48] = 0x0F00;
        p[167 * 256 + 207] = 0x00F0;
        p[23 * 256 + 48] = 0x0FFF;
        p[24 * 256 + 47] = 0x0FFF;
        let mut out = vec![0u8; 160 * 144 * 4];
        window(&p, 192, &mut out);
        assert_eq!(&out[..4], &[0, 0, 255, 255], "line 24, column 48 is the LCD's first pixel [R Sega's Game Gear VDP manual]");
        assert_eq!(&out[out.len() - 4..], &[0, 255, 0, 255], "line 167, column 207 its last");
        assert!(out.chunks(4).filter(|p| p[..3] == [255, 255, 255]).count() == 0, "nothing outside the window shows");
    }

    /// Three columns into two: a white column of each phase shows as two colour elements, never white.
    #[test]
    fn three_columns_become_two_by_their_colour_elements() {
        let white = 0x3F;
        let mut row = [0u8; 3 * WIDTH as usize];
        for phase in 0..3 {
            let mut line = [0u16; 256];
            line[FIRST_COLUMN + 30 + phase] = white;
            columns(&line, &mut row);
            let shown: Vec<[u8; 3]> = row.chunks(3).map(|p| [p[0], p[1], p[2]]).filter(|p| *p != [0, 0, 0]).collect();
            let want: &[[u8; 3]] = match phase {
                0 => &[[255, 255, 0]],
                1 => &[[0, 0, 255], [255, 0, 0]],
                _ => &[[0, 255, 255]],
            };
            assert_eq!(shown, want, "phase {phase}");
        }
        let mut line = [0u16; 256];
        line[7] = white;
        line[248] = white;
        columns(&line, &mut row);
        assert!(row.iter().all(|&c| c == 0), "the first and last eight columns are not shown");
    }

    /// A single lit line of the 224-line display against benryves's measured responses (forum topic 9562, 2007): line
    /// 3, the window's first, bright on row 0 and dim on row 1; line 4 bright on rows 0 and 1; line 5 dim on 0, bright
    /// on 1 and dim on 2; line 2 dim on row 0, and at the panel's foot lines 220 to 222 on its last two rows, which
    /// benryves's "221 to 223 do not appear to have any influence" disputes for line 221 (D-22).
    #[test]
    fn a_224_line_display_s_lines_light_the_rows_benryves_measured() {
        let mut s = Shrink::default();
        let mut out = vec![0u8; 160 * 146 * 4];
        let rows = |s: &mut Shrink, out: &mut Vec<u8>, lit: usize| -> Vec<(usize, u8)> {
            s.draw(&picture(224, 0, Some(lit), 0x3F), 224, 0, out);
            (0..146).map(|y| (y, column(out, y))).filter(|r| r.1 != 0).collect()
        };
        assert_eq!(rows(&mut s, &mut out, 3), [(0, 85), (1, 43)]);
        assert_eq!(rows(&mut s, &mut out, 4), [(0, 85), (1, 85)]);
        assert_eq!(rows(&mut s, &mut out, 5), [(0, 43), (1, 85), (2, 43)]);
        assert_eq!(rows(&mut s, &mut out, 2), [(0, 43)]);
        assert_eq!(rows(&mut s, &mut out, 220), [(144, 85), (145, 85)]);
        assert_eq!(rows(&mut s, &mut out, 221), [(144, 43), (145, 85)]);
        assert_eq!(rows(&mut s, &mut out, 222), [(145, 43)]);
        assert_eq!(rows(&mut s, &mut out, 223), []);
    }

    /// The 192-line display: line 0 on rows 8 and 9, as enik's comparisons with photographs found it (forum topic 9562,
    /// 2016), and eight rows of border at each end, as benryves's photographs showed.
    #[test]
    fn a_192_line_display_is_letterboxed_by_its_border() {
        let mut s = Shrink::default();
        let mut out = vec![0u8; 160 * 146 * 4];
        s.draw(&picture(192, 0x00, Some(0), 0x3F), 192, 0x00, &mut out);
        let lit: Vec<(usize, u8)> = (0..146).map(|y| (y, column(&out, y))).filter(|r| r.1 != 0).collect();
        assert_eq!(lit, [(8, 85), (9, 85)], "a one-line hyphen shown on two rows alike, as enik found Altered Beast's");
        s.draw(&picture(192, 0x00, None, 0), 192, 0x30, &mut out);
        let border: Vec<usize> = (0..146).filter(|&y| out[y * 640 + 2] == 255).collect();
        assert_eq!(border, (0..8).chain(138..146).collect::<Vec<_>>(), "only the border's rows are all border");
        assert!(out.chunks(4).all(|p| p[3] == 255));
    }
}
