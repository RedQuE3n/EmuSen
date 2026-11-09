//! Groups drawn as figures on a chart: each group a zigzag of bodies by its count, the figures shelf-packed in justified
//! rows after a leading box of three pointers, a compact tier past two rows, and the walk between them (EmuSen_Kit.md §7.31).
//! It names no console: the host hands it each item's group key, in the table's order.

/// One tier's measures in canvas units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FigureTier {
    pub body: f64,
    pub pitch: f64,
    /// The pitch of a figure of two.
    pub pair_pitch: f64,
    pub amplitude: f64,
    pub label_width: f64,
    pub row_pitch: f64,
    /// The zone at a figure's left where its group's label sits.
    pub zone: f64,
}

/// The chart's measures, as Constellation's Figures canvas sets them (constellation-alt/consoles-figures.html).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FigureGeometry {
    pub margin: f64,
    pub top: f64,
    pub bottom: f64,
    pub gap: f64,
    pub full: FigureTier,
    pub compact: FigureTier,
    pub pointers_width: f64,
    /// The pointers' stars about the row's centre line, from the box's left: the apex, then the lower left and right.
    pub apex: (f64, f64),
    pub lower_left: (f64, f64),
    pub lower_right: (f64, f64),
}

impl FigureGeometry {
    pub const CHART: FigureGeometry = FigureGeometry {
        margin: 64.0,
        top: 96.0,
        bottom: 564.0,
        gap: 64.0,
        full: FigureTier {
            body: 72.0,
            pitch: 110.0,
            pair_pitch: 130.0,
            amplitude: 20.0,
            label_width: 140.0,
            row_pitch: 232.0,
            zone: 100.0,
        },
        compact: FigureTier {
            body: 56.0,
            pitch: 96.0,
            pair_pitch: 112.0,
            amplitude: 16.0,
            label_width: 124.0,
            row_pitch: 168.0,
            zone: 90.0,
        },
        pointers_width: 360.0,
        apex: (180.0, -60.0),
        lower_left: (80.0, 36.0),
        lower_right: (280.0, 36.0),
    };
}

/// A node of the walk: an item by its index in the table, or one of the three pointers (0 the apex, 1 lower left, 2 lower right).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Node {
    Item(usize),
    Pointer(usize),
}

/// Where a node sits and where the walk goes from it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    /// Raised above its line's centre (its label above), else lowered (its label below); pointers are never raised.
    pub raised: bool,
    pub line: usize,
    pub left: Option<Node>,
    pub right: Option<Node>,
    pub up: Option<Node>,
    pub down: Option<Node>,
}

/// One group's figure: its box's left, the centre line of its first line, its bodies a line, and its first item.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Figure {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub count: usize,
    pub per_line: usize,
    pub first: usize,
}

/// The whole chart: the tier, the items in table order, the pointers, the groups in order of first appearance, each item's
/// group and the item drawn before it in its figure, and the chart's height in lines and units.
#[derive(Clone, Debug, PartialEq)]
pub struct FigureLayout {
    pub compact: bool,
    pub tier: FigureTier,
    pub items: Vec<Point>,
    pub pointers: [Point; 3],
    pub figures: Vec<Figure>,
    pub group_of: Vec<usize>,
    pub prior: Vec<Option<usize>>,
    pub lines: usize,
    pub height: f64,
    pub scrolls: bool,
}

enum Boxed {
    Pointers,
    Figure(usize),
}

struct Shelf {
    boxes: Vec<(Boxed, f64)>,
    width: f64,
    tall: usize,
}

/// The groups in order of first appearance, each with its items in table order, and each item's group.
fn groups_of<K: PartialEq>(keys: &[K]) -> (Vec<Vec<usize>>, Vec<usize>) {
    let mut firsts: Vec<&K> = Vec::new();
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut group_of = Vec::with_capacity(keys.len());
    for (i, k) in keys.iter().enumerate() {
        let g = match firsts.iter().position(|f| *f == k) {
            Some(g) => g,
            None => {
                firsts.push(k);
                groups.push(Vec::new());
                groups.len() - 1
            }
        };
        groups[g].push(i);
        group_of.push(g);
    }
    (groups, group_of)
}

/// The shelf packer: the pointers first in the flow, a figure of several lines a row group of its own (sharing it only with
/// pointers waiting alone), and the pointers moved to close their row.
fn pack(width: f64, groups: &[Vec<usize>], t: &FigureTier, g: &FigureGeometry) -> (Vec<Shelf>, Vec<(usize, usize, f64)>) {
    let safe = width - 2.0 * g.margin;
    let max_per = (((safe - t.label_width - t.zone) / t.pitch).floor() as i64 + 1).max(2) as usize;
    let shapes: Vec<(usize, usize, f64)> = groups
        .iter()
        .map(|list| {
            let n = list.len();
            let per = n.min(max_per);
            let pitch = if n == 2 { t.pair_pitch } else { t.pitch };
            (per, n.div_ceil(per), t.zone + (per as f64 - 1.0) * pitch + t.label_width)
        })
        .collect();
    let mut rows: Vec<Shelf> = Vec::new();
    let mut row: Option<Shelf> = None;
    let flow =
        std::iter::once((Boxed::Pointers, g.pointers_width, 1)).chain(shapes.iter().enumerate().map(|(i, s)| (Boxed::Figure(i), s.2, s.1)));
    for (b, w, lines) in flow {
        if lines > 1 {
            match row.take() {
                Some(mut r) if r.boxes.len() == 1 && matches!(r.boxes[0].0, Boxed::Pointers) && r.width + g.gap + w <= safe => {
                    r.boxes.push((b, w));
                    r.width += g.gap + w;
                    r.tall = lines;
                    rows.push(r);
                }
                other => {
                    rows.extend(other.filter(|r| !r.boxes.is_empty()));
                    rows.push(Shelf { boxes: vec![(b, w)], width: w, tall: lines });
                }
            }
            continue;
        }
        let fits = row.as_ref().is_some_and(|r| r.width + if r.boxes.is_empty() { 0.0 } else { g.gap } + w <= safe);
        if !fits {
            rows.extend(row.take().filter(|r| !r.boxes.is_empty()));
            row = Some(Shelf { boxes: Vec::new(), width: 0.0, tall: 1 });
        }
        let r = row.as_mut().expect("a row was opened");
        r.width += if r.boxes.is_empty() { 0.0 } else { g.gap } + w;
        r.boxes.push((b, w));
    }
    rows.extend(row.filter(|r| !r.boxes.is_empty()));
    for r in &mut rows {
        if let Some(i) = r.boxes.iter().position(|b| matches!(b.0, Boxed::Pointers)) {
            let p = r.boxes.remove(i);
            r.boxes.push(p);
        }
    }
    (rows, shapes)
}

/// The nearest node in x of a line, the first of two at the same distance.
fn nearest(line: &[(Node, f64)], x: f64) -> Option<Node> {
    let mut best: Option<(Node, f64)> = None;
    for &(n, nx) in line {
        if best.is_none_or(|(_, bx)| (nx - x).abs() < (bx - x).abs()) {
            best = Some((n, nx));
        }
    }
    best.map(|b| b.0)
}

/// Lays out `keys.len()` items grouped by key on a chart `width` wide: the full tier while it needs at most two lines, else
/// the compact tier, which scrolls when taller than the chart.
pub fn figure_layout<K: PartialEq>(width: f64, keys: &[K], g: &FigureGeometry) -> FigureLayout {
    let (groups, group_of) = groups_of(keys);
    let mut t = g.full;
    let (mut rows, mut shapes) = pack(width, &groups, &t, g);
    if rows.iter().map(|r| r.tall).sum::<usize>() > 2 {
        t = g.compact;
        (rows, shapes) = pack(width, &groups, &t, g);
    }
    let lines: usize = rows.iter().map(|r| r.tall).sum();
    let height = lines as f64 * t.row_pitch;
    let blank = Point { x: 0.0, y: 0.0, raised: false, line: 0, left: None, right: None, up: None, down: None };
    let mut items = vec![blank; keys.len()];
    let mut pointers = [blank; 3];
    let mut figures: Vec<Figure> =
        groups.iter().map(|l| Figure { x: 0.0, y: 0.0, width: 0.0, count: l.len(), per_line: 0, first: l[0] }).collect();
    let mut prior = vec![None; keys.len()];
    let mut line_y = g.top + t.row_pitch / 2.0;
    let mut line = 0;
    for r in &rows {
        let free = width - 2.0 * g.margin - r.width;
        let n = r.boxes.len();
        let spread = if n > 1 { g.gap + free / (n - 1) as f64 } else { 0.0 };
        let mut x = if n > 1 { g.margin } else { g.margin + free / 2.0 };
        for (bi, (b, w)) in r.boxes.iter().enumerate() {
            match *b {
                Boxed::Pointers => {
                    for (p, (dx, dy)) in [g.apex, g.lower_left, g.lower_right].into_iter().enumerate() {
                        pointers[p] = Point { x: x + dx, y: line_y + dy, line, ..blank };
                    }
                }
                Boxed::Figure(f) => {
                    let (per, _, _) = shapes[f];
                    let pitch = if groups[f].len() == 2 { t.pair_pitch } else { t.pitch };
                    figures[f] = Figure { x, y: line_y, width: *w, count: groups[f].len(), per_line: per, first: groups[f][0] };
                    for (k, &item) in groups[f].iter().enumerate() {
                        let (li, kk) = (k / per, k % per);
                        let raised = (kk + bi) % 2 == 0;
                        let y = line_y + li as f64 * t.row_pitch + if raised { -t.amplitude } else { t.amplitude };
                        items[item] =
                            Point { x: x + t.zone + t.label_width / 2.0 + kk as f64 * pitch, y, raised, line: line + li, ..blank };
                        prior[item] = k.checked_sub(1).map(|p| groups[f][p]);
                    }
                }
            }
            x += w + spread;
        }
        line += r.tall;
        line_y += r.tall as f64 * t.row_pitch;
    }
    walk(&mut items, &mut pointers, lines);
    FigureLayout {
        compact: t == g.compact,
        tier: t,
        items,
        pointers,
        figures,
        group_of,
        prior,
        lines,
        height,
        scrolls: height > g.bottom - g.top,
    }
}

/// Left and Right along a line, Up and Down to the nearest node in x of the next line; the pointers' apex above its two stars,
/// which sit between the line's last item and each other, and Right from the apex to the lower right star; standing alone,
/// they go Down to the ends of the next line.
fn walk(items: &mut [Point], pointers: &mut [Point; 3], lines: usize) {
    let mut by_line: Vec<Vec<(Node, f64)>> = vec![Vec::new(); lines.max(1)];
    for (p, at) in pointers.iter().enumerate() {
        by_line[at.line].push((Node::Pointer(p), at.x));
    }
    for (i, at) in items.iter().enumerate() {
        by_line[at.line].push((Node::Item(i), at.x));
    }
    for l in &mut by_line {
        l.sort_by(|a, b| a.1.total_cmp(&b.1));
    }
    let mut steps = Vec::new();
    for (li, l) in by_line.iter().enumerate() {
        for (k, &(n, x)) in l.iter().enumerate() {
            let left = k.checked_sub(1).map(|j| l[j].0);
            let up = li.checked_sub(1).and_then(|u| nearest(&by_line[u], x));
            steps.push((n, left, l.get(k + 1).map(|e| e.0), up, by_line.get(li + 1).and_then(|d| nearest(d, x))));
        }
    }
    for (n, left, right, up, down) in steps {
        let at = match n {
            Node::Item(i) => &mut items[i],
            Node::Pointer(p) => &mut pointers[p],
        };
        (at.left, at.right, at.up, at.down) = (left, right, up, down);
    }
    let line = &by_line[pointers[0].line];
    let before = line.iter().filter(|e| matches!(e.0, Node::Item(_))).map(|e| e.0).next_back();
    let (apex, left, right) = (Node::Pointer(0), Node::Pointer(1), Node::Pointer(2));
    for at in items.iter_mut().filter(|a| a.line == pointers[0].line) {
        if at.left == Some(apex) {
            at.left = before;
        }
        if at.right == Some(apex) {
            at.right = Some(left);
        }
    }
    pointers[0].left = before;
    pointers[0].right = Some(right);
    pointers[0].up = None;
    pointers[0].down = Some(left);
    pointers[1].up = Some(apex);
    pointers[2].up = Some(apex);
    pointers[1].right = Some(right);
    pointers[2].left = Some(left);
    pointers[1].left = before;
    if let Some(Node::Item(b)) = before {
        items[b].right = Some(left);
    }
    // Pointers standing alone open onto the next line at its ends, so a walk along a row and back takes the line whole.
    if before.is_none() && let Some(next) = by_line.get(pointers[0].line + 1) {
        pointers[1].down = next.first().map(|e| e.0);
        pointers[2].down = next.last().map(|e| e.0);
    }
}
