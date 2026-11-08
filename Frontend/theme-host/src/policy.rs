//! The sandbox's rules as plain data, with no Qt: which URL a theme's engine may reach, and which modules a theme's own files may
//! import (EmuSen_Qt.md §3.5 as built). The C++ interceptor asks `judge` for every URL; a refusal is the words the player reads.

use emusen_qml_rules::{check_theme, Rule, THEME_IMPORTS};
use std::path::{Component, Path, PathBuf};

/// What the engine is about to fetch, as `QQmlAbstractUrlInterceptor::DataType` numbers it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Qml,
    Script,
    Qmldir,
    Url,
}

impl Kind {
    /// Qt's numbers: QmlFile 0, JavaScriptFile 1, QmldirFile 2, UrlString 0x1000.
    pub fn from_qt(n: i32) -> Kind {
        match n {
            0 => Kind::Qml,
            1 => Kind::Script,
            2 => Kind::Qmldir,
            _ => Kind::Url,
        }
    }
}

/// Modules no theme engine resolves by any route, a quoted import or an inline component alike: the program's own modules and
/// those that reach files, processes, media or the network (EmuSen_Qt.md §3.5). Matched whole or as a prefix of dotted parts.
pub const NEVER_RESOLVED: [&str; 14] = [
    "EmuSen.Models",
    "EmuSen.App",
    "EmuSen.Play",
    "EmuSen.Endymion",
    "EmuSen.Esde",
    "EmuSen.ThemeHost",
    "QtMultimedia",
    "QtNetwork",
    "QtCore",
    "QtQml.XmlListModel",
    "QtQuick.LocalStorage",
    "QtQuick.Dialogs",
    "QtWebEngine",
    "Qt.labs",
];

/// Resource folders a theme's engine reads besides Qt's own: the modules it may see (`EmuSen.Theme`, and the kit it re-exports),
/// the EmuSen style's controls, and the kit's fonts.
const RESOURCES: [&str; 4] = ["/qt/qml/EmuSen/Theme/", "/qt/qml/EmuSen/Kit/", "/emusen/fonts/", "/qt-project.org/"];

/// One theme's rules: its folder, the cover stores, and Qt's own QML folder.
#[derive(Clone, Debug)]
pub struct Policy {
    /// The theme's folder as a URL prefix ending in `/`: `file:///…/` or `qrc:/…/`.
    folder: String,
    /// The theme's folder on disk, symbolic links resolved; none for a folder in the resources.
    folder_path: Option<PathBuf>,
    covers: Vec<PathBuf>,
    qt_qml: PathBuf,
}

/// Whether `module` is `prefix` or one of its dotted children.
fn under(module: &str, prefix: &str) -> bool {
    module == prefix || module.strip_prefix(prefix).is_some_and(|rest| rest.starts_with('.'))
}

/// A path with `.` and `..` taken out, as a URL's path is; symbolic links resolved where the file exists.
fn real(path: &Path) -> PathBuf {
    if let Ok(found) = path.canonicalize() {
        return found;
    }
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

/// The module a qmldir names by where it lies under an import root, `QtQuick/Controls/qmldir` → `QtQuick.Controls`.
pub fn module_of(relative: &str) -> Option<String> {
    let folder = relative.trim_start_matches('/').strip_suffix("qmldir")?.trim_end_matches('/');
    (!folder.is_empty()).then(|| folder.replace('/', "."))
}

/// Whether no theme engine may resolve `module`.
pub fn never_resolved(module: &str) -> bool {
    NEVER_RESOLVED.iter().any(|m| under(module, m))
}

/// The imports in a theme's QML file that are not on the allow-list, `line: words` each (qml-rules' theme profile, one owner).
pub fn refused_imports(file: &str, text: &str) -> Vec<String> {
    check_theme(file, text).into_iter().filter(|f| f.rule == Rule::Import).map(|f| format!("{}:{} imports {}", f.file, f.line, f.words)).collect()
}

/// The modules a theme's script file imports with `.import` that are not on the allow-list.
pub fn refused_script_imports(file: &str, text: &str) -> Vec<String> {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let module = line.trim().strip_prefix(".import ")?.split_whitespace().next()?;
            (!module.starts_with('"') && !THEME_IMPORTS.contains(&module)).then(|| format!("{file}:{} imports {module}", i + 1))
        })
        .collect()
}

impl Policy {
    /// `folder` is the theme's folder as a URL (`file:` or `qrc:`); `covers` the cover stores on disk; `qt_qml` Qt's QML folder.
    pub fn new(folder: &str, covers: &[String], qt_qml: &str) -> Policy {
        let folder = format!("{}/", folder.trim_end_matches('/'));
        let folder_path = folder.strip_prefix("file://").map(|p| real(Path::new(p)));
        let covers = covers.iter().filter(|c| !c.is_empty()).map(|c| real(Path::new(c))).collect();
        Policy { folder, folder_path, covers, qt_qml: real(Path::new(qt_qml)) }
    }

    /// The theme's folder as a URL prefix ending in `/`.
    pub fn folder(&self) -> &str {
        &self.folder
    }

    /// Whether `url` is a theme's own QML or script file, whose text `judge` must be given.
    pub fn reads(&self, url: &str, local: &str, kind: Kind) -> bool {
        matches!(kind, Kind::Qml | Kind::Script) && self.in_folder(url, local)
    }

    fn in_folder(&self, url: &str, local: &str) -> bool {
        match (&self.folder_path, local.is_empty()) {
            (Some(root), false) => real(Path::new(local)).starts_with(root),
            (None, true) => url.strip_prefix("qrc:").map(|p| format!("qrc:/{}", p.trim_start_matches('/'))).is_some_and(|u| u.starts_with(&self.folder)),
            _ => false,
        }
    }

    /// Empty when the engine may fetch `url`; else the words of its refusal. `local` is the URL's file path when it is a `file:`
    /// URL, and `text` the file's contents where `reads` asked for them.
    pub fn judge(&self, url: &str, local: &str, kind: Kind, text: &str) -> String {
        if url.is_empty() {
            return String::new();
        }
        if self.in_folder(url, local) {
            let file = url.strip_prefix(self.folder.as_str()).or_else(|| local.rsplit('/').next()).unwrap_or(url);
            let refused = match kind {
                Kind::Qml => refused_imports(file, text),
                Kind::Script => refused_script_imports(file, text),
                Kind::Qmldir | Kind::Url => Vec::new(),
            };
            return refused.into_iter().map(|r| format!("{r}, which a theme may not import (allowed: {})", THEME_IMPORTS.join(", "))).collect::<Vec<_>>().join("\n");
        }
        let scheme = url.split(':').next().unwrap_or("");
        match scheme {
            "file" => self.judge_file(url, local, kind),
            "qrc" => judge_resource(url, kind),
            "data" | "image" => String::new(),
            _ => format!("{url}: a theme reaches no network and no scheme but file:, qrc:, data: and image:"),
        }
    }

    fn judge_file(&self, url: &str, local: &str, kind: Kind) -> String {
        let path = real(Path::new(local));
        if let Ok(rest) = path.strip_prefix(&self.qt_qml) {
            return match module_of(&rest.to_string_lossy()).filter(|_| kind == Kind::Qmldir) {
                Some(module) if never_resolved(&module) => format!("{module} is a module no theme may use"),
                _ => String::new(),
            };
        }
        if kind == Kind::Url && self.covers.iter().any(|c| path.starts_with(c)) {
            return String::new();
        }
        format!("{url} is outside the theme's folder and the cover store")
    }
}

/// A folder's name without the version Qt adds when it probes for `import M 1.0`: `Theme.1.0` and `Theme.1` → `Theme`.
fn unversioned(folder: &str) -> &str {
    let mut name = folder;
    while let Some((head, tail)) = name.rsplit_once('.') {
        if tail.is_empty() || !tail.bytes().all(|b| b.is_ascii_digit()) {
            break;
        }
        name = head;
    }
    name
}

/// A resource: Qt's own and the theme's modules pass, the program's other resources do not. A qmldir is judged by the module it
/// is probed for, so Qt's versioned probes of an allowed module (`EmuSen/Theme.1.0/qmldir`) pass and are simply not found.
fn judge_resource(url: &str, kind: Kind) -> String {
    let path = format!("/{}", url.trim_start_matches("qrc:").trim_start_matches('/'));
    let path = if kind == Kind::Qmldir { path.split('/').map(unversioned).collect::<Vec<_>>().join("/") } else { path };
    let qml = path.strip_prefix("/qt/qml/");
    if let Some(module) = qml.filter(|_| kind == Kind::Qmldir).and_then(module_of).or_else(|| path.strip_prefix("/qt-project.org/imports/").and_then(module_of)) {
        if never_resolved(&module) {
            return format!("{module} is a module no theme may use");
        }
    }
    let style = qml.is_some_and(|rest| rest.strip_prefix("EmuSen/").is_some_and(|file| !file.contains('/')));
    let qt_own = qml.is_some_and(|rest| rest.starts_with("Qt"));
    if style || qt_own || RESOURCES.iter().any(|r| path.starts_with(r)) {
        return String::new();
    }
    format!("{url} is a resource of the program, not of the theme's modules")
}
