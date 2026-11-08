//! One view's element definitions as parsed, merged by type and name with the last value winning, then typed, ranged,
//! limited to their view, defaulted, bound and put in drawing order (EmuSen_BigPicture.md §12.2, §35.2).

use std::collections::BTreeMap;
use std::path::Path;

use super::catalog::{catalog, Computed, ElementSpec, Group, PropertySpec, PropertyType, Scope, Shape};
use super::diagnostics::{Code, Diagnostics, Severity};
use super::parse::ParseRun;
use super::resolved::{Binding, Element, View, DEFAULT_NAME};
use super::values::{self, Pair, ThemePath, Value, IMAGE_SHORTCUT};

/// A property's text once its variables are substituted, and where it was written.
#[derive(Clone, Debug)]
pub struct Raw {
    pub text: String,
    pub file: String,
    pub line: u32,
    pub from_variable: bool,
}

pub struct ViewBuilder {
    pub name: String,
    elements: Vec<ElementBuilder>,
}

struct ElementBuilder {
    spec: &'static ElementSpec,
    name: String,
    order: usize,
    // Insertion order kept, a later value replacing an earlier in place.
    properties: Vec<(String, Raw)>,
    keyed: Vec<(String, Vec<(String, Raw)>)>,
}

fn put<T>(list: &mut Vec<(String, T)>, key: &str, value: T) {
    match list.iter_mut().find(|(k, _)| k == key) {
        Some(slot) => slot.1 = value,
        None => list.push((key.to_owned(), value)),
    }
}

impl ViewBuilder {
    pub fn new(name: &str) -> Self {
        ViewBuilder { name: name.to_owned(), elements: vec![] }
    }

    pub fn merge(&mut self, spec: &'static ElementSpec, name: &str, order: &mut usize, properties: &[(String, Raw, Option<String>)]) {
        let index = match self.elements.iter().position(|e| e.spec.kind == spec.kind && e.name == name) {
            Some(i) => i,
            None => {
                self.elements.push(ElementBuilder { spec, name: name.to_owned(), order: *order, properties: vec![], keyed: vec![] });
                *order += 1;
                self.elements.len() - 1
            }
        };
        let element = &mut self.elements[index];
        for (property, value, key) in properties {
            match key {
                None => put(&mut element.properties, property, value.clone()),
                Some(key) => {
                    if !element.keyed.iter().any(|(k, _)| k == property) {
                        element.keyed.push((property.clone(), vec![]));
                    }
                    let by_key = &mut element.keyed.iter_mut().find(|(k, _)| k == property).expect("just added").1;
                    put(by_key, key, value.clone());
                }
            }
        }
    }

    pub fn resolve(&self, run: &ParseRun, d: &mut Diagnostics) -> View {
        let mut resolved: Vec<Element> = vec![];
        let mut primary: Option<String> = None;
        for element in &self.elements {
            if !self.allows(element.spec) {
                let (file, line) = element.first();
                d.add(
                    Severity::Warning,
                    Code::ViewRestricted,
                    &file,
                    line,
                    format!("<{} name=\"{}\"> cannot be used in the {} view and is ignored", element.spec.kind, element.name, self.name),
                );
                continue;
            }
            let Some(r) = element.resolve(&self.name, run, d) else { continue };
            if r.spec.group == Group::Primary {
                if let Some(first) = &primary {
                    let (file, line) = element.first();
                    d.add(
                        Severity::Warning,
                        Code::SecondPrimary,
                        &file,
                        line,
                        format!("{r} is a second primary element in the {} view after {first}, and is ignored", self.name),
                    );
                    continue;
                }
                primary = Some(r.to_string());
            }
            resolved.push(r);
        }
        if self.name == "system" || self.name == "gamelist" {
            let mut order = self.elements.len();
            for kind in ["helpsystem", "systemstatus"] {
                if self.elements.iter().any(|e| e.spec.kind == kind) {
                    continue;
                }
                let spec = catalog().find(kind).expect("in the catalogue");
                let builder = ElementBuilder { spec, name: DEFAULT_NAME.to_owned(), order, properties: vec![], keyed: vec![] };
                order += 1;
                if let Some(e) = builder.resolve(&self.name, run, d) {
                    resolved.push(e);
                }
            }
        }
        resolved.sort_by(|a, b| {
            let (za, zb) = (a.z_index.unwrap_or(f32::INFINITY), b.z_index.unwrap_or(f32::INFINITY));
            za.partial_cmp(&zb).unwrap_or(std::cmp::Ordering::Equal).then(a.order.cmp(&b.order))
        });
        View { name: self.name.clone(), elements: resolved }
    }

    fn allows(&self, spec: &ElementSpec) -> bool {
        match self.name.as_str() {
            "system" => spec.system,
            "gamelist" => spec.gamelist,
            _ => spec.all,
        }
    }
}

fn in_view(spec: &PropertySpec, view: &str) -> bool {
    match spec.only_in {
        Scope::System => view == "system",
        Scope::Gamelist => view == "gamelist",
        Scope::Any => true,
    }
}

fn lower_name(kind: PropertyType) -> &'static str {
    match kind {
        PropertyType::Pair => "pair of numbers",
        PropertyType::Boolean => "true or false",
        PropertyType::UInt => "whole number",
        PropertyType::Color => "hexadecimal colour",
        _ => "number",
    }
}

fn scope_name(scope: Scope) -> &'static str {
    match scope {
        Scope::System => "system",
        Scope::Gamelist => "gamelist",
        Scope::Any => "any",
    }
}

impl ElementBuilder {
    fn first(&self) -> (String, u32) {
        self.properties
            .iter()
            .map(|(_, r)| r)
            .chain(self.keyed.iter().flat_map(|(_, k)| k.iter().map(|(_, r)| r)))
            .next()
            .map(|r| (r.file.clone(), r.line))
            .unwrap_or_default()
    }

    // Types each value, applies view limits and ranges, then fills documented defaults; None when the element must not render.
    fn resolve(&self, view: &str, run: &ParseRun, d: &mut Diagnostics) -> Option<Element> {
        let mut explicit = BTreeMap::new();
        let mut render = true;
        for (name, raw) in &self.properties {
            let spec = self.spec.find(name).expect("checked when parsed");
            if !in_view(spec, view) {
                d.add(
                    Severity::Warning,
                    Code::ViewRestricted,
                    &raw.file,
                    raw.line,
                    format!(
                        "<{name}> of {} \"{}\" can only be used in the {} view and is ignored",
                        self.spec.kind,
                        self.name,
                        scope_name(spec.only_in)
                    ),
                );
                continue;
            }
            if let Some(value) = self.parse(spec, raw, d, &mut render) {
                explicit.insert(name.clone(), value);
            }
        }
        for (name, by_key) in &self.keyed {
            let paths = by_key.iter().map(|(key, raw)| (key.clone(), resolve_path(raw, d))).collect();
            explicit.insert(name.clone(), Value::Keyed(paths));
        }
        if !render {
            return None;
        }
        let mut effective = explicit.clone();
        for spec in &self.spec.properties {
            if !effective.contains_key(&spec.name) && in_view(spec, view) {
                if let Some(value) = self.default(spec, &mut effective, run) {
                    effective.insert(spec.name.clone(), value);
                }
            }
        }
        let z_index = match effective.get("zIndex") {
            Some(Value::Float(z)) => Some(*z),
            _ => None,
        };
        let bindings = self.bindings(view, &effective);
        Some(Element { spec: self.spec, name: self.name.clone(), order: self.order, z_index, explicit, effective, bindings })
    }

    fn parse(&self, spec: &PropertySpec, raw: &Raw, d: &mut Diagnostics, render: &mut bool) -> Option<Value> {
        let text = raw.text.trim();
        let kind = &self.spec.kind;
        let name = &self.name;
        let read_as = |d: &mut Diagnostics, what: String| {
            d.add(
                Severity::Warning,
                Code::LenientValue,
                &raw.file,
                raw.line,
                format!(
                    "<{}> of {kind} \"{name}\" is \"{}\", not a plain {}; it is read as {what}, as ES-DE reads it",
                    spec.name,
                    raw.text,
                    lower_name(spec.kind)
                ),
            )
        };
        match spec.kind {
            PropertyType::Pair => {
                let Some((pair, exact)) = values::esde_pair(&raw.text) else {
                    d.add(
                        Severity::Error,
                        Code::BadFormat,
                        &raw.file,
                        raw.line,
                        format!(
                            "<{}> of {kind} \"{name}\" is \"{text}\", which has no space between two values; ES-DE refuses such a pair too",
                            spec.name
                        ),
                    );
                    return None;
                };
                if !exact {
                    read_as(d, format!("{} {}", dotnet_float(pair.x), dotnet_float(pair.y)));
                }
                let clamped = values::clamp_pair(pair, spec);
                return Some(Value::Pair(Pair::new(values::finite(clamped.x), values::finite(clamped.y))));
            }
            PropertyType::Path if spec.shape != Shape::Keyed => return Some(Value::Path(resolve_path(raw, d))),
            PropertyType::Boolean => {
                let (b, exact) = values::esde_bool(&raw.text);
                if !exact {
                    read_as(d, b.to_string());
                }
                return Some(Value::Bool(b));
            }
            PropertyType::Color => {
                let Some((color, exact)) = values::esde_color(&raw.text) else {
                    d.add(
                        Severity::Error,
                        Code::BadFormat,
                        &raw.file,
                        raw.line,
                        format!(
                            "<{}> of {kind} \"{name}\" is \"{}\", which is not 6 or 8 characters long; ES-DE refuses such a colour too",
                            spec.name, raw.text
                        ),
                    );
                    return None;
                };
                if !exact {
                    read_as(d, color.to_string());
                }
                return Some(Value::Color(color));
            }
            PropertyType::UInt => {
                let (u, exact) = values::esde_uint(&raw.text);
                if !exact {
                    read_as(d, u.to_string());
                }
                if let Some((lo, hi)) = spec.valid_range.filter(|&(lo, hi)| u < lo || u > hi) {
                    d.add(
                        Severity::Warning,
                        Code::InvalidValue,
                        &raw.file,
                        raw.line,
                        format!("<{}> of {kind} \"{name}\" is {u}, outside {lo}-{hi}; the default applies, as in ES-DE", spec.name),
                    );
                    return None;
                }
                return Some(Value::UInt(values::clamp_uint(u, spec.min, spec.max)));
            }
            PropertyType::Float => {
                let (f, exact) = values::esde_float(&raw.text);
                if !exact {
                    read_as(d, dotnet_float(f));
                }
                return Some(Value::Float(values::finite(values::clamp(f, spec.min, spec.max))));
            }
            _ => {}
        }
        match spec.shape {
            Shape::Enum => {
                if spec.values.contains(&raw.text) {
                    return Some(Value::Str(raw.text.clone()));
                }
                d.add(
                    Severity::Warning,
                    Code::InvalidValue,
                    &raw.file,
                    raw.line,
                    format!(
                        "<{}> of {kind} \"{name}\" is \"{}\", not exactly one of {}; the default applies, as in ES-DE",
                        spec.name,
                        raw.text,
                        spec.values.join(", ")
                    ),
                );
                None
            }
            Shape::List => self.parse_list(spec, raw, text, d, render),
            _ => Some(Value::Str(raw.text.clone())),
        }
    }

    // An unknown imageType stops the element rendering, with a warning, as in ES-DE; a repeated one drops the property (§35.2).
    fn parse_list(&self, spec: &PropertySpec, raw: &Raw, text: &str, d: &mut Diagnostics, render: &mut bool) -> Option<Value> {
        let mut items = values::split(text);
        let image_type = spec.name == "imageType";
        let (kind, name) = (&self.spec.kind, &self.name);
        let invalid: Vec<&String> = items.iter().filter(|i| !spec.values.contains(i)).collect();
        if !invalid.is_empty() {
            let named = invalid.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
            if image_type {
                d.add(
                    Severity::Warning,
                    Code::InvalidImageType,
                    &raw.file,
                    raw.line,
                    format!("<imageType> of {kind} \"{name}\" names {named}; the element is not rendered, as in ES-DE, and the system stays themed"),
                );
                *render = false;
                return None;
            }
            d.add(
                Severity::Warning,
                Code::InvalidValue,
                &raw.file,
                raw.line,
                format!("<{}> of {kind} \"{name}\" names unknown {named}, which are ignored", spec.name),
            );
            items.retain(|i| spec.values.contains(i));
        }
        if image_type {
            let mut seen = items.clone();
            seen.sort();
            seen.dedup();
            if seen.len() != items.len() {
                d.add(
                    Severity::Warning,
                    Code::InvalidValue,
                    &raw.file,
                    raw.line,
                    format!("<imageType> of {kind} \"{name}\" repeats a type and is ignored"),
                );
                return None;
            }
        }
        if spec.max_items > 0 && items.len() > spec.max_items {
            d.add(
                Severity::Debug,
                Code::InvalidValue,
                &raw.file,
                raw.line,
                format!("<{}> of {kind} \"{name}\" lists {} entries; only the first {} are used", spec.name, items.len(), spec.max_items),
            );
            items.truncate(spec.max_items);
        }
        Some(Value::List(items))
    }

    fn default(&self, spec: &PropertySpec, effective: &mut BTreeMap<String, Value>, run: &ParseRun) -> Option<Value> {
        match spec.computed {
            Computed::SystemFullName => return Some(Value::Str(run.system.full_name.clone())),
            Computed::ElementWidth => {
                return match self.value("size", effective, run) {
                    Some(Value::Pair(p)) => Some(Value::Float(p.x)),
                    _ => None,
                }
            }
            Computed::OneAndAHalfFontSize => {
                return match self.value("fontSize", effective, run) {
                    Some(Value::Float(f)) => Some(Value::Float(f * 1.5)),
                    _ => None,
                }
            }
            Computed::HelpPosByOrientation => {
                return Some(Value::Pair(if run.selection.vertical() { Pair::new(0.012, 0.975) } else { Pair::new(0.012, 0.9515) }))
            }
            Computed::HelpFontSizeByOrientation => return Some(Value::Float(if run.selection.vertical() { 0.025 } else { 0.035 })),
            Computed::InterpolationByRotation => {
                let rotation = match self.value("rotation", effective, run) {
                    Some(Value::Float(r)) => r,
                    _ => 0.0,
                };
                let straight = ((rotation % 90.0) + 90.0) % 90.0 == 0.0;
                return Some(Value::Str(if straight { "nearest" } else { "linear" }.to_owned()));
            }
            Computed::ContainerForDescription => {
                return Some(Value::Bool(matches!(self.value("metadata", effective, run), Some(Value::Str(m)) if m == "description")))
            }
            Computed::DisplayRelativeForLastPlayed => {
                return Some(Value::Bool(matches!(self.value("metadata", effective, run), Some(Value::Str(m)) if m == "lastplayed")))
            }
            Computed::ContainerStartDelayByType => {
                let horizontal = matches!(self.value("containerType", effective, run), Some(Value::Str(t)) if t == "horizontal");
                return Some(Value::Float(if horizontal { 1.5 } else { 4.5 }));
            }
            Computed::None => {}
        }
        if let Some(other) = &spec.same_as {
            return self.value(other, effective, run);
        }
        let literal = spec.default.as_ref()?;
        let mut none = Diagnostics::default();
        let mut render = true;
        self.parse(spec, &Raw { text: literal.clone(), file: String::new(), line: 0, from_variable: false }, &mut none, &mut render)
    }

    // A property's effective value, filling and keeping its own default first when the theme did not set it.
    fn value(&self, name: &str, effective: &mut BTreeMap<String, Value>, run: &ParseRun) -> Option<Value> {
        if let Some(v) = effective.get(name) {
            return Some(v.clone());
        }
        let spec = self.spec.find(name)?;
        let d = self.default(spec, effective, run)?;
        effective.insert(name.to_owned(), d.clone());
        Some(d)
    }

    // What the element will show that the theme does not supply (EmuSen_BigPicture.md §12.2).
    fn bindings(&self, view: &str, effective: &BTreeMap<String, Value>) -> Vec<Binding> {
        let list = |name: &str| match effective.get(name) {
            Some(Value::List(l)) => l.clone(),
            _ => vec![],
        };
        let text = |name: &str| match effective.get(name) {
            Some(Value::Str(s)) => Some(s.clone()),
            _ => None,
        };
        let media = |types: Vec<String>| {
            let mut out: Vec<String> = vec![];
            for t in types {
                let expanded: Vec<String> = if t == "image" { IMAGE_SHORTCUT.iter().map(|s| s.to_string()).collect() } else { vec![t] };
                for e in expanded {
                    if !out.contains(&e) {
                        out.push(e);
                    }
                }
            }
            out
        };
        let bind = |kind: &str, names: Vec<String>| Binding { kind: kind.to_owned(), names };
        let listed = if view == "system" { "systems" } else { "games" };
        let mut bindings = vec![];
        match self.spec.kind.as_str() {
            "image" => {
                let types = list("imageType");
                if !types.is_empty() {
                    bindings.push(bind("media", media(types)));
                } else if effective.contains_key("gameOverridePath") {
                    bindings.push(bind("gameOverride", vec![]));
                }
            }
            "video" => bindings.push(bind("videoFallbackImage", media(list("imageType").into_iter().filter(|t| t != "none").collect()))),
            "carousel" | "grid" => {
                bindings.push(bind(listed, vec![]));
                if view == "gamelist" {
                    bindings.push(bind("media", list("imageType").into_iter().filter(|t| t != "none").collect()));
                }
            }
            "textlist" => bindings.push(bind(listed, vec![])),
            "text" => {
                if let Some(m) = text("metadata") {
                    bindings.push(bind("metadata", vec![m]));
                } else if let Some(s) = text("systemdata") {
                    bindings.push(bind("systemdata", vec![s]));
                }
            }
            "datetime" => {
                if let Some(m) = text("metadata") {
                    bindings.push(bind("metadata", vec![m]));
                }
            }
            "rating" => bindings.push(bind("metadata", vec!["rating".into()])),
            "badges" => bindings.push(bind("badges", list("slots"))),
            "gamelistinfo" => bindings.push(bind("gamelistinfo", vec![])),
            "helpsystem" => bindings.push(bind("help", list("entries"))),
            "clock" => bindings.push(bind("clock", vec![])),
            "systemstatus" => bindings.push(bind("systemstatus", list("entries"))),
            "gameselector" => bindings.push(bind("gameselector", text("selection").into_iter().collect())),
            _ => {}
        }
        bindings
    }
}

// A missing file is a warning when written out and a debug note when built from a variable.
fn resolve_path(raw: &Raw, d: &mut Diagnostics) -> ThemePath {
    let absolute = values::resolve_path(&raw.text, &raw.file);
    let exists = Path::new(&absolute).exists();
    if !exists {
        let severity = if raw.from_variable { Severity::Debug } else { Severity::Warning };
        d.add(severity, Code::PathMissing, &raw.file, raw.line, format!("\"{}\" not found (resolved to {absolute})", raw.text));
    }
    ThemePath { written: raw.text.trim().to_owned(), absolute, from_variable: raw.from_variable, exists }
}

// A float as .NET's invariant ToString writes it in a message: the shortest text that reads back.
fn dotnet_float(value: f32) -> String {
    if value.is_nan() {
        "NaN".into()
    } else if value.is_infinite() {
        if value > 0.0 { "∞" } else { "-∞" }.into()
    } else {
        format!("{value}")
    }
}
