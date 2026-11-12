//! Walking a console started with nothing in it, as console.rs and esde.rs do (EmuSen_Qt.md §7.9): the card's rows by
//! the pad and the pointer, the console's screen owning the pad, and the chord's Pause Card.
#![allow(dead_code)]

use emusen_kit_core::input::{PadControl, UiButton};
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::tree::{Node, Tree};
use emusen_qt_test::Scene;

/// Waits up to `seconds` for a condition the scene answers, letting Qt run between looks.
#[track_caller]
pub fn until(scene: &mut Scene<'_>, seconds: u64, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let start = std::time::Instant::now();
    while !done(scene) {
        if start.elapsed() >= std::time::Duration::from_secs(seconds) {
            panic!("{what}: not within {seconds} s; the focus in {}", focus_chain(scene).join(" < "));
        }
        scene.wait(8);
    }
}

/// The words of the item holding the focus.
pub fn focused_words(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    let Some(focus) = scene.focus(&tree) else { return String::new() };
    tree.nodes.iter().skip(focus as usize).find_map(|n| n.text.clone()).unwrap_or_default()
}

/// Down on a card until the focused row reads `words`, then A.
#[track_caller]
pub fn choose_row(scene: &mut Scene<'_>, words: &str) {
    for _ in 0..12 {
        if focused_words(scene).starts_with(words) {
            scene.pad(UiButton::Accept);
            return;
        }
        scene.pad(UiButton::Down);
    }
    panic!("no row reads {words}; the focus reads {}, in {}", focused_words(scene), focus_chain(scene).join(" < "));
}

/// The centre of the shown card's row whose label reads `label`.
pub fn row_centre(scene: &mut Scene<'_>, label: &str) -> (f64, f64) {
    let tree = scene.tree();
    card_rows(&tree)
        .into_iter()
        .find(|r| row_label(&tree, r) == Some(label))
        .map(|r| (r.x + r.w / 2.0, r.y + r.h / 2.0))
        .unwrap_or_else(|| panic!("no row reads {label}"))
}

/// The shown card's rows, top first: the one reading of them every row helper here makes.
fn card_rows(tree: &Tree) -> Vec<&Node> {
    let mut rows: Vec<&Node> = tree.nodes.iter().filter(|n| n.visible && n.ty.starts_with("MenuRow")).collect();
    rows.sort_by(|a, b| a.y.total_cmp(&b.y));
    rows
}

/// A card row's words of the part named `part`: its `label` or its `value`.
fn row_part<'t>(tree: &'t Tree, row: &Node, part: &str) -> Option<&'t str> {
    tree.nodes.iter().find(|n| n.parent == row.id && n.name == part).and_then(|n| n.text.as_deref())
}

/// A card row's label.
fn row_label<'t>(tree: &'t Tree, row: &Node) -> Option<&'t str> {
    row_part(tree, row, "label")
}

/// The shown card's rows, top first, each as its label and its value, empty where a row has none.
pub fn card_words(scene: &mut Scene<'_>) -> Vec<(String, String)> {
    let tree = scene.tree();
    card_rows(&tree)
        .into_iter()
        .map(|r| (row_label(&tree, r).unwrap_or_default().to_owned(), row_part(&tree, r, "value").unwrap_or_default().to_owned()))
        .collect()
}

/// Points over the middle of the game's screen, where the console's picture is drawn whatever its shape.
fn picture_points(scene: &mut Scene<'_>) -> Vec<(f64, f64)> {
    let tree = scene.tree();
    let game = &tree.nodes[tree.find_named("game").expect("the game's screen") as usize];
    let (cx, cy) = (game.x + game.w / 2.0, game.y + game.h / 2.0);
    let (dx, dy) = (game.w * 0.15, game.h * 0.15);
    [-1.0, -0.5, 0.0, 0.5, 1.0].iter().flat_map(|i| [-1.0, 0.0, 1.0].map(|j| (cx + i * dx, cy + j * dy))).collect()
}

/// The window's colours at `points`, as a grab of it reads them: red, green and blue.
fn seen(scene: &mut Scene<'_>, points: &[(f64, f64)]) -> Vec<[u8; 3]> {
    scene.colours_at(points).into_iter().flatten().map(|c| [c.r, c.g, c.b]).collect()
}

/// The blue the grab finds over the picture, on average: the client fixture's console core draws it by the button of its
/// pad held, B the most and Start less, and none with none held (emusen-native's test core under `AS_PLAYSTATION`,
/// its `PRESSED_BLUES`).
fn blue(scene: &mut Scene<'_>, points: &[(f64, f64)]) -> f64 {
    let seen = seen(scene, points);
    seen.iter().map(|c| f64::from(c[2])).sum::<f64>() / seen.len().max(1) as f64
}

/// The console's screen up, its picture drawn on the window as a grab reads it (more than one colour, not black), and
/// the pad the console's: B, A and Start (the RetroPad's, South, East and Start), each held on the pad, reach the console
/// as themselves, its picture turning blue while each is held (B bluer than Start, Start than A, as the core draws them) and back when it is let go, and not the program,
/// which stays on the console's screen unpaused.
#[track_caller]
pub fn console_owns_the_pad(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, console: &str) {
    until(scene, 20, "the console running", |s| {
        let problem = s.property("session", "problem");
        assert!(problem.is_empty() || problem.starts_with("The game plays without sound"), "the session: {problem}");
        s.property("session", "running") == "true" && s.property("screens", "shown") == "Game"
    });
    assert_eq!((scene.property("session", "console"), scene.property("session", "game")), (console.to_owned(), String::new()));
    until(scene, 20, "the first picture", |s| s.property("session", "firstPictureMs").parse::<f64>().is_ok_and(|ms| ms > 0.0));
    let points = picture_points(scene);
    // Nothing over the picture: the card that started or resumed it has slid away with its scrim.
    until(scene, 5, "the cards gone from over the console", |s| ["homeCardCanvas", "listCardCanvas", "cardCanvas"].iter().all(|c| s.property(c, "visible") == "false"));
    until(scene, 10, "the console's picture on the window", |s| {
        let seen = seen(s, &points);
        seen.len() == points.len() && seen.iter().any(|c| c != &seen[0]) && seen.iter().any(|c| c.iter().map(|v| u32::from(*v)).sum::<u32>() > 60)
    });
    until(scene, 5, "the console owning the pad", |s| s.pad_chord_only());
    scene.run_pad_clock();
    let mut held = Vec::new();
    for control in [PadControl::South, PadControl::East, PadControl::Start] {
        let button = emusen_qt_endymion::reading::sdl_button(control).unwrap();
        let before = blue(scene, &points);
        sdl.press(pad, button, true);
        until(scene, 5, &format!("{control:?} held reaching the console's picture (blue {before:.0} before)"), |s| blue(s, &points) > before + 60.0);
        // Steady: ten reads in a row, a few frames apart, agree within one step.
        let (mut last, mut same) = (-1.0, 0);
        until(scene, 5, &format!("{control:?}'s blue steady"), |s| {
            let now = blue(s, &points);
            same = if (now - last).abs() < 1.0 { same + 1 } else { 0 };
            last = now;
            same >= 10
        });
        held.push(last);
        sdl.press(pad, button, false);
        until(scene, 5, &format!("{control:?} let go in the console's picture"), |s| blue(s, &points) < before + 20.0);
        assert_eq!((scene.property("screens", "shown"), scene.property("session", "paused")), ("Game".into(), "false".into()), "{control:?} is the console's");
    }
    // The RetroPad's B is the bottom face and its A the right one: South reaches the core's B (0xFF), East its A (0x40),
    // and Start its Start (0xA0), each drawn apart, so each button reached the core as itself.
    let (b, a, start) = (held[0], held[1], held[2]);
    assert!(b > start + 40.0 && start > a + 40.0, "blues: B (South) {b:.0}, A (East) {a:.0}, Start {start:.0}: the core told them apart");
}

/// Guide pressed alone, held until the router reads it, then let go: with the Guide chords on, Guide alone opens the menu as it is let go (EmuSen_Kit.md §7.26).
#[track_caller]
pub fn guide_alone(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad) {
    let guide = emusen_qt_endymion::reading::sdl_button(PadControl::Guide).unwrap();
    sdl.press(pad, guide, true);
    until(scene, 5, "the router holding Guide", |s| s.pad_held(UiButton::Guide));
    sdl.press(pad, guide, false);
}

/// The middles of the shown card's rows, top first.
pub fn row_middles(scene: &mut Scene<'_>) -> Vec<(f64, f64)> {
    card_rows(&scene.tree()).iter().map(|n| (n.x + n.w / 2.0, n.y + n.h / 2.0)).collect()
}

/// Whether the shown card's row `index`, top first, lies under `at`.
pub fn row_box_holds(scene: &mut Scene<'_>, index: usize, at: (f64, f64)) -> bool {
    let tree = scene.tree();
    card_rows(&tree).get(index).is_some_and(|n| at.0 > n.x && at.0 < n.x + n.w && at.1 > n.y && at.1 < n.y + n.h)
}

/// The label of the shown card's row `index`, top first.
pub fn row_words(scene: &mut Scene<'_>, index: usize) -> String {
    let tree = scene.tree();
    let rows = card_rows(&tree);
    row_label(&tree, rows.get(index).expect("the card's row")).unwrap_or_default().to_owned()
}

/// The chord's Pause Card over the console, named as Mistress named its session.
#[track_caller]
pub fn pause(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, named: &str) {
    until(scene, 5, "the console owning the pad", |s| s.pad_chord_only());
    scene.run_pad_clock();
    guide_alone(scene, sdl, pad);
    until(scene, 10, "Guide pausing the console under the Pause Card", |s| s.property("session", "paused") == "true" && s.property("screens", "shown") == "Pause");
    until(scene, 5, "the card in place", |s| s.property("cardCanvas", "slide") == "0");
    assert!(scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some(named)), "the Pause Card names {named}");
}

/// The grid in place: its covers listed and holding the focus.
pub fn grid_ready(scene: &mut Scene<'_>) {
    until(scene, 5, "the grid's covers holding the focus", |s| s.property("gridCells", "count") != "0" && s.property("gridCells", "activeFocus") == "true");
}

/// A card's page in place: its focused row reads `words`.
pub fn row_focused(scene: &mut Scene<'_>, words: &str) {
    until(scene, 5, &format!("the focus on the row {words}"), |s| focused_words(s).starts_with(words));
}

/// Whether the shown card has a row whose label begins `label` ("Quit Game" for "Quit Game...").
pub fn row_shown(scene: &mut Scene<'_>, label: &str) -> bool {
    let tree = scene.tree();
    card_rows(&tree).into_iter().any(|r| row_label(&tree, r).is_some_and(|t| t.starts_with(label)))
}

/// The chord's Pause Card over the console named `named`, Leave, Quit Game and Yes: the session closed and `to` shown.
#[track_caller]
pub fn quit_game(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, named: &str, to: &str) {
    pause(scene, sdl, pad, named);
    choose_row(scene, "Leave");
    until(scene, 5, "the Leave page", |s| s.property("pause", "title") == "Leave" && row_shown(s, "Quit Game"));
    choose_row(scene, "Quit Game");
    until(scene, 5, "Quit Game's question", |s| row_shown(s, "Yes"));
    choose_row(scene, "Yes");
    until(scene, 10, &format!("Quit Game closing the console to {to}"), |s| s.property("session", "running") == "false" && s.property("screens", "shown") == to);
}

/// The chord's Pause Card over the console named `named`, Leave and Back to Library: the session suspended and `to` shown.
#[track_caller]
pub fn back_to_library(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, named: &str, to: &str) {
    pause(scene, sdl, pad, named);
    choose_row(scene, "Leave");
    until(scene, 5, "the Leave page", |s| s.property("pause", "title") == "Leave" && row_shown(s, "Back to Library"));
    choose_row(scene, "Back to Library");
    until(scene, 5, &format!("Back to Library showing {to}"), |s| s.property("screens", "shown") == to && s.property("cardCanvas", "visible") == "false");
}

/// The item holding the focus and those it sits in, each by its name and type, innermost first.
pub fn focus_chain(scene: &mut Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    let Some(at) = scene.focus(&tree) else { return Vec::new() };
    let mut out = Vec::new();
    let mut node = Some(&tree.nodes[at as usize]);
    while let Some(n) = node {
        out.push(format!("{}:{}", n.name, n.ty));
        node = tree.nodes.iter().find(|p| p.id == n.parent);
    }
    out
}

/// Whether the focus is inside the item named `name`.
pub fn focus_within(scene: &mut Scene<'_>, name: &str) -> bool {
    focus_chain(scene).iter().any(|n| n.split(':').next() == Some(name))
}
