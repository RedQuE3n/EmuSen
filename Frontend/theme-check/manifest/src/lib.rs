//! `theme.json`, schemas 1 and 2: a theme package's manifest read into plain data, with every way it breaks the schema as a finding
//! (EmuSen_Qt.md §3.3, §3.4, §5.2; `man theme-check`). It reads text only; what the files and token values hold is the caller's to check.

use std::fmt;

use serde_json::{Map, Value};

/// The newest schema number this build reads.
pub const SCHEMA: u64 = 2;

/// The `EmuSen.Theme` API this build carries, major and minor.
pub const API: (u64, u64) = (1, 23);

/// The cue names a theme may give its own file (EmuSen_Qt.md §3.2, `EmuSen_Interface.md` §23).
pub const CUES: [&str; 7] = ["scroll", "systembrowse", "select", "back", "favorite", "launch", "quicksysselect"];

const KEYS_1: [&str; 11] = ["schema", "api", "name", "author", "licence", "variants", "fontSizes", "screens", "tokens", "cues", "previews"];
const KEYS_2: [&str; 11] = ["schema", "api", "name", "author", "licence", "variants", "fontSizes", "faces", "tokens", "cues", "previews"];

/// A variant a theme is designed for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Variant {
    Dark,
    Light,
}

impl Variant {
    pub const ALL: [Variant; 2] = [Variant::Dark, Variant::Light];

    pub fn name(self) -> &'static str {
        match self {
            Variant::Dark => "dark",
            Variant::Light => "light",
        }
    }

    fn named(name: &str) -> Option<Variant> {
        Variant::ALL.into_iter().find(|v| v.name() == name)
    }
}

/// A screen a theme may provide; one it leaves out is the reference theme's (the splash: Moonrise).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Screen {
    Home,
    Consoles,
    Games,
    Splash,
    /// Settings (EmuSen_Qt.md §3.2, `Settings`): a theme that leaves it out gets the program's own.
    Settings,
    /// A game's page (§3.2, `Page`): a theme that leaves it out gets the program's own.
    Game,
}

impl Screen {
    pub const ALL: [Screen; 6] = [Screen::Home, Screen::Consoles, Screen::Games, Screen::Splash, Screen::Settings, Screen::Game];

    /// The name in `screens` and in `ThemeScreen.role`.
    pub fn name(self) -> &'static str {
        match self {
            Screen::Home => "home",
            Screen::Consoles => "consoles",
            Screen::Games => "games",
            Screen::Splash => "splash",
            Screen::Settings => "settings",
            Screen::Game => "game",
        }
    }

    /// The file inside the package.
    pub fn file(self) -> &'static str {
        match self {
            Screen::Home => "screens/Home.qml",
            Screen::Consoles => "screens/Consoles.qml",
            Screen::Games => "screens/Games.qml",
            Screen::Splash => "screens/Splash.qml",
            Screen::Settings => "screens/Settings.qml",
            Screen::Game => "screens/Game.qml",
        }
    }

    fn named(name: &str) -> Option<Screen> {
        Screen::ALL.into_iter().find(|s| s.name() == name)
    }

    /// Every screen's name, as a finding lists them: "home, consoles, … or game".
    fn listed() -> String {
        let names: Vec<&str> = Screen::ALL.iter().map(|s| s.name()).collect();
        let (last, rest) = names.split_last().expect("screens");
        format!("{} or {last}", rest.join(", "))
    }
}

/// A desktop screen a schema 2 theme may provide; omitted ones use the program's built-in desktop screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DesktopScreen {
    Library,
    Table,
    Playing,
    Settings,
}

impl DesktopScreen {
    pub const ALL: [DesktopScreen; 4] = [DesktopScreen::Library, DesktopScreen::Table, DesktopScreen::Playing, DesktopScreen::Settings];

    /// The name in `faces.desktop` and in `ThemeScreen.role`.
    pub fn name(self) -> &'static str {
        match self {
            DesktopScreen::Library => "library",
            DesktopScreen::Table => "table",
            DesktopScreen::Playing => "playing",
            DesktopScreen::Settings => "settings",
        }
    }

    /// The file inside the package.
    pub fn file(self) -> &'static str {
        match self {
            DesktopScreen::Library => "screens/desktop/Library.qml",
            DesktopScreen::Table => "screens/desktop/Table.qml",
            DesktopScreen::Playing => "screens/desktop/Playing.qml",
            DesktopScreen::Settings => "screens/desktop/Settings.qml",
        }
    }

    fn named(name: &str) -> Option<DesktopScreen> {
        DesktopScreen::ALL.into_iter().find(|s| s.name() == name)
    }

    /// Every desktop screen's name, as a finding lists them: "library, table, playing or settings".
    pub fn listed() -> String {
        let names: Vec<&str> = DesktopScreen::ALL.iter().map(|s| s.name()).collect();
        let (last, rest) = names.split_last().expect("desktop screens");
        format!("{} or {last}", rest.join(", "))
    }
}

/// A token override's value as written: the token's own type is the kit's to hold it to.
#[derive(Clone, Debug, PartialEq)]
pub enum TokenValue {
    Text(String),
    Number(f64),
    Switch(bool),
}

/// One token override: in a variant, a token's name and its value, and the line it was written on.
#[derive(Clone, Debug, PartialEq)]
pub struct TokenOverride {
    pub variant: Variant,
    pub name: String,
    pub value: TokenValue,
    pub line: usize,
}

/// A path inside the package, with its line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagePath {
    pub path: String,
    pub line: usize,
}

/// The manifest as read; a field the schema could not read is left empty and has a finding.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Manifest {
    pub schema: u64,
    pub api: (u64, u64),
    pub name: String,
    pub author: String,
    pub licence: String,
    pub variants: Vec<Variant>,
    pub font_sizes: Vec<f64>,
    pub font_sizes_line: usize,
    pub screens: Vec<Screen>,
    pub desktop_screens: Vec<DesktopScreen>,
    pub tokens: Vec<TokenOverride>,
    pub desktop_tokens: Vec<TokenOverride>,
    pub cues: Vec<(String, PackagePath)>,
    pub previews: Vec<PackagePath>,
}

/// Which rule a finding is under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// The file is not JSON, or a key is missing or of the wrong shape.
    Schema,
    /// The schema number or the API is newer than this build's.
    Api,
    /// A path that leaves the package.
    Path,
    /// A key the schema does not know: ignored, not a fault (§5.2).
    Note,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Kind::Schema => "schema",
            Kind::Api => "api",
            Kind::Path => "path",
            Kind::Note => "note",
        })
    }
}

/// What the schema found, on a line of `theme.json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub line: usize,
    pub kind: Kind,
    pub words: String,
}

impl Finding {
    /// Whether the finding refuses the theme; a note does not.
    pub fn refuses(&self) -> bool {
        self.kind != Kind::Note
    }
}

/// The 1-based line of a key reached by a path of keys, each looked for after the one before; 1 when not found.
pub fn line_of(text: &str, keys: &[&str]) -> usize {
    let mut at = 0;
    for key in keys {
        let quoted = format!("\"{key}\"");
        let mut from = at;
        loop {
            let Some(found) = text[from..].find(&quoted) else { return 1 };
            let pos = from + found;
            let after = text[pos + quoted.len()..].trim_start();
            if after.starts_with(':') {
                at = pos;
                break;
            }
            from = pos + quoted.len();
        }
    }
    text[..at].matches('\n').count() + 1
}

struct Reader<'a> {
    text: &'a str,
    findings: Vec<Finding>,
}

impl Reader<'_> {
    fn find(&mut self, keys: &[&str], kind: Kind, words: String) {
        let line = line_of(self.text, keys);
        self.findings.push(Finding { line, kind, words });
    }

    fn text_of(&mut self, object: &Map<String, Value>, key: &str, required: bool) -> String {
        match object.get(key) {
            Some(Value::String(s)) if !s.trim().is_empty() => s.clone(),
            Some(_) => {
                self.find(&[key], Kind::Schema, format!("{key} is a non-empty string"));
                String::new()
            }
            None => {
                if required {
                    self.find(&[], Kind::Schema, format!("{key} is missing"));
                }
                String::new()
            }
        }
    }

    fn names(&mut self, object: &Map<String, Value>, key: &str, required: bool) -> Vec<String> {
        self.names_at(&[key], object, key, required)
    }

    fn names_at(&mut self, keys: &[&str], object: &Map<String, Value>, key: &str, required: bool) -> Vec<String> {
        let label = keys.join(".");
        match object.get(key) {
            Some(Value::Array(items)) if items.iter().all(Value::is_string) => {
                let names: Vec<String> = items.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect();
                for (i, name) in names.iter().enumerate() {
                    if names[..i].contains(name) {
                        self.find(keys, Kind::Schema, format!("{label} names {name} twice"));
                    }
                }
                names
            }
            Some(_) => {
                self.find(keys, Kind::Schema, format!("{label} is a list of names"));
                Vec::new()
            }
            None => {
                if required {
                    self.find(&[], Kind::Schema, format!("{label} is missing"));
                }
                Vec::new()
            }
        }
    }

    fn path(&mut self, keys: &[&str], value: &Value) -> Option<PackagePath> {
        let line = line_of(self.text, keys);
        let Some(path) = value.as_str() else {
            self.find(keys, Kind::Schema, format!("{} is a path inside the package", keys.join(".")));
            return None;
        };
        if !inside_package(path) {
            self.find(keys, Kind::Path, format!("{path} leaves the package"));
            return None;
        }
        Some(PackagePath { path: path.to_owned(), line })
    }
}

/// Whether a path written in the package stays in it: relative, no `..`, no scheme, forward slashes.
pub fn inside_package(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains(':')
        && !path.contains('\\')
        && path.split('/').all(|part| part != ".." && !part.is_empty())
}

fn version(text: &str) -> Option<(u64, u64)> {
    let (major, minor) = text.split_once('.')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}

/// Reads `theme.json`'s text, the findings in line order; `Err` when it is not a JSON object, with the line of the fault.
pub fn parse(text: &str) -> Result<(Manifest, Vec<Finding>), Finding> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| Finding { line: e.line().max(1), kind: Kind::Schema, words: format!("not JSON: {e}") })?;
    let Value::Object(top) = value else {
        return Err(Finding { line: 1, kind: Kind::Schema, words: "theme.json holds one object".to_owned() });
    };
    let mut r = Reader { text, findings: Vec::new() };
    let mut m = Manifest::default();
    match top.get("schema").and_then(Value::as_u64) {
        Some(n) if (1..=SCHEMA).contains(&n) => m.schema = n,
        Some(n) if n > SCHEMA => r.find(&["schema"], Kind::Api, format!("written for theme.json schema {n}; this build reads {SCHEMA}")),
        Some(_) | None if top.contains_key("schema") => r.find(&["schema"], Kind::Schema, format!("schema is 1 or {SCHEMA}")),
        _ => r.find(&[], Kind::Schema, "schema is missing".to_owned()),
    }
    let future_v2 = m.schema == 0 && top.get("schema").and_then(Value::as_u64).is_some_and(|n| n > SCHEMA) && top.contains_key("faces");
    let v2 = m.schema >= 2 || future_v2;
    let schema_label = if m.schema >= 2 || future_v2 { SCHEMA } else { 1 };
    let valid_keys = if v2 { &KEYS_2 } else { &KEYS_1 };
    for key in top.keys() {
        if !valid_keys.contains(&key.as_str()) {
            if m.schema == 1 && key == "faces" {
                r.find(&["faces"], Kind::Schema, "faces requires schema 2".to_owned());
            } else if m.schema >= 2 && key == "screens" {
                r.find(&["screens"], Kind::Schema, "screens is replaced by faces in schema 2".to_owned());
            } else {
                r.find(&[key], Kind::Note, format!("{key} is not a key of schema {schema_label}, ignored"));
            }
        }
    }
    match top.get("api") {
        Some(Value::String(s)) => match version(s) {
            Some((major, minor)) if major == API.0 && minor <= API.1 => m.api = (major, minor),
            Some(v) => r.find(&["api"], Kind::Api, format!("written for EmuSen.Theme {}.{}; this build has {}.{}", v.0, v.1, API.0, API.1)),
            None => r.find(&["api"], Kind::Schema, "api is a version, major.minor".to_owned()),
        },
        Some(_) => r.find(&["api"], Kind::Schema, "api is a version, major.minor".to_owned()),
        None => r.find(&[], Kind::Schema, "api is missing".to_owned()),
    }
    m.name = r.text_of(&top, "name", true);
    m.author = r.text_of(&top, "author", true);
    m.licence = r.text_of(&top, "licence", true);
    for name in r.names(&top, "variants", true) {
        match Variant::named(&name) {
            Some(v) if !m.variants.contains(&v) => m.variants.push(v),
            Some(_) => {}
            None => r.find(&["variants"], Kind::Schema, format!("{name} is not a variant: dark or light")),
        }
    }
    if top.contains_key("variants") && m.variants.is_empty() {
        r.find(&["variants"], Kind::Schema, "variants names dark, light or both".to_owned());
    }
    if v2 {
        read_faces(&mut r, &mut m, &top);
    } else {
        for name in r.names(&top, "screens", true) {
            match Screen::named(&name) {
                Some(s) if !m.screens.contains(&s) => m.screens.push(s),
                Some(_) => {}
                None => r.find(&["screens"], Kind::Schema, format!("{name} is not a screen: {}", Screen::listed())),
            }
        }
    }
    if let Some(sizes) = top.get("fontSizes") {
        m.font_sizes_line = line_of(text, &["fontSizes"]);
        match sizes.as_array().map(|a| a.iter().map(Value::as_f64).collect::<Option<Vec<f64>>>()) {
            Some(Some(list)) if !list.is_empty() => m.font_sizes = list,
            _ => r.find(&["fontSizes"], Kind::Schema, "fontSizes is a list of text scales".to_owned()),
        }
    }
    if let Some(tokens) = top.get("tokens") {
        read_tokens(&mut r, &mut m, tokens, v2);
    }
    if let Some(cues) = top.get("cues") {
        match cues.as_object() {
            Some(map) => {
                for (cue, path) in map {
                    if !CUES.contains(&cue.as_str()) {
                        r.find(&["cues", cue], Kind::Schema, format!("{cue} is not a cue: {}", CUES.join(", ")));
                    } else if let Some(p) = r.path(&["cues", cue], path) {
                        m.cues.push((cue.clone(), p));
                    }
                }
            }
            None => r.find(&["cues"], Kind::Schema, "cues maps a cue's name to a file".to_owned()),
        }
    }
    if let Some(previews) = top.get("previews") {
        match previews.as_array() {
            Some(list) => {
                for item in list {
                    if let Some(p) = r.path(&["previews"], item) {
                        m.previews.push(p);
                    }
                }
            }
            None => r.find(&["previews"], Kind::Schema, "previews is a list of pictures".to_owned()),
        }
    }
    r.findings.sort_by_key(|f| f.line);
    Ok((m, r.findings))
}

fn read_faces(r: &mut Reader<'_>, m: &mut Manifest, top: &Map<String, Value>) {
    let Some(faces) = top.get("faces") else {
        r.find(&[], Kind::Schema, "faces is missing".to_owned());
        return;
    };
    let Some(map) = faces.as_object() else {
        r.find(&["faces"], Kind::Schema, "faces maps bigPicture and desktop to screen lists".to_owned());
        return;
    };
    for key in map.keys() {
        if key != "bigPicture" && key != "desktop" {
            r.find(&["faces", key], Kind::Schema, format!("{key} is not a face: bigPicture or desktop"));
        }
    }
    if map.contains_key("bigPicture") {
        for name in r.names_at(&["faces", "bigPicture"], map, "bigPicture", false) {
            match Screen::named(&name) {
                Some(s) if !m.screens.contains(&s) => m.screens.push(s),
                Some(_) => {}
                None => r.find(&["faces", "bigPicture"], Kind::Schema, format!("{name} is not a screen: {}", Screen::listed())),
            }
        }
    }
    if map.contains_key("desktop") {
        for name in r.names_at(&["faces", "desktop"], map, "desktop", false) {
            match DesktopScreen::named(&name) {
                Some(s) if !m.desktop_screens.contains(&s) => m.desktop_screens.push(s),
                Some(_) => {}
                None => r.find(&["faces", "desktop"], Kind::Schema, format!("{name} is not a desktop screen: {}", DesktopScreen::listed())),
            }
        }
    }
}

fn token_value(r: &mut Reader<'_>, keys: &[&str], name: &str, value: &Value) -> Option<TokenValue> {
    match value {
        Value::String(s) => Some(TokenValue::Text(s.clone())),
        Value::Number(n) => Some(TokenValue::Number(n.as_f64().unwrap_or(f64::NAN))),
        Value::Bool(b) => Some(TokenValue::Switch(*b)),
        _ => {
            r.find(keys, Kind::Schema, format!("{name} is a value: a colour, a face, a number or true/false"));
            None
        }
    }
}

fn read_tokens(r: &mut Reader<'_>, m: &mut Manifest, tokens: &Value, allow_desktop: bool) {
    let Some(variants) = tokens.as_object() else {
        r.find(&["tokens"], Kind::Schema, "tokens maps a variant to its overrides".to_owned());
        return;
    };
    for (variant_name, overrides) in variants {
        let keys = ["tokens", variant_name.as_str()];
        let Some(variant) = Variant::named(variant_name) else {
            r.find(&keys, Kind::Schema, format!("{variant_name} is not a variant: dark or light"));
            continue;
        };
        if !m.variants.contains(&variant) {
            r.find(&keys, Kind::Schema, format!("tokens for {variant_name}, which variants does not name"));
        }
        let Some(map) = overrides.as_object() else {
            r.find(&keys, Kind::Schema, format!("tokens.{variant_name} maps a token's name to a value"));
            continue;
        };
        for (name, value) in map {
            if name == "desktop" {
                if allow_desktop {
                    match value.as_object() {
                        Some(desktop_map) => {
                            for (dname, dvalue) in desktop_map {
                                let dkeys = ["tokens", variant_name.as_str(), "desktop", dname.as_str()];
                                if let Some(parsed) = token_value(r, &dkeys, dname, dvalue) {
                                    let line = line_of(r.text, &dkeys);
                                    m.desktop_tokens.push(TokenOverride { variant, name: dname.clone(), value: parsed, line });
                                }
                            }
                        }
                        None => r.find(
                            &["tokens", variant_name, "desktop"],
                            Kind::Schema,
                            format!("tokens.{variant_name}.desktop maps a token's name to a value"),
                        ),
                    }
                } else {
                    r.find(&["tokens", variant_name, "desktop"], Kind::Schema, "desktop token overrides require schema 2".to_owned());
                }
                continue;
            }
            let tkeys = ["tokens", variant_name.as_str(), name.as_str()];
            if let Some(parsed) = token_value(r, &tkeys, name, value) {
                let line = line_of(r.text, &tkeys);
                m.tokens.push(TokenOverride { variant, name: name.clone(), value: parsed, line });
            }
        }
    }
}
