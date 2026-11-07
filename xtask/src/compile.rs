//! One cargo build per library, with its PGO profile and the reuse cache, and the core ABI v1 sidecars after them.
//! See EmuSen_RustPlatform.md §28.9-§28.11 and Mars_Native.md §6.17.

use crate::files::{copy_keeping_time, sha256_hex};
use crate::libraries::{Library, Role};
use crate::{host, say};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Options {
    pub repo: PathBuf,
    pub profile: String,
    pub target_root: PathBuf,
    pub reuse: bool,
    pub cache: PathBuf,
    pub pgo: bool,
}

impl Options {
    /// Where cargo leaves `lib`'s file: `<root>/<name>/<profile>/<file>`.
    pub fn built(&self, lib: &Library) -> PathBuf {
        let rid = host::portable_rid();
        let file = if lib.role == Role::Program { host::program_file(&lib.name, &rid) } else { host::library_file(&lib.name, &rid) };
        self.target_root.join(&lib.name).join(&self.profile).join(file)
    }
}

/// A command with the variables `cargo run` gave this process removed, so a nested build sees the caller's environment.
pub fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut c = Command::new(program);
    for (k, _) in std::env::vars_os() {
        let k = k.to_string_lossy();
        if k.starts_with("CARGO_PKG_") || ["CARGO_MANIFEST_DIR", "CARGO_MANIFEST_PATH", "CARGO_CRATE_NAME", "CARGO_BIN_NAME", "CARGO_PRIMARY_PACKAGE"].contains(&k.as_ref()) {
            c.env_remove(k.as_ref());
        }
    }
    c
}

pub fn run(c: &mut Command, what: &str) -> Result<(), String> {
    let status = c.status().map_err(|e| format!("{what}: {e}"))?;
    if status.success() { Ok(()) } else { Err(format!("{what} failed ({status})")) }
}

/// The profile a build of `lib` is guided by: its hash, and the flag that names the hashed copy.
struct Pgo {
    hash: String,
    config: String,
}

/// A crate holding pgo/<name>.<target>.profdata is built with it when the manifest lists this compiler's flavour.
fn pgo(lib: &Library, o: &Options) -> Result<Option<Pgo>, String> {
    let dir = lib.folder(&o.repo);
    let profile = dir.join("pgo").join(format!("{}.{}.profdata", lib.name, host::rust_target()));
    if !o.pgo || !profile.is_file() {
        return Ok(None);
    }
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let said = command(&rustc).arg("-V").current_dir(&dir).output().map_err(|e| format!("rustc -V: {e}"))?;
    let rustc = String::from_utf8_lossy(&said.stdout).trim().to_string();
    let manifest = std::fs::read_to_string(profile.with_extension("pgo")).unwrap_or_default();
    let flavour = format!("flavour = {rustc} | {} | host", o.profile);
    if !manifest.lines().any(|l| l.trim() == flavour) {
        say::note(&format!("{} builds without its PGO profile, which was not trained for {rustc}, {}.", lib.core, o.profile));
        return Ok(None);
    }
    let hash = sha256_hex(&profile)?.to_uppercase();
    let copy = o.target_root.join(&lib.name).join("pgo").join(format!("{}-{}", lib.name, &hash[..16]));
    copy_keeping_time(&profile, &copy.with_extension("profdata"))?;
    copy_keeping_time(&profile.with_extension("pgo"), &copy.with_extension("pgo"))?;
    if std::env::var_os("RUSTFLAGS").is_some_and(|v| !v.is_empty()) || std::env::var_os("CARGO_ENCODED_RUSTFLAGS").is_some_and(|v| !v.is_empty()) {
        say::warn(&format!("RUSTFLAGS is set and overrides build.rustflags: {} builds without its PGO profile.", lib.core));
    }
    let config = format!("build.rustflags=['-Cprofile-use={}.profdata', '-Cllvm-args=-no-pgo-warn-mismatch']", copy.display());
    Ok(Some(Pgo { hash, config }))
}

/// The cache entry for a build here from the same committed inputs, named by library_key.py's key; None when an input is uncommitted.
fn cache_entry(lib: &Library, o: &Options, pgo: Option<&Pgo>) -> Option<PathBuf> {
    if !o.reuse {
        return None;
    }
    let script = o.repo.join(".github/scripts/library_key.py");
    let target = host::rust_target();
    let pgo = pgo.map_or("off", |p| p.hash.as_str());
    let said = command("python3").arg("-I").arg(&script).args(["local", ".", &lib.name, &target, &o.profile, pgo]).current_dir(lib.folder(&o.repo)).output().ok()?;
    let key = String::from_utf8_lossy(&said.stdout).trim().to_string();
    let valid = said.status.success() && key.len() == 24 && key.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    valid.then(|| o.cache.join(format!("{}-{target}-{}-{key}", lib.name, o.profile)))
}

/// Builds `lib`, or copies it from the reuse cache when a build of the same inputs is there; a failure stops the build.
pub fn build(lib: &Library, o: &Options) -> Result<(), String> {
    let dir = lib.folder(&o.repo);
    let pgo = pgo(lib, o)?;
    let built = o.built(lib);
    let file = built.file_name().unwrap().to_owned();
    let entry = cache_entry(lib, o, pgo.as_ref());
    if let Some(cached) = entry.as_ref().map(|e| e.join(&file)).filter(|c| c.is_file()) {
        copy_keeping_time(&cached, &built)?;
        say::note(&format!("{}: copied from {}, built before from the same inputs.", lib.core, cached.parent().unwrap().display()));
        return Ok(());
    }
    // Run from the crate's folder, as CI runs it, so a crate's own .cargo/config.toml applies - see Mercury_Native.md §8.7.
    let mut c = command("cargo");
    c.args(["build", "--profile", &o.profile, "--manifest-path"]).arg(dir.join("Cargo.toml")).arg("--target-dir").arg(o.target_root.join(&lib.name)).current_dir(&dir);
    if let Some(p) = &pgo {
        c.args(["--config", &p.config]);
    }
    run(&mut c, &format!("cargo build of {}", lib.core))?;
    if let Some(entry) = entry.filter(|_| built.is_file()) {
        store(&built, &entry, &file, o)?;
    }
    Ok(())
}

/// Stored under a unique name, then renamed, so a build in another checkout never copies half a file.
fn store(built: &Path, entry: &Path, file: &std::ffi::OsStr, o: &Options) -> Result<(), String> {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
    let partial = entry.join(format!("{}.{}-{nanos}", file.to_string_lossy(), std::process::id()));
    copy_keeping_time(built, &partial)?;
    std::fs::rename(&partial, entry.join(file)).map_err(|e| format!("{}: {e}", entry.display()))?;
    let script = o.repo.join(".github/scripts/library_key.py");
    let name = entry.file_name().unwrap().to_string_lossy().to_string();
    let _ = command("python3").arg("-I").arg(script).arg("prune").arg(&o.cache).args([name.as_str(), "4"]).status();
    Ok(())
}

/// Each library on the core ABI v1 gets its sidecar from the conformance kit's runner, which loads it once - see EmuSen_CoreAPI.md §7.1, §22.
pub fn sidecars(libs: &[&Library], o: &Options) -> Result<(), String> {
    let v1: Vec<&&Library> = libs.iter().filter(|l| l.abi_v1).collect();
    if v1.is_empty() {
        return Ok(());
    }
    let conform = o.repo.join("EmuSen/Cores/Shared/emusen-core-conform");
    let target = o.target_root.join("emusen-core-conform");
    let mut c = command("cargo");
    c.args(["build", "--release", "--bin", "emusen-core-conform", "--manifest-path"]).arg(conform.join("Cargo.toml")).arg("--target-dir").arg(&target).current_dir(&conform);
    run(&mut c, "cargo build of the conformance runner")?;
    let runner = target.join("release").join(host::program_file("emusen-core-conform", &host::portable_rid()));
    for lib in v1 {
        let mut c = command(&runner);
        c.arg("--sidecar").arg(o.built(lib));
        if !lib.ships {
            c.arg("--development");
        }
        run(&mut c, &format!("the sidecar of {}", lib.core))?;
    }
    Ok(())
}
