//! Cores' list measured as drawn (EmuSen_Qt.md §7.9, EmuSen_Kit.md §7.43), for cores_fit.rs on the software backend and
//! cores_fit_gpu.rs on Vulkan: over the world esde.rs holds Cores in, in the own look and under the ES-DE theme, each
//! cell's width as `ColumnFit` gave it against the width its words take as the cell lays them out, at the gate's sizes.
#![allow(dead_code)]

use super::console::{choose_row, until};
use super::{approved_console_core, hold, sandbox, wait_for};
use emusen_client_fixture::cores::installation_with_snes;
use emusen_client_fixture::library;
use emusen_client_fixture::World;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt_test::excuse::Excuses;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::Scene;
use serde_json::json;
use std::path::PathBuf;

/// Art Book Next, the theme esde.rs holds Cores over.
const THEME: &str = "art-book-next-es-de";

/// One cell of the list at one size: its words, its face, the width it was given and the width its words take, both in
/// the cell's own units, and whether it was cut.
#[derive(Debug)]
pub struct Cell {
    pub size: (i32, i32),
    /// The cell's left in its row, in the cell's own units: its column.
    pub x: f64,
    pub words: String,
    pub face: String,
    pub given: f64,
    pub takes: f64,
    pub cut: bool,
    /// The whole list a cell that gives way holds beside its words, and the width it takes in the cell's units.
    pub whole: Option<(String, f64)>,
}

/// The world and the cores folder esde.rs holds Cores over: VenusRT, the client fixture's console core approved.
pub fn world(name: &str) -> (World, PathBuf) {
    let world = sandbox(name, &[("nes", "Brass Lantern.nes")]);
    let cores = installation_with_snes(&world.dir("lib"));
    approved_console_core(&cores);
    (world, cores)
}

/// The list's cells as laid out: each one's place and words, from the tree.
fn laid_out(scene: &mut Scene<'_>) -> Vec<(String, i64, i64, i64)> {
    let tree = scene.tree();
    let Some(list) = tree.find_named("toolList").map(|i| tree.nodes[i as usize].clone()) else { return Vec::new() };
    let inside = |n: &emusen_qt_test::tree::Node| n.x >= list.x && n.y >= list.y && n.x < list.x + list.w && n.y < list.y + list.h;
    tree.nodes.iter().filter(|n| n.visible && n.ty.starts_with("ToneText") && inside(n)).map(|n| (n.text.clone().unwrap_or_default(), n.x as i64, n.w as i64, list.w as i64)).collect()
}

/// Until the list holds cells and two reads of them in a row agree: the columns fitted to the size and the words chosen.
pub fn steady(scene: &mut Scene<'_>) {
    let mut last = Vec::new();
    until(scene, 5, "the list's cells at rest", |s| {
        let now = laid_out(s);
        let done = !now.is_empty() && now == last;
        last = now;
        done
    });
}

/// Every visible cell of the list, at every size the gate holds; then Cores held by the gate as `held`, its findings
/// added to `report`.
pub fn cells(scene: &mut Scene<'_>, held: &str, report: &mut Vec<String>) -> Vec<Cell> {
    let mut out = Vec::new();
    for size in SIZES {
        scene.resize(size.0, size.1);
        steady(scene);
        let tree = scene.tree();
        let list = tree.find_named("toolList").map(|i| tree.nodes[i as usize].clone()).expect("the list");
        let inside = |n: &emusen_qt_test::tree::Node| n.x >= list.x && n.y >= list.y && n.x < list.x + list.w && n.y < list.y + list.h;
        let texts: Vec<_> = tree.nodes.iter().filter(|n| n.visible && n.ty.starts_with("ToneText") && inside(n)).cloned().collect();
        for n in texts {
            let id: u64 = n.id.parse().unwrap();
            let held = tree.nodes.iter().find(|c| c.parent == n.id && !c.visible && c.text.as_deref().is_some_and(|t| !t.is_empty())).cloned();
            let whole = held.map(|c| (c.text.clone().unwrap_or_default(), scene.number(c.id.parse().unwrap(), "implicitWidth")));
            out.push(Cell {
                size,
                x: scene.number(id, "x"),
                words: n.text.clone().unwrap_or_default(),
                face: n.face.clone().unwrap_or_default(),
                given: scene.number(id, "width"),
                takes: scene.number(id, "implicitWidth"),
                cut: n.elided,
                whole,
            });
        }
    }
    scene.resize(1280, 800);
    steady(scene);
    hold(scene, held, &emusen_qt_test::mockup::Mockup::default(), &mut Excuses::none(), report);
    out
}

/// The program on `cores`, Cores opened from Home's place: its cells, and its gate's findings in `report`; then under
/// the ES-DE theme from the Home card's Tools: the same.
pub fn both_looks(scene: &mut Scene<'_>, world: &World, cores: &std::path::Path, report: &mut Vec<String>) -> (Vec<Cell>, Vec<Cell>) {
    let theme = PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/emusen/bigpicture/corpus/whole").join(THEME);
    assert!(theme.exists(), "the theme at {}", theme.display());
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(scene, "library", "loading", "false", 20000);
    for _ in 0..40 {
        let tree = scene.tree();
        if let Some(first) = tree.find_named("place.0") {
            if scene.focus(&tree) == Some(first) || scene.focus_item(&tree, first) {
                break;
            }
        }
        scene.wait(50);
    }
    for _ in 0..emusen_qt_models::screens::CORES_PLACE {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Cores", 2000);
    wait_for(scene, "coresModel", "busy", "false", 10000);
    steady(scene);
    let own = cells(scene, "CoresFit", report);
    scene.pad(UiButton::Back);
    wait_for(scene, "screens", "shown", "Home", 2000);

    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["LibraryStyle"] = json!("Theme");
    settings["BigPictureTheme"] = json!(theme.to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    assert!(scene.set_property("program", "esdeChosen", "true"));
    assert!(scene.set_property("library", "platformLibrary", ""));
    assert!(scene.set_property("library", "platformLibrary", library().to_str().unwrap()));
    until(scene, 30, "the theme in place", |s| s.property("screens", "shown") == "Esde" && s.property("esdeModel", "view") == "system");
    scene.pad(UiButton::Menu);
    until(scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    choose_row(scene, "Tools");
    until(scene, 5, "the Tools page", |s| s.property("homeCardModel", "page") == "tools" && !super::console::focused_words(s).starts_with("Tools"));
    choose_row(scene, "Cores");
    until(scene, 5, "Cores", |s| s.property("screens", "shown") == "Cores");
    wait_for(scene, "coresModel", "busy", "false", 10000);
    steady(scene);
    let themed = cells(scene, "EsdeCoresFit", report);
    let _ = world;
    (own, themed)
}

/// Every way the list's words fail to fit, at every size: a cell cut, a cell given less than its words take, the status
/// column given other than its widest words (rounded up, as `ColumnFit` measures them), a core of several consoles
/// read as neither its whole list nor its first and a count, and a cell that gives way showing its first and a count
/// where its whole list fits or its whole list where it does not (EmuSen_Kit.md §7.106, decided 2026-11-11).
pub fn misfits(cells: &[Cell], several: &[(&str, &str)]) -> Vec<String> {
    let mut out = Vec::new();
    for c in cells {
        if let Some((whole, takes)) = &c.whole {
            let fits = *takes <= c.given + 0.01;
            if fits != (c.words == *whole) {
                out.push(format!("{:?}: \"{whole}\" takes {takes:.2} of {:.2} and the cell reads \"{}\"", c.size, c.given, c.words));
            }
        }
    }
    for c in cells.iter().filter(|c| c.cut || c.takes > c.given + 0.01) {
        out.push(format!("{:?}: \"{}\" takes {:.2} and is given {:.2}{}", c.size, c.words, c.takes, c.given, if c.cut { ", cut" } else { "" }));
    }
    for size in SIZES {
        let at: Vec<&Cell> = cells.iter().filter(|c| c.size == size).collect();
        let right = at.iter().map(|c| c.x).fold(f64::MIN, f64::max);
        let status: Vec<&&Cell> = at.iter().filter(|c| (c.x - right).abs() < 0.01).collect();
        let widest = status.iter().map(|c| c.takes).fold(0.0, f64::max).ceil();
        if status.first().is_none_or(|c| (c.given - widest).abs() > 0.01) {
            out.push(format!("{size:?}: the status column is given {:?}, its widest words {widest}", status.first().map(|c| c.given)));
        }
        for (whole, short) in several {
            if !at.iter().any(|c| c.words == *whole || c.words == *short) {
                out.push(format!("{size:?}: neither \"{whole}\" nor \"{short}\""));
            }
        }
    }
    out
}

/// The cells written out, one a line, for the record in EmuSen_Qt.md §7.9.
pub fn table(look: &str, cells: &[Cell]) -> String {
    cells
        .iter()
        .map(|c| format!("{look}\t{}x{}\t{}\t{}\t{:.2}\t{:.2}\t{}", c.size.0, c.size.1, c.words, c.face, c.given, c.takes, c.cut))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Both sides of giving way are held where the run meets them: `whole` read whole at a size whose room holds it, and its
/// first and a count, `short`, at a size whose room does not; and each core of one console, `one`, read whole at every
/// size (EmuSen_Kit.md §7.106).
pub fn both_sides(cells: &[Cell], whole: &str, short: &str, one: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let read = |words: &str| cells.iter().filter(|c| c.words == words).map(|c| c.size).collect::<Vec<_>>();
    if read(whole).is_empty() {
        out.push(format!("\"{whole}\" is read whole at no size"));
    }
    if read(short).is_empty() {
        out.push(format!("\"{short}\" is read at no size"));
    }
    for console in one {
        let at = read(console);
        for size in SIZES.iter().filter(|s| !at.contains(s)) {
            out.push(format!("{size:?}: \"{console}\" is not read whole"));
        }
    }
    out
}
