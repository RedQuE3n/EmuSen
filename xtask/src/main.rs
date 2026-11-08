//! EmuSen's build host, run as `cargo xtask <command>` from the repository root. See EmuSen_RustPlatform.md §28.11.

mod appimage;
mod check;
mod compile;
mod files;
mod host;
mod libraries;
mod prebuilt;
mod say;
mod stage;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage:
  cargo xtask stage --part cores|platform --out DIR [--profile release|dist] [--target-root DIR]
                    [--no-reuse] [--no-pgo] [--cache DIR] [--chdman FILE]
      build that part of libraries.tsv for this machine and lay it out in DIR/out and DIR/publish
  cargo xtask check-sidecars FOLDER
      fail when a library beside a sidecar in FOLDER is missing or is not the one the sidecar describes
  cargo xtask prebuilt --rid RID --out DIR [--from DIR]
      lay out the libraries CI built for RID, from FROM/RID/, for a publish for that platform
  PATCHELF=FILE cargo xtask appimage --out DIR [--tools DIR]
      bundle the kit's gallery program as DIR/kit-gallery-x86_64.AppImage with the tools appimage-tools.tsv pins,
      each checked against its sha256 first (tools from ~/.cache/emusen/toolchains/appimage unless named, and the
      distribution's patchelf where PATCHELF points)";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

/// `--name value` and `--flag` pairs after the command; a lone word is a positional argument.
fn options(args: &[String], flags: &[&str]) -> Result<(HashMap<String, String>, Vec<String>), String> {
    let (mut named, mut positional, mut i) = (HashMap::new(), Vec::new(), 0);
    while i < args.len() {
        let a = &args[i];
        if let Some(name) = a.strip_prefix("--") {
            if flags.contains(&name) {
                named.insert(name.to_string(), String::new());
            } else {
                let v = args.get(i + 1).ok_or(format!("--{name} needs a value"))?;
                named.insert(name.to_string(), v.clone());
                i += 1;
            }
        } else {
            positional.push(a.clone());
        }
        i += 1;
    }
    Ok((named, positional))
}

fn run(args: &[String]) -> Result<bool, String> {
    let repo = repo();
    match args.first().map(String::as_str) {
        Some("stage") => {
            let (o, _) = options(&args[1..], &["no-reuse", "no-pgo"])?;
            let part = match o.get("part").map(String::as_str) {
                Some("cores") => stage::Part::Cores,
                Some("platform") => stage::Part::Platform,
                _ => return Err(USAGE.into()),
            };
            let out = PathBuf::from(o.get("out").ok_or(USAGE)?);
            let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
            // The reuse cache stays off on CI and on Windows - see EmuSen_RustPlatform.md §28.9.
            let reuse_default = std::env::var("CI").as_deref() != Ok("true") && !cfg!(windows);
            let options = compile::Options {
                profile: o.get("profile").cloned().unwrap_or_else(|| "release".into()),
                target_root: o.get("target-root").map(PathBuf::from).unwrap_or_else(|| repo.join("target")),
                reuse: reuse_default && !o.contains_key("no-reuse"),
                cache: o.get("cache").map(PathBuf::from).unwrap_or_else(|| home.join(".cache/emusen/cores")),
                pgo: !o.contains_key("no-pgo"),
                repo: repo.clone(),
            };
            stage::run(part, &libraries::read(&repo)?, &options, o.get("chdman").map(PathBuf::from), &out)?;
            Ok(true)
        }
        Some("check-sidecars") => {
            let folder = PathBuf::from(args.get(1).ok_or(USAGE)?);
            let refused = check::check(&folder)?;
            refused.iter().for_each(|r| say::error(r));
            Ok(refused.is_empty())
        }
        Some("prebuilt") => {
            let (o, _) = options(&args[1..], &[])?;
            let rid = o.get("rid").ok_or(USAGE)?;
            let out = PathBuf::from(o.get("out").ok_or(USAGE)?);
            prebuilt::run(&libraries::read(&repo)?, rid, o.get("from").filter(|f| !f.is_empty()).map(Path::new), &out)?;
            Ok(true)
        }
        Some("appimage") => {
            let (o, _) = options(&args[1..], &[])?;
            let out = PathBuf::from(o.get("out").ok_or(USAGE)?);
            let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
            let tools = o.get("tools").map(PathBuf::from).unwrap_or_else(|| home.join(".cache/emusen/toolchains/appimage"));
            let image = appimage::run(&repo, &tools, &out)?;
            say::note(&format!("wrote {}", image.display()));
            Ok(true)
        }
        _ => Err(USAGE.into()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            say::error(&e);
            ExitCode::FAILURE
        }
    }
}
