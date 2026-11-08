//! `loader`: an ES-DE theme folder to a theme resolved for one system and one set of choices, with every diagnostic's
//! code, file and line. A translation of the C# loader (EmuSen_BigPicture.md §12, §31, §35; EmuSen_Qt.md §4.2).

pub mod capabilities;
pub mod catalog;
pub mod choices;
pub mod diagnostics;
mod parse;
pub mod resolved;
pub mod values;
mod view;
mod xml;

use std::collections::HashSet;
use std::path::Path;

pub use capabilities::{Animation, Capabilities, TransitionProfile};
pub use catalog::catalog;
pub use choices::{Choices, MediaPresence, Selection, SystemKind, ThemeSystem};
pub use diagnostics::{Code, Diagnostic, Diagnostics, Severity};
pub use parse::MAX_INCLUDE_DEPTH;
pub use resolved::{Binding, Element, Theme, View, DEFAULT_NAME};
pub use values::{Color, Pair, ThemePath, Value};

/// Loads the theme at a folder for one system; `media` None leaves the variant triggers unevaluated.
pub fn load(theme_directory: &str, system: &ThemeSystem, choices: &Choices, media: Option<&MediaPresence>) -> Theme {
    load_with(&capabilities::read(theme_directory), system, choices, media)
}

/// Loads a theme whose capabilities are already read, as a loop over systems and variants does.
pub fn load_with(caps: &Capabilities, system: &ThemeSystem, choices: &Choices, media: Option<&MediaPresence>) -> Theme {
    let mut d = Diagnostics { items: caps.diagnostics.clone() };
    let selection = Selection::resolve(caps, choices);
    let gamelist_variant = choices::apply_triggers(caps, selection.variant.as_deref(), media, choices.variant_triggers, &mut d);
    let entry = entry_file(&caps.directory, system);
    let caps_failed = caps.diagnostics.iter().any(|x| x.severity == Severity::Error);
    let empty = |name: &str| View { name: name.to_owned(), elements: vec![] };
    let Some(entry) = entry.filter(|_| !caps_failed) else {
        if entry_file(&caps.directory, system).is_none() {
            d.add(
                Severity::Error,
                Code::NoThemeFile,
                &caps.directory,
                0,
                format!("neither {}/theme.xml nor theme.xml exists", system.theme),
            );
        }
        return Theme {
            capabilities: caps.clone(),
            system: system.clone(),
            transitions: resolve_transitions(caps, &selection, None),
            selection,
            gamelist_variant,
            entry_file: entry_file(&caps.directory, system),
            system_view: empty("system"),
            gamelist_view: empty("gamelist"),
            sound_view: empty("all"),
            diagnostics: d.items,
            files_read: vec![],
        };
    };

    let for_system = parse::ParseRun::run(caps, &selection, selection.variant.as_deref(), system, &entry);
    let for_gamelist = if gamelist_variant == selection.variant {
        None
    } else {
        Some(parse::ParseRun::run(caps, &selection, gamelist_variant.as_deref(), system, &entry))
    };
    let gamelist_run = for_gamelist.as_ref().unwrap_or(&for_system);

    let mut seen = HashSet::new();
    for x in for_system.diagnostics.items.iter().chain(for_gamelist.iter().flat_map(|r| r.diagnostics.items.iter())) {
        if seen.insert(x.clone()) {
            d.items.push(x.clone());
        }
    }
    let system_view = for_system.finish("system", &mut d, &mut seen);
    let gamelist_view = gamelist_run.finish("gamelist", &mut d, &mut seen);
    let sound_view = for_system.finish("all", &mut d, &mut seen);
    let mut files_read: Vec<String> = vec![];
    for f in for_system.files_read.iter().chain(&gamelist_run.files_read) {
        if !files_read.contains(f) {
            files_read.push(f.clone());
        }
    }
    Theme {
        capabilities: caps.clone(),
        system: system.clone(),
        transitions: resolve_transitions(caps, &selection, for_system.variant_transitions.as_deref()),
        selection,
        gamelist_variant,
        entry_file: Some(entry),
        system_view,
        gamelist_view,
        sound_view,
        diagnostics: d.items,
        files_read,
    }
}

/// A system's own folder's theme.xml, else the theme's root one.
pub fn entry_file(theme_directory: &str, system: &ThemeSystem) -> Option<String> {
    let own = Path::new(theme_directory).join(&system.theme).join("theme.xml");
    if !system.theme.is_empty() && own.is_file() {
        return Some(values::full_path(&own.to_string_lossy()));
    }
    let root = Path::new(theme_directory).join("theme.xml");
    root.is_file().then(|| values::full_path(&root.to_string_lossy()))
}

/// A chosen profile, else Automatic: the variant's, the first declared, then instant.
pub fn resolve_transitions(caps: &Capabilities, selection: &Selection, variant_profile: Option<&str>) -> TransitionProfile {
    let declared = |name: &str| caps.transitions.iter().find(|t| t.name == name).cloned();
    if let Some(choice) = selection.transitions_choice.as_deref() {
        if capabilities::BUILT_IN_TRANSITIONS.contains(&choice) && !caps.suppressed_transitions.iter().any(|s| s == choice) {
            return TransitionProfile::built_in(choice);
        }
        if let Some(chosen) = declared(choice) {
            return chosen;
        }
    }
    if let Some(profile) = variant_profile.and_then(declared) {
        return profile;
    }
    caps.transitions.first().cloned().unwrap_or_else(|| TransitionProfile::built_in("builtin-instant"))
}
