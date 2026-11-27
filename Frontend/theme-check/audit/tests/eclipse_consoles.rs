// Eclipse's Choose a console held to its board's table, and held whole over the crowds (EmuSen_Qt.md §3.4.9, Proof).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

extern crate emusen_kit_qt;
extern crate emusen_qt_models;
extern crate emusen_qt_test;
extern crate emusen_theme_host;
extern crate emusen_theme_qt;

const PROGRAM: &str = env!("CARGO_BIN_EXE_theme-audit");
// The board's one table, which qt-app's oracle holds in the running program too.
const TABLE: &str = include_str!("../../../qt-app/tests/eclipse/consoles.tsv");

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// theme-audit on Eclipse over a catalogue, recording each state in `dir`: its lines and words, after it judged the package.
fn audit(catalogue: &str, focus: Option<&str>, dir: &Path) -> (String, String) {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).unwrap();
    let mut command = Command::new(PROGRAM);
    command.arg("--catalogue").arg(root().join(format!("Testing/themes/catalogues/{catalogue}.tsv"))).arg("--state").arg(dir);
    if let Some(name) = focus {
        command.args(["--focus", name]);
    }
    let out = command.arg(root().join("Frontend/themes/eclipse")).output().expect("theme-audit runs");
    let err = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(matches!(out.status.code(), Some(0 | 1)), "theme-audit could not run ({}): {err}", out.status);
    (String::from_utf8_lossy(&out.stdout).into_owned(), err)
}

/// One recorded item: its objectName, its box in pixels, its paints and its text.
#[derive(Debug, Default)]
struct Seen {
    at: [f64; 4],
    paints: Vec<String>,
    text: String,
}

/// The items of a recorded state by objectName, read from its spec's tree.
fn seen(spec: &str) -> BTreeMap<String, Seen> {
    let mut items = BTreeMap::new();
    for line in spec.lines().skip_while(|l| *l != "tree").skip(1).take_while(|l| l.starts_with(' ')) {
        let fact = |key: &str| -> Option<String> {
            let at = line.find(&format!(" {key}="))? + key.len() + 2;
            let rest = &line[at..];
            Some(if let Some(quoted) = rest.strip_prefix('"') { quoted[..quoted.find('"').unwrap_or(quoted.len())].to_owned() } else { rest.split(' ').next().unwrap_or("").to_owned() })
        };
        let Some(name) = fact("name") else { continue };
        let at: Vec<f64> = fact("at").unwrap_or_default().split(',').filter_map(|n| n.parse().ok()).collect();
        let paints = fact("paints").map(|p| p.split(',').map(str::to_owned).collect()).unwrap_or_default();
        items.insert(name, Seen { at: [at[0], at[1], at[2], at[3]], paints, text: fact("text").unwrap_or_default() });
    }
    items
}

/// A number as the spec writes it: two places at most.
fn written(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// The table's built rows whose box is in plain board pixels: name, x or right edge, y or middle, w, h, and the two flags.
fn boxes() -> Vec<(String, [Option<f64>; 4], bool, bool)> {
    TABLE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("name\t"))
        .map(|l| l.split('\t').collect::<Vec<_>>())
        // A ShapePath row is a path of its Shape, which the recording holds as the Shape: qt-app's oracle holds the path.
        .filter(|c| c.len() == 14 && c[12] == "built" && c[4] != "ShapePath")
        .map(|c| {
            let n = |p: &str| p.parse::<f64>().ok();
            // A right edge in the width's column (with no width given) or in x's (with the width beside it).
            let right = c[7].strip_prefix("right:").or_else(|| c[5].strip_prefix("right:")).and_then(n);
            let middle = c[6].strip_prefix("mid:").and_then(n);
            let x = right.map(|g| 1920.0 - g).or_else(|| n(c[5]));
            let w = n(c[7]);
            (c[0].to_owned(), [x, middle.or_else(|| n(c[6])), w, n(c[8])], right.is_some(), middle.is_some())
        })
        .collect()
}

/// The plates' lattice from the table: plate.0's x, y, width and height, and the gap plate.1's rule stands it after plate.0 by.
fn lattice() -> (f64, f64, f64, f64, f64) {
    let row = |name: &str| TABLE.lines().map(|l| l.split('\t').collect::<Vec<_>>()).find(|c| c[0] == name).unwrap_or_else(|| panic!("the table's {name}"));
    let (p0, p1) = (row("plate.0"), row("plate.1"));
    let n = |v: &str| v.parse::<f64>().unwrap_or_else(|_| panic!("plate.0's box in plain pixels: {v}"));
    let gap = p1[5].strip_prefix("after:plate.0:").map(n).expect("plate.1 after plate.0 by the grid's gap");
    (n(p0[5]), n(p0[6]), n(p0[7]), n(p0[8]), gap)
}

fn states(dir: &Path, size: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("Consoles-dark-") && n.ends_with(&format!("-{size}.spec")))
        .map(|n| (n.clone(), std::fs::read_to_string(dir.join(&n)).unwrap()))
        .collect();
    found.sort();
    found
}

#[test]
fn qt_3_4_9_eclipse_s_consoles_stands_where_its_board_stands_it() {
    let dir = std::env::temp_dir().join(format!("theme-audit-eclipse-consoles-{}", std::process::id()));
    let (out, err) = audit("eleven", Some("Super Nintendo"), &dir);
    let mut differences = Vec::new();
    let mut held = 0;
    for (size, height) in [("1280x800", 800.0), ("1920x1200", 1200.0)] {
        // A board pixel in the window's pixels: at the board's shape the window's height over the board's 1200.
        let unit = height / 1200.0;
        let recorded = states(&dir, size);
        assert!(!recorded.is_empty(), "Consoles recorded at {size}: {err}");
        for (file, spec) in &recorded {
            let items = seen(spec);
            for (name, edges, right, middle) in boxes() {
                // The backdrop is recorded as one line, its parts held by qt-app's oracle (EmuSen_Qt.md §3.4.9, Proof).
                if name.starts_with("backdrop") {
                    if !items.contains_key("consolesBackdrop") {
                        differences.push(format!("{file}: the backdrop is not recorded"));
                    }
                    continue;
                }
                // Every other element the board draws is recorded in every state; a missing one is a difference.
                let Some(item) = items.get(&name) else {
                    differences.push(format!("{file}: {name} is not recorded"));
                    continue;
                };
                held += 1;
                let at = item.at;
                let x = if right { at[0] + at[2] } else { at[0] };
                // A row centred on a line keeps its centre where the board's stands, whatever size Type.size gives its words.
                let y = if middle { at[1] + at[3] / 2.0 } else { at[1] };
                for (i, (want, got)) in edges.iter().zip([x, y, at[2], at[3]]).enumerate() {
                    let Some(want) = want else { continue };
                    // At the board's shape a right edge stands where the board's does, so every edge is the board's px times the unit.
                    let want = want * unit;
                    // A centre is made of the spec's y and half its height, each written to two places: 0.005 and 0.0025 apart at most.
                    let apart = if i == 1 && middle { (want - got).abs() > 0.0075 + 1e-9 } else { written(want) != written(got) };
                    if apart {
                        differences.push(format!("{file}: {name} edge {i}: the board {}, the screen {got}", written(want)));
                    }
                }
            }
            // The chosen maker's plates on the board's lattice: plate.0's box and the grid's one gap (board line 62).
            let px = unit;
            let (x0, y0, w, h, gap) = lattice();
            for (name, item) in items.iter().filter(|(n, _)| n.starts_with("plate.")) {
                let (c, r) = (((item.at[0] / px - x0) / (w + gap)).round(), ((item.at[1] / px - y0) / (h + gap)).round());
                let want = [(x0 + (w + gap) * c) * px, (y0 + (h + gap) * r) * px, w * px, h * px].map(written);
                if want != item.at {
                    differences.push(format!("{file}: {name} at {:?}, the board's lattice {want:?}", item.at));
                }
                held += 1;
            }
            if let Some(fill) = items.get("makerFill.0") {
                assert!(fill.paints.iter().any(|p| p == "#F6E7D8FF"), "{file}: the chosen maker is filled in the accent: {:?}", fill.paints);
            }
            let label = |i: usize| items.get(&format!("consoleFactLabel.{i}")).map(|s| s.text.as_str()).unwrap_or("");
            let labels: Vec<&str> = (0..4).map(label).collect();
            assert_eq!(labels, emusen_qt_models::words::FACT_LABELS, "{file}: the facts' four labels");
            let place = items.get("consolePlace").map(|s| s.text.as_str()).unwrap_or("");
            assert!(place.contains(" of "), "{file}: the maker and the place: {place:?}");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(held > 20, "the table's rows were found on the screen ({held})");
    assert!(differences.is_empty(), "{} differences from the board:\n{}", differences.len(), differences.join("\n"));
    assert!(out.trim().is_empty(), "no finding over the eleven consoles:\n{out}\n{err}");
}

/// Eclipse's Choose a console judged over one crowd at every size.
fn holds(catalogue: &str) {
    let dir = std::env::temp_dir().join(format!("theme-audit-eclipse-{catalogue}-{}", std::process::id()));
    let (out, err) = audit(catalogue, None, &dir);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(out.trim().is_empty(), "{catalogue}: no finding at any size:\n{out}\n{err}");
    assert!(err.contains("theme-audit: held Consoles in "), "{catalogue}: Consoles judged: {err}");
}

#[test]
fn qt_3_4_9_eclipse_s_consoles_holds_one_console_at_every_size() {
    holds("one-console");
}

#[test]
fn qt_3_4_9_eclipse_s_consoles_holds_30_consoles_at_every_size() {
    holds("n30");
}

#[test]
fn qt_3_4_9_eclipse_s_consoles_holds_40_consoles_of_one_maker_at_every_size() {
    holds("maker40");
}
