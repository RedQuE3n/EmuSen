//! Serenity's slang half without a device: a `.slangp` preset and its `.slang` sources read as RetroArch writes them,
//! each stage compiled by Shaderc, what the SPIR-V binds read back from it, and the compiled stages kept in SQLite.
//!
//! The C# `EmuSen.Serenity.Slang` forwards here; its own half was deleted on 2026-11-05 (EmuSen_Backend.md §40.6), and
//! the parity classes hold this one to the answers recorded from it. The built-in filters and the presets they are
//! written as are here too (`filters`). Shaderc and SQLite are the libraries the program already ships: Shaderc found
//! beside the program, SQLite lent by the host, never a second copy.

pub mod cache;
pub mod chain;
pub mod compiler;
pub mod filters;
pub mod format;
pub mod lend;
pub mod parameters;
pub mod preset;
pub mod reflection;
pub mod ring;
pub mod source;
// SQLite, moved to its own crate for the library to share (EmuSen_Backend.md §28.2).
pub use emusen_sqlite as sqlite;
pub mod text;
pub mod vulkan;
pub mod word;

use emusen_galaxia::path_key;

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

/// `Path.GetFullPath`, with the paths it refuses: on Windows an empty or all-space path is "empty", elsewhere an empty one is an empty value.
pub(crate) fn full(path: &str) -> Result<String, Error> {
    if cfg!(windows) && path.bytes().all(|b| b == b' ') {
        return Err(Error::Argument { message: "The path is empty.".to_string(), parameter: "path".to_string() });
    }
    if path.is_empty() {
        return Err(Error::Argument { message: "The value cannot be an empty string.".to_string(), parameter: "path".to_string() });
    }
    if path.contains('\0') {
        return Err(Error::Argument { message: "Null character in path.".to_string(), parameter: "path".to_string() });
    }
    Ok(path_key::key_text(path))
}

/// `Path.GetFullPath(Path.Combine(directory, relative))`.
pub(crate) fn resolve(directory: &str, relative: &str) -> Result<String, Error> {
    full(&std::path::Path::new(directory).join(relative).to_string_lossy())
}

/// A file's lines, or why it could not be read.
pub(crate) fn read_lines(path: &str) -> Result<Vec<String>, Error> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(text::lines(&bytes)),
        Err(error) => Err(Error::Io { message: format!("{path}: {error}"), denied: error.kind() == std::io::ErrorKind::PermissionDenied }),
    }
}
