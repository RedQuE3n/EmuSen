//! The loaded audits (man theme-check): each screen a package lists placed through the theme host in an engine of its own,
//! its views fed the synthetic library, then held at the gate's five sizes in both variants to kit-core's fit, pad reach,
//! pointer reach and no-chrome rules, and once to stillness under reduced motion. One line per finding.

use core::pin::Pin;
use std::collections::HashSet;
use std::path::Path;

use cxx_qt::QObject;
use emusen_kit_core::audit::{self, fit, no_chrome, pad_reach, pointer_reach, reached, Fault, Rect};
use emusen_kit_core::input::{ElementId, UiButton, WalkForm};
use emusen_kit_core::tokens::Variant;
use emusen_kit_qt::kinds;
use emusen_kit_qt::tokens::qobject::Tokens;
use emusen_qt_test::ffi::ffi::harness_messages;
use emusen_qt_test::harness::{edges, palette, Measures, SETTLE_MS, SIZES};
use emusen_qt_test::spec::{self, Header};
use emusen_qt_test::tree::Tree;
use emusen_qt_test::{Harness, Scene};
use emusen_theme_manifest::Screen;
use emusen_qt_models::library::LibraryView;
use emusen_theme_qt::host;

use crate::world::Synthetic;

/// The scene, written beside the synthetic library and loaded from there.
const SCENE: &str = include_str!("../qml/Audit.qml");

/// Qt's key codes for the four directions, which the kit's `Frame` answers as the pad's (EmuSen_Qt.md §1.7) and `KeyNavigation` natively.
fn key_of(button: UiButton) -> i32 {
    match button {
        UiButton::Left => 0x0100_0012,
        UiButton::Up => 0x0100_0013,
        UiButton::Right => 0x0100_0014,
        _ => 0x0100_0015,
    }
}

/// `ThemeHost.Role`'s value for a screen, in the enum's order.
fn role(screen: Screen) -> i32 {
    match screen {
        Screen::Home => 0,
        Screen::Consoles => 1,
        Screen::Games => 2,
        Screen::Splash => 3,
    }
}

fn title(screen: Screen) -> &'static str {
    match screen {
        Screen::Home => "Home",
        Screen::Consoles => "Consoles",
        Screen::Games => "Games",
        Screen::Splash => "Splash",
    }
}

/// An object of the theme's engine by its number in the harness's tree, which is its QObject's address (qt-test's `idOf`).
fn object(id: u64) -> Pin<&'static mut QObject> {
    // Safety: the id was read from the live tree of the shown window, and the screen it names lives until the host loads again.
    unsafe { Pin::new_unchecked(&mut *(id as *mut QObject)) }
}

/// The kit's `Tokens` of the theme's engine, which is not the harness's: each engine has its own singletons.
fn tokens(screen: u64) -> Option<Pin<&'static mut Tokens>> {
    host::kit_tokens(object(screen))
}

/// The items a fault names, so its line can say what they are.
fn items(fault: &Fault) -> Vec<ElementId> {
    match fault {
        Fault::PastItsLimit { item, limit } => [Some(*item), *limit].into_iter().flatten().collect(),
        Fault::WordsPastOwner { words, owner } => vec![*words, *owner],
        Fault::Overlap { a, b } => vec![*a, *b],
        Fault::CutAtEdge { item, area, .. } => vec![*item, *area],
        Fault::Covered { item, by } => vec![*item, *by],
        Fault::ScrollsInTooLittleRoom { item, .. }
        | Fault::Ellipsis { item }
        | Fault::RunsPastItsBox { item }
        | Fault::CutAtFoot { item }
        | Fault::TooSmall { item, .. }
        | Fault::TitleCut { item }
        | Fault::FooterCut { item }
        | Fault::Unreachable { item }
        | Fault::ClickDiffers { item }
        | Fault::WheelDoesNotMove { item }
        | Fault::Chrome { item }
        | Fault::Unnamed { item }
        | Fault::OutsidePalette { item, .. }
        | Fault::LowContrast { item, .. } => vec![*item],
    }
}

/// An item as a reader finds it in the QML: its type, its objectName or words, and its box in window pixels.
fn described(tree: &Tree, id: ElementId) -> String {
    let Some(n) = tree.nodes.get(id.0 as usize) else { return format!("{}", id.0) };
    let label = if !n.name.is_empty() { n.name.clone() } else { n.text.as_ref().map(|t| format!("{t:?}")).unwrap_or_default() };
    format!("{} {} {label} at {:.0},{:.0} {:.0}x{:.0}", id.0, n.ty, n.x, n.y, n.w, n.h)
}

/// One finding's line: the screen's file, its kind, the state and the words.
struct Lines<'a> {
    shown: &'a str,
    out: Vec<String>,
}

impl Lines<'_> {
    fn add(&mut self, screen: Screen, kind: &str, words: &str) {
        let line = format!("{}/{}:1 {kind} {words}", self.shown, screen.file());
        if !self.out.contains(&line) {
            self.out.push(line);
        }
    }
}

/// Places a screen in a fresh engine and gives its root's number in the tree, or the host's words when it cannot.
fn place(scene: &mut Scene<'_>, folder: &str, screen: Screen) -> Result<u64, String> {
    scene.set_property("scene", "role", &role(screen).to_string());
    scene.set_property("host", "folder", "");
    scene.set_property("host", "folder", folder);
    scene.wait(100);
    let warnings = scene.property("host", "warnings");
    if scene.property("scene", "placed") != "true" {
        return Err(if warnings.is_empty() { "the host placed nothing".to_owned() } else { warnings });
    }
    let tree = scene.tree();
    let slot = tree.find_named("slot").map(|i| tree.nodes[i as usize].id.clone()).ok_or("the scene has no slot")?;
    let root = tree.nodes.iter().find(|n| n.parent == slot).ok_or("the placed screen is not in the window")?;
    root.id.parse().map_err(|_| "the screen's number".to_owned())
}

/// The views fed as the program's host feeds them, the kit's tokens set to the big screen with reduced motion on.
fn feed(screen: u64, library: &Synthetic, view: &LibraryView, overrides: &[(bool, String, String)]) -> Result<(), String> {
    host::library(object(screen)).ok_or("EmuSen.Theme's Library is not in the theme's engine")?.show(view, &library.home_lines);
    host::session(object(screen)).ok_or("no Session")?.show(&library.session);
    host::device(object(screen)).ok_or("no Device")?.show(&library.device);
    let mut kit = tokens(screen).ok_or("the kit's Tokens are not in the theme's engine")?;
    kit.as_mut().set_form(kinds::form_of(WalkForm::BigScreen));
    kit.as_mut().set_reduced_motion(true);
    host::set_token_overrides(object(screen), overrides);
    Ok(())
}

/// Whether the screen comes to rest under reduced motion: no animation running and no frame swapped for two seconds.
fn still(scene: &mut Scene<'_>) -> Option<String> {
    let mut waited = 0;
    while !scene.running().is_empty() && waited < SETTLE_MS {
        scene.wait(50);
        waited += 50;
    }
    scene.wait(100);
    let before = scene.frames();
    scene.wait(2000);
    let swapped = scene.frames() - before;
    let running = scene.running();
    (swapped > 0 || !running.is_empty())
        .then(|| format!("not still under reduced motion: {swapped} frames swapped in 2 s, running {}", running.join(", ")))
}

/// One pass of states: the library it was fed, named in each line when it is the crowd, whose consoles the pad must sweep.
struct Pass<'a> {
    variants: &'a [Variant],
    label: &'a str,
    consoles: Option<&'a [String]>,
}

/// Every state of one placed screen in a pass: the rules at each size in each variant, recorded into `state` when it is named.
fn states(scene: &mut Scene<'_>, screen: Screen, id: u64, pass: &Pass<'_>, state: Option<&Path>, lines: &mut Lines<'_>) {
    for &variant in pass.variants {
        let name = format!("{}{}", if variant == Variant::Dark { "dark" } else { "light" }, pass.label);
        if let Some(mut kit) = tokens(id) {
            kit.as_mut().set_variant(if variant == Variant::Dark { kinds::Variant::Dark } else { kinds::Variant::Light });
        }
        for (w, h) in SIZES {
            scene.resize(w, h);
            scene.wait(150);
            let tree = scene.tree();
            let focus = scene.focus(&tree).map(ElementId);
            // The crowd's reach is the sweep's; every element on the screen is still held to the pointer.
            let walk = if pass.consoles.is_some() { Vec::new() } else { scene.walk_by(&tree, |s, button| s.key(key_of(button), 0)) };
            let walked = edges(&walk);
            let pointed: HashSet<ElementId> =
                if pass.consoles.is_some() { tree.operable().into_iter().map(ElementId).collect() } else { reached(focus, &walked) };
            let measures = Measures::of(WalkForm::BigScreen, f64::from(h));
            let panel = Rect::new(0.0, 0.0, f64::from(w), f64::from(h));
            let smallest = measures.floor * audit::DESKTOP_CAPS * measures.unit;
            let reach = if pass.consoles.is_some() { Vec::new() } else { pad_reach(&tree.root, focus, &walked) };
            let found = [
                ("fit", fit(&tree.root, panel, measures.slack, smallest)),
                ("reach", reach),
                ("pointer", pointer_reach(&tree.root, &pointed)),
                ("chrome", no_chrome(&tree.root, WalkForm::BigScreen)),
            ];
            for (kind, faults) in found {
                for fault in faults {
                    let who: Vec<String> = items(&fault).into_iter().map(|i| described(&tree, i)).collect();
                    lines.add(screen, kind, &format!("{w}x{h} {name}: {fault} ({})", who.join("; ")));
                }
            }
            if let Some(dir) = state {
                let file = format!("{}-{}-{w}x{h}", title(screen), name.replace(", ", "-").replace(' ', "-"));
                let declared = palette(variant);
                let header =
                    Header { state: &file, case: title(screen), kind: "menu", width: w, height: h, measures, palette: &declared };
                let _ = std::fs::write(dir.join(format!("{file}.spec")), spec::write(&tree, &tree.root, &header, focus.map(|f| f.0), &walk));
                scene.grab(&dir.join(format!("{file}.png")).display().to_string());
            }
            if let Some(missed) = pass.consoles.and_then(|all| swept(scene, all, (f64::from(w), f64::from(h)))) {
                lines.add(screen, "reach", &format!("{w}x{h} {name}: {missed}"));
            }
        }
    }
}

/// What Qt logged since the last call, the theme's engine's warnings among them (it writes them to the log as well as to the host).
fn logged() -> Vec<String> {
    harness_messages().to_string().split('\u{1e}').map(|m| m.trim().replace('\n', " ")).filter(|m| !m.is_empty()).collect()
}

/// The name a reader hears for an item: its accessible name, else the first words under it.
fn spoken(tree: &Tree, index: u32) -> String {
    let node = &tree.nodes[index as usize];
    if !node.heard.is_empty() {
        return node.heard.clone();
    }
    let mut ancestors = vec![node.id.as_str()];
    for n in &tree.nodes[index as usize + 1..] {
        if !ancestors.contains(&n.parent.as_str()) {
            continue;
        }
        if let Some(text) = n.text.as_ref().filter(|t| !t.is_empty()) {
            return text.clone();
        }
        ancestors.push(n.id.as_str());
    }
    String::new()
}

/// The focused element's item, its spoken name, and whether its centre is inside the window, where the player can see it.
fn focused(scene: &mut Scene<'_>, size: (f64, f64)) -> Option<(u64, String, bool)> {
    let tree = scene.tree();
    scene.focus(&tree).map(|f| {
        let n = &tree.nodes[f as usize];
        let (x, y) = (n.x + n.w / 2.0, n.y + n.h / 2.0);
        (tree.id_of(f), spoken(&tree, f), x >= 0.0 && y >= 0.0 && x < size.0 && y < size.1)
    })
}

/// The pad's sweep of a crowded Choose a console: Up and Left until the focus stops, then each row along and back with Down
/// between, as a player looks for one console among many. None when every console's name was heard on the focus while it
/// was inside the window; else how many were and the first few that were not.
fn swept(scene: &mut Scene<'_>, consoles: &[String], size: (f64, f64)) -> Option<String> {
    let mut heard: Vec<String> = Vec::new();
    if let Some((_, name, true)) = focused(scene, size) {
        heard.push(name);
    }
    let limit = consoles.len() * 3 + 40;
    let mut press = |scene: &mut Scene<'_>, button: UiButton| -> bool {
        let before = focused(scene, size);
        scene.key(key_of(button), 0);
        // The view brought the new focus into sight before it is read.
        let tree = scene.tree();
        scene.settle(&tree);
        let after = focused(scene, size);
        if let Some((_, name, true)) = &after {
            if !heard.contains(name) {
                heard.push(name.clone());
            }
        }
        after.is_some() && after != before
    };
    let mut presses = 0;
    for button in [UiButton::Up, UiButton::Left] {
        while presses < limit && press(scene, button) {
            presses += 1;
        }
    }
    let mut along = UiButton::Right;
    loop {
        let mut seen = Vec::new();
        while presses < limit && press(scene, along) {
            presses += 1;
            let now = focused(scene, size);
            if seen.contains(&now) {
                break;
            }
            seen.push(now);
        }
        presses += 1;
        if presses >= limit || !press(scene, UiButton::Down) {
            break;
        }
        along = if along == UiButton::Right { UiButton::Left } else { UiButton::Right };
    }
    let missed: Vec<&String> = consoles.iter().filter(|c| !heard.contains(c)).collect();
    (!missed.is_empty()).then(|| {
        let first: Vec<&str> = missed.iter().take(5).map(|s| s.as_str()).collect();
        format!("the pad's sweep heard {} of {} consoles on the screen; not {}", consoles.len() - missed.len(), consoles.len(), first.join(", "))
    })
}

/// The words of a panic the harness raised (a view that never settles, a window that never shows).
fn panic_words(payload: &(dyn std::any::Any + Send)) -> String {
    payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned())).unwrap_or_default()
}

/// The findings of the package at `folder` (named `shown` in the lines), its screens loaded in turn.
pub fn run(folder: &Path, shown: &str, state: Option<&Path>) -> Result<Vec<String>, String> {
    let shown = shown.trim_end_matches('/');
    let text = std::fs::read_to_string(folder.join("theme.json")).map_err(|e| format!("{shown}/theme.json: {e}"))?;
    let mut lines = Lines { shown, out: Vec::new() };
    let package = match emusen_theme_host::package::read(&text) {
        Ok(p) => p,
        Err(words) => return Ok(words.lines().map(|l| format!("{shown}/{l}")).collect()),
    };
    if let Some(dir) = state {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let real = std::fs::canonicalize(folder).map_err(|e| format!("{shown}: {e}"))?;
    let library = crate::world::make()?;
    let file = library.world.root.join("Audit.qml");
    std::fs::write(&file, SCENE).map_err(|e| format!("the scene: {e}"))?;
    std::panic::set_hook(Box::new(|_| {}));
    let mut harness = Harness::new();
    let mut scene = harness.load(&format!("file://{}", file.display()), 1280, 800);
    for screen in Screen::ALL.into_iter().filter(|s| package.screens.contains(s)) {
        logged();
        let id = match place(&mut scene, &real.display().to_string(), screen) {
            Ok(id) => id,
            Err(words) => {
                words.lines().for_each(|w| lines.add(screen, "load", w));
                continue;
            }
        };
        if let Err(words) = feed(id, &library, &library.view, &package.colour_overrides()) {
            lines.add(screen, "load", &words);
            continue;
        }
        scene.resize(1280, 800);
        if let Some(words) = still(&mut scene) {
            lines.add(screen, "motion", &words);
        }
        let both = Pass { variants: &[Variant::Dark, Variant::Light], label: "", consoles: None };
        let done = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| states(&mut scene, screen, id, &both, state, &mut lines)));
        if let Err(payload) = done {
            lines.add(screen, "audit", &panic_words(payload.as_ref()));
        }
        if screen == Screen::Consoles {
            // Placed afresh over the crowd, as the program shows Choose a console once the library holds 120 consoles.
            let crowd = Pass { variants: &[Variant::Dark], label: ", 120 consoles", consoles: Some(&library.crowd_names) };
            match place(&mut scene, &real.display().to_string(), screen).and_then(|id| feed(id, &library, &library.crowd, &package.colour_overrides()).map(|_| id)) {
                Err(words) => words.lines().for_each(|w| lines.add(screen, "load", w)),
                Ok(id) => {
                    let done = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| states(&mut scene, screen, id, &crowd, state, &mut lines)));
                    if let Err(payload) = done {
                        lines.add(screen, "audit", &panic_words(payload.as_ref()));
                    }
                }
            }
        }
        logged().iter().for_each(|w| lines.add(screen, "warning", w));
    }
    Ok(lines.out)
}
