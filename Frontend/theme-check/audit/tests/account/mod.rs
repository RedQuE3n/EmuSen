// The parts theme-audit's tests run side by side (man theme-check, PARTS).

use emusen_qt_test::harness::SIZES;

/// One part of a run: one window size without the crowd (`--sizes WxH --crowd without`), or the crowd at every size (`--crowd only`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Size(i32, i32),
    Crowd,
}

/// Every part, the sizes in the gate's order and the crowd last.
pub fn parts() -> Vec<Part> {
    SIZES.iter().map(|&(w, h)| Part::Size(w, h)).chain([Part::Crowd]).collect()
}
