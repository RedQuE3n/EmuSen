//! The Themes tab's facts as plain data (EmuSen_Qt.md §3.8): every theme in Galaxia's themes store as a row, the reference
//! first; the stored id resolved to a folder; a package's check, run by `theme-check` or `esde-check`'s measure; the ES-DE
//! theme the library is drawn with, and its choices as capabilities.xml declares them. No Qt: `themes_model.rs` puts it on Qt.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::esde_shelves::{self, theme_key, CHOICES, STYLE, STYLE_OWN, STYLE_THEME, THEME};
use emusen_esde::check as esde;
use emusen_esde::loader::capabilities::{self, Capabilities, BUILT_IN_TRANSITIONS, FONT_SIZE_ORDER};
use emusen_theme_check::loaded;
use serde_json::Value;

/// The reference theme's id, the one compiled into the program's resources.
pub const REFERENCE: &str = "emusen";

/// The setting that holds the chosen theme's id (EmuSen_Config_Reference.md §7.3).
pub const SETTING: &str = "EmuSenTheme";

/// The tab's navigation, one word a kind, in `Kind`'s order.
pub const KINDS: [&str; 2] = ["EmuSen", "ES-DE"];

/// What a folder holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A package against EmuSen.Theme: `theme.json`.
    EmuSen,
    /// An ES-DE theme: `capabilities.xml` or `theme.xml`.
    Esde,
}

impl Kind {
    pub fn index(self) -> usize {
        match self {
            Kind::EmuSen => 0,
            Kind::Esde => 1,
        }
    }
}

/// The last check of a package, kept while the program runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Checked {
    Not,
    Running,
    Ok,
    Findings(Vec<String>),
}

/// One theme as the tab lists it.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: String,
    pub name: String,
    pub kind: Kind,
    pub author: String,
    /// `EmuSen.Theme` major.minor, or "ES-DE".
    pub api: String,
    /// The variants it is drawn in, as words.
    pub variants: String,
    /// The first preview's URL; empty for none.
    pub preview: String,
    pub current: bool,
    pub checked: Checked,
}

impl Row {
    /// Offered Use: a theme whose last check was clean, not already the one in use.
    pub fn usable(&self) -> bool {
        self.checked == Checked::Ok && !self.current
    }

    /// The row's value on the list: in use, checking, its check's outcome, or nothing yet.
    pub fn value(&self) -> String {
        match (&self.checked, self.current) {
            (_, true) => "In use".to_owned(),
            (Checked::Running, _) => "Checking…".to_owned(),
            (Checked::Ok, _) => "Checked".to_owned(),
            (Checked::Findings(f), _) if f.len() == 1 => "1 finding".to_owned(),
            (Checked::Findings(f), _) => format!("{} findings", f.len()),
            (Checked::Not, _) => String::new(),
        }
    }

    /// The detail's line under the name: who made it, against what, in which variants.
    pub fn line(&self) -> String {
        let made = if self.author.is_empty() { String::new() } else { format!("By {} · ", self.author) };
        match self.kind {
            Kind::EmuSen => format!("{made}EmuSen.Theme {} · {}", self.api, self.variants),
            Kind::Esde => format!("{made}ES-DE theme · {}", self.variants),
        }
    }

    /// The detail's words on the check, or why Use is not offered.
    pub fn findings(&self) -> String {
        let check = match &self.checked {
            Checked::Not => "Not checked yet. A theme is checked before it can be used.".to_owned(),
            Checked::Running => "Checking…".to_owned(),
            Checked::Ok => "Checked: nothing found.".to_owned(),
            Checked::Findings(f) => f.join("\n"),
        };
        if self.current {
            "In use.".to_owned()
        } else {
            check
        }
    }
}

/// A folder name that resolves inside the store: no separator, not `.` or `..`, not hidden.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty() && !id.starts_with('.') && !id.contains(['/', '\\', '\0'])
}

/// The kind of theme a folder holds, if any.
pub fn kind_of(folder: &Path) -> Option<Kind> {
    if folder.join("theme.json").is_file() {
        return Some(Kind::EmuSen);
    }
    let caps = esde::theme_root(folder).join("capabilities.xml").is_file();
    (caps || folder.join("theme.xml").is_file()).then_some(Kind::Esde)
}

/// The id the settings hold; the reference's where none is set.
pub fn chosen_of(settings: &Value) -> String {
    settings.get(SETTING).and_then(Value::as_str).map(str::trim).filter(|id| !id.is_empty()).unwrap_or(REFERENCE).to_owned()
}

/// What an id names.
#[derive(Clone, Debug, PartialEq)]
pub enum Resolved {
    Reference,
    Theme { folder: PathBuf, kind: Kind },
    /// No folder of a theme by that name in the store.
    Missing,
}

/// The id resolved through the themes store (Galaxia's `DIR_THEMES`): the reference, a folder directly under the store, or missing.
pub fn resolve(id: &str, store: Option<&Path>) -> Resolved {
    if id.is_empty() || id == REFERENCE {
        return Resolved::Reference;
    }
    let Some(store) = store.filter(|_| valid_id(id)) else { return Resolved::Missing };
    let folder = store.join(id);
    match kind_of(&folder) {
        Some(kind) => Resolved::Theme { folder, kind },
        None => Resolved::Missing,
    }
}

impl Resolved {
    /// The folder the theme host loads: an EmuSen package's; empty for the reference and anything it cannot draw.
    pub fn host_folder(&self) -> String {
        match self {
            Resolved::Theme { folder, kind: Kind::EmuSen } => folder.to_string_lossy().into_owned(),
            _ => String::new(),
        }
    }

    /// Why the reference is shown in place of the chosen theme; empty when it is shown as chosen.
    pub fn words(&self, id: &str) -> String {
        match self {
            Resolved::Missing => format!("The theme “{id}” is not in the themes folder, so EmuSen's own theme is shown."),
            Resolved::Theme { kind: Kind::Esde, .. } => format!("“{id}” is an ES-DE theme, which is chosen under ES-DE, so EmuSen's own theme is shown."),
            _ => String::new(),
        }
    }
}

/// A folder as a URL with `path` joined: `qrc:` and `file:` folders as given, a plain path as a `file:` URL.
fn url_in(folder: &str, path: &str) -> String {
    let folder = folder.trim_end_matches('/');
    let folder = if folder.starts_with("qrc:") || folder.starts_with("file:") { folder.to_owned() } else { format!("file://{folder}") };
    format!("{folder}/{path}")
}

/// A package's row from its `theme.json`; `folder` is where its previews are, as a path or a URL.
pub fn package_row(id: &str, manifest: &str, folder: &str) -> Row {
    let parsed = emusen_theme_manifest::parse(manifest).ok().map(|(m, _)| m);
    let named = |name: &str| if name.is_empty() { id.to_owned() } else { name.to_owned() };
    let (name, author, api, variants, preview) = match &parsed {
        Some(m) => {
            let names: Vec<&str> = m.variants.iter().map(|v| v.name()).collect();
            let preview = m.previews.first().map(|p| url_in(folder, &p.path)).unwrap_or_default();
            (named(&m.name), m.author.clone(), format!("{}.{}", m.api.0, m.api.1), names.join(" and "), preview)
        }
        None => (id.to_owned(), String::new(), String::new(), String::new(), String::new()),
    };
    Row { id: id.to_owned(), name, kind: Kind::EmuSen, author, api, variants, preview, current: false, checked: Checked::Not }
}

/// The reference's row: its manifest from the resources, held clean by the build's own tests, so checked.
pub fn reference_row(manifest: &str, folder: &str) -> Row {
    Row { checked: Checked::Ok, ..package_row(REFERENCE, manifest, folder) }
}

/// An ES-DE theme's row from its `capabilities.xml`: its name, and its selectable variants as a count.
fn esde_row(id: &str, folder: &Path) -> Row {
    let root = esde::theme_root(folder);
    let caps = emusen_esde::loader::capabilities::read(&root.to_string_lossy());
    let variants = caps.variants.iter().filter(|v| v.selectable).count();
    let variants = match variants {
        0 => "one look".to_owned(),
        1 => "one variant".to_owned(),
        n => format!("{n} variants"),
    };
    let name = if caps.theme_name.is_empty() { id.to_owned() } else { caps.theme_name };
    Row { id: id.to_owned(), name, kind: Kind::Esde, author: String::new(), api: "ES-DE".to_owned(), variants, preview: String::new(), current: false, checked: Checked::Not }
}

/// Every theme folder directly under the store, in name order; hidden, partial and replaced folders, and one named as the reference, left out.
pub fn scan(store: &Path) -> Vec<Row> {
    let mut folders: Vec<PathBuf> = std::fs::read_dir(store).map(|r| r.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect()).unwrap_or_default();
    folders.sort();
    folders
        .iter()
        .filter_map(|folder| {
            let id = folder.file_name()?.to_str()?;
            if !valid_id(id) || id == REFERENCE || id.ends_with(".part") || id.ends_with(".old") {
                return None;
            }
            match kind_of(folder)? {
                Kind::EmuSen => Some(package_row(id, &std::fs::read_to_string(folder.join("theme.json")).unwrap_or_default(), &folder.to_string_lossy())),
                Kind::Esde => Some(esde_row(id, folder)),
            }
        })
        .collect()
}

/// The tab after a read: every row, the reference first, the one in use marked; the store; the id stored; and why the
/// reference stands in for it, if it does.
#[derive(Clone, Debug, PartialEq)]
pub struct Listing {
    pub rows: Vec<Row>,
    pub store: Option<PathBuf>,
    pub chosen: String,
    pub words: String,
    /// The ES-DE theme the library is drawn with, where it is one of the store's.
    pub esde: Option<InUse>,
}

/// The listing from the settings, the store and the reference's manifest and folder.
pub fn listing(settings: &Value, store: Option<&Path>, reference: &str, reference_folder: &str) -> Listing {
    let chosen = chosen_of(settings);
    let resolved = resolve(&chosen, store);
    let mut rows = vec![reference_row(reference, reference_folder)];
    rows.extend(store.map(scan).unwrap_or_default());
    let esde = mark(&mut rows, settings, store);
    Listing { rows, store: store.map(Path::to_path_buf), words: resolved.words(&chosen), chosen, esde }
}

/// Marks the row in use: the ES-DE theme the settings make the library's look, else the EmuSen theme they choose (the
/// reference where that one cannot be drawn). Returns the ES-DE theme's choices when it is one of the store's.
pub fn mark(rows: &mut [Row], settings: &Value, store: Option<&Path>) -> Option<InUse> {
    let themed = esde_shelves::settings_of(settings).themed;
    let esde = if themed { esde_in_use(settings, store, rows) } else { None };
    let chosen = chosen_of(settings);
    let own = if resolve(&chosen, store).host_folder().is_empty() { REFERENCE } else { chosen.as_str() };
    for row in rows.iter_mut() {
        row.current = match row.kind {
            Kind::EmuSen => !themed && row.id == own,
            Kind::Esde => esde.as_ref().is_some_and(|e| e.id == row.id),
        };
    }
    esde
}

/// The ES-DE theme the library is drawn with, as the tab lists it: its row's id, the folder ES-DE loads, and its choices.
#[derive(Clone, Debug, PartialEq)]
pub struct InUse {
    pub id: String,
    pub root: PathBuf,
    pub choices: Vec<Choice>,
}

/// The store's ES-DE theme whose folder the settings' `BigPictureTheme` names; none for a folder outside the store.
fn esde_in_use(settings: &Value, store: Option<&Path>, rows: &[Row]) -> Option<InUse> {
    let named = theme_key(settings.get(THEME).and_then(Value::as_str).unwrap_or_default());
    let store = store?;
    rows.iter().filter(|r| r.kind == Kind::Esde).find_map(|r| {
        let root = esde::theme_root(&store.join(&r.id));
        let key = theme_key(&root.to_string_lossy());
        (key == named).then(|| {
            let stored = settings.get(CHOICES).and_then(|c| c.get(&key)).cloned().unwrap_or(Value::Null);
            let caps = capabilities::read(&root.to_string_lossy());
            InUse { id: r.id.clone(), root, choices: choices_of(&caps, &stored) }
        })
    })
}

/// The stored value of a choice that means "the loader decides": the screen's nearest ratio, the variant's transitions.
pub const AUTOMATIC: &str = "automatic";

/// One of an ES-DE theme's choices: its key among the stored choices (`esde-qt`'s `shelves::settings` reads it), its
/// row's words, each option as its stored value and its words, and the one in force.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub key: &'static str,
    pub label: &'static str,
    pub options: Vec<(String, String)>,
    pub at: usize,
}

impl Choice {
    /// The words of the option in force.
    pub fn value(&self) -> &str {
        self.options.get(self.at).map_or("", |o| o.1.as_str())
    }

    /// Steps `by` options round the list and gives the stored value now in force.
    pub fn step(&mut self, by: i32) -> &str {
        let n = self.options.len() as i64;
        self.at = (self.at as i64 + i64::from(by)).rem_euclid(n.max(1)) as usize;
        &self.options[self.at].0
    }
}

/// A capability's words: its English label where the theme gives one, else its name.
fn label_of(labels: &BTreeMap<String, String>, name: &str) -> String {
    labels.get("en_US").filter(|l| !l.is_empty()).cloned().unwrap_or_else(|| name.to_owned())
}

/// Every choice the capabilities declare, in ES-DE's order, each at its stored value, else at the option the loader
/// falls back to (`Selection::resolve`, `resolve_transitions`).
pub fn choices_of(caps: &Capabilities, stored: &Value) -> Vec<Choice> {
    let given = |key: &str| {
        let lower = format!("{}{}", key[..1].to_lowercase(), &key[1..]);
        stored.get(key).or_else(|| stored.get(&lower)).and_then(Value::as_str).map(str::to_owned)
    };
    let named = |names: &[String]| names.iter().map(|n| (n.clone(), n.clone())).collect::<Vec<_>>();
    let automatic = || (AUTOMATIC.to_owned(), "Automatic".to_owned());
    let mut out = Vec::new();
    let mut add = |key: &'static str, label: &'static str, options: Vec<(String, String)>, fallback: usize| {
        if options.is_empty() {
            return;
        }
        let at = given(key).and_then(|v| options.iter().position(|o| o.0 == v)).unwrap_or(fallback);
        out.push(Choice { key, label, options, at });
    };
    let variants = caps.variants.iter().filter(|v| v.selectable).map(|v| (v.name.clone(), label_of(&v.labels, &v.name))).collect();
    add("Variant", "Variant", variants, 0);
    add("ColorScheme", "Colour scheme", caps.color_schemes.iter().map(|c| (c.name.clone(), label_of(&c.labels, &c.name))).collect(), 0);
    let medium = caps.font_sizes.iter().position(|f| f == FONT_SIZE_ORDER[0]).unwrap_or(0);
    add("FontSize", "Font size", named(&caps.font_sizes), medium);
    let english = caps.languages.iter().position(|l| l == "en_US").unwrap_or(0);
    add("Language", "Language", named(&caps.languages), english);
    if !caps.aspect_ratios.is_empty() {
        add("AspectRatio", "Aspect ratio", std::iter::once(automatic()).chain(named(&caps.aspect_ratios)).collect(), 0);
    }
    let profiles = caps.transitions.iter().filter(|t| t.selectable).map(|t| (t.name.clone(), label_of(&t.labels, &t.name)));
    let built_in = BUILT_IN_TRANSITIONS.iter().filter(|b| !caps.suppressed_transitions.iter().any(|s| s == *b)).map(|b| (b.to_string(), b.to_string()));
    add("Transitions", "Transitions", std::iter::once(automatic()).chain(profiles).chain(built_in).collect(), 0);
    out
}

/// The settings with an ES-DE theme's folder as the library's look.
pub fn choose_esde(settings: &mut Value, root: &Path) {
    object(settings);
    settings[STYLE] = Value::from(STYLE_THEME);
    settings[THEME] = Value::from(root.to_string_lossy().into_owned());
}

/// The settings with an EmuSen theme as the library's look, the ES-DE theme's folder and choices kept for a later return.
pub fn choose_own(settings: &mut Value, id: &str) {
    object(settings);
    settings[SETTING] = Value::from(id);
    settings[STYLE] = Value::from(STYLE_OWN);
}

/// The settings with one choice of the ES-DE theme in `root` stored under its folder's key.
pub fn store_choice(settings: &mut Value, root: &Path, key: &str, value: &str) {
    object(settings);
    let choices = &mut settings[CHOICES];
    object(choices);
    let theme = &mut choices[theme_key(&root.to_string_lossy())];
    object(theme);
    theme[key] = Value::from(value);
}

/// A value made an object where it is anything else, so a key can be set on it.
fn object(value: &mut Value) {
    if !value.is_object() {
        *value = Value::Object(Default::default());
    }
}

/// `theme-audit` as `theme-check` finds it, else beside the running program.
fn audit_program() -> Option<PathBuf> {
    loaded::program().or_else(|| Some(std::env::current_exe().ok()?.parent()?.join(loaded::PROGRAM)).filter(|p| p.is_file()))
}

/// A package checked as the install gate checks it: `theme-check`'s both steps on an EmuSen package, `esde-check`'s measure on
/// an ES-DE theme. Slow (the loaded step loads every screen), so it runs off the GUI thread.
pub fn check(kind: Kind, folder: &Path) -> Checked {
    match kind {
        Kind::EmuSen => check_package(folder, audit_program().as_deref()),
        Kind::Esde => check_esde(folder),
    }
}

/// `theme-check` on one package with `program` as its loaded step; findings named under the folder's name, notes left out.
pub fn check_package(folder: &Path, program: Option<&Path>) -> Checked {
    let shown = folder.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let args = ["theme-check".to_owned(), shown];
    let real = folder.to_string_lossy().into_owned();
    match emusen_theme_check::run_with(&args, |_| Ok(real.clone()), program) {
        Ok((_, 0)) => Checked::Ok,
        Ok((text, _)) => {
            // The last line is the count; a note refuses nothing.
            let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
            let found = lines[..lines.len().saturating_sub(1)].iter().filter(|l| l.split(' ').nth(1) != Some("note"));
            Checked::Findings(found.map(|l| l.to_string()).collect())
        }
        Err(words) => Checked::Findings(vec![words]),
    }
}

/// `esde-check`'s measure over EmuSen's systems: clean when it is themed for every one and says no error.
pub fn check_esde(folder: &Path) -> Checked {
    let m = esde::measure(folder, &esde::SYSTEMS);
    let mut found: Vec<String> = m
        .diagnostics
        .iter()
        .filter(|d| d.0 == emusen_esde::loader::Severity::Error)
        .map(|(_, code, file, line, message)| format!("{file}:{line} {code} {message}"))
        .collect();
    if !m.loads() {
        found.push(format!("themed for {} of {} systems", m.themed_systems, m.systems));
    }
    if found.is_empty() {
        Checked::Ok
    } else {
        Checked::Findings(found)
    }
}
