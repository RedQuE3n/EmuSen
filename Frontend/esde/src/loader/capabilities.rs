//! capabilities.xml: variants with labels and triggers, colour schemes, font sizes, languages, aspect ratios and
//! transition profiles; without it ES-DE does not load the theme (EmuSen_BigPicture.md §12.2, §35.3).

use std::collections::BTreeMap;
use std::path::Path;

use super::diagnostics::{Code, Diagnostic, Diagnostics, Severity};
use super::values::{split, try_bool};
use super::xml::{self, Node};

pub const FILE_NAME: &str = "capabilities.xml";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Animation {
    Instant,
    Slide,
    Fade,
}

/// A variant trigger: noVideos or noMedia, the media it looks for, and the variant it switches to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Override {
    pub trigger: String,
    pub media_types: Vec<String>,
    pub use_variant: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    pub name: String,
    pub labels: BTreeMap<String, String>,
    pub selectable: bool,
    pub overrides: Vec<Override>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorScheme {
    pub name: String,
    pub labels: BTreeMap<String, String>,
}

/// A transitions profile; the kinds the file leaves out are filled by THEMES.md's rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransitionProfile {
    pub name: String,
    pub labels: BTreeMap<String, String>,
    pub selectable: bool,
    pub system_to_system: Animation,
    pub system_to_gamelist: Animation,
    pub gamelist_to_gamelist: Animation,
    pub gamelist_to_system: Animation,
    pub startup_to_system: Animation,
    pub startup_to_gamelist: Animation,
}

impl TransitionProfile {
    /// A built-in profile: one animation for every kind.
    pub fn built_in(name: &str) -> Self {
        let a = match name {
            "builtin-slide" => Animation::Slide,
            "builtin-fade" => Animation::Fade,
            _ => Animation::Instant,
        };
        TransitionProfile {
            name: name.to_owned(),
            labels: BTreeMap::new(),
            selectable: true,
            system_to_system: a,
            system_to_gamelist: a,
            gamelist_to_gamelist: a,
            gamelist_to_system: a,
            startup_to_system: a,
            startup_to_gamelist: a,
        }
    }
}

/// What capabilities.xml declares, in its own order except where THEMES.md fixes the menu order.
#[derive(Clone, Debug)]
pub struct Capabilities {
    pub directory: String,
    pub theme_name: String,
    pub variants: Vec<Variant>,
    pub color_schemes: Vec<ColorScheme>,
    pub font_sizes: Vec<String>,
    pub languages: Vec<String>,
    pub aspect_ratios: Vec<String>,
    pub transitions: Vec<TransitionProfile>,
    pub suppressed_transitions: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

/// THEMES.md's menu order for font sizes.
pub const FONT_SIZE_ORDER: [&str; 5] = ["medium", "large", "small", "x-large", "x-small"];

/// The fixed aspect-ratio table, horizontal names in menu order; each but 1:1 has a _vertical form.
pub const ASPECT_RATIO_TABLE: [(&str, f64, f64); 12] = [
    ("16:9", 16.0, 9.0),
    ("16:10", 16.0, 10.0),
    ("3:2", 3.0, 2.0),
    ("4:3", 4.0, 3.0),
    ("5:3", 5.0, 3.0),
    ("5:4", 5.0, 4.0),
    ("8:7", 8.0, 7.0),
    ("19.5:9", 19.5, 9.0),
    ("20:9", 20.0, 9.0),
    ("21:9", 21.0, 9.0),
    ("32:9", 32.0, 9.0),
    ("1:1", 1.0, 1.0),
];

pub const LANGUAGE_TABLE: [&str; 21] = [
    "en_US", "en_GB", "bs_BA", "ca_ES", "de_DE", "es_ES", "fr_FR", "hr_HR", "it_IT", "nl_NL", "pl_PL", "pt_BR", "pt_PT", "ro_RO", "ru_RU",
    "sr_RS", "sv_SE", "ja_JP", "ko_KR", "zh_CN", "zh_TW",
];

pub const BUILT_IN_TRANSITIONS: [&str; 3] = ["builtin-instant", "builtin-slide", "builtin-fade"];

pub const MEDIA_TYPES: [&str; 10] =
    ["miximage", "marquee", "screenshot", "titlescreen", "cover", "backcover", "3dbox", "physicalmedia", "fanart", "video"];

impl Capabilities {
    pub fn is_declared_variant(&self, name: &str) -> bool {
        self.variants.iter().any(|v| v.name == name)
    }

    pub fn find_variant(&self, name: &str) -> Option<&Variant> {
        self.variants.iter().find(|v| v.name == name)
    }
}

/// Width over height for a table name, vertical forms inverted; None for a name not in the table.
pub fn ratio(name: &str) -> Option<f64> {
    let (horizontal, vertical) = match name.strip_suffix("_vertical") {
        Some(h) => (h, true),
        None => (name, false),
    };
    let &(n, w, h) = ASPECT_RATIO_TABLE.iter().find(|(n, _, _)| *n == horizontal)?;
    if vertical && n == "1:1" {
        return None;
    }
    Some(if vertical { h / w } else { w / h })
}

pub fn is_vertical(aspect_ratio: Option<&str>) -> bool {
    aspect_ratio.is_some_and(|a| a.ends_with("_vertical"))
}

/// Reads a theme folder's capabilities.xml.
pub fn read(theme_directory: &str) -> Capabilities {
    let mut d = Diagnostics::default();
    let file = Path::new(theme_directory).join(FILE_NAME).to_string_lossy().into_owned();
    let fallback =
        Path::new(theme_directory.trim_end_matches('/')).file_name().map(|n| n.to_string_lossy().to_uppercase()).unwrap_or_default();
    let empty = |d: Diagnostics| Capabilities {
        directory: theme_directory.to_owned(),
        theme_name: fallback.clone(),
        variants: vec![],
        color_schemes: vec![],
        font_sizes: vec![],
        languages: vec![],
        aspect_ratios: vec![],
        transitions: vec![],
        suppressed_transitions: vec![],
        diagnostics: d.items,
    };
    let Ok(bytes) = std::fs::read(&file) else {
        d.add(Severity::Error, Code::CapabilitiesMissing, &file, 0, "capabilities.xml is missing, so the theme is not loaded");
        return empty(d);
    };
    let text = String::from_utf8_lossy(&bytes);
    if strip_comments_and_declarations(&text).trim().is_empty() {
        return empty(d);
    }
    let root = match xml::parse(&text) {
        Ok((root, leniency)) => {
            if let Some(l) = leniency {
                d.add(Severity::Warning, Code::LenientXml, &file, 0, format!("not well-formed XML, read as ES-DE reads it: {l}"));
            }
            root
        }
        Err(e) => {
            d.add(Severity::Error, Code::MalformedXml, &file, e.line, e.message);
            return empty(d);
        }
    };
    if root.name != "themeCapabilities" {
        d.add(Severity::Warning, Code::WrongRoot, &file, root.line, format!("root is <{}>, not <themeCapabilities>", root.name));
    }
    let theme_name =
        root.child("themeName").map(|n| n.value.trim().to_owned()).filter(|n| !n.is_empty()).unwrap_or_else(|| fallback.clone());
    let mut variants: Vec<Variant> = vec![];
    let mut schemes: Vec<ColorScheme> = vec![];
    let mut font_sizes: Vec<String> = vec![];
    let mut languages: Vec<String> = vec![];
    let mut aspect_ratios: Vec<String> = vec![];
    let mut transitions: Vec<TransitionProfile> = vec![];
    let mut suppressed: Vec<String> = vec![];

    for child in &root.children {
        match child.name.as_str() {
            "themeName" => {}
            "aspectRatio" => add_checked(
                &mut aspect_ratios,
                child,
                ratio(child.value.trim()).is_some(),
                "is not in the aspect-ratio table",
                &mut d,
                &file,
            ),
            "fontSize" => add_checked(
                &mut font_sizes,
                child,
                FONT_SIZE_ORDER.contains(&child.value.trim()),
                "is not one of the five font sizes",
                &mut d,
                &file,
            ),
            "language" => add_checked(
                &mut languages,
                child,
                LANGUAGE_TABLE.contains(&child.value.trim()),
                "is not a supported language",
                &mut d,
                &file,
            ),
            "colorScheme" => {
                let names: Vec<&str> = schemes.iter().map(|s| s.name.as_str()).collect();
                if let Some(name) = named_entry(child, &names, "colour scheme", &mut d, &file) {
                    schemes.push(ColorScheme { name, labels: labels(child) });
                }
            }
            "variant" => {
                let names: Vec<&str> = variants.iter().map(|v| v.name.as_str()).collect();
                if let Some(name) = named_entry(child, &names, "variant", &mut d, &file) {
                    variants.push(read_variant(child, name, &mut d, &file));
                }
            }
            "transitions" => {
                let names: Vec<&str> = transitions.iter().map(|t| t.name.as_str()).collect();
                if let Some(name) = named_entry(child, &names, "transitions profile", &mut d, &file) {
                    if BUILT_IN_TRANSITIONS.contains(&name.as_str()) {
                        d.add(Severity::Warning, Code::Reserved, &file, child.line, format!("transitions name \"{name}\" is reserved"));
                    } else if let Some(profile) = read_transitions(child, name, &mut d, &file) {
                        transitions.push(profile);
                    }
                }
            }
            "suppressTransitionProfiles" => {
                for entry in child.children_named("entry") {
                    let value = entry.value.trim();
                    if BUILT_IN_TRANSITIONS.contains(&value) {
                        suppressed.push(value.to_owned());
                    } else {
                        d.add(
                            Severity::Warning,
                            Code::InvalidValue,
                            &file,
                            entry.line,
                            format!("\"{value}\" is not a built-in transitions profile"),
                        );
                    }
                }
            }
            other => {
                d.add_about(Severity::Warning, Code::UnknownTag, &file, child.line, other, format!("<{other}> is not a capabilities tag"))
            }
        }
    }

    if !languages.is_empty() && !languages.iter().any(|l| l == "en_US") {
        d.add(
            Severity::Warning,
            Code::LanguageWithoutEnglish,
            &file,
            0,
            "languages are declared without en_US, so the language configuration is not loaded",
        );
        languages.clear();
    }
    // Aspect ratios in the table's order whatever the file's, a row's vertical form after its horizontal.
    aspect_ratios.sort_by_key(|r| (index_in_table(r), is_vertical(Some(r))));
    font_sizes.sort_by_key(|f| FONT_SIZE_ORDER.iter().position(|o| o == f));

    Capabilities {
        directory: theme_directory.to_owned(),
        theme_name,
        variants,
        color_schemes: schemes,
        font_sizes,
        languages,
        aspect_ratios,
        transitions,
        suppressed_transitions: suppressed,
        diagnostics: d.items,
    }
}

// An empty file, or one holding only comments and a declaration, declares nothing, as THEMES.md says it may.
fn strip_comments_and_declarations(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    loop {
        let next =
            [("<!--", "-->"), ("<?xml", "?>")].iter().filter_map(|(o, c)| rest.find(o).map(|i| (i, *o, *c))).min_by_key(|(i, _, _)| *i);
        let Some((i, open, close)) = next else { break };
        match rest[i + open.len()..].find(close) {
            Some(end) => {
                out.push_str(&rest[..i]);
                rest = &rest[i + open.len() + end + close.len()..];
            }
            None => break,
        }
    }
    out.push_str(rest);
    out
}

fn index_in_table(ratio: &str) -> usize {
    let horizontal = ratio.strip_suffix("_vertical").unwrap_or(ratio);
    ASPECT_RATIO_TABLE.iter().position(|(n, _, _)| *n == horizontal).unwrap_or(usize::MAX)
}

fn add_checked(list: &mut Vec<String>, element: &Node, valid: bool, problem: &str, d: &mut Diagnostics, file: &str) {
    let value = element.value.trim();
    let tag = &element.name;
    if !valid {
        d.add(Severity::Warning, Code::InvalidValue, file, element.line, format!("<{tag}>{value}</{tag}> {problem} and is not loaded"));
    } else if list.iter().any(|v| v == value) {
        d.add(Severity::Warning, Code::Duplicate, file, element.line, format!("{tag} \"{value}\" is declared twice"));
    } else {
        list.push(value.to_owned());
    }
}

fn named_entry(element: &Node, existing: &[&str], what: &str, d: &mut Diagnostics, file: &str) -> Option<String> {
    let name = element.attribute("name").map(str::trim).unwrap_or("");
    if name.is_empty() {
        d.add(Severity::Warning, Code::MissingName, file, element.line, format!("a {what} has no name and is not loaded"));
        return None;
    }
    if existing.contains(&name) {
        d.add(
            Severity::Warning,
            Code::Duplicate,
            file,
            element.line,
            format!("{what} \"{name}\" is declared twice; the second is not loaded"),
        );
        return None;
    }
    if what == "variant" && name == "all" {
        d.add(Severity::Warning, Code::Reserved, file, element.line, "\"all\" is a reserved variant name");
        return None;
    }
    Some(name.to_owned())
}

// A label without a language attribute is en_US's.
fn labels(element: &Node) -> BTreeMap<String, String> {
    element.children_named("label").map(|l| (l.attribute("language").unwrap_or("en_US").to_owned(), l.value.trim().to_owned())).collect()
}

fn read_variant(element: &Node, name: String, d: &mut Diagnostics, file: &str) -> Variant {
    // A variant without <selectable> is not offered, as in ES-DE (§35.3).
    let selectable = read_selectable(element.child("selectable"), false, d, file);
    let mut overrides = vec![];
    for o in element.children_named("override") {
        let triggers: Vec<&Node> = o.children_named("trigger").collect();
        if triggers.len() != 1 {
            d.add(
                Severity::Warning,
                Code::InvalidValue,
                file,
                o.line,
                format!("an override needs exactly one trigger, not {}, and is ignored", triggers.len()),
            );
            continue;
        }
        let trigger = triggers[0].value.trim();
        if trigger != "noVideos" && trigger != "noMedia" {
            d.add(Severity::Warning, Code::InvalidValue, file, o.line, format!("trigger \"{trigger}\" is not noVideos or noMedia"));
            continue;
        }
        let use_variant = o.child("useVariant").map(|u| u.value.trim()).unwrap_or("");
        if use_variant.is_empty() {
            d.add(Severity::Warning, Code::InvalidValue, file, o.line, "an override has no useVariant");
            continue;
        }
        let mut media = vec!["video".to_owned()];
        if trigger == "noMedia" {
            media = o.child("mediaType").map(|m| split(&m.value)).unwrap_or_default();
            if media.is_empty() {
                media = vec!["miximage".to_owned()];
            }
            for m in media.iter().filter(|m| !MEDIA_TYPES.contains(&m.as_str())) {
                d.add(Severity::Warning, Code::InvalidValue, file, o.line, format!("mediaType \"{m}\" is not a media type"));
            }
            media.retain(|m| MEDIA_TYPES.contains(&m.as_str()));
        }
        overrides.push(Override { trigger: trigger.to_owned(), media_types: media, use_variant: use_variant.to_owned() });
    }
    Variant { name, labels: labels(element), selectable, overrides }
}

fn read_transitions(element: &Node, name: String, d: &mut Diagnostics, file: &str) -> Option<TransitionProfile> {
    let mut kind = |tag: &str| -> Option<Animation> {
        let e = element.child(tag)?;
        match e.value.trim() {
            "instant" => Some(Animation::Instant),
            "slide" => Some(Animation::Slide),
            "fade" => Some(Animation::Fade),
            other => {
                d.add(Severity::Warning, Code::InvalidValue, file, e.line, format!("<{tag}> \"{other}\" is not instant, slide or fade"));
                None
            }
        }
    };
    let (s2s, s2g, g2g, g2s, u2s, u2g) = (
        kind("systemToSystem"),
        kind("systemToGamelist"),
        kind("gamelistToGamelist"),
        kind("gamelistToSystem"),
        kind("startupToSystem"),
        kind("startupToGamelist"),
    );
    if [s2s, s2g, g2g, g2s, u2s, u2g].iter().all(Option::is_none) {
        d.add(
            Severity::Warning,
            Code::InvalidValue,
            file,
            element.line,
            format!("transitions \"{name}\" defines none of the six types and is not loaded"),
        );
        return None;
    }
    let system_to_system = s2s.unwrap_or(Animation::Instant);
    let gamelist_to_gamelist = g2g.unwrap_or(Animation::Instant);
    Some(TransitionProfile {
        name,
        labels: labels(element),
        selectable: read_selectable(element.child("selectable"), true, d, file),
        system_to_system,
        system_to_gamelist: s2g.unwrap_or(Animation::Instant),
        gamelist_to_gamelist,
        gamelist_to_system: g2s.unwrap_or(Animation::Instant),
        startup_to_system: u2s.unwrap_or(system_to_system),
        startup_to_gamelist: u2g.unwrap_or(gamelist_to_gamelist),
    })
}

// ES-DE's reading of selectable: false when the first character is 0, f, F, n or N, otherwise true, blank included (§35.3).
fn read_selectable(element: Option<&Node>, when_missing: bool, d: &mut Diagnostics, file: &str) -> bool {
    let Some(element) = element else { return when_missing };
    let text = &element.value;
    let value = !matches!(text.chars().next(), Some('0' | 'f' | 'F' | 'n' | 'N'));
    if try_bool(text).is_none() {
        d.add(
            Severity::Warning,
            Code::LenientValue,
            file,
            element.line,
            format!("<{}> \"{text}\" is not true or false; it is read as {value}, as ES-DE reads it", element.name),
        );
    }
    value
}
