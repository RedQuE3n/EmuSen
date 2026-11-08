// SyntheticTheme as the C# fixtures write it: a small ES-DE theme written for the tests into a temporary folder; nothing is copied from a real theme.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use emusen_esde::loader::{self, values::full_path, Choices, Code, MediaPresence, Severity, Theme, ThemeSystem};

pub struct Synth {
    pub root: PathBuf,
}

static NEXT: AtomicUsize = AtomicUsize::new(0);

impl Synth {
    pub fn new() -> Self {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
        let root = std::env::temp_dir().join("EmuSenSyntheticTheme").join(format!("{}-{n}-{nanos:x}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        Synth { root }
    }

    pub fn dir(&self) -> String {
        self.root.to_string_lossy().into_owned()
    }

    pub fn capabilities(&self, body: &str) -> &Self {
        self.file("capabilities.xml", &format!("<themeCapabilities>\n{body}\n</themeCapabilities>"))
    }

    pub fn theme(&self, body: &str) -> &Self {
        self.theme_at(body, "theme.xml")
    }

    pub fn theme_at(&self, body: &str, file: &str) -> &Self {
        self.file(file, &format!("<theme>\n{body}\n</theme>"))
    }

    pub fn file(&self, relative: &str, text: &str) -> &Self {
        let path = self.root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
        self
    }

    // An asset of the given name, so path properties resolve to a file that exists.
    pub fn asset(&self, relative: &str) -> &Self {
        self.file(relative, "asset")
    }

    pub fn path_of(&self, relative: &str) -> String {
        full_path(&self.root.join(relative).to_string_lossy())
    }

    pub fn load(&self) -> Theme {
        self.load_with(&Choices::default(), &snes(), None)
    }

    pub fn load_choices(&self, choices: &Choices) -> Theme {
        self.load_with(choices, &snes(), None)
    }

    pub fn load_with(&self, choices: &Choices, system: &ThemeSystem, media: Option<&MediaPresence>) -> Theme {
        loader::load(&self.dir(), system, choices, media)
    }
}

impl Drop for Synth {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub fn snes() -> ThemeSystem {
    ThemeSystem::new("snes", "Super Nintendo", "snes")
}

pub fn exists(path: &Path) -> bool {
    path.exists()
}

/// The player's choices, set one at a time as the C# tests' object initialisers set them.
pub trait Pick {
    fn variant(self, v: &str) -> Self;
    fn scheme(self, v: &str) -> Self;
    fn font(self, v: &str) -> Self;
    fn language(self, v: &str) -> Self;
    fn aspect(self, v: &str) -> Self;
    fn transitions(self, v: &str) -> Self;
    fn screen(self, w: i32, h: i32) -> Self;
}

impl Pick for Choices {
    fn variant(mut self, v: &str) -> Self {
        self.variant = Some(v.into());
        self
    }
    fn scheme(mut self, v: &str) -> Self {
        self.color_scheme = Some(v.into());
        self
    }
    fn font(mut self, v: &str) -> Self {
        self.font_size = Some(v.into());
        self
    }
    fn language(mut self, v: &str) -> Self {
        self.language = Some(v.into());
        self
    }
    fn aspect(mut self, v: &str) -> Self {
        self.aspect_ratio = Some(v.into());
        self
    }
    fn transitions(mut self, v: &str) -> Self {
        self.transitions = Some(v.into());
        self
    }
    fn screen(mut self, w: i32, h: i32) -> Self {
        self.screen_width = w;
        self.screen_height = h;
        self
    }
}

pub fn ch() -> Choices {
    Choices::default()
}

pub fn count(theme: &Theme, code: Code) -> usize {
    theme.diagnostics.iter().filter(|d| d.code == code).count()
}

pub fn count_at(theme: &Theme, code: Code, severity: Severity) -> usize {
    theme.diagnostics.iter().filter(|d| d.code == code && d.severity == severity).count()
}

pub fn has_error(theme: &Theme, code: Code) -> bool {
    theme.errors().any(|d| d.code == code)
}

pub fn assert_unthemed(theme: &Theme, code: Code) {
    assert!(!theme.is_themed(), "themed: {:#?}", theme.diagnostics);
    assert!(has_error(theme, code), "no {code:?} error: {:#?}", theme.diagnostics);
}

pub fn shown(theme: &Theme) -> String {
    theme.diagnostics.iter().map(|d| d.to_string()).collect::<Vec<_>>().join("\n")
}

pub fn color(theme: &Theme, name: &str) -> String {
    theme.gamelist_view.find("text", name).unwrap().color("color").unwrap().to_string()
}

pub fn text(theme: &Theme, view: &str, name: &str) -> Option<String> {
    theme.view(view).find("text", name).and_then(|e| e.string("text")).map(str::to_owned)
}
