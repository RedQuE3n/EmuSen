//! Rows of bodies on a picker: where each tile sits and where the walk goes from it (EmuSen_Kit.md §7.19).
//! A leading row of wide tiles spans the margins; the bodies follow in centred rows of fixed tiles. It names no console.

/// The picker's measures in canvas units, as the courts canvas sets them (EmuSen_Interface.md §24.1, LunaP §222).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanetGeometry {
    pub margin: f64,
    pub gap: f64,
    pub lead_top: f64,
    pub lead_height: f64,
    pub tile_width: f64,
    pub tile_height: f64,
    pub top: f64,
    pub pitch: f64,
    /// Where a row's orbit line crosses its tiles, from the tile's top.
    pub orbit: f64,
    /// How far the focused tile rises.
    pub lift: f64,
}

impl PlanetGeometry {
    pub const COURTS: PlanetGeometry = PlanetGeometry {
        margin: 64.0,
        gap: 18.0,
        lead_top: 100.0,
        lead_height: 72.0,
        tile_width: 176.0,
        tile_height: 190.0,
        top: 186.0,
        pitch: 200.0,
        orbit: 52.0,
        lift: 6.0,
    };
}

/// One tile: its box, its row (0 for the leading row, then the bodies' rows from 1), and its neighbours by direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub row: usize,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub up: Option<usize>,
    pub down: Option<usize>,
}

/// The whole layout: every tile in index order (the leading row's first), and each bodies' row's orbit line.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlanetLayout {
    pub tiles: Vec<Placed>,
    pub orbits: Vec<f64>,
}

/// How many fixed tiles fit across a width between the margins.
pub fn capacity(width: f64, g: &PlanetGeometry) -> usize {
    (((width - 2.0 * g.margin + g.gap) / (g.tile_width + g.gap)).floor() as isize).max(1) as usize
}

/// Lays out `leading` wide tiles and `bodies` fixed ones across `width`: as few rows as fit, as even as they divide.
pub fn planet_layout(width: f64, leading: usize, bodies: usize, g: &PlanetGeometry) -> PlanetLayout {
    let mut rows: Vec<(usize, usize)> = Vec::new();
    if leading > 0 {
        rows.push((0, leading));
    }
    let mut tiles = Vec::with_capacity(leading + bodies);
    let lead_width = (width - 2.0 * g.margin - (leading.saturating_sub(1)) as f64 * g.gap) / leading.max(1) as f64;
    for i in 0..leading {
        let x = g.margin + i as f64 * (lead_width + g.gap);
        tiles.push(Placed { x, y: g.lead_top, width: lead_width, height: g.lead_height, row: 0, left: None, right: None, up: None, down: None });
    }
    let mut orbits = Vec::new();
    if bodies > 0 {
        let count = bodies.div_ceil(capacity(width, g));
        let per_row = bodies.div_ceil(count);
        for r in 0..count {
            let first = r * per_row;
            let length = per_row.min(bodies - first);
            let span = length as f64 * (g.tile_width + g.gap) - g.gap;
            let left = (width - span) / 2.0;
            let y = g.top + r as f64 * g.pitch;
            orbits.push(y + g.orbit);
            rows.push((leading + first, length));
            for k in 0..length {
                let x = left + k as f64 * (g.tile_width + g.gap);
                tiles.push(Placed { x, y, width: g.tile_width, height: g.tile_height, row: r + 1, left: None, right: None, up: None, down: None });
            }
        }
    }
    for (r, &(first, length)) in rows.iter().enumerate() {
        for i in first..first + length {
            tiles[i].left = (i > first).then(|| i - 1);
            tiles[i].right = (i + 1 < first + length).then_some(i + 1);
            let centre = tiles[i].x + tiles[i].width / 2.0;
            tiles[i].up = r.checked_sub(1).map(|above| nearest(&tiles, rows[above], centre));
            tiles[i].down = rows.get(r + 1).map(|&below| nearest(&tiles, below, centre));
        }
    }
    PlanetLayout { tiles, orbits }
}

/// The tile of a row whose centre is nearest a point across; the first of two at the same distance.
fn nearest(tiles: &[Placed], (first, length): (usize, usize), centre: f64) -> usize {
    let distance = |i: usize| (tiles[i].x + tiles[i].width / 2.0 - centre).abs();
    (first..first + length).fold(first, |best, i| if distance(i) < distance(best) { i } else { best })
}
