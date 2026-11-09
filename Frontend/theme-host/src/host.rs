//! `ThemeHost`: the program's handle on one EmuSen theme (EmuSen_Qt.md §3.5). Setting `folder` reads the package and makes its
//! sandboxed engine, destroying the one before; `place` creates a screen the theme provides into a slot of the program's scene.

use crate::package::{self, Package};
use emusen_theme_manifest::Screen;
use crate::policy::Policy;
use crate::sandbox::ffi::{sandbox_engine, sandbox_new, sandbox_place, sandbox_qt_qml, sandbox_read, sandbox_take_log, watch_alive, watch_new, Sandbox, Watch};
use core::pin::Pin;
use cxx::UniquePtr;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QStringList};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!(<QtQuick/QQuickItem>);
        type QQuickItem = crate::sandbox::ffi::QQuickItem;
    }

    /// The screens a theme may provide.
    #[qenum(ThemeHost)]
    enum Role {
        Home,
        Consoles,
        Games,
        Splash,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, folder, READ, WRITE = set_folder, NOTIFY)]
        #[qproperty(QStringList, cover_stores, cxx_name = "coverStores", READ, WRITE = set_cover_stores, NOTIFY)]
        #[qproperty(QString, name, READ, NOTIFY)]
        #[qproperty(bool, ready, READ, NOTIFY)]
        #[qproperty(bool, failed, READ, NOTIFY)]
        #[qproperty(QString, warnings, READ, NOTIFY)]
        #[qproperty(i32, generation, READ, NOTIFY)]
        #[qproperty(i32, live_engines, cxx_name = "liveEngines", READ, NOTIFY)]
        type ThemeHost = super::ThemeHostRust;
    }

    unsafe extern "RustQt" {
        /// Reads the package at the folder and makes its engine; empty for none.
        fn set_folder(self: Pin<&mut ThemeHost>, folder: QString);
        /// The folders whose files a theme may show as pictures; a loaded theme is loaded again under them.
        fn set_cover_stores(self: Pin<&mut ThemeHost>, stores: QStringList);
        /// Whether the theme provides a screen for the role; one it omits is the reference's.
        #[qinvokable]
        fn provides(self: &ThemeHost, role: Role) -> bool;
        /// Creates the role's screen in the theme's engine as a child of `slot`, filling it; false, with `failed` set, if it fails.
        #[qinvokable]
        unsafe fn place(self: Pin<&mut ThemeHost>, role: Role, slot: *mut QQuickItem) -> bool;
    }
}

/// The host's state. The sandbox is declared before the policy its interceptor reads, so it is dropped first.
pub struct ThemeHostRust {
    folder: QString,
    cover_stores: QStringList,
    name: QString,
    ready: bool,
    failed: bool,
    warnings: QString,
    generation: i32,
    /// How many engines this host made are still alive: one while a theme is loaded, none otherwise.
    live_engines: i32,
    sandbox: UniquePtr<Sandbox>,
    policy: Option<Box<Policy>>,
    package: Option<Package>,
    engines: Vec<UniquePtr<Watch>>,
}

impl Default for ThemeHostRust {
    fn default() -> Self {
        ThemeHostRust {
            folder: QString::default(),
            cover_stores: QStringList::default(),
            name: QString::default(),
            ready: false,
            failed: false,
            warnings: QString::default(),
            generation: 0,
            live_engines: 0,
            sandbox: UniquePtr::null(),
            policy: None,
            package: None,
            engines: Vec::new(),
        }
    }
}

/// The manifest's screen for a role.
fn screen(role: qobject::Role) -> Screen {
    match role {
        qobject::Role::Consoles => Screen::Consoles,
        qobject::Role::Games => Screen::Games,
        qobject::Role::Splash => Screen::Splash,
        _ => Screen::Home,
    }
}

/// A folder as a URL: `file:` and `qrc:` as given, a plain path as a `file:` URL; no trailing `/`.
pub fn folder_url(folder: &str) -> String {
    let trimmed = folder.trim_end_matches('/');
    if trimmed.starts_with("file:") || trimmed.starts_with("qrc:") {
        trimmed.to_owned()
    } else {
        format!("file://{trimmed}")
    }
}

impl qobject::ThemeHost {
    pub fn set_folder(mut self: Pin<&mut Self>, folder: QString) {
        if self.folder == folder {
            return;
        }
        self.as_mut().rust_mut().folder = folder;
        self.as_mut().folder_changed();
        self.reload();
    }

    pub fn set_cover_stores(mut self: Pin<&mut Self>, stores: QStringList) {
        if self.cover_stores == stores {
            return;
        }
        self.as_mut().rust_mut().cover_stores = stores;
        self.as_mut().cover_stores_changed();
        if !self.folder.is_empty() {
            self.reload();
        }
    }

    /// The engine before goes first, then the package is read and a new engine made; `generation` tells the slots to place again.
    fn reload(mut self: Pin<&mut Self>) {
        self.as_mut().unload();
        // Down before up, so a slot bound to `ready` lets its screen go before the next theme's is placed.
        if self.ready {
            self.as_mut().rust_mut().ready = false;
            self.as_mut().ready_changed();
        }
        let folder = self.folder.to_string();
        let loaded = if folder.is_empty() { Ok(None) } else { self.as_mut().load(&folder).map(Some) };
        let (name, ready, failed, warnings) = match &loaded {
            Ok(Some(p)) => (p.name.as_str(), true, false, String::new()),
            Ok(None) => ("", false, false, String::new()),
            Err(words) => ("", false, true, words.clone()),
        };
        self.as_mut().rust_mut().name = QString::from(name);
        self.as_mut().rust_mut().ready = ready;
        self.as_mut().rust_mut().failed = failed;
        self.as_mut().rust_mut().warnings = QString::from(warnings.as_str());
        self.as_mut().rust_mut().package = loaded.ok().flatten();
        let generation = self.generation + 1;
        self.as_mut().rust_mut().generation = generation;
        let live = self.engines.iter().filter(|w| watch_alive(w)).count() as i32;
        self.as_mut().rust_mut().live_engines = live;
        self.as_mut().live_engines_changed();
        self.as_mut().name_changed();
        self.as_mut().ready_changed();
        self.as_mut().failed_changed();
        self.as_mut().warnings_changed();
        self.as_mut().generation_changed();
    }

    fn unload(mut self: Pin<&mut Self>) {
        let mut rust = self.as_mut().rust_mut();
        rust.sandbox = UniquePtr::null();
        rust.policy = None;
        rust.package = None;
        rust.engines.retain(|w| watch_alive(w));
    }

    fn load(mut self: Pin<&mut Self>, folder: &str) -> Result<Package, String> {
        let folder = folder_url(folder);
        let mut text = QString::default();
        if !sandbox_read(&QString::from(format!("{folder}/theme.json").as_str()), &mut text) {
            return Err(format!("{folder}/theme.json cannot be read"));
        }
        let package = package::read(&text.to_string())?;
        let covers: Vec<String> = self.cover_stores.iter().map(|s| s.to_string()).collect();
        let policy = Box::new(Policy::new(&folder, &covers, &sandbox_qt_qml().to_string()));
        let sandbox = sandbox_new(&policy);
        let engine = sandbox_engine(&sandbox);
        let watch = unsafe { watch_new(engine) };
        let mut rust = self.as_mut().rust_mut();
        rust.policy = Some(policy);
        rust.sandbox = sandbox;
        rust.engines.push(watch);
        Ok(package)
    }

    /// The theme's engine, which the program's feed hands the views (EmuSen_Qt.md §3.5.1); null while none is loaded.
    pub fn engine(&self) -> *mut cxx_qt::QObject {
        self.sandbox.as_ref().map_or(std::ptr::null_mut(), sandbox_engine)
    }

    /// The package's colour overrides, as `Package::colour_overrides` gives them; none while no package is loaded.
    pub fn colour_overrides(&self) -> Vec<(bool, String, String)> {
        self.package.as_ref().map(Package::colour_overrides).unwrap_or_default()
    }

    pub fn provides(&self, role: qobject::Role) -> bool {
        self.package.as_ref().is_some_and(|p| p.screens.contains(&screen(role)))
    }

    /// # Safety
    /// `slot` is an item of the program's scene, alive for the call.
    pub unsafe fn place(mut self: Pin<&mut Self>, role: qobject::Role, slot: *mut qobject::QQuickItem) -> bool {
        if !self.provides(role) {
            return false;
        }
        let Some(policy) = self.policy.as_ref() else {
            return false;
        };
        let url = QString::from(format!("{}{}", policy.folder(), screen(role).file()).as_str());
        let placed = match self.as_mut().rust_mut().sandbox.as_mut() {
            Some(sandbox) => unsafe { sandbox_place(sandbox, &url, slot) },
            None => false,
        };
        self.as_mut().take_log();
        if !placed && !self.failed {
            self.as_mut().rust_mut().failed = true;
            self.as_mut().failed_changed();
        }
        placed
    }

    /// The engine's warnings and the sandbox's refusals, added to `warnings`.
    fn take_log(mut self: Pin<&mut Self>) {
        let lines = match self.as_mut().rust_mut().sandbox.as_mut() {
            Some(sandbox) => sandbox_take_log(sandbox).to_string(),
            None => return,
        };
        if lines.is_empty() {
            return;
        }
        let before = self.warnings.to_string();
        let all = if before.is_empty() { lines } else { format!("{before}\n{lines}") };
        self.as_mut().rust_mut().warnings = QString::from(all.as_str());
        self.as_mut().warnings_changed();
    }
}
