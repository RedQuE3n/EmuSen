//! Serenity's slang half without a device: a `.slangp` preset and its `.slang` sources read as RetroArch writes them,
//! each stage compiled by Shaderc, what the SPIR-V binds read back from it, and the compiled stages kept in SQLite.
//!
//! The C# `EmuSen.Serenity.Slang` forwards here behind `EMUSEN_SERENITY_NATIVE`. Every rule reproduces the C#'s, which
//! is its oracle until its gate; EmuSen_RustPlatform.md §16 says how that was held. Shaderc and SQLite are the
//! libraries the program already ships, lent by the host or opened by path, never a second copy.

pub mod cache;
pub mod compiler;
pub mod parameters;
pub mod preset;
pub mod reflection;
pub mod source;
pub mod sqlite;
pub mod text;
pub mod word;

use emusen_galaxia::dotnet_path::{Style, combine, full_path};

/// Why a preset or a source could not be read, as the C# exception it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// `FileNotFoundException`, with the file it names.
    NotFound { message: String, file: String },
    /// `InvalidDataException`.
    InvalidData(String),
    /// `ArgumentException`, with the parameter it names.
    Argument { message: String, parameter: String },
    /// A file that is there and could not be read: `UnauthorizedAccessException` when access was refused, else `IOException`.
    Io { message: String, denied: bool },
}

impl Error {
    pub fn message(&self) -> &str {
        match self {
            Error::NotFound { message, .. } | Error::InvalidData(message) | Error::Argument { message, .. } | Error::Io { message, .. } => message,
        }
    }
}

/// `Path.GetFullPath`, with the two paths it refuses.
pub(crate) fn full(path: &str) -> Result<String, Error> {
    if path.is_empty() {
        return Err(Error::Argument { message: "The value cannot be an empty string.".to_string(), parameter: "path".to_string() });
    }
    if path.contains('\0') {
        return Err(Error::Argument { message: "Null character in path.".to_string(), parameter: "path".to_string() });
    }
    Ok(full_path(path))
}

/// `Path.GetFullPath(Path.Combine(directory, relative))`.
pub(crate) fn resolve(directory: &str, relative: &str) -> Result<String, Error> {
    full(&combine(Style::HOST, directory, relative))
}

/// A file's lines, or why it could not be read.
pub(crate) fn read_lines(path: &str) -> Result<Vec<String>, Error> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(text::lines(&bytes)),
        Err(error) => Err(Error::Io { message: format!("{path}: {error}"), denied: error.kind() == std::io::ErrorKind::PermissionDenied }),
    }
}
