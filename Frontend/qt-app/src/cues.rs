//! `EsdeCues`: an ES-DE theme's navigation sounds through Endymion's interface player (EmuSen_Qt.md §4.15). Each of
//! ES-DE's seven names plays the theme's own file where it has one, else the kit's cue that stands for it, as Mistress's
//! `NavigationSounds` does; the player is made and the files decoded off the GUI thread, and come back through the queue.

use core::pin::Pin;
use std::collections::BTreeMap;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use emusen_kit_core::sound::{quick_select_samples, Cue};
use emusen_platform_client::sounds::{decode, InterfacePlayer};
use emusen_platform_client::Platform;
use emusen_qt_endymion::bell_player;

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
        #[qproperty(QString, platform_library)]
        /// The theme's sounds as the painter gives them, `{name: file}`.
        #[qproperty(QString, sounds)]
        /// AppSettings.NavigationSounds: off plays nothing.
        #[qproperty(bool, on)]
        #[qproperty(QString, problem, READ, NOTIFY)]
        /// The key of the sound last played and how many have played, as the player was asked.
        #[qproperty(QString, rung, READ, NOTIFY)]
        #[qproperty(i32, rings, READ, NOTIFY)]
        type EsdeCues = super::EsdeCuesRust;
    }

    extern "RustQt" {
        /// The scene's cue (`navigate`, `select`, `back`, `quicksysselect`) in the view it was asked in.
        #[qinvokable]
        fn play(self: Pin<&mut EsdeCues>, name: &QString, view: &QString);
        /// A game launched from the view.
        #[qinvokable]
        fn launch(self: Pin<&mut EsdeCues>);
        /// A game's favourite turned.
        #[qinvokable]
        fn favourite(self: Pin<&mut EsdeCues>);
    }

    impl cxx_qt::Threading for EsdeCues {}
    impl cxx_qt::Initialize for EsdeCues {}
}

/// ES-DE's seven navigation sounds and the kit's cue each falls back to; quick system select, which is no cue of the
/// kit's, rings the kit's own two bells for it (`QUICK_SELECT`), as Mistress's `NavigationSounds` does.
pub const NAMES: [(&str, Option<Cue>); 7] = [
    ("scroll", Some(Cue::Move)),
    ("systembrowse", Some(Cue::Section)),
    ("quicksysselect", None),
    ("select", Some(Cue::Choose)),
    ("back", Some(Cue::Back)),
    ("favorite", Some(Cue::Favourite)),
    ("launch", Some(Cue::Launch)),
];

/// The key the kit's quick system select is kept under.
const QUICK_SELECT: &str = "kit:quicksysselect";

/// ES-DE's name for a scene's cue: a step is `systembrowse` on the system view and `scroll` in a gamelist.
pub fn esde_name<'a>(cue: &'a str, view: &str) -> &'a str {
    match (cue, view) {
        ("navigate", "system") => "systembrowse",
        ("navigate", _) => "scroll",
        _ => cue,
    }
}

/// The key the player keeps a sound under: the theme's file where it gave one, else the kit's cue; none for no name of the seven.
pub fn key_of(name: &str, themed: &[String]) -> Option<String> {
    let cue = NAMES.iter().find(|(n, _)| *n == name)?.1;
    Some(if themed.iter().any(|t| t == name) { format!("theme:{name}") } else { cue.map_or(QUICK_SELECT, bell_player::key).to_owned() })
}

#[derive(Default)]
pub struct EsdeCuesRust {
    platform_library: QString,
    sounds: QString,
    on: bool,
    problem: QString,
    rung: QString,
    rings: i32,
    player: Option<InterfacePlayer>,
    /// The names the theme's files are kept under.
    themed: Vec<String>,
    /// The latest opening asked for; an older one that finishes later is dropped.
    opening: u64,
}

/// The player with the kit's cues, and each of the theme's files decoded and kept under its name; a sounds text or a
/// file that does not read is said in the words returned, the other sounds kept.
fn open(library: &str, sounds: &str) -> Result<(InterfacePlayer, Vec<String>, String), String> {
    let player = bell_player::open_player(library)?;
    let quick: Vec<u8> = quick_select_samples().iter().flat_map(|s| s.to_le_bytes()).collect();
    player.remember(QUICK_SELECT, &quick).map_err(|e| e.to_string())?;
    let platform = Platform::load(library).map_err(|e| e.to_string())?;
    let mut problems = Vec::new();
    let files: BTreeMap<String, String> = match sounds.trim() {
        "" => BTreeMap::new(),
        text => serde_json::from_str(text).unwrap_or_else(|e| {
            problems.push(format!("The theme's sounds could not be read: {e}"));
            BTreeMap::new()
        }),
    };
    let mut themed = Vec::new();
    for (name, file) in files.iter().filter(|(n, _)| NAMES.iter().any(|(k, _)| k == n)) {
        let kept = decode(&platform, file).map_err(|e| e.to_string()).and_then(|bytes| match bytes {
            Some(bytes) => player.remember(&format!("theme:{name}"), &bytes).map(|()| true).map_err(|e| e.to_string()),
            None => Ok(false),
        });
        match kept {
            Ok(true) => themed.push(name.clone()),
            Ok(false) => {}
            Err(words) => problems.push(format!("The theme's {name} sound could not be played: {words}")),
        }
    }
    Ok((player, themed, problems.join(" ")))
}

impl cxx_qt::Initialize for qobject::EsdeCues {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_platform_library_changed(|cues| cues.reopen()).release();
        self.as_mut().on_sounds_changed(|cues| cues.reopen()).release();
    }
}

impl qobject::EsdeCues {
    pub fn play(self: Pin<&mut Self>, name: &QString, view: &QString) {
        let (name, view) = (name.to_string(), view.to_string());
        self.ring(esde_name(&name, &view));
    }

    pub fn launch(self: Pin<&mut Self>) {
        self.ring("launch");
    }

    pub fn favourite(self: Pin<&mut Self>) {
        self.ring("favorite");
    }

    fn ring(mut self: Pin<&mut Self>, name: &str) {
        let Some(key) = key_of(name, &self.themed) else { return };
        let played = self.player.as_ref().filter(|_| self.on).map(|p| p.play(&key));
        match played {
            Some(Ok(_)) => {
                self.as_mut().rust_mut().rung = QString::from(key.as_str());
                self.as_mut().rung_changed();
                let rings = self.rings.wrapping_add(1);
                self.as_mut().rust_mut().rings = rings;
                self.as_mut().rings_changed();
            }
            Some(Err(e)) => self.as_mut().publish(QString::from(e.to_string().as_str())),
            None => {}
        }
    }

    fn reopen(mut self: Pin<&mut Self>) {
        let (library, sounds) = (self.platform_library.to_string(), self.sounds.to_string());
        if library.is_empty() {
            return;
        }
        let opening = self.opening + 1;
        self.as_mut().rust_mut().opening = opening;
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let opened = open(&library, &sounds);
            let _ = qt.queue(move |mut cues: Pin<&mut qobject::EsdeCues>| {
                if cues.opening != opening {
                    return;
                }
                match opened {
                    Ok((player, themed, problems)) => {
                        cues.as_mut().rust_mut().player = Some(player);
                        cues.as_mut().rust_mut().themed = themed;
                        cues.publish(QString::from(problems.as_str()));
                    }
                    Err(words) => cues.publish(QString::from(words.as_str())),
                }
            });
        });
    }

    fn publish(mut self: Pin<&mut Self>, problem: QString) {
        if self.problem != problem {
            self.as_mut().rust_mut().problem = problem;
            self.problem_changed();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_is_systembrowse_on_the_system_view_and_scroll_in_a_gamelist() {
        assert_eq!(esde_name("navigate", "system"), "systembrowse");
        assert_eq!(esde_name("navigate", "gamelist"), "scroll");
        assert_eq!(esde_name("quicksysselect", "gamelist"), "quicksysselect");
    }

    #[test]
    fn the_themes_file_plays_where_it_gave_one_else_the_kits_cue() {
        let themed = vec!["select".to_owned()];
        assert_eq!(key_of("select", &themed).as_deref(), Some("theme:select"));
        assert_eq!(key_of("back", &themed).as_deref(), Some(bell_player::key(Cue::Back)));
        assert_eq!(key_of("quicksysselect", &[]).as_deref(), Some(QUICK_SELECT));
        assert_eq!(key_of("navigate", &themed), None, "the scene's own name is never a key");
    }
}
