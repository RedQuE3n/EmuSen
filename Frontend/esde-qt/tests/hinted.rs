//! The reference glyph hinted by the system's FreeType at every help size measured on ES-DE 3.4.1's captures (EmuSen_Qt.md
//! §4.12.1): CarAlt's rost (CFF) and ES-DE's own Akrobat (TrueType, the bytecode hinter's heights). No Qt, no GPU.

mod common;

use std::path::PathBuf;

use emusen_esde::fonts::REFERENCE;
use emusen_esde_qt::hinted::rows;

fn rost() -> String {
    common::corpus().join("caralt-es-de/assets/fonts/rost.otf").to_string_lossy().into_owned()
}

fn akrobat() -> String {
    let root = PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/emusen/esde-oracle/bin/AppDir/usr/share/es-de/resources");
    emusen_esde::fonts::locate(&[root.join("fonts")]).expect("ES-DE's own face in the oracle's ES-DE").to_string_lossy().into_owned()
}

fn heights(path: &str, sizes: &[f64]) -> Vec<(f64, f64)> {
    let s = REFERENCE.chars().next().unwrap();
    sizes.iter().map(|&px| (px, f64::from(rows(path, px as f32, s).expect("FreeType").expect("the face's S")))).collect()
}

#[test]
fn the_reference_glyph_is_hinted_at_every_measured_size_in_both_faces() {
    // rost: the copies' sizes (fontSize × height); the step between 20.16 and 20.32 px is the box's on the captures.
    let rost_sizes = [11.52, 12.0, 12.8, 13.6, 14.4, 15.2, 16.0, 17.28, 17.6, 19.2, 19.36, 19.52, 19.68, 19.84, 20.0, 20.16, 20.32, 20.4, 20.48, 20.64, 20.8, 22.8];
    let rost_want = [8.0, 9.0, 9.0, 10.0, 10.0, 11.0, 12.0, 12.0, 13.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 15.0, 15.0, 15.0, 15.0, 15.0, 16.0];
    // Akrobat: the own-face copies' sizes; 9 and 10 at 11.52 and 13.6 px are the bytecode hinter's, light hinting's 8 and 9.
    let akrobat_sizes = [11.52, 12.8, 13.6, 14.4, 15.2, 16.0, 19.2, 19.36, 19.52, 19.68, 19.84, 20.0, 20.16, 20.32, 20.48, 20.64, 20.8];
    let akrobat_want = [9.0, 9.0, 10.0, 10.0, 10.0, 12.0, 13.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0, 14.0];
    let want = |sizes: &[f64], hs: &[f64]| sizes.iter().copied().zip(hs.iter().copied()).collect::<Vec<_>>();
    assert_eq!(
        (heights(&rost(), &rost_sizes), heights(&akrobat(), &akrobat_sizes)),
        (want(&rost_sizes, &rost_want), want(&akrobat_sizes, &akrobat_want)),
        "the S hinted in rost and in Akrobat"
    );
}

// Art Book Next's help at 1280 × 800, in ES-DE's help face: 0.04583333 × 800 in f32, 36.666664 px, is 2346/64 truncated as ES-DE gives it to
// FreeType, where the S renders 27 rows; rounded to 2347/64 it renders 26 (EmuSen_Qt.md §4.12.4). At 1920 × 1200 both give 41.
#[test]
fn the_size_is_truncated_to_sixty_fourths_where_the_hinter_turns() {
    let s = REFERENCE.chars().next().unwrap();
    let semibold = std::path::Path::new(&akrobat()).with_file_name(emusen_esde::fonts::HELP_FILE).to_string_lossy().into_owned();
    let at = |px: f32| rows(&semibold, px, s).expect("FreeType").expect("the face's S");
    assert_eq!((at(0.045_833_33_f32 * 800.0), at(2347.0 / 64.0), at(0.045_833_33_f32 * 1200.0)), (27, 26, 41));
}
