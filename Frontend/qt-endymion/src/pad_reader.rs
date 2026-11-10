//! `PadReader`: Endymion's pads read on a thread of their own and put on the GUI thread through `qt_thread().queue`, one
//! tick at a time, as presses for `PadRouter.push` and the help bar's family (EmuSen_Kit.md §7.15); the same reader
//! reads them for a running game, which finds the hand-over by `gamePads` (§7.17).

use core::pin::Pin;
use std::sync::Arc;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use emusen_kit_qt::kinds::{self, Family};
use emusen_platform_client::pads::Pads;
use emusen_platform_client::{Lent, Platform};

use crate::game::GamePads;
use crate::reader::{Delivery, Reader};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Pad"]
        type PadButton = emusen_kit_qt::kinds::ffi::PadButton;
        #[namespace = "Families"]
        type Family = emusen_kit_qt::kinds::ffi::Family;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, platform_library)]
        #[qproperty(QString, only_pads)]
        #[qproperty(Family, family, READ, NOTIFY)]
        #[qproperty(i32, connected, READ, NOTIFY)]
        #[qproperty(QString, problem, READ, NOTIFY)]
        #[qproperty(u64, game_pads, READ, CONSTANT)]
        /// The game hears nothing while true (`PadRouter.guideHeld`), and after it, nothing until every button is let go.
        #[qproperty(bool, game_muted)]
        type PadReader = super::PadReaderRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// A button of the interface went down or up on the pads read, before the player's swap: what `PadRouter.push` takes.
        #[qsignal]
        fn button(self: Pin<&mut PadReader>, button: PadButton, down: bool);

        /// The seats or the pads connected changed, or a button went down or up on one (EmuSen_Input.md §8.9).
        #[qsignal]
        fn seats_changed(self: Pin<&mut PadReader>);

        /// A pad came or went, in the notice's words (EmuSen_Input.md §8.13.5); the pads present at the start say nothing.
        #[qsignal]
        fn notice(self: Pin<&mut PadReader>, words: QString);
    }

    impl cxx_qt::Threading for PadReader {}
    impl cxx_qt::Initialize for PadReader {}
}

/// The reader's properties: the platform library's path and, for a test, SDL's pads to read alone (their ids, separated
/// by spaces; empty reads every pad); the family of the pad last used, how many pads are read, and the last failure; the
/// game's hand-over, as a number a `SessionModel` is named.
pub struct PadReaderRust {
    platform_library: QString,
    only_pads: QString,
    family: Family,
    connected: i32,
    problem: QString,
    game_pads: u64,
    game_muted: bool,
    game: Arc<GamePads>,
    reader: Option<Reader>,
}

impl Default for PadReaderRust {
    fn default() -> Self {
        let game = Arc::new(GamePads::default());
        PadReaderRust {
            game_pads: GamePads::address(&game),
            game_muted: false,
            game,
            platform_library: QString::default(),
            only_pads: QString::default(),
            family: kinds::family_of(emusen_kit_core::glyph::PadFamily::Generic),
            connected: 0,
            problem: QString::default(),
            reader: None,
        }
    }
}

/// `onlyPads` read: none for empty text, else every id the text holds.
fn only(text: &QString) -> Option<Vec<u32>> {
    let text = text.to_string();
    let ids: Vec<u32> = text.split([' ', ',']).filter_map(|id| id.trim().parse().ok()).collect();
    (!text.trim().is_empty()).then_some(ids)
}

/// SDL lent to the platform library at `path` and its pads made, on the reader's thread.
fn open_pads(path: &str) -> Result<Pads, String> {
    let words = |e: emusen_platform_client::Error| e.to_string();
    let platform: Arc<Platform> = Platform::load(path).map_err(words)?;
    let sdl = Lent::open("libSDL3.so.0").map_err(words)?;
    platform.lend_sdl(&sdl).map_err(words)?;
    Pads::sdl(&platform).map_err(words)
}

impl cxx_qt::Initialize for qobject::PadReader {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_platform_library_changed(|reader| reader.start()).release();
        self.as_mut().on_game_muted_changed(|reader| reader.game.set_muted(reader.game_muted)).release();
        self.as_mut()
            .on_only_pads_changed(|reader| {
                if let Some(running) = reader.reader.as_ref() {
                    running.show_only(only(&reader.only_pads));
                }
            })
            .release();
    }
}

impl qobject::PadReader {
    /// Starts reading once the platform library is named; a second name is not taken while a reader runs.
    fn start(mut self: Pin<&mut Self>) {
        let path = self.platform_library.to_string();
        if path.is_empty() || self.reader.is_some() {
            return;
        }
        let qt = self.qt_thread();
        let reader = Reader::spawn(
            move || open_pads(&path),
            only(&self.only_pads),
            Some(self.game.clone()),
            move |delivery| qt.queue(move |reader: Pin<&mut qobject::PadReader>| reader.apply(delivery)).is_ok(),
        );
        self.as_mut().rust_mut().reader = Some(reader);
    }

    /// A tick applied on the GUI thread: each press emitted in order, then the family, the count and the problem published.
    fn apply(mut self: Pin<&mut Self>, delivery: Delivery) {
        let tick = match delivery {
            Ok(tick) => tick,
            Err(words) => return self.publish_problem(QString::from(words.as_str())),
        };
        for event in &tick.events {
            self.as_mut().button(kinds::pad_button(event.button), event.down);
        }
        if tick.seats {
            self.as_mut().seats_changed();
        }
        for words in &tick.notices {
            self.as_mut().notice(QString::from(words.as_str()));
        }
        if let Some(family) = tick.family.map(kinds::family_of).filter(|f| *f != self.family) {
            self.as_mut().rust_mut().family = family;
            self.as_mut().family_changed();
        }
        if let Some(count) = tick.connected.map(|n| i32::try_from(n).unwrap_or(i32::MAX)).filter(|n| *n != self.connected) {
            self.as_mut().rust_mut().connected = count;
            self.as_mut().connected_changed();
        }
        self.publish_problem(QString::default());
    }

    fn publish_problem(mut self: Pin<&mut Self>, problem: QString) {
        if self.problem != problem {
            self.as_mut().rust_mut().problem = problem;
            self.problem_changed();
        }
    }
}
