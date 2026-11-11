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
}

/// The world and the cores folder esde.rs holds Cores over: VenusRT, the client fixture's console core approved.
pub fn world(name: &str) -> (World, PathBuf) {
    let world = sandbox(name, &[("nes", "Brass Lantern.nes")]);
    let cores = installation_with_snes(&world.dir("lib"));
    approved_console_core(&cores);
    (world, cores)
}

/// No finding is named for the tester: the gate holds every state whole.
fn nothing_named(_: &str, _: &emusen_qt_test::tree::Tree, _: (i32, i32)) -> bool {
    false
}

/// Every visible cell of the list, at every size the gate holds; then Cores held by the gate as `held`, its findings
/// added to `report`.
pub fn cells(scene: &mut Scene<'_>, held: &str, report: &mut Vec<String>) -> Vec<Cell> {
    let mut out = Vec::new();
    for size in SIZES {
        scene.resize(size.0, size.1);
        scene.wait(150);
        let tree = scene.tree();
        let list = tree.find_named("toolList").map(|i| tree.nodes[i as usize].clone()).expect("the list");
        let inside = |n: &emusen_qt_test::tree::Node| n.x >= list.x && n.y >= list.y && n.x < list.x + list.w && n.y < list.y + list.h;
        let texts: Vec<_> = tree.nodes.iter().filter(|n| n.visible && n.ty.starts_with("ToneText") && inside(n)).cloned().collect();
        for n in texts {
            let id: u64 = n.id.parse().unwrap();
            out.push(Cell {
                size,
                x: scene.number(id, "x"),
                words: n.text.clone().unwrap_or_default(),
                face: n.face.clone().unwrap_or_default(),
                given: scene.number(id, "width"),
                takes: scene.number(id, "implicitWidth"),
                cut: n.elided,
            });
        }
    }
    scene.resize(1280, 800);
    scene.wait(100);
    hold(scene, held, &emusen_qt_test::mockup::Mockup::default(), nothing_named, report);
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
    scene.wait(300);
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
    until(scene, 5, "the Tools page", |s| s.property("homeCardModel", "page") == "tools");
    scene.wait(300);
    choose_row(scene, "Cores");
    until(scene, 5, "Cores", |s| s.property("screens", "shown") == "Cores");
    wait_for(scene, "coresModel", "busy", "false", 10000);
    scene.wait(300);
    let themed = cells(scene, "EsdeCoresFit", report);
    let _ = world;
    (own, themed)
}

/// For each size, the share of their words' width each content column was given (`ColumnFit`'s `kept`): the column's
/// width over its widest words as drawn, whole pixels rounded up; the first column, which fills, is left out.
pub fn shares(cells: &[Cell]) -> Vec<((i32, i32), Vec<f64>)> {
    let mut out = Vec::new();
    for size in SIZES {
        let at: Vec<&Cell> = cells.iter().filter(|c| c.size == size).collect();
        let mut xs: Vec<f64> = at.iter().map(|c| c.x).collect();
        xs.sort_by(f64::total_cmp);
        xs.dedup_by(|a, b| (*a - *b).abs() < 0.01);
        let shares = xs
            .iter()
            .skip(1)
            .map(|x| {
                let column: Vec<&&Cell> = at.iter().filter(|c| (c.x - x).abs() < 0.01).collect();
                let widest = column.iter().map(|c| c.takes).fold(0.0, f64::max).ceil();
                column[0].given / widest
            })
            .collect();
        out.push((size, shares));
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
