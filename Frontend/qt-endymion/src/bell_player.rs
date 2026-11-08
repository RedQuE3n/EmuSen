//! `BellPlayer`: the kit's cues played through Endymion's interface player, each rendered once by kit-core and kept by
//! the player under its name (EmuSen_Kit.md §7.15). The player is made off the GUI thread and comes back through
//! `qt_thread().queue`.

use core::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use emusen_kit_core::sound::{Cue as SoundCue, DEFAULT_VOLUME};
use emusen_kit_qt::kinds::{self, Cue};
use emusen_platform_client::sounds::InterfacePlayer;
use emusen_platform_client::{Lent, Platform};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Cues"]
        type Cue = emusen_kit_qt::kinds::ffi::Cue;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, platform_library)]
        #[qproperty(f64, volume)]
        #[qproperty(bool, ready, READ, NOTIFY)]
        #[qproperty(QString, problem, READ, NOTIFY)]
        type BellPlayer = super::BellPlayerRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        /// Plays a cue, replacing the one still playing; nothing before the player is ready.
        #[qinvokable]
        fn play(self: Pin<&mut BellPlayer>, cue: Cue);
    }

    impl cxx_qt::Threading for BellPlayer {}
    impl cxx_qt::Initialize for BellPlayer {}
}

/// The name each cue is kept under, in kit-core's order.
const KEYS: [&str; SoundCue::ALL.len()] = ["move", "section", "choose", "back", "favourite", "launch", "saved", "loaded", "refused"];

/// The name a cue is kept under by the player.
pub fn key(cue: SoundCue) -> &'static str {
    SoundCue::ALL.iter().position(|c| *c == cue).map_or(KEYS[0], |i| KEYS[i])
}

pub struct BellPlayerRust {
    platform_library: QString,
    volume: f64,
    ready: bool,
    problem: QString,
    player: Option<InterfacePlayer>,
    opening: bool,
}

impl Default for BellPlayerRust {
    fn default() -> Self {
        BellPlayerRust {
            platform_library: QString::default(),
            volume: DEFAULT_VOLUME,
            ready: false,
            problem: QString::default(),
            player: None,
            opening: false,
        }
    }
}

/// The player made once SDL is lent, every cue rendered and remembered: 48 kHz stereo floats, little-endian, as the player keeps them.
pub fn open_player(path: &str) -> Result<InterfacePlayer, String> {
    let words = |e: emusen_platform_client::Error| e.to_string();
    let platform = Platform::load(path).map_err(words)?;
    let sdl = Lent::open("libSDL3.so.0").map_err(words)?;
    platform.lend_sdl(&sdl).map_err(words)?;
    let player = InterfacePlayer::new(&platform).map_err(words)?;
    for cue in SoundCue::ALL {
        let bytes: Vec<u8> = cue.samples().iter().flat_map(|s| s.to_le_bytes()).collect();
        player.remember(key(cue), &bytes).map_err(words)?;
    }
    Ok(player)
}

impl cxx_qt::Initialize for qobject::BellPlayer {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_platform_library_changed(|player| player.open()).release();
        self.as_mut()
            .on_volume_changed(|mut player| {
                let volume = player.volume as f32;
                let set = player.player.as_ref().map(|p| p.set_volume(volume));
                if let Some(Err(e)) = set {
                    player.as_mut().publish(None, QString::from(e.to_string().as_str()));
                }
            })
            .release();
    }
}

impl qobject::BellPlayer {
    pub fn play(mut self: Pin<&mut Self>, cue: Cue) {
        let Some(cue) = kinds::sound_cue(cue) else { return };
        let played = self.player.as_ref().map(|p| p.play(key(cue)));
        if let Some(Err(e)) = played {
            self.as_mut().publish(None, QString::from(e.to_string().as_str()));
        }
    }

    /// Makes the player off the GUI thread once the platform library is named; it comes back through the queue.
    fn open(mut self: Pin<&mut Self>) {
        let path = self.platform_library.to_string();
        if path.is_empty() || self.opening || self.player.is_some() {
            return;
        }
        self.as_mut().rust_mut().opening = true;
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let opened = open_player(&path);
            let _ = qt.queue(move |mut player: Pin<&mut qobject::BellPlayer>| {
                player.as_mut().rust_mut().opening = false;
                match opened {
                    Ok(made) => {
                        let _ = made.set_volume(player.volume as f32);
                        player.as_mut().rust_mut().player = Some(made);
                        player.publish(Some(true), QString::default());
                    }
                    Err(words) => player.publish(Some(false), QString::from(words.as_str())),
                }
            });
        });
    }

    fn publish(mut self: Pin<&mut Self>, ready: Option<bool>, problem: QString) {
        if let Some(ready) = ready.filter(|r| *r != self.ready) {
            self.as_mut().rust_mut().ready = ready;
            self.as_mut().ready_changed();
        }
        if self.problem != problem {
            self.as_mut().rust_mut().problem = problem;
            self.problem_changed();
        }
    }
}
