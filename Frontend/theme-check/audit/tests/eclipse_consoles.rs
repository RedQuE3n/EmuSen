// EmuSen_Qt.md §3.4.8: Eclipse's Choose a console held to its board, the table eclipse_consoles.tsv, as theme-audit records
// it over the canvases' eleven consoles at the board's shape (1280 x 800 and 1920 x 1200), every box in design units as the
// table gives it; and held whole, with no finding, over one console, thirty, one maker of forty and 120.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

extern crate emusen_kit_qt;
extern crate emusen_qt_models;
extern crate emusen_qt_test;
extern crate emusen_theme_host;
extern crate emusen_theme_qt;

const PROGRAM: &str = env!("CARGO_BIN_EXE_theme-audit");
const TABLE: &str = include_str!("eclipse_consoles.tsv");

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

/// The table's built rows with a single objectName and a box on the board: name, then x (or right=), y, w, h in board px, "-" free.
fn boxes() -> Vec<(String, [Option<f64>; 4], bool)> {
    TABLE
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| l.split('\t').collect::<Vec<_>>())
        .filter(|c| c.len() >= 8 && c[7] == "built" && c[2] != "-" && !c[2].ends_with(".N"))
        .map(|c| {
            let parts: Vec<&str> = c[4].split(',').collect();
            let right = parts.first().is_some_and(|p| p.starts_with("right="));
            let n = |p: &str| p.trim_start_matches("right=").parse::<f64>().ok();
            (c[2].to_owned(), [n(parts.first().unwrap_or(&"-")), parts.get(1).and_then(|p| n(p)), parts.get(2).and_then(|p| n(p)), parts.get(3).and_then(|p| n(p))], right)
        })
        .collect()
}

fn states(dir: &Path, size: &str) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("Consoles-") && n.ends_with(&format!("-{size}.spec")))
        .map(|n| (n.clone(), std::fs::read_to_string(dir.join(&n)).unwrap()))
        .collect();
    found.sort();
    found
}

#[test]
fn qt_3_4_8_eclipse_s_consoles_stands_where_its_board_stands_it() {
    let dir = std::env::temp_dir().join(format!("theme-audit-eclipse-consoles-{}", std::process::id()));
    let (out, err) = audit("eleven", Some("Super Nintendo"), &dir);
    assert!(out.trim().is_empty(), "no finding over the eleven consoles:\n{out}\n{err}");
    let mut differences = Vec::new();
    let mut held = 0;
    for (size, height) in [("1280x800", 800.0), ("1920x1200", 1200.0)] {
        // A board pixel in the window's pixels: at the board's shape the window's height over the board's 1200.
        let unit = height / 1200.0;
        let recorded = states(&dir, size);
        assert!(!recorded.is_empty(), "Consoles recorded at {size}: {err}");
        for (file, spec) in &recorded {
            let items = seen(spec);
            for (name, edges, right) in boxes() {
                let Some(item) = items.get(&name) else { continue };
                held += 1;
                let at = item.at;
                let x = if right { at[0] + at[2] } else { at[0] };
                for (i, (want, got)) in edges.iter().zip([x, at[1], at[2], at[3]]).enumerate() {
                    let Some(want) = want else { continue };
                    // At the board's shape a right edge stands where the board's does, so every edge is the board's px times the unit.
                    let want = want * unit;
                    if written(want) != got {
                        differences.push(format!("{file}: {name} edge {i}: the board {}, the screen {got}", written(want)));
                    }
                }
            }
            // The plates of the chosen maker on the board's lattice: 390 + 256 c px across, 250 + 198 r px down, 236 by 178 px.
            let px = unit;
            for (name, item) in items.iter().filter(|(n, _)| n.starts_with("plate.")) {
                let (c, r) = (((item.at[0] / px - 390.0) / 256.0).round(), ((item.at[1] / px - 250.0) / 198.0).round());
                let want = [(390.0 + 256.0 * c) * px, (250.0 + 198.0 * r) * px, 236.0 * px, 178.0 * px].map(written);
                if want != item.at {
                    differences.push(format!("{file}: {name} at {:?}, the board's lattice {want:?}", item.at));
                }
                held += 1;
            }
            if let Some(fill) = items.get("makerFill.0") {
                assert!(fill.paints.iter().any(|p| p == "#F6E7D8FF"), "{file}: the chosen maker is filled in the accent: {:?}", fill.paints);
            }
            let facts = items.get("consoleFacts").map(|s| s.text.as_str()).unwrap_or("");
            assert!(facts.contains(" played"), "{file}: the facts carry the played count: {facts:?}");
            let place = items.get("consolePlace").map(|s| s.text.as_str()).unwrap_or("");
            assert!(place.contains(" of "), "{file}: the maker and the place: {place:?}");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(held > 20, "the table's rows were found on the screen ({held})");
    assert!(differences.is_empty(), "{} differences from the board:\n{}", differences.len(), differences.join("\n"));
}

#[test]
fn qt_3_4_8_eclipse_s_consoles_holds_one_to_120_consoles_at_every_size() {
    for catalogue in ["one-console", "n30", "maker40", "n120"] {
        let dir = std::env::temp_dir().join(format!("theme-audit-eclipse-{catalogue}-{}", std::process::id()));
        let (out, err) = audit(catalogue, None, &dir);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(out.trim().is_empty(), "{catalogue}: no finding at any size:\n{out}\n{err}");
        assert!(err.contains("theme-audit: held Consoles in "), "{catalogue}: Consoles judged: {err}");
    }
}
