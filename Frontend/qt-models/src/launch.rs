//! The questions asked before a game opens (EmuSen_Backend.md §29.40.8, EmuSen_Firmware.md §3; EmuSen_Qt.md §6.3 items
//! 2 and 6): the approval of the core the game would open on, where it waits for the player's, and the firmware the
//! engine requires that the library does not hold. Both are read from the runtime and asked in the question form; the
//! words are those of C#'s `MainWindow.CoreApproval` and `PromptForMissingFirmwareAsync`, rewritten for a form with no
//! file picker. No Qt here.

use crate::question::Question;
use crate::tools::cores::approval::{self, Asking};
use emusen_crystal_client::shapes::{CoreRow, FirmwareNeed, Route, SystemRow};
use emusen_crystal_client::Runtime;
use std::path::Path;

/// The core a game would open on that waits for approval, or whose approved default could not be read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Awaiting {
    pub asking: Asking,
    /// The engine the game opens on instead when the player says Not Now.
    pub instead: Option<String>,
    /// Why the console's approved default could not be read, so the game opens on `instead` with nothing to ask.
    pub held: Option<String>,
}

fn asking(file: &str, name: &str, authors: &[String], licence: &str, systems: &[String], sha256: &str) -> Asking {
    Asking {
        name: name.to_owned(),
        file: file.to_owned(),
        authors: authors.to_vec(),
        licence: licence.to_owned(),
        consoles: systems.to_vec(),
        sha256: sha256.to_owned(),
        ..Asking::default()
    }
}

/// The listing's row of the libretro core whose hash the route names.
fn waiting(cores: &[CoreRow], sha256: &str) -> Option<Asking> {
    cores.iter().find_map(|c| {
        let l = c.libretro.as_ref().filter(|l| l.sha256.eq_ignore_ascii_case(sha256))?;
        let name = l.name.clone().or_else(|| c.name.clone()).unwrap_or_default();
        Some(asking(&l.file, &name, &l.authors, l.license.as_deref().unwrap_or_default(), &c.systems, sha256))
    })
}

/// The core the routed game would open on, where it waits for approval: the routed engine's library, or a libretro
/// core's whose hash the listing gives; none otherwise, and none for a core the installation vouches for.
pub fn awaiting(route: &Route, cores: &[CoreRow]) -> Option<Awaiting> {
    let (engine, system) = (route.engine.as_deref(), route.system.as_deref());
    if engine.is_none() && system.is_none() {
        return None;
    }
    if let Some(kept) = &route.default_held {
        let asking = waiting(cores, &kept.sha256).unwrap_or_else(|| asking(&kept.stem, &kept.stem, &[], "", &[], &kept.sha256));
        return Some(Awaiting { asking, instead: engine.map(str::to_owned), held: Some(kept.reason.clone().unwrap_or_default()) });
    }
    if let Some(named) = &route.waiting_default {
        return waiting(cores, &named.sha256).map(|asking| Awaiting { asking, instead: engine.map(str::to_owned), held: None });
    }
    for c in cores {
        let unnamed = engine.is_none() && c.engine.is_none() && system.is_some_and(|s| c.systems.iter().any(|x| x == s));
        if !unnamed && (engine.is_none() || c.engine.as_deref() != engine) {
            continue;
        }
        if let Some(l) = &c.libretro {
            let asks = l.trust.as_deref() == Some("needs_approval") && c.trust.as_deref() != Some("needs_refresh") && !l.sha256.is_empty();
            let name = l.name.clone().or_else(|| c.name.clone()).or_else(|| engine.map(str::to_owned)).unwrap_or_default();
            return asks.then(|| Awaiting { asking: asking(&l.file, &name, &l.authors, l.license.as_deref().unwrap_or_default(), &c.systems, &l.sha256), ..Awaiting::default() });
        }
        if unnamed || c.trust.as_deref() != Some("needs_approval") {
            return None;
        }
        let file = c.library.as_deref().and_then(|l| Path::new(l).file_name()).map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
        let name = c.name.clone().or_else(|| engine.map(str::to_owned)).unwrap_or_default();
        return Some(Awaiting {
            asking: asking(&file, &name, &c.authors, c.license.as_deref().unwrap_or_default(), &c.systems, c.sha256.as_deref().unwrap_or_default()),
            ..Awaiting::default()
        });
    }
    None
}

/// A core's systems as a list names their consoles, the table's display name, for the question's words.
pub fn shown_names(systems: &[String], table: &[SystemRow]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for id in systems {
        let row = table.iter().find(|s| &s.id == id);
        let name = row.map(|s| if !s.display_name.is_empty() { s.display_name.clone() } else if !s.console.is_empty() { s.console.clone() } else { id.clone() }).unwrap_or_else(|| id.clone());
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

/// The approval the game waits for, read from the runtime; none where nothing waits or the runtime cannot say.
pub fn approval_of(runtime: &Runtime, path: &str) -> Option<Awaiting> {
    let route = runtime.route(path).ok()?;
    let cores = runtime.cores().ok()?;
    let mut found = awaiting(&route, &cores)?;
    if let Ok(table) = runtime.systems() {
        found.asking.consoles = shown_names(&found.asking.consoles, &table);
    }
    Some(found)
}

/// The engine the game opens on, as the route names it.
pub fn engine_of(runtime: &Runtime, path: &str) -> Option<String> {
    runtime.route(path).ok()?.engine
}

/// The approval's question: the same words as Cores' (EmuSen_Interface.md §15.6).
pub fn approval_question(found: &Awaiting) -> Question {
    approval::question(&found.asking)
}

/// The line saying a game opened on the fallback because its console's default waits for approval.
pub fn fallback_notice(name: &str, instead: &str) -> String {
    format!("{name} waits for your approval, so this game opens on {instead}.")
}

/// The line saying a game opened on the fallback because its console's approved default could not be read, and why.
pub fn held_notice(name: &str, instead: &str, why: &str) -> String {
    format!("{name} could not be used, so this game opens on {instead}: {why}")
}

/// The line saying the firmware a game needs could not be checked, so it opens without the question (EmuSen_Qt.md §7.9).
pub fn firmware_unchecked(why: &str) -> String {
    format!("The firmware this game needs could not be checked, so it opens without asking: {why}")
}

/// The line saying an approval was recorded, or why not.
pub fn approved_notice(found: &Awaiting, refused: Option<&str>) -> String {
    match refused {
        None => format!("Approved {} ({}).", found.asking.name, found.asking.file),
        Some(why) => format!("{} could not be approved: {why}", found.asking.file),
    }
}

/// The firmware question for one required file: play without it, or look again once the player has put it in the
/// firmware folder; B goes back without starting the game. EmuSen never fetches firmware, and the form has no file list
/// of the folder (EmuSen_Interface.md §18.10).
pub fn firmware_question(need: &FirmwareNeed, folder: &str) -> Question {
    let chip = need.chip.clone().filter(|c| !c.is_empty());
    let what = chip.clone().map_or_else(|| "Firmware".to_owned(), |c| format!("{c} firmware"));
    let size = crate::words::grouped(need.size as usize);
    let purpose = need.purpose.as_ref().filter(|p| !p.is_empty()).map(|p| format!(" for {p}")).unwrap_or_default();
    let without = match need.replacement.as_ref().and_then(|r| r.effect.clone()).filter(|e| !e.is_empty()) {
        Some(effect) => format!("Without it, EmuSen's open replacement runs in its place: {effect}"),
        None => format!("Without it, {} is not emulated.", chip.unwrap_or_else(|| "that part of the console".to_owned())),
    };
    let without = if without.ends_with('.') { without } else { without + "." };
    Question {
        title: format!("{what} needed"),
        sentences: vec![
            format!("This game asks for {}{purpose}, {size} bytes: a dump of your own, which EmuSen never downloads.", need.file),
            format!("Put it in {folder}, then choose Check Again. {without}"),
        ],
        caption: String::new(),
        choices: vec!["Play Without It".to_owned(), "Check Again".to_owned()],
        back: None,
    }
}

/// What the player answered the firmware question.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirmwareAnswer {
    PlayWithout,
    CheckAgain,
    Back,
}

pub fn firmware_answer(index: i32) -> FirmwareAnswer {
    match index {
        0 => FirmwareAnswer::PlayWithout,
        1 => FirmwareAnswer::CheckAgain,
        _ => FirmwareAnswer::Back,
    }
}
