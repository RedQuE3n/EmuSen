//! The loaded audits (man theme-check): each screen a package lists placed through the theme host in an engine of its own,
//! its views fed the synthetic library, then held at the gate's five sizes in both variants to kit-core's fit, pad reach,
//! pointer reach, no-chrome, name and contrast rules, and once to stillness under reduced motion; then in each state a run names
//! (man theme-check, STATES), fed through the same surfaces. One line per finding.

use core::pin::Pin;
use std::collections::HashSet;
use std::path::Path;

use cxx_qt::QObject;
use emusen_kit_core::audit::{self, contrast, fit, named, no_chrome, pad_reach, pointer_reach, reached, Fault, Rect};
use emusen_kit_core::input::{ElementId, UiButton, WalkForm};
use emusen_kit_core::tokens::Variant;
use emusen_kit_qt::kinds;
use emusen_kit_qt::tokens::qobject::Tokens;
use emusen_qt_test::ffi::ffi::harness_messages;
use emusen_qt_test::harness::{edges, palette, Landing, Measures, Step, SETTLE_MS, SIZES};
use emusen_qt_test::spec::{self, Header};
use emusen_qt_test::tree::Tree;
use emusen_qt_test::{Harness, Scene};
use emusen_theme_manifest::Screen;
use emusen_qt_models::library::{Kind, LibraryView, ALL};
use emusen_qt_models::words;
use emusen_theme_qt::browse::facts_of;
use emusen_theme_qt::host;
use emusen_theme_qt::session::SessionFacts;
use emusen_theme_qt::view as theme_view;

use crate::catalogue::Entry;
use crate::states::{State, DEFAULT};
use crate::world::Synthetic;

/// The scene, written beside the synthetic library and loaded from there.
const SCENE: &str = include_str!("../qml/Audit.qml");

/// Qt's key code for Return, which a theme's Walkable answers as A.
const RETURN: i32 = 0x0100_0004;

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
    /// Where the audit is: the size and the variant being judged, named when a state cannot be read.
    at: String,
}

impl Lines<'_> {
    fn add(&mut self, screen: Screen, kind: &str, words: &str) {
        let line = format!("{}/{}:1 {kind} {words}", self.shown, screen.file());
        if !self.out.contains(&line) {
            self.out.push(line);
        }
    }

    /// Runs one pass of a screen; a panic in it (a view that never settles) is a finding of its own, naming where it stopped.
    fn guarded(&mut self, screen: Screen, pass: impl FnOnce(&mut Self)) {
        self.at = "placed, before its first size".to_owned();
        if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pass(&mut *self))) {
            let words = format!(
                "{}: the audit stopped here, so this state and the rest of its pass were not judged: {}",
                self.at,
                panic_words(payload.as_ref())
            );
            self.add(screen, "stopped", &words);
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

/// What the host feeds a placed screen beside the library: the session, the places it declines, and whether motion is reduced.
struct Fed<'a> {
    session: &'a SessionFacts,
    declined: &'a [&'a str],
    reduced: bool,
}

/// The views fed as the program's host feeds them, the kit's tokens set to the big screen, with reduced motion on but for a held clock.
fn feed(screen: u64, library: &Synthetic, view: &LibraryView, fed: &Fed<'_>, overrides: &[(bool, String, String)], section: Option<&str>) -> Result<(), String> {
    let mut theme = host::library(object(screen)).ok_or("EmuSen.Theme's Library is not in the theme's engine")?;
    theme.as_mut().show_declined(&fed.declined.iter().map(|d| (*d).to_owned()).collect::<Vec<_>>());
    theme.as_mut().show(view, &library.home_lines(view));
    // The Games screen's section as the program's showBrowse gives it: the named one, else the first console's, else the All shelf.
    let first = view.rows.rows().iter().find(|r| r.kind == Some(Kind::Console)).map(|r| r.key.clone()).or(Some(ALL.to_owned()));
    if let Some(key) = section.map(str::to_owned).or(first) {
        let mut browse = host::browse(object(screen)).ok_or("no Browse")?;
        browse.as_mut().show_games(theme_view::games_of(view, &key, words::now()));
        browse.as_mut().show(&facts_of(view, &key, "", 0, (false, false, false), false));
    }
    host::session(object(screen)).ok_or("no Session")?.show(fed.session);
    host::device(object(screen)).ok_or("no Device")?.show(&library.device);
    let mut kit = tokens(screen).ok_or("the kit's Tokens are not in the theme's engine")?;
    kit.as_mut().set_form(kinds::form_of(WalkForm::BigScreen));
    kit.as_mut().set_reduced_motion(fed.reduced);
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
    /// The name of the element the pad walks the focus to before the state is judged.
    focus: Option<&'a str>,
    /// A held press of A on the focus while the state is judged.
    press: bool,
    /// The animation clock is held, so nothing moves and the picture is taken without waiting for rest.
    held: bool,
}

impl<'a> Pass<'a> {
    fn plain(variants: &'a [Variant], label: &'a str, consoles: Option<&'a [String]>) -> Pass<'a> {
        Pass { variants, label, consoles, focus: None, press: false, held: false }
    }
}

/// Every state of one placed screen in a pass: the rules at each size in each variant, recorded into `state` when it is named.
fn states(scene: &mut Scene<'_>, screen: Screen, id: u64, pass: &Pass<'_>, state: Option<&Path>, lines: &mut Lines<'_>) {
    for &variant in pass.variants {
        let name = format!("{}{}", if variant == Variant::Dark { "dark" } else { "light" }, pass.label);
        if let Some(mut kit) = tokens(id) {
            kit.as_mut().set_variant(if variant == Variant::Dark { kinds::Variant::Dark } else { kinds::Variant::Light });
        }
        for (w, h) in SIZES {
            lines.at = format!("{w}x{h} {name}");
            scene.resize(w, h);
            scene.wait(150);
            let mut tree = scene.tree();
            // A title or a name may elide; the program's fixed words may not.
            tree.root.mark_data(&words::is_fixed);
            // What lies behind each text as drawn, pictures and translucent fills included, before the walk moves anything.
            scene.sample_behind(&mut tree, f64::from(w), f64::from(h));
            let focus = scene.focus(&tree).map(ElementId);
            // The crowd's reach is the sweep's; every element on the screen is still held to the pointer.
            let walk = if pass.consoles.is_some() { Vec::new() } else { scene.walk_by(&tree, |s, button| s.key(key_of(button), 0)) };
            let walked = edges(&walk);
            // A state's focus and held press are put on after the walk, and the screen judged and recorded as they leave it.
            if let Some(name) = pass.focus {
                walk_to(scene, &tree, &walk, name);
            }
            if pass.press {
                scene.key_hold(RETURN, true);
            }
            // Reach and pointer are the walk's tree's, whose numbers the walk's edges name; the rest are judged on what is shown.
            let held_tree = (pass.focus.is_some() || pass.press).then(|| {
                if !pass.held {
                    scene.rest();
                }
                let mut shown = scene.tree();
                shown.root.mark_data(&words::is_fixed);
                scene.sample_behind(&mut shown, f64::from(w), f64::from(h));
                shown
            });
            let seen = held_tree.as_ref().unwrap_or(&tree);
            let shown = scene.focus(seen).map(ElementId);
            // The splash is watched, not operated: nothing on it is clicked, whatever stands at its centre.
            let pointed: HashSet<ElementId> = if screen == Screen::Splash {
                HashSet::new()
            } else if pass.consoles.is_some() {
                tree.operable().into_iter().map(ElementId).collect()
            } else {
                reached(focus, &walked)
            };
            let measures = Measures::of(WalkForm::BigScreen, f64::from(h));
            let panel = Rect::new(0.0, 0.0, f64::from(w), f64::from(h));
            let smallest = measures.floor * audit::DESKTOP_CAPS * measures.unit;
            let reach = if pass.consoles.is_some() { Vec::new() } else { pad_reach(&tree.root, focus, &walked) };
            let found = [
                ("fit", fit(&seen.root, panel, measures.slack, smallest), seen),
                ("reach", reach, &tree),
                ("pointer", pointer_reach(&tree.root, &pointed), &tree),
                ("chrome", no_chrome(&seen.root, WalkForm::BigScreen), seen),
                ("name", named(&seen.root), seen),
                ("contrast", contrast(&seen.root), seen),
            ];
            for (kind, faults, judged) in found {
                for fault in faults {
                    let who: Vec<String> = items(&fault).into_iter().map(|i| described(judged, i)).collect();
                    lines.add(screen, kind, &format!("{w}x{h} {name}: {fault} ({})", who.join("; ")));
                }
            }
            if let Some(dir) = state {
                let file = format!("{}-{}-{w}x{h}", title(screen), name.replace(", ", "-").replace(' ', "-"));
                let declared = palette(variant);
                let header =
                    Header { state: &file, case: title(screen), kind: "menu", width: w, height: h, measures, palette: &declared };
                let _ = std::fs::write(dir.join(format!("{file}.spec")), spec::write(seen, &seen.root, &header, shown.map(|f| f.0), &walk));
                // The walk's last put-back moved the focus home, so whatever that started ends before the picture; a held clock moves nothing.
                if pass.held {
                    scene.wait(100);
                } else {
                    scene.rest();
                }
                let picture = dir.join(format!("{file}.png"));
                scene.grab(&picture.display().to_string());
                if let Some(fault) = misfit(&picture, (w as u32, h as u32)) {
                    lines.add(screen, "recording", &format!("{w}x{h} {name}: {fault}"));
                }
            }
            if let Some(missed) = pass.consoles.and_then(|all| swept(scene, all, (f64::from(w), f64::from(h)))) {
                lines.add(screen, "reach", &format!("{w}x{h} {name}: {missed}"));
            }
            if pass.press {
                scene.key_hold(RETURN, false);
                if !pass.held {
                    scene.rest();
                }
            }
        }
    }
}

/// Walks the focus by the pad, along the walk's own presses, to the element a reader hears as `name`; false when no walk leads there.
fn walk_to(scene: &mut Scene<'_>, tree: &Tree, walk: &[Step], name: &str) -> bool {
    let Some(start) = scene.focus(tree) else { return false };
    let target = (0..tree.nodes.len() as u32).find(|&i| tree.nodes[i as usize].operable && spoken(tree, i) == name);
    let Some(target) = target else { return false };
    // Breadth first over the walk's presses, each element reached by its fewest.
    let mut came: std::collections::HashMap<u32, (u32, UiButton)> = std::collections::HashMap::new();
    let mut queue = std::collections::VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        if at == target {
            break;
        }
        for step in walk.iter().filter(|s| s.from == at) {
            if let Landing::At(to) = step.to {
                if to != start && !came.contains_key(&to) {
                    came.insert(to, (at, step.button));
                    queue.push_back(to);
                }
            }
        }
    }
    let mut presses = Vec::new();
    let mut at = target;
    while at != start {
        let Some(&(from, button)) = came.get(&at) else { return false };
        presses.push(button);
        at = from;
    }
    for button in presses.into_iter().rev() {
        scene.key(key_of(button), 0);
    }
    true
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

/// The pad's sweep of a crowded Choose a console: Up and Left until the focus stops, then each row along to its right end and
/// back to its left with Down between, as a player looks for one console among many. None when every console's name was heard on the focus while it
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
    // Each row to both of its ends, since Down may land anywhere along the next (the orbits land nearest in x).
    loop {
        for along in [UiButton::Right, UiButton::Left] {
            let mut seen = Vec::new();
            while presses < limit && press(scene, along) {
                presses += 1;
                let now = focused(scene, size);
                if seen.contains(&now) {
                    break;
                }
                seen.push(now);
            }
        }
        presses += 1;
        if presses >= limit || !press(scene, UiButton::Down) {
            break;
        }
    }
    let missed: Vec<&String> = consoles.iter().filter(|c| !heard.contains(c)).collect();
    (!missed.is_empty()).then(|| {
        let first: Vec<&str> = missed.iter().take(5).map(|s| s.as_str()).collect();
        format!("the pad's sweep heard {} of {} consoles on the screen; not {}", consoles.len() - missed.len(), consoles.len(), first.join(", "))
    })
}

/// A recorded picture whose size is not its state's, in words; None when it is the state's size or was not written.
fn misfit(picture: &Path, size: (u32, u32)) -> Option<String> {
    png_size(picture).filter(|&got| got != size).map(|(pw, ph)| format!("the picture is {pw}x{ph}, not the state's size"))
}

/// A PNG's width and height, from its header.
fn png_size(path: &Path) -> Option<(u32, u32)> {
    let head = std::fs::read(path).ok()?;
    let word = |at: usize| head.get(at..at + 4).map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]));
    (head.get(12..16) == Some(b"IHDR")).then_some(())?;
    Some((word(16)?, word(20)?))
}

/// The words of a panic the harness raised (a view that never settles, a window that never shows).
fn panic_words(payload: &(dyn std::any::Any + Send)) -> String {
    payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned())).unwrap_or_default()
}

/// What one run of the audit is asked: the recordings' folder, a catalogue, the Games screen's section and the states.
pub struct Asked<'a> {
    pub state: Option<&'a Path>,
    pub catalogue: Option<(&'a str, &'a [Entry])>,
    pub section: Option<&'a str>,
    pub states: Vec<&'static State>,
}

/// Places a screen afresh and feeds it; the host's words, a line each, when it cannot.
fn placed(scene: &mut Scene<'_>, folder: &str, screen: Screen, feeding: impl FnOnce(u64) -> Result<(), String>) -> Result<u64, String> {
    let id = place(scene, folder, screen)?;
    feeding(id).map(|_| id)
}

/// The findings of the package at `folder` (named `shown` in the lines), its screens loaded in turn in each state asked.
/// With a catalogue (`man theme-check`, --catalogue), Choose a console alone is placed over it, its states labelled by its name.
pub fn run(folder: &Path, shown: &str, asked: &Asked<'_>) -> Result<Vec<String>, String> {
    let (state, section) = (asked.state, asked.section);
    let shown = shown.trim_end_matches('/');
    let text = std::fs::read_to_string(folder.join("theme.json")).map_err(|e| format!("{shown}/theme.json: {e}"))?;
    let mut lines = Lines { shown, out: Vec::new(), at: String::new() };
    let package = match emusen_theme_host::package::read(&text) {
        Ok(p) => p,
        Err(words) => return Ok(words.lines().map(|l| format!("{shown}/{l}")).collect()),
    };
    if let Some(dir) = state {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let real = std::fs::canonicalize(folder).map_err(|e| format!("{shown}: {e}"))?.display().to_string();
    std::panic::set_hook(Box::new(|_| {}));
    let mut harness = Harness::new();
    let library = crate::world::make(&mut harness, asked.catalogue.map(|(_, entries)| entries))?;
    let file = library.world.root.join("Audit.qml");
    std::fs::write(&file, SCENE).map_err(|e| format!("the scene: {e}"))?;
    let mut scene = harness.load(&format!("file://{}", file.display()), 1280, 800);
    // The covers' folder, the one a theme may show pictures from, as the program's start reads it from the settings.
    scene.set_property("host", "coverStores", &library.media.display().to_string());
    let overrides = package.colour_overrides();
    let both = [Variant::Dark, Variant::Light];
    if let (Some((name, _)), Some((view, names))) = (asked.catalogue, library.catalogue.as_ref()) {
        let screen = Screen::Consoles;
        logged();
        let label = format!(", {name}");
        let pass = Pass::plain(&both, &label, Some(names));
        let session = library.session(view, None)?;
        let fed = Fed { session: &session, declined: &[], reduced: true };
        match placed(&mut scene, &real, screen, |id| feed(id, &library, view, &fed, &overrides, section)) {
            Err(words) => words.lines().for_each(|w| lines.add(screen, "load", w)),
            Ok(id) => lines.guarded(screen, |lines| states(&mut scene, screen, id, &pass, state, lines)),
        }
        logged().iter().for_each(|w| lines.add(screen, "warning", w));
        return Ok(lines.out);
    }
    for asked_state in &asked.states {
        let screens: Vec<Screen> = Screen::ALL.into_iter().filter(|s| package.screens.contains(s) && asked_state.screens.contains(s)).collect();
        if screens.is_empty() {
            continue;
        }
        let view = library.shaped(asked_state.library)?;
        let session = library.session(&view, asked_state.game)?;
        let default = asked_state.name == DEFAULT;
        let label = if default { String::new() } else { format!(", {}", asked_state.name) };
        // The element a state walks the focus to is the place a reader hears by its name.
        let focus = asked_state.focus.and_then(|key| place_name(&view, key));
        for screen in screens {
            logged();
            match asked_state.clock {
                Some(ms) => held(&mut scene, &real, screen, (&library, &view, &session), ms, &label, (state, section, &overrides), &mut lines),
                None => {
                    let fed = Fed { session: &session, declined: asked_state.declined, reduced: true };
                    let pass = Pass { focus: focus.as_deref(), press: asked_state.press, ..Pass::plain(&both, &label, None) };
                    match placed(&mut scene, &real, screen, |id| feed(id, &library, &view, &fed, &overrides, section)) {
                        Err(words) => words.lines().for_each(|w| lines.add(screen, "load", w)),
                        Ok(id) => lines.guarded(screen, |lines| {
                            if default {
                                lines.at = "1280x800, judged still under reduced motion".to_owned();
                                scene.resize(1280, 800);
                                if let Some(words) = still(&mut scene) {
                                    lines.add(screen, "motion", &words);
                                }
                            }
                            states(&mut scene, screen, id, &pass, state, lines);
                        }),
                    }
                    if default && screen == Screen::Consoles {
                        // Placed afresh over the crowd, as the program shows Choose a console once the library holds 120 consoles.
                        let crowd = Pass::plain(&[Variant::Dark], ", 120 consoles", Some(&library.crowd_names));
                        match placed(&mut scene, &real, screen, |id| feed(id, &library, &library.crowd, &fed, &overrides, section)) {
                            Err(words) => words.lines().for_each(|w| lines.add(screen, "load", w)),
                            Ok(id) => lines.guarded(screen, |lines| states(&mut scene, screen, id, &crowd, state, lines)),
                        }
                    }
                }
            }
            logged().iter().for_each(|w| lines.add(screen, "warning", w));
        }
    }
    Ok(lines.out)
}

/// A place's name as Home's list gives it, by its key.
fn place_name(view: &LibraryView, key: &str) -> Option<String> {
    use emusen_theme_qt::view::Cell;
    theme_view::places(view, &[], &[]).into_iter().find_map(|row| match (row.first(), row.get(1)) {
        (Some(Cell::Text(k)), Some(Cell::Text(name))) if k == key => Some(name.clone()),
        _ => None,
    })
}

/// A screen with its animation clock held `ms` after it is placed, motion allowed: placed afresh in each variant, since a
/// change of variant animates its colours, then judged and recorded at the five sizes as every state is.
#[allow(clippy::too_many_arguments)]
fn held(
    scene: &mut Scene<'_>,
    folder: &str,
    screen: Screen,
    (library, view, session): (&Synthetic, &LibraryView, &SessionFacts),
    ms: i64,
    label: &str,
    (state, section, overrides): (Option<&Path>, Option<&str>, &[(bool, String, String)]),
    lines: &mut Lines<'_>,
) {
    for variant in [Variant::Dark, Variant::Light] {
        scene.hold_clock(Some(0));
        let fed = Fed { session, declined: &[], reduced: false };
        let id = placed(scene, folder, screen, |id| {
            feed(id, library, view, &fed, overrides, section)?;
            let mut kit = tokens(id).ok_or("the kit's Tokens are not in the theme's engine")?;
            kit.as_mut().set_variant(if variant == Variant::Dark { kinds::Variant::Dark } else { kinds::Variant::Light });
            Ok(())
        });
        match id {
            Err(words) => words.lines().for_each(|w| lines.add(screen, "load", w)),
            Ok(id) => {
                scene.hold_clock(Some(ms));
                let pass = Pass { held: true, ..Pass::plain(std::slice::from_ref(&variant), label, None) };
                lines.guarded(screen, |lines| states(scene, screen, id, &pass, state, lines));
            }
        }
        scene.hold_clock(None);
    }
}

#[cfg(test)]
mod tests {
    use super::misfit;

    /// The eight-byte signature and an IHDR chunk's head: all a reader of the size looks at.
    fn header(w: u32, h: u32) -> Vec<u8> {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        png.extend(w.to_be_bytes());
        png.extend(h.to_be_bytes());
        png.extend([8, 6, 0, 0, 0]);
        png
    }

    #[test]
    fn qt_3_7_a_picture_scaled_by_the_desktop_is_a_recording_finding() {
        let dir = std::env::temp_dir().join(format!("theme-audit-misfit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let picture = dir.join("Home-dark-1280x800.png");
        std::fs::write(&picture, header(1600, 1000)).unwrap();
        assert_eq!(misfit(&picture, (1280, 800)).as_deref(), Some("the picture is 1600x1000, not the state's size"));
        std::fs::write(&picture, header(1280, 800)).unwrap();
        assert_eq!(misfit(&picture, (1280, 800)), None, "a picture at its state's size");
        assert_eq!(misfit(&dir.join("absent.png"), (1280, 800)), None, "no picture, no size to judge");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
