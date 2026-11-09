//! Groups of bodies on nested half-ellipses: where each body sits, which three groups are shown, how a long group turns, and
//! where the walk goes (EmuSen_Kit.md §7.30). Bodies come as group keys in a list's order; it names no console and no maker.

use std::f64::consts::PI;

/// The orbits' measures in canvas units, as the reference theme's Choose a console sets them (EmuSen_Kit.md §7.30).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitGeometry {
    /// Slot 0's radii, and what each slot outward adds.
    pub first_rx: f64,
    pub first_ry: f64,
    pub step_rx: f64,
    pub step_ry: f64,
    /// Bodies' spacing along an orbit, at its mean radius.
    pub pitch: f64,
    /// How many slots are on screen.
    pub slots: i32,
    /// A body's centre stays below this line and this far inside each side; on the innermost slot below `inner_top`,
    /// so a focused body's ring clears the leading row (decided 2026-11-08).
    pub top: f64,
    pub inner_top: f64,
    pub side: f64,
    /// A group's label: this far in from its window's end toward the centre, right-aligned no further right than the
    /// width less `label_margin`, its box `label_half` above the point and never above `label_top`.
    pub label_in: f64,
    pub label_margin: f64,
    pub label_half: f64,
    pub label_top: f64,
    /// The ellipses past a window's end are drawn only where the arc is below this line.
    pub more_below: f64,
}

impl OrbitGeometry {
    pub const REFERENCE: OrbitGeometry = OrbitGeometry {
        first_rx: 260.0,
        first_ry: 220.0,
        step_rx: 180.0,
        step_ry: 150.0,
        pitch: 150.0,
        slots: 3,
        top: 178.0,
        inner_top: 197.0,
        side: 100.0,
        label_in: 60.0,
        label_margin: 64.0,
        label_half: 10.0,
        label_top: 152.0,
        more_below: 150.0,
    };
}

/// One slot's half-ellipse about (`centre`, 0): its radii, the angle window a body's centre may take (radians, measured
/// as y grows downward, so π/2 is the lowest point), the mean radius the arc is measured at, and the window's arc length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ellipse {
    pub centre: f64,
    pub rx: f64,
    pub ry: f64,
    pub from: f64,
    pub to: f64,
    pub mean: f64,
    pub span: f64,
}

impl Ellipse {
    /// The point at an angle.
    pub fn at(&self, angle: f64) -> (f64, f64) {
        (self.centre + self.rx * angle.cos(), self.ry * angle.sin())
    }
}

/// Slot `slot`'s ellipse across `width`; a slot inside the first shrinks toward the centre and stops at nothing.
pub fn ellipse(slot: i32, width: f64, g: &OrbitGeometry) -> Ellipse {
    let rx = (g.first_rx + f64::from(slot) * g.step_rx).max(0.0);
    let ry = (g.first_ry + f64::from(slot) * g.step_ry).max(0.0);
    let centre = width / 2.0;
    let top = if slot <= 0 { g.inner_top } else { g.top };
    let below = if ry > 0.0 { PI - (top / ry).min(1.0).asin() } else { PI / 2.0 };
    let inside = if rx > 0.0 { ((g.side - centre) / rx).clamp(-1.0, 1.0).acos() } else { PI / 2.0 };
    let to = below.min(inside).max(PI / 2.0);
    let mean = (rx + ry) / 2.0;
    Ellipse { centre, rx, ry, from: PI - to, to, mean, span: (2.0 * to - PI) * mean }
}

/// How many bodies slot `slot` shows whole at `pitch` apart.
pub fn capacity(slot: i32, width: f64, g: &OrbitGeometry) -> usize {
    (ellipse(slot, width, g).span / g.pitch).floor() as usize + 1
}

/// One group's orbit: its bodies (`first` and `size`, in the list's order of the group), its slot, and what is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orbit {
    pub first: usize,
    pub size: usize,
    pub slot: i32,
    pub ellipse: Ellipse,
    pub shown: bool,
    /// More bodies than the window holds: the orbit turns under the focus, `offset` along the arc.
    pub turning: bool,
    pub offset: f64,
    /// The label's right edge and top.
    pub label: (f64, f64),
    /// The ellipses' centre past the window's left (earlier bodies wait) and right (later ones) ends, where drawn.
    pub before: Option<(f64, f64)>,
    pub after: Option<(f64, f64)>,
    /// Where a jump to this orbit lands, by the walk's index: its shown body nearest the focus across, else its first.
    pub reach: usize,
}

/// One body: its orbit (an index of `orbits`), its angle and point, and whether it is inside a shown window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct World {
    pub orbit: usize,
    pub angle: f64,
    pub x: f64,
    pub y: f64,
    pub shown: bool,
}

/// Where the walk goes from an item, by the walk's index: the leading items first, then the bodies.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Steps {
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub up: Option<usize>,
    pub down: Option<usize>,
}

/// The whole layout: the orbits innermost first (the groups by size, the fewest first), the bodies in the list's order,
/// the walk over the leading items and the bodies, and the first slot's group, kept for the next layout.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OrbitLayout {
    pub orbits: Vec<Orbit>,
    pub worlds: Vec<World>,
    pub walk: Vec<Steps>,
    pub base: i32,
}

/// Lays out bodies whose groups are `groups` (a key each, in the list's order) under `leading` items centred at the given
/// x, with the walk's `focus`; `kept` is the previous layout's base, so the shown groups move only when the focus leaves them.
pub fn orbit_layout<K: PartialEq>(width: f64, groups: &[K], leading: &[f64], focus: usize, kept: Option<i32>, g: &OrbitGeometry) -> OrbitLayout {
    let mut members: Vec<Vec<usize>> = Vec::new();
    let mut keys: Vec<&K> = Vec::new();
    for (i, key) in groups.iter().enumerate() {
        match keys.iter().position(|k| *k == key) {
            Some(at) => members[at].push(i),
            None => {
                keys.push(key);
                members.push(vec![i]);
            }
        }
    }
    members.sort_by_key(Vec::len);
    let count = members.len() as i32;
    let focused = focus.checked_sub(leading.len()).and_then(|b| members.iter().position(|m| m.contains(&b)));
    let base = match count {
        0 => 0,
        1 => {
            if members[0].len() <= capacity(1, width, g) {
                -1
            } else {
                -2
            }
        }
        2 => -1,
        _ => {
            let top = count - g.slots;
            let at = focused.map(|f| f as i32);
            match (kept, at) {
                (Some(b), Some(f)) if (0..=top).contains(&b) => b.min(f).max(f - g.slots + 1),
                (Some(b), None) if (0..=top).contains(&b) => b,
                (_, Some(f)) => (f - 1).clamp(0, top),
                (_, None) => 0,
            }
        }
    };
    let mut orbits = Vec::with_capacity(members.len());
    let mut worlds = vec![World { orbit: 0, angle: 0.0, x: 0.0, y: 0.0, shown: false }; groups.len()];
    for (o, list) in members.iter().enumerate() {
        let slot = o as i32 - base;
        let e = ellipse(slot, width, g);
        let shown = (0..g.slots).contains(&slot);
        let total = (list.len() - 1) as f64 * g.pitch;
        let turning = total > e.span;
        let at = focus.checked_sub(leading.len()).and_then(|b| list.iter().position(|&w| w == b)).unwrap_or(0);
        // The window holds gaps + 1 bodies; the focus stands at the middle one (left of the middle when even), and the set is
        // centred in the window, so an orbit of even capacity shows all of it rather than one fewer about a centred focus.
        let gaps = (e.span / g.pitch).floor();
        let lead = (gaps as usize / 2) as f64 * g.pitch;
        let start = at as f64 * g.pitch - lead - (e.span - gaps * g.pitch) / 2.0;
        let offset = if turning { start.clamp(0.0, total - e.span) } else { 0.0 };
        let slack = 0.5_f64.to_radians();
        for (k, &w) in list.iter().enumerate() {
            let along = k as f64 * g.pitch;
            let angle = if turning { e.to - (along - offset) / e.mean } else { PI / 2.0 + (total / 2.0 - along) / e.mean };
            let (x, y) = e.at(angle);
            worlds[w] = World { orbit: o, angle, x, y, shown: shown && angle >= e.from - slack && angle <= e.to + slack };
        }
        let (ex, ey) = e.at(e.from);
        let (dx, length) = (ex - e.centre, (ex - e.centre).hypot(ey));
        let (lx, ly) = if length > 0.0 { (ex - g.label_in * dx / length, ey - g.label_in * ey / length) } else { (ex, ey) };
        let label = (lx.min(width - g.label_margin), (ly - g.label_half).max(g.label_top));
        let past = g.pitch / 2.0 / e.mean;
        let drawn = |p: (f64, f64)| (p.1 >= g.more_below).then_some(p);
        let before = (turning && offset > 0.5).then(|| e.at(e.to + past)).and_then(drawn);
        let after = (turning && offset < total - e.span - 0.5).then(|| e.at(e.from - past)).and_then(drawn);
        orbits.push(Orbit { first: list[0], size: list.len(), slot, ellipse: e, shown, turning, offset, label, before, after, reach: 0 });
    }
    let lead = leading.len();
    let nearest_shown = |o: &Orbit, list: &[usize], x: f64| -> usize {
        let best = list.iter().filter(|&&w| worlds[w].shown).fold(None, |best: Option<usize>, &w| match best {
            Some(b) if (worlds[b].x - x).abs() <= (worlds[w].x - x).abs() => Some(b),
            _ => Some(w),
        });
        lead + best.unwrap_or(o.first)
    };
    let nearest_lead = |x: f64| -> Option<usize> {
        (0..lead).fold(None, |best: Option<usize>, i| match best {
            Some(b) if (leading[b] - x).abs() <= (leading[i] - x).abs() => Some(b),
            _ => Some(i),
        })
    };
    let across = match focus.checked_sub(lead) {
        Some(b) => worlds.get(b).map_or(0.0, |w| w.x),
        None => leading.get(focus).copied().unwrap_or(width / 2.0),
    };
    let reaches: Vec<usize> = orbits.iter().zip(&members).map(|(o, list)| nearest_shown(o, list, across)).collect();
    let mut walk = vec![Steps::default(); lead + groups.len()];
    for (i, step) in walk.iter_mut().enumerate().take(lead) {
        step.left = i.checked_sub(1);
        step.right = (i + 1 < lead).then_some(i + 1);
        step.down = orbits.iter().position(|o| o.shown).map(|o| nearest_shown(&orbits[o], &members[o], leading[i]));
    }
    for (o, list) in members.iter().enumerate() {
        for (k, &w) in list.iter().enumerate() {
            let x = worlds[w].x;
            walk[lead + w] = Steps {
                left: k.checked_sub(1).map(|p| lead + list[p]),
                right: list.get(k + 1).map(|&n| lead + n),
                up: if o > 0 { Some(nearest_shown(&orbits[o - 1], &members[o - 1], x)) } else { nearest_lead(x) },
                down: orbits.get(o + 1).map(|below| nearest_shown(below, &members[o + 1], x)),
            };
        }
    }
    for (o, reach) in orbits.iter_mut().zip(reaches) {
        o.reach = reach;
    }
    OrbitLayout { orbits, worlds, walk, base }
}
