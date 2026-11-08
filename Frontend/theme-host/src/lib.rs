//! emusen-theme-host, the sandboxed host of an EmuSen theme (EmuSen_Qt.md §3.5): the policy as plain data (`policy`), the package
//! read through the manifest's owner (`package`), the C++ shells' bridge (`sandbox`) and the QML type the program places (`host`).

pub mod host;
pub mod package;
pub mod policy;
pub mod sandbox;
