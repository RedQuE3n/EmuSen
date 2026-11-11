//! `EsdeModel`: the painter's state on Qt, one per `View` (EmuSen_Qt.md §4.8.4). The library's texts are read and the
//! themes loaded on a data thread; the scene runs here on the GUI thread, and its lists reach QML through `lists.rs`.

use core::pin::Pin;
use std::cell::RefCell;
use std::rc::Rc;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QDateTime, QString, QTimeZone};
use emusen_esde::scene::{DateTime, System};
use emusen_kit_core::input::UiButton;

use crate::painter::{self, Action, Painter, ThemeCache};
use crate::shelves::{self, Settings};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, settings)]
        #[qproperty(QString, shelves)]
        #[qproperty(QString, media)]
        #[qproperty(QString, presence)]
        #[qproperty(f64, width)]
        #[qproperty(f64, height)]
        #[qproperty(bool, swap)]
        #[qproperty(QString, device)]
        /// The time the clock and the relative dates read, in seconds since 1970; 0 reads the wall clock.
        #[qproperty(i64, now)]
        #[qproperty(bool, themed, READ, NOTIFY)]
        #[qproperty(QString, problem, READ, NOTIFY)]
        #[qproperty(QString, view, READ, NOTIFY)]
        #[qproperty(bool, moving, READ, NOTIFY)]
        #[qproperty(i32, next_change_ms, READ, NOTIFY)]
        #[qproperty(i32, revision, READ, NOTIFY)]
        #[qproperty(QString, letters, READ, NOTIFY)]
        #[qproperty(QString, letters_mark, READ, NOTIFY)]
        #[qproperty(f64, letters_px, READ, NOTIFY)]
        #[qproperty(f64, letters_opacity, READ, NOTIFY)]
        /// The shown system's shelf, by its name in the shelves' text, and the primary's selected row.
        #[qproperty(QString, shelf, READ, NOTIFY)]
        #[qproperty(i32, selected, READ, NOTIFY)]
        /// The theme's navigation sounds, `{name: file}`, from the first system's theme as ES-DE reads them.
        #[qproperty(QString, sounds, READ, NOTIFY)]
        type EsdeModel = super::EsdeModelRust;
    }

    extern "RustQt" {
        /// The clock moved on by `ms` milliseconds (the FrameAnimation's frame time, or the Timer's wait).
        #[qinvokable]
        fn tick(self: Pin<&mut EsdeModel>, ms: f64);
        /// The router's press of a `Kit.PadButton`; the scene repeats a held direction itself, so a repeat is ignored.
        #[qinvokable]
        fn pressed(self: Pin<&mut EsdeModel>, button: i32, repeat: bool, jump: i32);
        #[qinvokable]
        fn released(self: Pin<&mut EsdeModel>, button: i32);
        #[qinvokable]
        fn hover(self: Pin<&mut EsdeModel>, row: i32);
        #[qinvokable]
        fn tap(self: Pin<&mut EsdeModel>, row: i32);
        #[qinvokable]
        fn wheel(self: Pin<&mut EsdeModel>, notches: i32);
        /// The painter's measurement of an element, by the `id` role's text.
        #[qinvokable]
        fn measured(self: Pin<&mut EsdeModel>, id: &QString, w: f64, h: f64);
        /// Reads the theme folder again, as an update on disk asks.
        #[qinvokable]
        fn reload(self: Pin<&mut EsdeModel>);
        /// A row of the primary selected as a step to it, as the host's List card jumps.
        #[qinvokable]
        fn select(self: Pin<&mut EsdeModel>, row: i32);

        #[qsignal]
        fn played(self: Pin<&mut EsdeModel>, path: QString);
        #[qsignal]
        fn favourite(self: Pin<&mut EsdeModel>, path: QString);
        #[qsignal]
        fn menu(self: Pin<&mut EsdeModel>);
        #[qsignal]
        fn options(self: Pin<&mut EsdeModel>);
        /// A navigation sound's name (navigate, select, back, quicksysselect) for the host to play.
        #[qsignal]
        fn cued(self: Pin<&mut EsdeModel>, name: QString);
    }

    impl cxx_qt::Threading for EsdeModel {}
    impl cxx_qt::Initialize for EsdeModel {}
}

// A read-only property set from Rust: the field changed and its signal raised, only when the value differs.
macro_rules! publish {
    ($m:expr, $field:ident, $signal:ident, $value:expr) => {{
        let value = $value;
        if $m.$field != value {
            $m.as_mut().rust_mut().$field = value;
            $m.as_mut().$signal();
        }
    }};
}

/// The four texts a load reads, as the GUI thread hands them to the load's thread: each QString shared, not copied, and
/// turned into Rust text on that thread (EmuSen_Qt.md §4.9).
pub struct LoadTexts(pub [QString; 4]);

// Sound: the copies are made here and moved whole, and Qt's implicit sharing counts atomically (EmuSen_Qt.md §4.7).
unsafe impl Send for LoadTexts {}

impl LoadTexts {
    /// The GUI thread's part: four shared copies, megabytes or not.
    pub fn of(texts: [&QString; 4]) -> LoadTexts {
        LoadTexts(texts.map(QString::clone))
    }

    /// The texts as Rust text, on the load's thread.
    pub fn read(self) -> [String; 4] {
        self.0.map(|t| t.to_string())
    }
}

/// The painter shared with the list models of the same view, on the GUI thread only.
pub type Shared = Rc<RefCell<Painter>>;

#[derive(Default)]
pub struct EsdeModelRust {
    settings: QString,
    shelves: QString,
    media: QString,
    presence: QString,
    width: f64,
    height: f64,
    swap: bool,
    device: QString,
    now: i64,
    themed: bool,
    problem: QString,
    view: QString,
    moving: bool,
    next_change_ms: i32,
    revision: i32,
    letters: QString,
    letters_mark: QString,
    letters_px: f64,
    letters_opacity: f64,
    shelf: QString,
    selected: i32,
    sounds: QString,
    pub shared: Shared,
    cache: ThemeCache,
    /// The latest load asked for; an older load that finishes later is dropped.
    load: u64,
    load_asked: bool,
}

/// The local time at `pinned` seconds since 1970, or the wall clock's at 0, for the clock element and the dates.
fn local_now(pinned: i64) -> DateTime {
    let now = if pinned == 0 { QDateTime::current_date_time() } else { QDateTime::from_secs_since_epoch(pinned, &QTimeZone::system_time_zone()) };
    let (d, t) = (now.date(), now.time());
    let u = |v: i32| u32::try_from(v).unwrap_or(0);
    DateTime::new(d.year(), u(d.month()), u(d.day()), u(t.hour()), u(t.minute()), u(t.second()))
}

impl cxx_qt::Initialize for qobject::EsdeModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_settings_changed(|m| m.ask_load()).release();
        self.as_mut().on_shelves_changed(|m| m.ask_load()).release();
        self.as_mut().on_media_changed(|m| m.ask_load()).release();
        self.as_mut().on_presence_changed(|m| m.ask_load()).release();
        self.as_mut().on_width_changed(|m| m.ask_load()).release();
        self.as_mut().on_height_changed(|m| m.ask_load()).release();
        self.as_mut().on_device_changed(|mut m| {
            let device = shelves::device(&m.device.to_string());
            m.as_mut().rust_mut().shared.borrow_mut().set_device(device);
            m.publish();
        })
        .release();
        self.as_mut().on_now_changed(|mut m| {
            let now = local_now(m.now);
            m.as_mut().rust_mut().shared.borrow_mut().set_now(now);
            m.publish();
        })
        .release();
        self.as_mut().on_swap_changed(|mut m| {
            let swap = *m.swap();
            m.as_mut().rust_mut().shared.borrow_mut().set_swap(swap);
            m.publish();
        })
        .release();
    }
}

impl qobject::EsdeModel {
    // One load for the texts and the size changed together: the library publishes its four texts one after another.
    fn ask_load(mut self: Pin<&mut Self>) {
        if !self.load_asked {
            self.as_mut().rust_mut().load_asked = true;
            let _ = self.qt_thread().queue(|mut m| {
                m.as_mut().rust_mut().load_asked = false;
                m.load();
            });
        }
    }

    // The texts read, the shelves built and their themes resolved off the GUI thread; the stage installed back on it.
    fn load(mut self: Pin<&mut Self>) {
        let (w, h) = (self.width.round() as i32, self.height.round() as i32);
        if w <= 0 || h <= 0 || self.settings.is_empty() {
            return;
        }
        let texts = LoadTexts::of([&self.settings, &self.shelves, &self.media, &self.presence]);
        let (qt, cache) = (self.qt_thread(), self.cache.clone());
        let load = self.load + 1;
        self.as_mut().rust_mut().load = load;
        std::thread::spawn(move || {
            let [settings_text, shelves_text, media_text, presence_text] = texts.read();
            let settings = shelves::settings(&settings_text);
            let found = shelves::shelves(&shelves_text, &media_text).map(|s| painter::systems(&cache, &settings, s, &shelves::presence(&presence_text), w, h));
            let _ = qt.queue(move |model| model.install(load, settings, found));
        });
    }

    // A text that did not read installs nothing and is the problem, so the host shows its words.
    fn install(mut self: Pin<&mut Self>, load: u64, settings: Settings, found: Result<Vec<System>, String>) {
        if load != self.load {
            return;
        }
        let (systems, problem) = match found {
            Ok(systems) => {
                let problem = painter::problem(&systems);
                (systems, problem)
            }
            Err(words) => (Vec::new(), words),
        };
        let problem = QString::from(problem.as_str());
        let (w, h) = (self.width, self.height);
        {
            let mut p = self.shared.borrow_mut();
            p.settings = settings;
            p.install(systems, w, h, local_now(self.now));
        }
        let themed = self.shared.borrow().stage.is_some();
        let sounds = QString::from(painter::sounds(&self.shared.borrow()).as_str());
        publish!(self, sounds, sounds_changed, sounds);
        publish!(self, problem, problem_changed, problem);
        publish!(self, themed, themed_changed, themed);
        self.publish();
    }

    /// The frame's state onto the properties, and the revision the list models refresh on.
    fn publish(mut self: Pin<&mut Self>) {
        let (view, moving, next, overlay, shelf, selected) = {
            let p = self.shared.borrow();
            let next = p.frame.next_change_ms.map_or(0, |ms| i32::try_from(ms).unwrap_or(i32::MAX).max(1));
            let (shelf, selected) = p.selection();
            (QString::from(p.view()), p.frame.moving, next, p.frame.overlay.clone(), QString::from(shelf), selected)
        };
        publish!(self, view, view_changed, view);
        publish!(self, shelf, shelf_changed, shelf);
        publish!(self, selected, selected_changed, selected);
        publish!(self, moving, moving_changed, moving);
        publish!(self, next_change_ms, next_change_ms_changed, next);
        let o = overlay.unwrap_or(emusen_esde::scene::ScrollLetters { letters: String::new(), star: false, px: 0.0, opacity: 0.0 });
        publish!(self, letters, letters_changed, QString::from(o.letters.as_str()));
        let mark = if o.star { emusen_esde::marks::MarkId::Star.name() } else { String::new() };
        publish!(self, letters_mark, letters_mark_changed, QString::from(mark.as_str()));
        publish!(self, letters_px, letters_px_changed, o.px);
        publish!(self, letters_opacity, letters_opacity_changed, o.opacity);
        publish!(self, revision, revision_changed, self.revision.wrapping_add(1));
    }

    fn act(mut self: Pin<&mut Self>, action: Option<Action>) {
        self.as_mut().publish();
        match action {
            Some(Action::Played(path)) => self.as_mut().played(QString::from(path.as_str())),
            Some(Action::Favourite(path)) => self.as_mut().favourite(QString::from(path.as_str())),
            Some(Action::Menu) => self.as_mut().menu(),
            Some(Action::Options) => self.as_mut().options(),
            Some(Action::Cue(name)) => self.as_mut().cued(QString::from(name)),
            None => {}
        }
    }

    pub fn tick(self: Pin<&mut Self>, ms: f64) {
        {
            let mut p = self.shared.borrow_mut();
            p.set_now(local_now(self.now));
            p.tick(ms);
        }
        self.publish();
    }

    pub fn pressed(self: Pin<&mut Self>, button: i32, repeat: bool, _jump: i32) {
        let Some(button) = usize::try_from(button).ok().and_then(|i| UiButton::ALL.get(i).copied()) else { return };
        let action = self.shared.borrow_mut().pressed(button, repeat);
        self.act(action);
    }

    pub fn released(self: Pin<&mut Self>, button: i32) {
        let Some(button) = usize::try_from(button).ok().and_then(|i| UiButton::ALL.get(i).copied()) else { return };
        self.shared.borrow_mut().released(button);
        self.publish();
    }

    pub fn hover(self: Pin<&mut Self>, row: i32) {
        self.shared.borrow_mut().hover(row);
        self.publish();
    }

    pub fn tap(self: Pin<&mut Self>, row: i32) {
        let action = self.shared.borrow_mut().tap(row);
        self.act(action);
    }

    pub fn wheel(self: Pin<&mut Self>, notches: i32) {
        self.shared.borrow_mut().wheel(notches);
        self.publish();
    }

    pub fn measured(self: Pin<&mut Self>, id: &QString, w: f64, h: f64) {
        let Ok(id) = id.to_string().parse::<u64>() else { return };
        let changed = self.shared.try_borrow_mut().is_ok_and(|mut p| p.measured(id, w, h));
        if changed {
            // Published from the event loop: a measurement arrives while a list is signalling its rows.
            let _ = self.qt_thread().queue(|m| m.publish());
        }
    }

    pub fn select(self: Pin<&mut Self>, row: i32) {
        self.shared.borrow_mut().choose(row);
        self.publish();
    }

    pub fn reload(self: Pin<&mut Self>) {
        self.cache.forget();
        self.load();
    }
}
