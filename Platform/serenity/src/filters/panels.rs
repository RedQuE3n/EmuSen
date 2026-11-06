//! The filters with fixed passes: crt-lottes, the handhelds' LCDs and the simple effects. See EmuSen_Serenity.md §3.4,
//! §3.5 and §3.7, and EmuSen_DevicePicture.md §3.2.

use super::number::fixed;
use super::{Filter, Pass, Scale, Setting, glsl, setting};

fn f(value: f64) -> String {
    fixed(value, 6)
}

fn rgb(rgb: i32) -> String {
    format!("vec3({}, {}, {})", f(((rgb >> 16) & 0xFF) as f64 / 255.0), f(((rgb >> 8) & 0xFF) as f64 / 255.0), f((rgb & 0xFF) as f64 / 255.0))
}

/// crt-lottes.slang's own parameters for the eleven the port kept, labelled for a person.
pub fn lottes_settings() -> Vec<Setting> {
    vec![
        setting("hardScan", "Scanline hardness", -8.0, -20.0, 0.0, 1.0),
        setting("hardPix", "Pixel hardness", -3.0, -20.0, 0.0, 1.0),
        setting("warpX", "Curvature, horizontal", 0.031, 0.0, 0.125, 0.01),
        setting("warpY", "Curvature, vertical", 0.041, 0.0, 0.125, 0.01),
        setting("maskDark", "Mask dark", 0.5, 0.0, 2.0, 0.1),
        setting("maskLight", "Mask light", 1.5, 0.0, 2.0, 0.1),
        setting("brightBoost", "Brightness boost", 1.0, 0.0, 2.0, 0.05),
        setting("hardBloomPix", "Bloom softness, horizontal", -1.5, -2.0, -0.5, 0.1),
        setting("hardBloomScan", "Bloom softness, vertical", -2.0, -4.0, -1.0, 0.1),
        setting("bloomAmount", "Bloom amount", 0.15, 0.0, 1.0, 0.05),
        setting("shape", "Filter kernel shape", 2.0, 0.0, 10.0, 0.05),
    ]
}

pub fn lottes() -> Filter {
    let consoles: Vec<&str> = super::screens::televisions();
    Filter::plain(
        "CRT (Lottes)",
        "CRT (Lottes)",
        vec![Pass::new(glsl::LOTTES.to_string(), Scale::Viewport)],
        Some(&consoles),
        "crt-lottes by Timothy Lottes, public domain, from libretro's slang-shaders",
        lottes_settings(),
    )
}

pub fn monochrome_panel(lightest: i32, light: i32, dark: i32, darkest: i32) -> String {
    glsl::MONOCHROME_PANEL.replace("P0", &rgb(lightest)).replace("P1", &rgb(light)).replace("P2", &rgb(dark)).replace("P3", &rgb(darkest))
}

pub fn dot_matrix(background: i32, coverage: f64) -> String {
    glsl::DOT_MATRIX.replace("BG", &rgb(background)).replace("COVER", &f(coverage))
}

pub fn colour_panel(m: &[f64; 9], luminance: f64) -> String {
    let mut text = glsl::COLOUR_PANEL.replace("LUM", &f(luminance));
    for (i, name) in ["M00", "M01", "M02", "M10", "M11", "M12", "M20", "M21", "M22"].iter().enumerate() {
        text = text.replace(name, &f(m[i]));
    }
    text
}

pub fn stripe_grid(floor: f64, gain: f64, gap: f64, row_dim: f64) -> String {
    glsl::STRIPE_GRID.replace("FLOOR", &f(floor)).replace("GAIN", &f(gain)).replace("GAP", &f(gap)).replace("ROWDIM", &f(row_dim))
}

/// SameBoy's measured palettes: the four shades and the colour of a switched-off panel.
pub const DMG: [i32; 5] = [0xC6DE8C, 0x84A563, 0x396139, 0x081810, 0xD2E6A6];
pub const POCKET: [i32; 5] = [0xC2CE93, 0x818D66, 0x3A4C3A, 0x07100E, 0xCFDAAC];
pub const LIGHT: [i32; 5] = [0x7FE2C3, 0x56B495, 0x357862, 0x0A1C15, 0x91EAD0];

/// Pokefan531's sRGB matrices, rows of output from linear input, and the luminance each is scaled by.
pub const GBC_GBA_MATRIX: [f64; 9] = [0.905, 0.195, -0.10, 0.10, 0.65, 0.25, 0.1575, 0.1425, 0.70];
pub const GBC_GBA_LUMINANCE: f64 = 0.91;
pub const SP101_MATRIX: [f64; 9] = [0.96, 0.11, -0.07, 0.0325, 0.89, 0.0775, 0.001, -0.03, 1.029];
pub const SP101_LUMINANCE: f64 = 0.935;

const GAME_BOY: &[&str] = &["GB"];
const GAME_BOY_ADVANCE: &[&str] = &["GBA"];

fn response(initial: f64) -> Setting {
    setting("response", "LCD response (ghosting)", initial as f32, 0.0, 0.9, 0.01)
}

fn shadow(initial: f64) -> Setting {
    setting("shadowOpacity", "Dot shadow", initial as f32, 0.0, 1.0, 0.05)
}

fn history(glsl: String, scale: Scale, history: i32) -> Pass {
    let mut pass = Pass::new(glsl, scale);
    pass.history = history;
    pass
}

fn monochrome(name: &str, p: [i32; 5], response_initial: f64, shadow_initial: f64) -> Filter {
    Filter::plain(
        name,
        name,
        vec![history(monochrome_panel(p[0], p[1], p[2], p[3]), Scale::Source, 3), Pass::new(dot_matrix(p[4], 0.86), Scale::Viewport)],
        Some(GAME_BOY),
        "palettes measured by SameBoy (LIJI32, MIT); response after Harlequin's gameboy shader; shadow after SameBoy's MonoLCD",
        vec![response(response_initial), shadow(shadow_initial)],
    )
}

fn colour(name: &str, matrix: &[f64; 9], luminance: f64, stripes: (f64, f64, f64, f64), consoles: &[&str], credit: &str, response_initial: f64) -> Filter {
    Filter::plain(
        name,
        name,
        vec![history(colour_panel(matrix, luminance), Scale::Source, 1), Pass::new(stripe_grid(stripes.0, stripes.1, stripes.2, stripes.3), Scale::Viewport)],
        Some(consoles),
        credit,
        vec![response(response_initial)],
    )
}

pub fn handhelds() -> Vec<Filter> {
    vec![
        monochrome("Game Boy LCD", DMG, 0.33, 0.25),
        monochrome("Game Boy Pocket LCD", POCKET, 0.12, 0.2),
        monochrome("Game Boy Light LCD", LIGHT, 0.12, 0.1),
        colour("Game Boy Color LCD", &GBC_GBA_MATRIX, GBC_GBA_LUMINANCE, (0.5, 1.25, 0.0, 0.85), GAME_BOY, "colour matrix measured by Pokefan531 (public domain); stripes after fishku's authentic_gbc (CC0)", 0.33),
        colour("Game Boy Advance LCD", &GBC_GBA_MATRIX, GBC_GBA_LUMINANCE, (0.45, 1.2, 0.25, 0.85), GAME_BOY_ADVANCE, "colour matrix measured by Pokefan531 (public domain); stripes and gap after mGBA's agb001 layout", 0.25),
        colour("Game Boy Advance SP (AGS-101) LCD", &SP101_MATRIX, SP101_LUMINANCE, (0.6, 1.15, 0.0, 0.9), GAME_BOY_ADVANCE, "colour matrix measured by Pokefan531 (public domain)", 0.0),
    ]
}

/// The simple effects, and the plain picture, as one-pass filters whose image is the frame stretched to the rectangle:
/// keyed `effect.<Name>.nearest` and `effect.<Name>.linear`.
pub fn effects() -> Vec<Filter> {
    let mut all = Vec::new();
    for bilinear in [false, true] {
        for (name, text) in [("None", glsl::PLAIN), ("Scanlines", glsl::SCANLINES), ("Crt", glsl::SIMPLE_CRT)] {
            let mut pass = Pass::new(text.to_string(), Scale::Viewport);
            pass.linear_source = bilinear;
            let key = format!("effect.{name}.{}", if bilinear { "linear" } else { "nearest" });
            let mut filter = Filter::plain(&key, name, vec![pass], None, "EmuSen's own", Vec::new());
            filter.rows_once = true;
            all.push(filter);
        }
    }
    all
}
