//! The loader's output: a theme resolved for one system and one set of choices, its two views and its sounds, every
//! element typed and defaulted (EmuSen_BigPicture.md §12.2). Immutable once built.

use std::collections::BTreeMap;
use std::fmt;

use super::capabilities::{Capabilities, TransitionProfile};
use super::catalog::{ElementSpec, Group};
use super::choices::{Selection, ThemeSystem};
use super::diagnostics::{Diagnostic, Severity};
use super::values::{Color, Pair, ThemePath, Value};

/// Data an element shows that the theme does not supply: a game's media, its metadata, the clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    pub kind: String,
    pub names: Vec<String>,
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.names.is_empty() {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "{}:{}", self.kind, self.names.join(","))
        }
    }
}

/// One element after merging, typing and defaults.
#[derive(Clone, Debug)]
pub struct Element {
    pub spec: &'static ElementSpec,
    pub name: String,
    /// The order of the element's first definition, which breaks zIndex ties.
    pub order: usize,
    /// None for helpsystem, clock and systemstatus, which draw above everything.
    pub z_index: Option<f32>,
    /// What the theme set, typed.
    pub explicit: BTreeMap<String, Value>,
    /// What the theme set, plus every documented default that has a value.
    pub effective: BTreeMap<String, Value>,
    pub bindings: Vec<Binding>,
}

impl Element {
    pub fn kind(&self) -> &str {
        &self.spec.kind
    }
    pub fn get(&self, property: &str) -> Option<&Value> {
        self.effective.get(property)
    }
    pub fn float(&self, property: &str) -> Option<f32> {
        match self.get(property) {
            Some(Value::Float(f)) => Some(*f),
            _ => None,
        }
    }
    pub fn uint(&self, property: &str) -> Option<u32> {
        match self.get(property) {
            Some(Value::UInt(u)) => Some(*u),
            _ => None,
        }
    }
    pub fn bool(&self, property: &str) -> Option<bool> {
        match self.get(property) {
            Some(Value::Bool(b)) => Some(*b),
            _ => None,
        }
    }
    pub fn pair(&self, property: &str) -> Option<Pair> {
        match self.get(property) {
            Some(Value::Pair(p)) => Some(*p),
            _ => None,
        }
    }
    pub fn color(&self, property: &str) -> Option<Color> {
        match self.get(property) {
            Some(Value::Color(c)) => Some(*c),
            _ => None,
        }
    }
    pub fn path(&self, property: &str) -> Option<&ThemePath> {
        match self.get(property) {
            Some(Value::Path(p)) => Some(p),
            _ => None,
        }
    }
    pub fn string(&self, property: &str) -> Option<&str> {
        match self.get(property) {
            Some(Value::Str(s)) => Some(s),
            _ => None,
        }
    }
    pub fn list(&self, property: &str) -> &[String] {
        match self.get(property) {
            Some(Value::List(l)) => l,
            _ => &[],
        }
    }
    pub fn keyed(&self, property: &str) -> Option<&BTreeMap<String, ThemePath>> {
        match self.get(property) {
            Some(Value::Keyed(k)) => Some(k),
            _ => None,
        }
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} \"{}\"", self.kind(), self.name)
    }
}

/// The name of the help bar and status indicators a view gets when its theme defines none, as ES-DE 3.4.1 draws them (§39).
pub const DEFAULT_NAME: &str = "(default)";

/// One view's elements in drawing order: zIndex low to high, ties by first definition, the unindexed last.
#[derive(Clone, Debug)]
pub struct View {
    pub name: String,
    pub elements: Vec<Element>,
}

impl View {
    pub fn find(&self, kind: &str, name: &str) -> Option<&Element> {
        self.elements.iter().find(|e| e.kind() == kind && e.name == name)
    }

    pub fn of_kind<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a Element> + 'a {
        self.elements.iter().filter(move |e| e.kind() == kind)
    }

    pub fn primary(&self) -> Option<&Element> {
        self.elements.iter().find(|e| e.spec.group == Group::Primary)
    }
}

/// A theme resolved for one system and one set of choices.
#[derive(Clone, Debug)]
pub struct Theme {
    pub capabilities: Capabilities,
    pub system: ThemeSystem,
    pub selection: Selection,
    /// The variant the gamelist view was built with, after the triggers; the system view uses selection.variant.
    pub gamelist_variant: Option<String>,
    pub entry_file: Option<String>,
    pub system_view: View,
    pub gamelist_view: View,
    /// The all view, which holds only the navigation sounds.
    pub sound_view: View,
    pub transitions: TransitionProfile,
    pub diagnostics: Vec<Diagnostic>,
    pub files_read: Vec<String>,
}

impl Theme {
    /// An error unthemes the system; the views stay readable for inspection but must not be drawn.
    pub fn is_themed(&self) -> bool {
        self.diagnostics.iter().all(|d| d.severity != Severity::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Error)
    }

    pub fn sounds(&self) -> BTreeMap<&str, &ThemePath> {
        self.sound_view.of_kind("sound").filter_map(|s| s.path("path").map(|p| (s.name.as_str(), p))).collect()
    }

    pub fn view(&self, name: &str) -> &View {
        match name {
            "system" => &self.system_view,
            "gamelist" => &self.gamelist_view,
            _ => panic!("a view is system or gamelist, not {name}"),
        }
    }
}
