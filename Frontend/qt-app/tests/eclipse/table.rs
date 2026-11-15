//! Eclipse's games view held to its table (`games.tsv`, EmuSen_Qt.md §3.4.10): every built row's checks against the
//! scene's items, a board pixel converted by kit-core's design height over the board's height.

use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::tokens::Colour;
use emusen_qt_test::tree::{Node, Tree};
use emusen_qt_test::Scene;

pub const TABLE: &str = include_str!("games.tsv");

/// Two measures in canvas units are one where they differ by less than this: Qt's scene holds positions in single
/// precision (about 7 digits), so a 3840-pixel window rounds at a few ten-thousandths of a unit; not a design allowance.
const SAME: f64 = 1e-3;

/// One built row: the item it names, the board's height and its checks.
pub struct Row {
    pub element: String,
    pub item: String,
    pub board_h: f64,
    pub checks: Vec<(String, String)>,
}

impl Row {
    /// The value of a check by its key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.checks.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }
}

/// The built rows of the table whose item is given, in its order.
pub fn built() -> Vec<Row> {
    TABLE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty() && !l.starts_with("element\t"))
        .map(|l| l.split('\t').collect::<Vec<_>>())
        .filter(|f| f.len() > 6 && f[4] == "built" && !f[6].is_empty())
        .map(|f| Row {
            element: f[0].to_owned(),
            item: f[5].to_owned(),
            board_h: f[2].parse().expect("a board height"),
            checks: f[6].split_whitespace().map(|c| c.split_once('=').map(|(k, v)| (k.to_owned(), v.to_owned())).expect("key=value")).collect(),
        })
        .collect()
}

fn named<'t>(tree: &'t Tree, name: &str) -> Option<&'t Node> {
    tree.find_named(name).map(|i| &tree.nodes[i as usize])
}

fn same(a: f64, b: f64) -> bool {
    (a - b).abs() < SAME
}

/// The differences between the rows given and the scene at the window's height `h` and width `w`, each a line; `states`
/// names the rows held in this state (the games or the empty one).
pub fn differences(scene: &mut Scene<'_>, rows: &[&Row], w: f64, h: f64) -> Vec<String> {
    let text_scale = *scene.kit_tokens().expect("the program's tokens").text_scale();
    let tree = scene.tree();
    let unit = h / DESIGN_HEIGHT;
    let canvas_width = w / unit;
    let mut out = Vec::new();
    for row in rows {
        let units = |px: &str| px.parse::<f64>().expect("a number") * DESIGN_HEIGHT / row.board_h;
        let get = |key: &str| row.get(key);
        if row.item == "gamesTile" {
            out.extend(tiles(scene, &tree, row, unit, canvas_width, &units));
            continue;
        }
        let Some(node) = named(&tree, &row.item) else {
            out.push(format!("{}: {} is not in the scene", row.element, row.item));
            continue;
        };
        if get("shown") == Some("no") {
            if node.visible {
                out.push(format!("{}: {} is shown", row.element, row.item));
            }
            continue;
        }
        if !node.visible {
            out.push(format!("{}: {} is not shown", row.element, row.item));
            continue;
        }
        let (x, y, nw, nh) = (node.x / unit, node.y / unit, node.w / unit, node.h / unit);
        for (key, value) in &row.checks {
            let (read, want): (String, String) = match key.as_str() {
                "x" => (x.to_string(), units(value).to_string()),
                "y" => (y.to_string(), units(value).to_string()),
                "w" => (nw.to_string(), units(value).to_string()),
                "h" => (nh.to_string(), units(value).to_string()),
                "right" => ((canvas_width - x - nw).to_string(), units(value).to_string()),
                "cy" => ((y + nh / 2.0).to_string(), units(value).to_string()),
                "below" => {
                    let (above, gap) = value.split_once(':').expect("item:gap");
                    let Some(a) = named(&tree, above) else {
                        out.push(format!("{}: {above} is not in the scene", row.element));
                        continue;
                    };
                    (y.to_string(), ((a.y + a.h) / unit + units(gap)).to_string())
                }
                "after" => {
                    let (before, gap) = value.split_once(':').expect("item:gap");
                    let Some(a) = named(&tree, before) else {
                        out.push(format!("{}: {before} is not in the scene", row.element));
                        continue;
                    };
                    (x.to_string(), ((a.x + a.w) / unit + units(gap)).to_string())
                }
                // A type size is the item's own pixel size in canvas units: Type.size's of the board's, the kit's rule at the text
                // scale in force and never under the legibility floor, as Qt holds it whole.
                "font" => (node.size.to_string(), emusen_kit_qt::type_size::legible(units(value), text_scale).round().to_string()),
                "ink" => {
                    let read = node.ink.clone().unwrap_or_default();
                    if Colour::parse(&read) != Colour::parse(value) {
                        out.push(format!("{}: {} ink {read}, the table {value}", row.element, row.item));
                    }
                    continue;
                }
                "colour" => {
                    // The item's own colour where others draw over it, as the ground under the disc.
                    let read = scene.property(&row.item, "color");
                    if Colour::parse(&read) != Colour::parse(value) {
                        out.push(format!("{}: {} colour {read}, the table {value}", row.element, row.item));
                    }
                    continue;
                }
                "fill" => {
                    // Read as drawn, inside the box's left end at its middle (its centre where it is narrower than its ends'
                    // offset), clear of the words a pill or a disc carries at its centre.
                    let at = if node.w > 8.0 { (node.x + 3.0, node.y + node.h / 2.0) } else { (node.x + node.w / 2.0, node.y + node.h / 2.0) };
                    let read = scene.colours_at(&[at])[0];
                    let want = Colour::parse(value).expect("a colour");
                    if read.map(|c| (c.r, c.g, c.b)) != Some((want.r, want.g, want.b)) {
                        out.push(format!("{}: {} fill {read:?}, the table {value}", row.element, row.item));
                    }
                    continue;
                }
                other => panic!("{}: no such check {other}", row.element),
            };
            if !same(read.parse().unwrap(), want.parse().unwrap()) {
                out.push(format!("{}: {} {key} {read}, the board {want}", row.element, row.item));
            }
        }
    }
    out
}

/// The tiles by the board's rule: as many columns of the tile's width and gap as fit between its left edge and its right
/// margin, the rows shown placed down from its top, each tile at its column and row.
fn tiles(scene: &mut Scene<'_>, tree: &Tree, row: &Row, unit: f64, canvas_width: f64, units: &dyn Fn(&str) -> f64) -> Vec<String> {
    let get = |key: &str| row.get(key);
    let (left, top, tw, th, gap, right) = (units(get("x").unwrap()), units(get("y").unwrap()), units(get("w").unwrap()), units(get("h").unwrap()), units(get("gap").unwrap()), units(get("right").unwrap()));
    let rows: usize = get("rows").unwrap().parse().unwrap();
    let columns = (((canvas_width - left - right + gap) / (tw + gap)).floor() as usize).max(1);
    let count: usize = scene.property("gamesGrid", "count").parse().unwrap_or(0);
    let mut out = Vec::new();
    for i in 0..count.min(columns * rows) {
        let name = format!("gamesTile.{i}");
        let Some(n) = named(tree, &name) else {
            out.push(format!("{}: {name} is not in the scene", row.element));
            continue;
        };
        let want = (left + (i % columns) as f64 * (tw + gap), top + (i / columns) as f64 * (th + gap), tw, th);
        let read = (n.x / unit, n.y / unit, n.w / unit, n.h / unit);
        if !(same(read.0, want.0) && same(read.1, want.1) && same(read.2, want.2) && same(read.3, want.3)) {
            out.push(format!("{}: {name} at {read:?}, the board's rule {want:?} ({columns} columns)", row.element));
        }
    }
    if right + left + columns as f64 * (tw + gap) - gap > canvas_width + SAME {
        out.push(format!("{}: {columns} columns pass the right margin", row.element));
    }
    out
}
