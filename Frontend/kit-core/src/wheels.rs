//! Groups as wheels turned under a fixed top, and a dial along the foot holding the places and one stop per group: the
//! wheel's slots by their distance from the focus, the dial's camera, and a walk that does not change as the wheel turns
//! (EmuSen_Kit.md §7.32). It names no console: the host hands it each item's group key, in the table's order.

/// The wheel's and the dial's measures in canvas units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelGeometry {
    /// Slots around the wheel, the first at the top.
    pub slots: usize,
    /// Items shown either side of the top; the rest wait behind the rim.
    pub reach: usize,
    /// The dial's left edge and the margin it keeps from the right.
    pub dial_x: f64,
    pub margin: f64,
    /// The width of one dial stop.
    pub pitch: f64,
}

impl WheelGeometry {
    /// Orrery's Wheelwork canvas (orrery-alt/consoles-wheelwork.html).
    pub const WHEELWORK: WheelGeometry = WheelGeometry { slots: 8, reach: 3, dial_x: 64.0, margin: 64.0, pitch: 136.0 };

    /// The angle between two slots, in degrees.
    pub fn step(&self) -> f64 {
        360.0 / self.slots as f64
    }
}

/// Where the walk goes from a node. Nodes are the items in table order, then the dial's stops: the places, then one per group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Steps {
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub up: Option<usize>,
    pub down: Option<usize>,
    /// The shoulders: the previous and next group's stop on the dial.
    pub previous: Option<usize>,
    pub next: Option<usize>,
}

/// The wheels and the dial: each item's group and place in it, each group's first item and size, every node's steps, and
/// each dial stop's centre before the camera.
#[derive(Clone, Debug, PartialEq)]
pub struct Wheels {
    pub places: usize,
    pub group_of: Vec<usize>,
    pub place_in: Vec<usize>,
    pub firsts: Vec<usize>,
    pub sizes: Vec<usize>,
    pub steps: Vec<Steps>,
    pub dial_xs: Vec<f64>,
    pub dial_width: f64,
    /// The dial's shown width, and whether the dial is longer than it.
    pub view: f64,
    pub scrolls: bool,
}

impl Wheels {
    /// The node of the first dial stop.
    pub fn dial_start(&self) -> usize {
        self.group_of.len()
    }

    /// Each group's sum of its items' `weights` (a console's games, say); an item without a weight counts nothing.
    pub fn totals(&self, weights: &[f64]) -> Vec<f64> {
        let mut sums = vec![0.0; self.sizes.len()];
        for (g, w) in self.group_of.iter().zip(weights) {
            sums[*g] += w;
        }
        sums
    }

    /// Each group's rank by size, the fewest first and a tie in the groups' order, as concentric rings are filled from the inside.
    pub fn ranks(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.sizes.len()).collect();
        order.sort_by_key(|g| (self.sizes[*g], *g));
        let mut ranks = vec![0; order.len()];
        for (r, g) in order.into_iter().enumerate() {
            ranks[g] = r;
        }
        ranks
    }

    /// The dial stop of group `g`.
    pub fn stop_of(&self, g: usize) -> usize {
        self.dial_start() + self.places + g
    }
}

/// The wheel as the focus stands: which group it shows, dimmed while a place is focused; each item's distance from the top
/// (on the wheel only), whether it is shown, how many of the wheel wait unshown, and the dial's camera.
#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    pub wheel: Option<usize>,
    pub dimmed: bool,
    pub turns: Vec<i64>,
    pub on_wheel: Vec<bool>,
    pub shown: Vec<bool>,
    pub hidden: usize,
    /// The dial stop lit for the focus: the focused stop, or a focused item's group's.
    pub lit: Option<usize>,
    pub camera: f64,
}

/// Groups `keys` by first appearance and lays the dial out `width` wide after `places` stops of its own.
pub fn wheels<K: PartialEq>(width: f64, keys: &[K], places: usize, g: &WheelGeometry) -> Wheels {
    let mut firsts_keys: Vec<&K> = Vec::new();
    let (mut group_of, mut place_in, mut firsts, mut sizes) = (Vec::new(), Vec::new(), Vec::new(), Vec::<usize>::new());
    for (i, k) in keys.iter().enumerate() {
        let group = firsts_keys.iter().position(|f| *f == k).unwrap_or_else(|| {
            firsts_keys.push(k);
            firsts.push(i);
            sizes.push(0);
            sizes.len() - 1
        });
        group_of.push(group);
        place_in.push(sizes[group]);
        sizes[group] += 1;
    }
    let n = keys.len();
    let stops = places + sizes.len();
    let stop = |s: usize| n + s;
    // Each group's items in their order, so a step along the wheel is a lookup.
    let mut members: Vec<Vec<usize>> = sizes.iter().map(|s| Vec::with_capacity(*s)).collect();
    for (i, gr) in group_of.iter().enumerate() {
        members[*gr].push(i);
    }
    let shoulder = |gr: Option<usize>| gr.filter(|x| *x < sizes.len()).map(|x| stop(places + x));
    let mut steps = Vec::with_capacity(n + stops);
    for i in 0..n {
        let (gr, k) = (group_of[i], place_in[i]);
        steps.push(Steps {
            // Past a wheel's ends the walk runs on into the next group's first and the previous group's last.
            left: k.checked_sub(1).map(|p| members[gr][p]).or_else(|| gr.checked_sub(1).and_then(|p| members[p].last().copied())),
            right: members[gr].get(k + 1).copied().or_else(|| members.get(gr + 1).map(|m| m[0])),
            up: None,
            down: Some(stop(places + gr)),
            previous: shoulder(gr.checked_sub(1)),
            next: shoulder(Some(gr + 1)),
        });
    }
    for s in 0..stops {
        let group = s.checked_sub(places);
        steps.push(Steps {
            left: s.checked_sub(1).map(stop),
            right: (s + 1 < stops).then(|| stop(s + 1)),
            // A place goes Up to the first world, so every node is reached from every other.
            up: group.map_or(firsts.first().copied(), |gr| Some(firsts[gr])),
            down: None,
            previous: group.and_then(|gr| shoulder(gr.checked_sub(1))),
            next: shoulder(Some(group.map_or(0, |gr| gr + 1))),
        });
    }
    let dial_width = stops as f64 * g.pitch;
    let view = width - g.dial_x - g.margin;
    Wheels {
        places,
        group_of,
        place_in,
        firsts,
        sizes,
        steps,
        dial_xs: (0..stops).map(|s| g.dial_x + (s as f64 + 0.5) * g.pitch).collect(),
        dial_width,
        view,
        scrolls: dial_width > view,
    }
}

/// The wheel with `current` focused (a node, or none): an item puts its group's wheel at its place; a group's stop turns
/// that wheel to its first; a place dims the first group's wheel; the camera centres the lit stop, clamped to the dial.
pub fn turn(w: &Wheels, current: Option<usize>, g: &WheelGeometry) -> Turn {
    let n = w.group_of.len();
    let current = current.filter(|c| *c < w.steps.len());
    let (wheel, top, dimmed, lit) = match current {
        Some(i) if i < n => (Some(w.group_of[i]), w.place_in[i], false, Some(w.stop_of(w.group_of[i]))),
        Some(s) if s >= w.stop_of(0) => (Some(s - w.stop_of(0)), 0, false, Some(s)),
        Some(s) => ((!w.sizes.is_empty()).then_some(0), 0, true, Some(s)),
        None => ((!w.sizes.is_empty()).then_some(0), 0, false, None),
    };
    let on_wheel: Vec<bool> = w.group_of.iter().map(|gr| Some(*gr) == wheel).collect();
    let turns: Vec<i64> = (0..n).map(|i| if on_wheel[i] { w.place_in[i] as i64 - top as i64 } else { 0 }).collect();
    let shown: Vec<bool> = (0..n).map(|i| on_wheel[i] && turns[i].unsigned_abs() as usize <= g.reach).collect();
    let hidden = wheel.map_or(0, |gr| w.sizes[gr]) - shown.iter().filter(|s| **s).count();
    let centre = lit.map_or(w.dial_xs.first().copied().unwrap_or(g.dial_x), |s| w.dial_xs[s - n]) - g.dial_x;
    let camera = if w.scrolls { (centre - w.view / 2.0).clamp(0.0, w.dial_width - w.view) } else { 0.0 };
    Turn { wheel, dimmed, turns, on_wheel, shown, hidden, lit, camera }
}
