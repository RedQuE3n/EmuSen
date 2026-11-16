//! A picture's hues, most prominent first: what a theme colours a game's corona and its backdrop's fields with
//! (EmuSen_Qt.md §3.4.8, R-20261115-27, -28). Read once a picture, off the GUI thread, by the caller's cache.

use cxx_qt_lib::{QImage, TransformationMode};
use emusen_kit_core::tokens::Colour;

/// The width a picture is read at: enough pixels to weigh its colours, few enough to read in a moment; a constant of the method.
pub const READ_WIDTH: i32 = 64;

/// A pixel weighs towards a hue when it has colour (saturation at least this) and is neither near black nor near white.
pub const LEAST_SATURATION: f64 = 0.25;
pub const LIGHTNESS: (f64, f64) = (0.12, 0.88);

/// The hue circle in bins of this many degrees.
pub const BIN: f64 = 10.0;

/// Two hues listed are at least this far apart: the board's nearest two, 218 and 251 degrees (Main.dc.html:20-21), are 33 apart.
pub const DISTINCT: f64 = 30.0;

/// A hue is listed only when it carries at least this share of the picture's colour.
pub const LEAST_SHARE: f64 = 0.05;

/// The hues of a picture's pixels, most prominent first, at most `most`, each `DISTINCT` from those before it; none for a
/// picture with no colour (greys, black, white).
pub fn hues(pixels: &[Colour], most: usize) -> Vec<f64> {
    let bins = (360.0 / BIN) as usize;
    // Each bin's weight and the weighted sum of its hues' unit vectors, so a bin's hue is its pixels' circular mean.
    let mut weight = vec![0.0; bins];
    let mut sum = vec![(0.0, 0.0); bins];
    for pixel in pixels.iter().filter(|p| p.a > 0) {
        let (hue, saturation, lightness) = pixel.to_hsl();
        if saturation < LEAST_SATURATION || lightness < LIGHTNESS.0 || lightness > LIGHTNESS.1 {
            continue;
        }
        let bin = ((hue / BIN) as usize).min(bins - 1);
        let w = saturation * pixel.alpha();
        weight[bin] += w;
        sum[bin].0 += w * hue.to_radians().cos();
        sum[bin].1 += w * hue.to_radians().sin();
    }
    let total: f64 = weight.iter().sum();
    if total <= 0.0 {
        return Vec::new();
    }
    // A bin's prominence is its own weight and its two neighbours', so a hue split across a bin's edge is one hue.
    let around = |i: usize| [(i + bins - 1) % bins, i, (i + 1) % bins];
    let mut listed: Vec<f64> = Vec::new();
    let mut order: Vec<usize> = (0..bins).collect();
    order.sort_by(|&a, &b| {
        let w = |i: usize| around(i).iter().map(|&j| weight[j]).sum::<f64>();
        w(b).total_cmp(&w(a)).then(a.cmp(&b))
    });
    for i in order {
        if listed.len() == most {
            break;
        }
        let (w, x, y) = around(i).iter().fold((0.0, 0.0, 0.0), |(w, x, y), &j| (w + weight[j], x + sum[j].0, y + sum[j].1));
        if weight[i] <= 0.0 || w < LEAST_SHARE * total {
            continue;
        }
        let hue = (y.atan2(x).to_degrees() + 360.0) % 360.0;
        if listed.iter().all(|h| emusen_kit_core::world::wrap_hue(hue - h).abs() >= DISTINCT) {
            listed.push(hue);
        }
    }
    listed
}

/// The hues of the picture in the file at `path`, read at `READ_WIDTH` with Qt's image readers; none for a file Qt cannot read.
pub fn hues_of_file(path: &str, most: usize) -> Vec<f64> {
    let Ok(data) = std::fs::read(path) else { return Vec::new() };
    let Some(image) = QImage::from_data(&data, None) else { return Vec::new() };
    let small = if image.width() > READ_WIDTH { image.scaled_to_width(READ_WIDTH, TransformationMode::SmoothTransformation) } else { image };
    let mut pixels = Vec::with_capacity((small.width() * small.height()).max(0) as usize);
    for y in 0..small.height() {
        for x in 0..small.width() {
            let c = small.pixel_color(x, y);
            pixels.push(Colour { r: c.red() as u8, g: c.green() as u8, b: c.blue() as u8, a: c.alpha() as u8 });
        }
    }
    hues(&pixels, most)
}
