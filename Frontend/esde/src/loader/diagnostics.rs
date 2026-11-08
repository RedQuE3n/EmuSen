//! THEMES.md's three outcomes: an error unthemes the system, a warning resets a value, a debug note is silent
//! (EmuSen_BigPicture.md §12.3). Each diagnostic carries a code, the file and the line.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Debug,
    Warning,
    Error,
}

macro_rules! codes {
    ($($name:ident),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Code { $($name),* }

        impl Code {
            pub fn name(self) -> &'static str {
                match self { $(Code::$name => stringify!($name)),* }
            }
        }
    };
}

codes!(
    CapabilitiesMissing,
    NoThemeFile,
    MalformedXml,
    WrongRoot,
    MisplacedTag,
    UnknownTag,
    UnknownElement,
    UnknownProperty,
    UnknownView,
    MissingName,
    LegacySyntax,
    NoValue,
    BadFormat,
    InvalidValue,
    InvalidImageType,
    UndefinedVariable,
    IncludeMissing,
    IncludeSkipped,
    IncludeLoop,
    PathMissing,
    Undeclared,
    Duplicate,
    Reserved,
    ViewRestricted,
    SecondPrimary,
    LanguageWithoutEnglish,
    UnusedAttribute,
    LenientValue,
    LenientXml,
    IgnoredTag,
);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Code,
    pub file: String,
    pub line: u32,
    pub message: String,
    /// What an unknown tag, element or property names: the tag, the element type, or type.property; empty otherwise.
    pub subject: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = std::path::Path::new(&self.file).file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
        write!(f, "{:?} {} {}:{}: {}", self.severity, self.code.name(), name, self.line, self.message)
    }
}

/// Diagnostics in the order they happen.
#[derive(Clone, Debug, Default)]
pub struct Diagnostics {
    pub items: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn add(&mut self, severity: Severity, code: Code, file: &str, line: u32, message: impl Into<String>) {
        self.add_about(severity, code, file, line, "", message);
    }

    pub fn add_about(&mut self, severity: Severity, code: Code, file: &str, line: u32, subject: &str, message: impl Into<String>) {
        self.items.push(Diagnostic { severity, code, file: file.to_owned(), line, message: message.into(), subject: subject.to_owned() });
    }

    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|d| d.severity == Severity::Error)
    }
}
