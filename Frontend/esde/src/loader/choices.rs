//! The system a theme is loaded for, the player's choices, their resolution against the capabilities, and the variant
//! triggers (EmuSen_BigPicture.md §12.2 and §12.4).

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use super::capabilities::{self, Capabilities, FONT_SIZE_ORDER};
use super::diagnostics::{Code, Diagnostics, Severity};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SystemKind {
    #[default]
    Regular,
    AutoCollection,
    CustomCollection,
}

/// The system a theme is loaded for; its three names feed the system variables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeSystem {
    pub name: String,
    pub full_name: String,
    pub theme: String,
    pub kind: SystemKind,
}

impl ThemeSystem {
    pub fn new(name: &str, full_name: &str, theme: &str) -> Self {
        ThemeSystem { name: name.into(), full_name: full_name.into(), theme: theme.into(), kind: SystemKind::Regular }
    }

    pub fn of_kind(name: &str, full_name: &str, theme: &str, kind: SystemKind) -> Self {
        ThemeSystem { kind, ..ThemeSystem::new(name, full_name, theme) }
    }

    /// The twelve documented system variables; the kind-specific forms are empty for the other kinds.
    pub fn variables(&self) -> BTreeMap<String, String> {
        let mut variables = BTreeMap::new();
        for (key, value) in [("name", &self.name), ("fullName", &self.full_name), ("theme", &self.theme)] {
            let when = |k: SystemKind| if self.kind == k { value.clone() } else { String::new() };
            variables.insert(format!("system.{key}"), value.clone());
            variables.insert(format!("system.{key}.autoCollections"), when(SystemKind::AutoCollection));
            variables.insert(format!("system.{key}.customCollections"), when(SystemKind::CustomCollection));
            variables.insert(format!("system.{key}.noCollections"), when(SystemKind::Regular));
        }
        variables
    }
}

/// Which media a system's games have, for the variant triggers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MediaPresence {
    pub types: HashSet<String>,
}

impl MediaPresence {
    pub fn of(types: &[&str]) -> Self {
        MediaPresence { types: types.iter().map(|t| t.to_string()).collect() }
    }

    pub fn has_videos(&self) -> bool {
        self.types.contains("video")
    }
}

/// The player's choices, as the settings would store them; None means Automatic or not chosen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choices {
    pub variant: Option<String>,
    pub color_scheme: Option<String>,
    pub font_size: Option<String>,
    pub language: Option<String>,
    pub aspect_ratio: Option<String>,
    pub transitions: Option<String>,
    pub screen_width: i32,
    pub screen_height: i32,
    pub variant_triggers: bool,
}

impl Default for Choices {
    fn default() -> Self {
        Choices {
            variant: None,
            color_scheme: None,
            font_size: None,
            language: None,
            aspect_ratio: None,
            transitions: None,
            screen_width: 1280,
            screen_height: 800,
            variant_triggers: true,
        }
    }
}

/// The choices after THEMES.md's rules and the loader's fallbacks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub variant: Option<String>,
    pub color_scheme: Option<String>,
    pub font_size: Option<String>,
    pub language: Option<String>,
    pub aspect_ratio: Option<String>,
    pub transitions_choice: Option<String>,
}

impl Selection {
    pub fn vertical(&self) -> bool {
        capabilities::is_vertical(self.aspect_ratio.as_deref())
    }

    pub fn resolve(caps: &Capabilities, choices: &Choices) -> Selection {
        let variant = match &choices.variant {
            Some(v) if caps.is_declared_variant(v) => Some(v.clone()),
            _ => caps.variants.iter().find(|x| x.selectable).or(caps.variants.first()).map(|v| v.name.clone()),
        };
        let color_scheme = match &choices.color_scheme {
            Some(c) if caps.color_schemes.iter().any(|s| &s.name == c) => Some(c.clone()),
            _ => caps.color_schemes.first().map(|s| s.name.clone()),
        };
        let font_size = match &choices.font_size {
            Some(f) if caps.font_sizes.contains(f) => Some(f.clone()),
            _ if caps.font_sizes.iter().any(|f| f == FONT_SIZE_ORDER[0]) => Some(FONT_SIZE_ORDER[0].to_owned()),
            _ => caps.font_sizes.first().cloned(),
        };
        let language = match &choices.language {
            Some(l) if caps.languages.contains(l) => Some(l.clone()),
            _ if caps.languages.iter().any(|l| l == "en_US") => Some("en_US".to_owned()),
            _ => None,
        };
        let aspect_ratio = match &choices.aspect_ratio {
            Some(a) if a != "automatic" && caps.aspect_ratios.contains(a) => Some(a.clone()),
            _ => nearest(&caps.aspect_ratios, choices.screen_width, choices.screen_height),
        };
        Selection { variant, color_scheme, font_size, language, aspect_ratio, transitions_choice: choices.transitions.clone() }
    }
}

/// Automatic picks the declared ratio nearest the screen's, measured as |log(r1/r2)|.
pub fn nearest(declared: &[String], width: i32, height: i32) -> Option<String> {
    if declared.is_empty() || width <= 0 || height <= 0 {
        return declared.first().cloned();
    }
    let screen = width as f64 / height as f64;
    let distance = |r: &String| (capabilities::ratio(r).unwrap_or(1.0) / screen).ln().abs();
    let mut best = &declared[0];
    for r in &declared[1..] {
        if distance(r) < distance(best) {
            best = r;
        }
    }
    Some(best.clone())
}

/// noMedia before noVideos, one step only, gamelist view only (EmuSen_BigPicture.md §12.2).
pub fn apply_triggers(
    caps: &Capabilities,
    variant: Option<&str>,
    media: Option<&MediaPresence>,
    enabled: bool,
    d: &mut Diagnostics,
) -> Option<String> {
    let (Some(variant), Some(media), true) = (variant, media, enabled) else { return variant.map(str::to_owned) };
    let Some(declared) = caps.find_variant(variant) else { return Some(variant.to_owned()) };
    for trigger in ["noMedia", "noVideos"] {
        for o in declared.overrides.iter().filter(|o| o.trigger == trigger) {
            let fires = if trigger == "noMedia" { !o.media_types.iter().any(|m| media.types.contains(m)) } else { !media.has_videos() };
            if !fires {
                continue;
            }
            if caps.is_declared_variant(&o.use_variant) {
                return Some(o.use_variant.clone());
            }
            let file = Path::new(&caps.directory).join(capabilities::FILE_NAME).to_string_lossy().into_owned();
            d.add(Severity::Warning, Code::Undeclared, &file, 0, format!("override variant \"{}\" is not declared", o.use_variant));
        }
    }
    Some(variant.to_owned())
}
