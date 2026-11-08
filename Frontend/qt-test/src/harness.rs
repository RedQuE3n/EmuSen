//! The harness: one offscreen application per process, scenes loaded from a QML URL, the audit over their trees, and the pad through the router (EmuSen_Kit.md §7.9).

use core::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QQuickStyle, QString, QUrl};
use emusen_kit_core::audit::{self, Edge, Fault, State};
use emusen_kit_core::input::{ElementId, UiButton, WalkForm};
use emusen_kit_core::tokens::{self, Colour, ColourToken, Variant};
use emusen_kit_qt::kinds;
use emusen_kit_qt::pad::qobject::PadRouter;

use crate::ffi::ffi;
use crate::tree::Tree;

/// The four sizes the gate runs at (EmuSen_Interface.md §24.7).
pub const SIZES: [(i32, i32); 4] = [(1280, 800), (1920, 1080), (1280, 720), (3840, 2160)];

/// The engine is declared first so it drops before the application, as Qt requires.
pub struct Harness {
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
    pub fn new() -> Harness {
        if std::env::var_os("QT_QPA_PLATFORM").is_none() {
            std::env::set_var("QT_QPA_PLATFORM", "offscreen");
        }
        if std::env::var_os("QT_QUICK_BACKEND").is_none() {
            std::env::set_var("QT_QUICK_BACKEND", "software");
        }
        if std::env::var_os("QT_FORCE_STDERR_LOGGING").is_none() {
            std::env::set_var("QT_FORCE_STDERR_LOGGING", "1");
        }
        QQuickStyle::set_fallback_style(&QString::from("Basic"));
        QQuickStyle::set_style(&QString::from("EmuSen"));
        let app = QGuiApplication::new();
        let mut engine = QQmlApplicationEngine::new();
        let created = Arc::new(AtomicUsize::new(0));
        let slot = Arc::clone(&created);
        if let Some(engine) = engine.as_mut() {
            engine.on_object_created(move |_, object, _| slot.store(object as usize, Ordering::SeqCst)).release();
        }
        Harness { engine, app, created, now_ms: 0 }
    }

    /// Loads a scene whose root is a Window and shows it at a size; the window must come up exposed and active.
    pub fn load(&mut self, url: &str, width: i32, height: i32) -> Scene<'_> {
        self.created.store(0, Ordering::SeqCst);
        let engine = self.engine.as_mut().expect("the QML engine");
        engine.load(&QUrl::from(url));
        let mut waited = 0;
        while self.created.load(Ordering::SeqCst) == 0 && waited < 3000 {
            ffi::harness_wait(20);
            waited += 20;
        }
        let window = self.created.load(Ordering::SeqCst) as *mut QObject;
        assert!(!window.is_null(), "{url} did not create a root object in {waited} ms; Qt's words are above");
        let shown = unsafe { ffi::harness_show(Pin::new_unchecked(&mut *window), width, height) };
        assert!(shown, "{url}: the window did not come up exposed and active");
        ffi::harness_wait(50);
        Scene { harness: self, window }
    }

    fn router(&mut self) -> Option<Pin<&mut PadRouter>> {
        let engine = self.engine.as_mut()?;
        let object = unsafe { ffi::harness_singleton(engine, &QString::from("EmuSen.Kit"), &QString::from("PadRouter")) };
        if object.is_null() {
            return None;
        }
        let router = unsafe { <PadRouter as Upcast<QObject>>::from_base_ptr(object as *const QObject) } as *mut PadRouter;
        if router.is_null() {
            return None;
        }
        Some(unsafe { Pin::new_unchecked(&mut *router) })
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

    pub fn resize(&mut self, width: i32, height: i32) {
        ffi::harness_resize(self.window(), width, height);
        self.wait(30);
    }

    pub fn wait(&mut self, ms: i32) {
        ffi::harness_wait(ms);
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

    pub fn right_click(&mut self, x: f64, y: f64) {
        ffi::harness_click(self.window(), x, y, 2);
        self.wait(20);
    }

    pub fn key(&mut self, key: i32, modifiers: i32) {
        ffi::harness_key(self.window(), key, modifiers);
        self.wait(20);
    }

    pub fn wheel(&mut self, x: f64, y: f64, notches: i32) {
        ffi::harness_wheel(self.window(), x, y, notches * 120);
        self.wait(20);
    }

    /// The active focus item's number in a tree, if any.
    pub fn focus(&mut self, tree: &Tree) -> Option<u32> {
        let id = ffi::harness_focus(self.window());
        tree.index_of(id)
    }

    pub fn focus_item(&mut self, tree: &Tree, index: u32) -> bool {
        let id = tree.id_of(index);
        let done = ffi::harness_focus_item(self.window(), id);
        self.wait(20);
        done
    }

    pub fn grab(&mut self, path: &str) -> bool {
        ffi::harness_grab(self.window(), &QString::from(path))
    }

    /// A property of the object with this objectName (or the root's), as text.
    pub fn property(&mut self, object_name: &str, property: &str) -> String {
        ffi::harness_property(self.window(), &QString::from(object_name), &QString::from(property)).to_string()
    }

    /// A pad press through the router's queue, down then up, on the harness's own clock; the router's wall clock is off.
    pub fn pad(&mut self, button: UiButton) {
        let (down, up) = {
            let now = self.harness.now_ms;
            self.harness.now_ms += 32;
            (now, now + 16)
        };
        if let Some(mut router) = self.harness.router() {
            router.as_mut().set_clock_running(false);
            router.as_mut().push(kinds::pad_button(button), true);
            router.as_mut().pump_at(down);
            router.as_mut().push(kinds::pad_button(button), false);
            router.as_mut().pump_at(up);
        }
        self.wait(20);
    }

    /// Where a press of each direction from each operable element lands: the edges pad reach is judged by.
    pub fn record_edges(&mut self, tree: &Tree) -> Vec<Edge> {
        let mut edges = Vec::new();
        for from in tree.operable() {
            for button in [UiButton::Up, UiButton::Down, UiButton::Left, UiButton::Right] {
                if !self.focus_item(tree, from) {
                    continue;
                }
                self.pad(button);
                if let Some(to) = self.focus(tree) {
                    if to != from {
                        edges.push(Edge { from: ElementId(from), button, to: ElementId(to) });
                    }
                }
            }
        }
        edges
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
        let declared = palette(variant);
        let scale = (height / 800.0).max(1.0);
        let state = State {
            panel: audit::Rect::new(0.0, 0.0, width, height),
            form,
            focus: focus.map(ElementId),
            edges,
            declared: &declared,
            slack: if scale > 1.0 { 1.5 } else { 1.0 },
            smallest: if form == WalkForm::Desktop { 0.0 } else { audit::smallest_capitals() * scale },
        };
        audit::check(&tree.root, &state)
    }
}

/// The colours a state may paint: the variant's eighteen tokens.
pub fn palette(variant: Variant) -> Vec<Colour> {
    ColourToken::ALL.iter().map(|t| tokens::colour(variant, *t)).collect()
}
