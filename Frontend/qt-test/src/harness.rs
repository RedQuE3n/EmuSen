//! The harness: one offscreen application per process, scenes loaded from a QML URL, the audit over their trees, and the pad through the router (EmuSen_Kit.md §7.9).

use core::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use cxx_qt_lib::{QGuiApplication, QMap, QMapPair_QString_QVariant, QQmlApplicationEngine, QQuickStyle, QString, QUrl};
use emusen_kit_core::audit::{self, Edge, Fault, State};
use emusen_kit_core::input::{ElementId, UiButton, WalkForm};
use emusen_kit_core::tokens::{self, Colour, ColourToken, Variant};
use emusen_kit_qt::kinds;
use emusen_kit_qt::pad::qobject::PadRouter;
use emusen_kit_qt::tokens::qobject::Tokens;

use crate::ffi::ffi;
use crate::tree::Tree;

/// How long a view may take to come to rest before a state is refused: over twelve of the longest motion token's steps (Reel, 400 ms).
pub const SETTLE_MS: i32 = 5000;

/// Qt's `qt.quick.dirty` account of a span, one entry a synced frame: the items it found dirty and why, or none.
fn dirty_items(log: &str) -> String {
    let mut frames: Vec<Vec<&str>> = Vec::new();
    for line in log.lines() {
        match line.strip_prefix("QSGNode: ") {
            Some(item) => frames.last_mut().map_or((), |f| f.push(item)),
            None => frames.push(Vec::new()),
        }
    }
    let each: Vec<String> = frames.iter().map(|f| if f.is_empty() { "[no item]".to_owned() } else { format!("[{}]", f.join(", ")) }).collect();
    if each.is_empty() { "no frame synced".to_owned() } else { each.join(" ") }
}

/// The scene's clock, where it has one: the object and property whose new minute `assert_still` reads as a change.
pub const CLOCK: (&str, &str) = ("clock", "time");

/// `PathView.StrictlyEnforceRange`: the current item held in the highlight range, so the offset follows the current index.
const STRICTLY_ENFORCE_RANGE: f64 = 2.0;

/// The kit's property an action declares a value Left and Right step with (`ActionButton.stepped`), and the most
/// presses the walk makes on through one (EmuSen_Kit.md §7.58).
const STEPPED: &str = "stepped";
const STEPS_PRESSED: usize = 64;

/// Messages Qt logs of its own accord that no code of ours causes, each with its reason (EmuSen_Kit.md §7.14).
pub const QT_INTERNAL: [(&str, &str); 2] = [
    // Qt Multimedia converting a clip's frame to a picture where there is no RHI, the software backend's path (EmuSen_Kit.md §7.50).
    ("No RHI backend. Using CPU conversion.", "Qt Multimedia on the software backend"),
    // QtCore's, once Qt Multimedia starts in a process whose application was made on a test runner's thread; the program makes it on main.
    ("QApplication was not created in the main() thread", "the test runner's thread"),
];

/// The sizes the gate runs at (EmuSen_Interface.md §24.7): 1280 × 800, a Steam Deck–class size kept as a fit size; the
/// handheld's own 1920 × 1200 (a Legion Go S, decided 2026-11-08); 1920 × 1080, 1280 × 720 and 3840 × 2160.
pub const SIZES: [(i32, i32); 5] = [(1280, 800), (1920, 1200), (1920, 1080), (1280, 720), (3840, 2160)];

/// The engine and the held clock are declared first so they drop before the application, as Qt requires.
pub struct Harness {
    /// Qt's animation clock while the harness holds it (EmuSen_Kit.md §7.36).
    clock: Option<cxx::UniquePtr<ffi::HeldClock>>,
    engine: cxx::UniquePtr<QQmlApplicationEngine>,
    app: cxx::UniquePtr<QGuiApplication>,
    created: Arc<AtomicUsize>,
    now_ms: i32,
}

impl Default for Harness {
    fn default() -> Self {
        Harness::new()
    }
}

impl Harness {
    /// Offscreen, the software backend, Qt's log on the console (Fedora's Qt sends it to journald when stderr is no TTY) and the EmuSen style, set before Qt starts; one per process.
    /// Always one device pixel a window pixel, whatever the desktop's scale, so a grab is the size its state names.
    pub fn new() -> Harness {
        std::env::set_var("QT_ENABLE_HIGHDPI_SCALING", "0");
        std::env::set_var("QT_SCALE_FACTOR", "1");
        std::env::remove_var("QT_SCREEN_SCALE_FACTORS");
        if std::env::var_os("QT_QPA_PLATFORM").is_none() {
            std::env::set_var("QT_QPA_PLATFORM", "offscreen");
        }
        if std::env::var_os("QT_QUICK_BACKEND").is_none() {
            std::env::set_var("QT_QUICK_BACKEND", "software");
        }
        if std::env::var_os("QT_FORCE_STDERR_LOGGING").is_none() {
            std::env::set_var("QT_FORCE_STDERR_LOGGING", "1");
        }
        ffi::harness_keep_messages();
        QQuickStyle::set_fallback_style(&QString::from("Basic"));
        QQuickStyle::set_style(&QString::from("EmuSen"));
        let app = QGuiApplication::new();
        let mut engine = QQmlApplicationEngine::new();
        let created = Arc::new(AtomicUsize::new(0));
        let slot = Arc::clone(&created);
        if let Some(engine) = engine.as_mut() {
            engine.on_object_created(move |_, object, _| slot.store(object as usize, Ordering::SeqCst)).release();
        }
        Harness { clock: None, engine, app, created, now_ms: 0 }
    }

    /// Loads a scene whose root is a Window and shows it at a size; the window must come up exposed and active.
    pub fn load(&mut self, url: &str, width: i32, height: i32) -> Scene<'_> {
        self.load_prepared(url, width, height, |_| {})
    }

    /// `load`, with `prepare` given the window after it is made and before it is shown, as a host that adopts a device
    /// prepares its window (EmuSen_Kit.md §7.16).
    pub fn load_prepared(&mut self, url: &str, width: i32, height: i32, prepare: impl FnOnce(*mut QObject)) -> Scene<'_> {
        self.load_initial(url, width, height, &QMap::default(), prepare)
    }

    /// `load_prepared`, the root's properties set to `initial` as it is made, as a program's start sets them with
    /// QQmlApplicationEngine::setInitialProperties (EmuSen_Qt.md §4.9); later loads start from none.
    pub fn load_initial(
        &mut self,
        url: &str,
        width: i32,
        height: i32,
        initial: &QMap<QMapPair_QString_QVariant>,
        prepare: impl FnOnce(*mut QObject),
    ) -> Scene<'_> {
        self.created.store(0, Ordering::SeqCst);
        let mut engine = self.engine.as_mut().expect("the QML engine");
        engine.as_mut().set_initial_properties(initial);
        engine.as_mut().load(&QUrl::from(url));
        engine.as_mut().set_initial_properties(&QMap::default());
        let mut waited = 0;
        while self.created.load(Ordering::SeqCst) == 0 && waited < 3000 {
            ffi::harness_wait(20);
            waited += 20;
        }
        let window = self.created.load(Ordering::SeqCst) as *mut QObject;
        assert!(!window.is_null(), "{url} did not create a root object in {waited} ms; Qt's words are above");
        prepare(window);
        let shown = unsafe { ffi::harness_show(Pin::new_unchecked(&mut *window), width, height) };
        assert!(shown, "{url}: the window did not come up exposed and active");
        ffi::harness_wait(50);
        Scene { harness: self, window }
    }

    fn singleton<T: Upcast<QObject>>(&mut self, name: &str) -> Option<Pin<&mut T>> {
        let engine = self.engine.as_mut()?;
        let object = unsafe { ffi::harness_singleton(engine, &QString::from("EmuSen.Kit"), &QString::from(name)) };
        if object.is_null() {
            return None;
        }
        let typed = unsafe { T::from_base_ptr(object as *const QObject) } as *mut T;
        if typed.is_null() {
            return None;
        }
        Some(unsafe { Pin::new_unchecked(&mut *typed) })
    }

    fn router(&mut self) -> Option<Pin<&mut PadRouter>> {
        self.singleton::<PadRouter>("PadRouter")
    }

    /// The kit's Tokens singleton, to switch the variant or the form under a scene.
    pub fn tokens(&mut self) -> Option<Pin<&mut Tokens>> {
        self.singleton::<Tokens>("Tokens")
    }

    /// Every warning and error Qt has logged since the last call, QML's own among them.
    pub fn messages(&mut self) -> Vec<String> {
        let all = ffi::harness_messages().to_string();
        all.split('\u{1e}').filter(|m| !m.is_empty()).map(str::to_owned).collect()
    }

    /// Fails on any message Qt logged that is not one of `QT_INTERNAL`'s, each named with the reason it is Qt's (EmuSen_Kit.md §7.14).
    pub fn assert_quiet(&mut self) {
        let ours: Vec<String> = self.messages().into_iter().filter(|m| !QT_INTERNAL.iter().any(|(text, _)| m.contains(text))).collect();
        assert!(ours.is_empty(), "Qt logged {} warnings or errors:\n{}", ours.len(), ours.join("\n"));
    }

    pub fn process_events(&self) {
        if let Some(app) = self.app.as_ref() {
            app.process_events();
        }
    }
}

/// A shown scene: its window, and the verbs over it.
pub struct Scene<'a> {
    harness: &'a mut Harness,
    window: *mut QObject,
}

impl Scene<'_> {
    fn window(&mut self) -> Pin<&mut QObject> {
        unsafe { Pin::new_unchecked(&mut *self.window) }
    }

    /// Resizes the window, waits for Qt to lay every item out, and settles its views.
    pub fn resize(&mut self, width: i32, height: i32) {
        ffi::harness_resize(self.window(), width, height);
        let tree = self.tree();
        self.settle(&tree);
    }

    /// Waits until every view of the tree is at rest as Qt declares it: a flickable or path view neither moving nor
    /// flicking, and a path view that keeps its current item in its highlight range at the offset that item rests at,
    /// Qt's `fmod(count - currentIndex, count)`, since Qt declares a step to a current index as no movement
    /// (EmuSen_Kit.md §7.11). It polls those declarations and fails past `SETTLE_MS`, so an unsettled state is never read.
    pub fn settle(&mut self, tree: &Tree) {
        let views: Vec<(u64, bool)> = tree
            .nodes
            .iter()
            .filter(|n| n.kept.contains_key("contentY") || n.kept.contains_key("offset"))
            .map(|n| (n.id.parse().unwrap_or(0), n.kept.contains_key("offset")))
            .collect();
        let mut waited = 0;
        while let Some(&(id, _)) = views.iter().find(|&&(id, path)| !self.at_rest(id, path)) {
            if waited >= SETTLE_MS {
                let node = tree.nodes.iter().find(|n| n.id.parse::<u64>().ok() == Some(id));
                let named = node.map(|n| format!("{} {}", n.ty, n.name)).unwrap_or_default();
                let mut get = |property: &str| ffi::harness_get(self.window(), id, &QString::from(property));
                let (moving, flicking) = (get("moving"), get("flicking"));
                panic!("a view ({id:#x} {named}; moving {moving}, flicking {flicking}) still moving after {SETTLE_MS} ms");
            }
            self.wait(10);
            waited += 10;
        }
    }

    fn at_rest(&mut self, id: u64, path: bool) -> bool {
        if ffi::harness_turning(self.window(), id) {
            return false;
        }
        let mut get = |property: &str| ffi::harness_get(self.window(), id, &QString::from(property));
        if get("moving") != 0.0 || get("flicking") != 0.0 {
            return false;
        }
        if !path || get("highlightRangeMode") != STRICTLY_ENFORCE_RANGE {
            return true;
        }
        let (count, current, offset) = (get("count"), get("currentIndex"), get("offset"));
        if count < 1.0 || current < 0.0 {
            return true;
        }
        let apart = (offset - (count - current).rem_euclid(count)).abs();
        apart < 1e-6 || (apart - count).abs() < 1e-6
    }

    pub fn wait(&mut self, ms: i32) {
        ffi::harness_wait(ms);
    }

    /// The harness's `assert_quiet`, for a test that holds a scene.
    pub fn assert_quiet(&mut self) {
        self.harness.assert_quiet();
    }

    /// The window's item tree, read into kit-core's audit items.
    pub fn tree(&mut self) -> Tree {
        let json = ffi::harness_tree(self.window()).to_string();
        Tree::from_json(&json).expect("the tree")
    }

    pub fn click(&mut self, x: f64, y: f64) {
        ffi::harness_click(self.window(), x, y, 1);
        self.wait(20);
    }

    /// The pointer moved to a point and left resting there, nothing pressed.
    pub fn hover(&mut self, x: f64, y: f64) {
        ffi::harness_hover(self.window(), x, y);
        self.wait(20);
    }

    /// The left button pressed at one point, moved in steps to another and let go there.
    pub fn drag(&mut self, from: (f64, f64), to: (f64, f64)) {
        ffi::harness_drag(self.window(), from.0, from.1, to.0, to.1);
        self.wait(20);
    }

    pub fn right_click(&mut self, x: f64, y: f64) {
        ffi::harness_click(self.window(), x, y, 2);
        self.wait(20);
    }

    pub fn key(&mut self, key: i32, modifiers: i32) {
        ffi::harness_key(self.window(), key, modifiers);
        self.wait(20);
    }

    /// Holds a key down (a press held, as a player holds A), or lets it up.
    pub fn key_hold(&mut self, key: i32, down: bool) {
        ffi::harness_key_hold(self.window(), key, down);
        self.wait(20);
    }

    /// Holds every animation's clock at `ms` since it was first held, moved there a frame at a time; `None` lets it run again
    /// on Qt's own driver (EmuSen_Kit.md §7.36).
    pub fn hold_clock(&mut self, ms: Option<i64>) {
        match ms {
            Some(ms) => {
                if let Some(clock) = self.harness.clock.get_or_insert_with(ffi::harness_clock_hold).as_mut() {
                    ffi::harness_clock_move(clock, ms);
                }
            }
            None => self.harness.clock = None,
        }
        self.wait(20);
    }

    pub fn wheel(&mut self, x: f64, y: f64, notches: i32) {
        ffi::harness_wheel(self.window(), x, y, notches * 120);
        self.wait(20);
    }

    /// The walkable element holding the active focus, by its number in a tree, if any; a focus on an item made since the
    /// tree was read (a carousel's card scrolled in) is found by its nearest ancestor the tree holds.
    pub fn focus(&mut self, tree: &Tree) -> Option<u32> {
        let id = ffi::harness_focus(self.window());
        if let Some(index) = tree.index_of(id) {
            return Some(tree.walkable_of(index));
        }
        let now = self.tree();
        let mut at = now.index_of(id)? as usize;
        loop {
            // The item itself, or one a view made again where the tree's stood (a delegate scrolled out and back).
            if let Some(index) = tree.same_as(&now.nodes[at]) {
                return Some(tree.walkable_of(index));
            }
            at = now.nodes.iter().position(|n| n.id == now.nodes[at].parent)?;
        }
    }

    /// The walk's `focus`, read through `live`, the objects that stand for the tree's items now, never through the tree's own
    /// numbers, whose objects a view may have deleted and whose addresses a new item may hold.
    fn landed(&mut self, tree: &Tree, live: &[u64]) -> Option<u32> {
        // A focus inside no element of the tree (a row a list made as it scrolled) lands on none, not on the list's content.
        let known = |id: u64| live.iter().position(|&l| l != 0 && l == id).map(|i| tree.walkable_of(i as u32)).filter(|&w| tree.nodes[w as usize].operable);
        let id = ffi::harness_focus(self.window());
        if let Some(index) = known(id) {
            return Some(index);
        }
        let now = self.tree();
        let mut at = now.index_of(id)? as usize;
        loop {
            if let Some(index) = now.nodes[at].id.parse().ok().and_then(known) {
                return Some(index);
            }
            at = now.nodes.iter().position(|n| n.id == now.nodes[at].parent)?;
        }
    }

    /// Where a press from `from` landed: another element, the same one, or none of the tree's.
    fn landing(&mut self, tree: &Tree, live: &[u64], from: u32) -> Landing {
        match self.landed(tree, live) {
            Some(to) if to == from => Landing::Same,
            Some(to) => Landing::At(to),
            None => Landing::Lost,
        }
    }

    pub fn focus_item(&mut self, tree: &Tree, index: u32) -> bool {
        let id = tree.id_of(index);
        let done = ffi::harness_focus_item(self.window(), id);
        self.wait(5);
        done
    }

    /// The frames the window has swapped since the first call, at Qt's `frameSwapped`.
    pub fn frames(&mut self) -> u64 {
        ffi::harness_frames(self.window())
    }

    /// The animations running under the window's items, one "Class in name" a line, the name its nearest named item's.
    pub fn running(&mut self) -> Vec<String> {
        ffi::harness_running(self.window()).to_string().lines().map(str::to_owned).collect()
    }

    /// Waits until no transition runs: every running animation loops without end (a design's resting motion), and with none
    /// running, the window swaps no frame for 100 ms; it fails past `SETTLE_MS`, naming what still runs (EmuSen_Kit.md §7.11).
    pub fn rest(&mut self) {
        let mut waited = 0;
        loop {
            let running = self.running();
            let passing: Vec<&String> = running.iter().filter(|r| !r.ends_with(" looping")).collect();
            if passing.is_empty() {
                if !running.is_empty() {
                    return;
                }
                let before = self.frames();
                self.wait(100);
                waited += 100;
                if self.frames() == before {
                    return;
                }
            } else {
                self.wait(50);
                waited += 50;
            }
            if waited >= SETTLE_MS {
                let running = self.running();
                panic!("a transition still running after {SETTLE_MS} ms: {}", if running.is_empty() { "frames still swapping".to_owned() } else { running.join(", ") });
            }
        }
    }

    /// Once every animation has stopped (within `SETTLE_MS`), the window swaps no frame for two seconds; a span in which the
    /// scene's clock read a new minute is measured once more, since that minute is a change (EmuSen_Qt.md §9.3.2).
    pub fn assert_still(&mut self, what: &str) {
        for again in [false, true] {
            let mut waited = 0;
            while !self.running().is_empty() && waited < SETTLE_MS {
                self.wait(50);
                waited += 50;
            }
            let minute = self.property(CLOCK.0, CLOCK.1);
            self.wait(100);
            ffi::harness_watch_dirty(true);
            let before = self.frames();
            self.wait(2000);
            let swapped = self.frames() - before;
            let dirty = dirty_items(&ffi::harness_dirty().to_string());
            ffi::harness_watch_dirty(false);
            let running = self.running();
            if swapped == 0 && running.is_empty() {
                return;
            }
            let ticked = self.property(CLOCK.0, CLOCK.1) != minute;
            assert!(
                ticked && !again,
                "{what}: {swapped} frames swapped in 2 s with nothing changing; running: {running:?}; dirty, as Qt synced them: {dirty}"
            );
        }
    }

    pub fn grab(&mut self, path: &str) -> bool {
        ffi::harness_grab(self.window(), &QString::from(path))
    }

    /// Sets a property of the object with this objectName (or the root's) from text, a dotted path reaching into an object property; Qt converts.
    pub fn set_property(&mut self, object_name: &str, property: &str, value: &str) -> bool {
        let done = ffi::harness_set_property(self.window(), &QString::from(object_name), &QString::from(property), &QString::from(value));
        self.wait(30);
        done
    }

    /// Deletes the named object at once, as the window's own teardown deletes a placed screen with it, then waits a moment.
    pub fn destroy(&mut self, object_name: &str) -> bool {
        let done = ffi::harness_destroy(self.window(), &QString::from(object_name));
        self.wait(30);
        done
    }

    /// `set_property` without the wait, so the next grab is the first frame drawn after the change.
    pub fn set_property_now(&mut self, object_name: &str, property: &str, value: &str) -> bool {
        ffi::harness_set_property(self.window(), &QString::from(object_name), &QString::from(property), &QString::from(value))
    }

    /// The variant the tokens paint, switched under the shown scene.
    pub fn set_variant(&mut self, variant: Variant) {
        if let Some(tokens) = self.harness.tokens() {
            tokens.set_variant(kinds::Variant { repr: if variant == Variant::Light { 1 } else { 0 } });
        }
        self.wait(30);
    }

    /// The kit's tokens of the scene's own engine, the program's, which every card reads.
    pub fn kit_tokens(&mut self) -> Option<Pin<&mut Tokens>> {
        self.harness.tokens()
    }

    /// The form the tokens read, switched under the shown scene.
    pub fn set_token_form(&mut self, form: WalkForm) {
        if let Some(tokens) = self.harness.tokens() {
            tokens.set_form(kinds::form_of(form));
        }
        self.wait(30);
    }

    /// The colours drawn at these window points, from one grab.
    pub fn colours_at(&mut self, points: &[(f64, f64)]) -> Vec<Option<Colour>> {
        let list = points.iter().map(|(x, y)| format!("{},{}", x.round() as i64, y.round() as i64)).collect::<Vec<_>>().join(";");
        let text = ffi::harness_colours_at(self.window(), &QString::from(list.as_str())).to_string();
        text.split(';').map(crate::tree::colour).collect()
    }

    /// How many points of a grid within 0.35 of each item's size from its centre differ by more than 24 levels from the
    /// colour most of them show, from one grab: an empty ring, filled or not, shows one colour there (EmuSen_Kit.md §7.12).
    pub fn ink_inside(&mut self, tree: &Tree, items: &[u32]) -> Vec<(u32, usize)> {
        let mut spans = Vec::new();
        let mut points = Vec::new();
        for &i in items {
            let n = &tree.nodes[i as usize];
            let (cx, cy, r) = (n.x + n.w / 2.0, n.y + n.h / 2.0, 0.35 * n.w.min(n.h));
            let start = points.len();
            for row in -4..=4 {
                for column in -4..=4 {
                    let (dx, dy) = (r * f64::from(column) / 4.0, r * f64::from(row) / 4.0);
                    if dx * dx + dy * dy <= r * r {
                        points.push((cx + dx, cy + dy));
                    }
                }
            }
            spans.push((i, start..points.len()));
        }
        let colours = self.colours_at(&points);
        let near =
            |a: &Colour, b: &Colour| [(a.r, b.r), (a.g, b.g), (a.b, b.b)].iter().all(|(x, y)| (i32::from(*x) - i32::from(*y)).abs() <= 24);
        spans
            .into_iter()
            .map(|(i, span)| {
                let seen: Vec<Colour> = colours[span].iter().flatten().copied().collect();
                let most = seen.iter().max_by_key(|c| seen.iter().filter(|o| near(c, o)).count()).copied();
                (i, most.map_or(0, |m| seen.iter().filter(|c| !near(c, &m)).count()))
            })
            .collect()
    }

    /// What is behind every visible text, from one grab with no words drawn, given to the tree for the contrast rule (EmuSen_Kit.md §7.10).
    pub fn sample_behind(&mut self, tree: &mut Tree, width: f64, height: f64) {
        let wanted = tree.behind_points(width, height);
        let points: Vec<(f64, f64)> = wanted.iter().flat_map(|(_, points)| points.iter().copied()).collect();
        ffi::harness_hide_words(self.window(), true);
        let colours = self.colours_at(&points);
        ffi::harness_hide_words(self.window(), false);
        let mut at = 0;
        for (index, points) in wanted {
            let behind = colours[at..at + points.len()].iter().flatten().copied().collect();
            at += points.len();
            tree.set_behind(index, behind);
        }
    }

    /// The pixels of the window's grab with no match in a reference picture within `reach` pixels and `tolerance` levels,
    /// either way round, or a loose box's own levels inside it, marked into `marks` when it is not empty; None when the
    /// picture cannot be read or its size differs.
    pub fn difference(&mut self, reference: &str, tolerance: i32, reach: i32, loose: &[([i32; 4], i32)], marks: &str) -> Option<usize> {
        let boxes: Vec<String> = loose.iter().map(|([x, y, w, h], levels)| format!("{x},{y},{w},{h},{levels}")).collect();
        let n = ffi::harness_difference(
            self.window(),
            &QString::from(reference),
            tolerance,
            reach,
            &QString::from(boxes.join(";").as_str()),
            &QString::from(marks),
        );
        usize::try_from(n).ok()
    }

    /// A property of the object with this objectName (or the root's), as text.
    pub fn property(&mut self, object_name: &str, property: &str) -> String {
        ffi::harness_property(self.window(), &QString::from(object_name), &QString::from(property)).to_string()
    }

    /// A number property of a tree's item by its id (a `Node`'s `id`, read as a number), in the item's own units; NaN where it has none.
    pub fn number(&mut self, id: u64, property: &str) -> f64 {
        ffi::harness_get(self.window(), id, &QString::from(property))
    }

    /// Counts the property's change signal from now, as a QSignalSpy does; false where it has none to count.
    pub fn watch(&mut self, object_name: &str, property: &str) -> bool {
        ffi::harness_watch(self.window(), &QString::from(object_name), &QString::from(property))
    }

    /// Records a list model's changes from now, as a view hears them; false where the object is no item model.
    pub fn watch_rows(&mut self, object_name: &str) -> bool {
        ffi::harness_watch_rows(self.window(), &QString::from(object_name))
    }

    /// From now, each change signal of the watched model's own property written into `rows_told` as the property's name,
    /// in order with the rows and resets; false where the model is not watched or the property has no change signal.
    pub fn watch_order(&mut self, object_name: &str, property: &str) -> bool {
        ffi::harness_watch_order(self.window(), &QString::from(object_name), &QString::from(property))
    }

    /// What the watched model told since the last read, as "first-last" for each dataChanged and "reset" for each reset,
    /// comma-separated in order; empty where nothing was told or it is not watched.
    pub fn rows_told(&mut self, object_name: &str) -> String {
        ffi::harness_rows_told(self.window(), &QString::from(object_name)).to_string()
    }

    /// How many times a watched property said it changed since `watch`; none where it is not watched.
    pub fn notified(&mut self, object_name: &str, property: &str) -> Option<usize> {
        usize::try_from(ffi::harness_notified(self.window(), &QString::from(object_name), &QString::from(property))).ok()
    }

    /// The pad router of the engine whose item holds the focus, as the program hands a hosted theme's engine its presses
    /// (EmuSen_Qt.md §3.5.2); the scene's own otherwise.
    fn router(&mut self) -> Option<Pin<&mut PadRouter>> {
        let object = ffi::harness_focused_singleton(self.window(), &QString::from("EmuSen.Kit"), &QString::from("PadRouter"));
        if object.is_null() {
            return self.harness.router();
        }
        // Safety: the kit registers PadRouter as this type in every engine, which keeps it alive as long as itself.
        let typed = unsafe { PadRouter::from_base_ptr(object as *const QObject) } as *mut PadRouter;
        Some(unsafe { Pin::new_unchecked(&mut *typed) })
    }

    /// A pad press through the router's queue, down then up, on the harness's own clock; the router's wall clock is off.
    pub fn pad(&mut self, button: UiButton) {
        let (down, up) = {
            let now = self.harness.now_ms;
            self.harness.now_ms += 32;
            (now, now + 16)
        };
        if let Some(mut router) = self.router() {
            router.as_mut().set_clock_running(false);
            router.as_mut().push(kinds::pad_button(button), true);
            router.as_mut().pump_at(down);
            router.as_mut().push(kinds::pad_button(button), false);
            router.as_mut().pump_at(up);
        }
        self.wait(5);
    }

    /// A pad button pressed and left held, on the harness's clock; `pad_up` lets it go. The router's repeats are its own.
    pub fn pad_down(&mut self, button: UiButton) {
        self.pad_edge(button, true);
    }

    /// The release of a button `pad_down` held.
    pub fn pad_up(&mut self, button: UiButton) {
        self.pad_edge(button, false);
    }

    fn pad_edge(&mut self, button: UiButton, down: bool) {
        let at = self.harness.now_ms;
        self.harness.now_ms += 16;
        if let Some(mut router) = self.harness.router() {
            router.as_mut().set_clock_running(false);
            router.as_mut().push(kinds::pad_button(button), down);
            router.as_mut().pump_at(at);
        }
        self.wait(5);
    }

    /// Whether the router gives the menu chord alone, as it does while a game owns the pad.
    pub fn pad_chord_only(&mut self) -> bool {
        self.router().is_some_and(|router| *router.chord_only())
    }

    /// Whether the router holds a button, as its last pump left it.
    pub fn pad_held(&mut self, button: UiButton) -> bool {
        self.router().is_some_and(|router| router.is_held(kinds::pad_button(button)))
    }

    /// The router pumped by the frame's timer on the wall clock again, as a real pad needs it; `pad` turns it off.
    pub fn run_pad_clock(&mut self) {
        if let Some(router) = self.harness.router() {
            router.set_clock_running(true);
        }
        if let Some(router) = self.router() {
            router.set_clock_running(true);
        }
    }

    /// Where a press of each direction from each operable element lands, kept whatever the outcome, as a recording keeps it;
    /// what each press moved (a list's selection, a tab, a value, a scroll) is put back before the next, as InterfaceSpec §3's walk does.
    pub fn walk(&mut self, tree: &Tree) -> Vec<Step> {
        self.walk_by(tree, |scene, button| scene.pad(button))
    }

    /// The walk with each press made by `press`: a real pad's, where `walk` pushes into the router's queue.
    pub fn walk_by(&mut self, tree: &Tree, mut press: impl FnMut(&mut Self, UiButton)) -> Vec<Step> {
        let mut steps = Vec::new();
        // The object standing for each of the tree's items: a view deletes a delegate scrolled out of it and makes another
        // when it scrolls back, so a press can leave the tree's numbers naming nothing, or a new item at a freed address.
        let mut live = tree.ids.clone();
        for from in tree.operable() {
            for button in [UiButton::Up, UiButton::Down, UiButton::Left, UiButton::Right] {
                let id = live[from as usize];
                if id == 0 || !ffi::harness_focus_item(self.window(), id) {
                    continue;
                }
                self.wait(5);
                press(self, button);
                // A focus on no item the walk knows is found by reading the tree, which marks what it reads as read.
                let mut to = self.landing(tree, &live, from);
                // A stepped action keeps the focus while it steps its value; the pad presses on, as a player does, until it moves.
                let mut more = STEPS_PRESSED;
                while to == Landing::Same && more > 0 && ffi::harness_get(self.window(), id, &QString::from(STEPPED)) == 1.0 {
                    press(self, button);
                    to = self.landing(tree, &live, from);
                    more -= 1;
                }
                let mut read = !live.contains(&ffi::harness_focus(self.window()));
                steps.push(Step { from, button, to });
                // Items made since the tree was read stand for the tree's own once found; a scope's focus is then given back to them.
                for _ in 0..3 {
                    self.put_back(tree, &live);
                    if !std::mem::take(&mut read) && !ffi::harness_made_since(self.window()) {
                        break;
                    }
                    live = tree.found_in(&self.tree());
                }
            }
        }
        steps
    }

    /// Writes back every selection, value, scroll and focus scope's remembered item that differs from the tree's, through
    /// `live`, then settles the views; a path's offset is not written, since it follows its current index by the view's own
    /// animation.
    fn put_back(&mut self, before: &Tree, live: &[u64]) {
        for (i, node) in before.nodes.iter().enumerate().filter(|(_, n)| !n.scoped.is_empty()) {
            let want = node.scoped.parse().ok().and_then(|id| before.index_of(id)).map_or(0, |w| live[w as usize]);
            if want != 0 && live[i] != 0 && ffi::harness_scoped(self.window(), live[i]) != want {
                ffi::harness_give_focus(self.window(), want);
            }
        }
        for (i, node) in before.nodes.iter().enumerate() {
            for (property, value) in node.kept.iter().filter(|(property, _)| *property != "offset") {
                let name = QString::from(property.as_str());
                let now = ffi::harness_get(self.window(), live[i], &name);
                if now.is_finite() && value.is_finite() && (now - value).abs() > 1e-6 {
                    ffi::harness_put(self.window(), live[i], &name, *value);
                }
            }
        }
        self.settle(before);
    }

    /// The edges pad reach is judged by: the presses that moved the focus.
    pub fn record_edges(&mut self, tree: &Tree) -> Vec<Edge> {
        edges(&self.walk(tree))
    }

    /// kit-core's rules over the tree at this window size, for a form, with the focus and the edges given.
    pub fn audit(
        &mut self,
        tree: &Tree,
        form: WalkForm,
        variant: Variant,
        focus: Option<u32>,
        edges: &[Edge],
        width: f64,
        height: f64,
    ) -> Vec<Fault> {
        self.audit_item(&tree.root, form, variant, focus, edges, width, height)
    }

    /// The same over one item and what is under it, as a player sees an open sheet alone.
    pub fn audit_item(
        &mut self,
        root: &audit::Item,
        form: WalkForm,
        variant: Variant,
        focus: Option<u32>,
        edges: &[Edge],
        width: f64,
        height: f64,
    ) -> Vec<Fault> {
        let declared = palette(variant);
        let measures = Measures::of(form, height);
        let state = State {
            panel: audit::Rect::new(0.0, 0.0, width, height),
            form,
            focus: focus.map(ElementId),
            edges,
            declared: &declared,
            slack: measures.slack,
            smallest: measures.floor * audit::DESKTOP_CAPS * measures.unit,
        };
        audit::check(root, &state)
    }
}

/// Where one press from an element landed: another element, the same one, or no element of the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Landing {
    At(u32),
    Same,
    Lost,
}

/// One press of the walk: from an element, a direction, and where the focus landed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub from: u32,
    pub button: UiButton,
    pub to: Landing,
}

/// The presses of a walk that moved the focus, as kit-core's pad reach reads them.
pub fn edges(steps: &[Step]) -> Vec<Edge> {
    steps
        .iter()
        .filter_map(|s| match s.to {
            Landing::At(to) => Some(Edge { from: ElementId(s.from), button: s.button, to: ElementId(to) }),
            Landing::Same | Landing::Lost => None,
        })
        .collect()
}

/// The audit's measures of a state: screen pixels to a design pixel, the slack, and the smallest words as a size of the desktop face (0 where none is held).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measures {
    pub unit: f64,
    pub slack: f64,
    pub floor: f64,
}

impl Measures {
    pub fn of(form: WalkForm, height: f64) -> Measures {
        let unit = (height / 800.0).max(1.0);
        let slack = if unit > 1.0 { 1.5 } else { 1.0 };
        if form == WalkForm::Desktop {
            Measures { unit: 1.0, slack, floor: 0.0 }
        } else {
            Measures { unit, slack, floor: audit::SMALLEST_TEXT }
        }
    }
}

/// The colours a state may paint: the variant's eighteen tokens.
pub fn palette(variant: Variant) -> Vec<Colour> {
    ColourToken::ALL.iter().map(|t| tokens::colour(variant, *t)).collect()
}
