//! The controller drawing (EmuSen_Kit.md §7.62): a controller drawn from a drawing file, its regions named by the
//! controls a host's controller declares, laid out in a box with a label and a leader for each, and lit by the focus
//! and the controls held. A controller with no drawing of its own, or one whose drawing lacks a declared control, is
//! drawn as the generic controller, derived from the declared controls alone. No Qt here, and no controller is named:
//! which file stands for which controller is the host's.

pub mod drawing;
pub mod layout;
pub mod shape;

pub use drawing::{Drawing, Part, Region, Side, Stick, Word};
pub use layout::{data_url, lay_out, paints, svg, walk, words, Laid, Lit, Paint};
pub use shape::{Bounds, Shape};

/// The generic controller's own drawing: a two-stick pad with every RetroPad control, as libretro names them, and
/// each stick's directions by its axes (`LeftY-` is up).
pub const GENERIC: &str = include_str!("generic.json");

/// Where a control the generic drawing has no place for stands: in a row under the shell, this far apart.
const EXTRA_TOP: f64 = 640.0;
const EXTRA_STEP: f64 = 96.0;
const EXTRA_RADIUS: f64 = 30.0;

/// The generic drawing for a controller declaring `controls`: the generic pad, with a round button under it for each
/// declared control it has no region for, named by the control.
pub fn generic(controls: &[String]) -> Drawing {
    let mut d = Drawing::parse(GENERIC).expect("the generic drawing reads");
    let extra: Vec<&String> = controls.iter().filter(|c| d.region(c).is_none()).collect();
    if !extra.is_empty() {
        let first = (d.width - EXTRA_STEP * (extra.len() as f64 - 1.0)) / 2.0;
        for (i, control) in extra.iter().enumerate() {
            let shape = Shape::Circle { cx: first + EXTRA_STEP * i as f64, cy: EXTRA_TOP, r: EXTRA_RADIUS };
            d.regions.push(Region {
                control: (*control).clone(),
                name: (*control).clone(),
                shape,
                fill: Some("#2E2F35".into()),
                stroke: Some("#3E4048".into()),
                width: 3.0,
                arm: false,
                caption: None,
                mark: None,
                behind: false,
                side: Some(Side::Bottom),
            });
        }
        d.width = d.width.max(EXTRA_STEP * extra.len() as f64);
        d.height = d.height.max(EXTRA_TOP + EXTRA_RADIUS + 8.0);
    }
    d
}

/// The drawing that stands for a controller: its own file where the file reads and has a region for every declared
/// control, else the generic drawing; and whether it is the generic one.
pub fn drawing_for(file: Option<&str>, controls: &[String]) -> (Drawing, bool) {
    match file.map(Drawing::parse) {
        Some(Ok(d)) if layout::covers(&d, controls) => (d, false),
        _ => (generic(controls), true),
    }
}
