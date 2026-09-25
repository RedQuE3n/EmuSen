//! The PGO profile a build is given, checked against this compiler and this source - see Mars_Native.md §6.17.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    for path in ["src", "Cargo.toml", "Cargo.lock"] {
        println!("cargo:rerun-if-changed={path}");
    }
    let source = format!("{:016x}", source_digest(&root));

    let flags: Vec<String> = env::var("CARGO_ENCODED_RUSTFLAGS")
        .unwrap_or_default()
        .split('\x1f')
        .filter(|f| !f.is_empty())
        .map(String::from)
        .collect();
    let used = codegen_option(&flags, "profile-use");
    let generating = codegen_option(&flags, "profile-generate").is_some() || flags.iter().any(|f| f == "-Cprofile-generate");

    // OUT_DIR is <target dir>[/<triple>]/<profile>/build/marsrt-<hash>/out.
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let profile_dir = out.ancestors().nth(3).expect("OUT_DIR's profile directory").to_path_buf();
    let cargo_profile = profile_dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let target = env::var("TARGET").unwrap_or_default();
    let explicit = profile_dir.parent().and_then(Path::file_name).is_some_and(|n| n == target.as_str());

    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let verbose = Command::new(&rustc)
        .arg("-vV")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let compiler = verbose.lines().next().unwrap_or("rustc ?").trim().to_string();
    let llvm = verbose.lines().find_map(|l| l.strip_prefix("LLVM version: ")).unwrap_or("?").trim().to_string();
    let flavour = format!("{compiler} | {cargo_profile} | {}", if explicit { target.as_str() } else { "host" });

    let mut lines = vec![format!("source = {source}"), format!("flavour = {flavour}"), format!("llvm = {llvm}")];
    let (state, warning) = match &used {
        None if generating => ("instrumented", None),
        None => ("none", None),
        Some(path) => {
            let manifest = Path::new(path).with_extension("pgo");
            println!("cargo:rerun-if-changed={path}");
            println!("cargo:rerun-if-changed={}", manifest.display());
            let Ok(text) = fs::read_to_string(&manifest) else {
                fail(&format!("-Cprofile-use={path} has no manifest beside it ({}); a profile is used only with the manifest pgo/train.sh writes", manifest.display()));
            };
            let values = |key: &str| -> Vec<String> {
                text.lines()
                    .filter_map(|l| l.split_once('='))
                    .filter(|(k, _)| k.trim() == key)
                    .map(|(_, v)| v.trim().to_string())
                    .collect()
            };
            let written = values("llvm").into_iter().next().unwrap_or_default();
            if major(&written) != major(&llvm) {
                fail(&format!(
                    "MarsRT's PGO profile was written by llvm-profdata {written}, and this rustc ({compiler}) uses LLVM {llvm}. Refresh the profile with pgo/train.sh, or build without it: -p:EmuSenPgo=false for dotnet, no -Cprofile-use for cargo. See Mars_Native.md §6.17."
                ));
            }
            lines.push(format!("profile = {path}"));
            let trained = values("source").into_iter().next().unwrap_or_default();
            if !values("flavour").iter().any(|f| *f == flavour) {
                ("untrained", Some(format!("MarsRT's PGO profile was not trained for this build ({flavour}): it builds, and no function in it is profile-guided. See Mars_Native.md §6.17.")))
            } else if trained != source {
                ("stale", Some(format!("MarsRT's PGO profile was trained on source {trained}, and this is {source}: it builds, and each function changed since loses its guidance. Refresh with pgo/train.sh. See Mars_Native.md §6.17.")))
            } else {
                ("matched", None)
            }
        }
    };
    lines.insert(0, format!("state = {state}"));
    if let Some(message) = warning {
        println!("cargo:warning={message}");
        lines.push(format!("warning = {message}"));
    }
    let _ = fs::write(profile_dir.join("marsrt-pgo.txt"), lines.join("\n") + "\n");
}

// The value of -C<name>=..., given as one flag or two.
fn codegen_option(flags: &[String], name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    let mut previous_c = false;
    for flag in flags {
        let body = flag.strip_prefix("-C").filter(|b| !b.is_empty()).or(if previous_c { Some(flag.as_str()) } else { None });
        if let Some(value) = body.and_then(|b| b.strip_prefix(&prefix)) {
            return Some(value.to_string());
        }
        previous_c = flag == "-C";
    }
    None
}

fn major(version: &str) -> &str {
    version.split('.').next().unwrap_or(version)
}

fn fail(message: &str) -> ! {
    eprintln!("error: {message}");
    exit(1);
}

// FNV-1a over what the library is built from: src/ but its tests, Cargo.toml and Cargo.lock, by path, carriage returns dropped.
fn source_digest(root: &Path) -> u64 {
    let mut files = vec![root.join("Cargo.toml"), root.join("Cargo.lock")];
    collect(&root.join("src"), &root.join("src").join("tests"), &mut files);
    let mut named: Vec<(String, PathBuf)> = files
        .into_iter()
        .map(|p| (p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/"), p))
        .collect();
    named.sort();
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut feed = |bytes: &[u8]| {
        for &b in bytes {
            if b != b'\r' {
                hash = (hash ^ b as u64).wrapping_mul(0x0100_0000_01b3);
            }
        }
    };
    for (name, path) in named {
        feed(name.as_bytes());
        feed(&[0]);
        feed(&fs::read(&path).unwrap_or_default());
        feed(&[0]);
    }
    hash
}

fn collect(dir: &Path, skip: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path == skip {
            continue;
        }
        if path.is_dir() {
            collect(&path, skip, into);
        } else {
            into.push(path);
        }
    }
}
