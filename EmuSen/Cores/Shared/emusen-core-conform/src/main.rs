//! `emusen-core-conform --core LIB --image FILE [--self-delimiting] [--frames N] [--input SCRIPT] [--settings KEY=VALUE]... [--file WHICH=PATH]...
//! [--report REPORT.json]` runs the core suite and exits 0 when every case passes; `emusen-core-conform --sidecar LIB`
//! writes the library's sidecar. See EmuSen_CoreAPI.md §12.3 and §21.

use std::path::PathBuf;
use std::process::ExitCode;

use emusen_core_conform::{Options, Script, run, sidecar};

fn usage() -> ExitCode {
    eprintln!("usage: emusen-core-conform --core LIB --image FILE [--self-delimiting] [--frames N] [--input SCRIPT] [--settings KEY=VALUE]... [--file WHICH=PATH]... [--report REPORT.json]");
    eprintln!("       emusen-core-conform --sidecar LIB");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut core = None;
    let mut image = None;
    let mut frames = 600u64;
    let mut script = None;
    let mut settings = String::new();
    let mut files = Vec::new();
    let mut report = None;
    let mut self_delimiting = false;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--self-delimiting" {
            self_delimiting = true;
            i += 1;
            continue;
        }
        let value = args.get(i + 1).cloned();
        let Some(v) = value else { return usage() };
        match args[i].as_str() {
            "--sidecar" => {
                return match sidecar::write(&PathBuf::from(&v)) {
                    Ok(path) => {
                        println!("{}", path.display());
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("error: {e}");
                        ExitCode::FAILURE
                    }
                };
            }
            "--core" => core = Some(PathBuf::from(v)),
            "--image" => image = Some(PathBuf::from(v)),
            "--frames" => match v.parse() {
                Ok(n) if n > 0 => frames = n,
                _ => return usage(),
            },
            "--input" => match std::fs::read_to_string(&v).map_err(|e| e.to_string()).and_then(|t| Script::parse(&t)) {
                Ok(s) => script = Some(s),
                Err(e) => {
                    eprintln!("error: {e}");
                    return ExitCode::from(2);
                }
            },
            "--settings" => {
                settings.push_str(&v);
                settings.push('\n');
            }
            "--file" => {
                let Some((which, path)) = v.split_once('=') else { return usage() };
                let (Ok(which), Ok(data)) = (which.parse::<u32>(), std::fs::read(path)) else { return usage() };
                files.push((which, data));
            }
            "--report" => report = Some(PathBuf::from(v)),
            _ => return usage(),
        }
        i += 2;
    }
    let (Some(core), Some(image)) = (core, image) else { return usage() };
    let image = match std::fs::read(&image) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: {}: {e}", image.display());
            return ExitCode::from(2);
        }
    };
    let result = run(&core, &Options { self_delimiting, image, frames, settings, files, script });
    for c in &result.cases {
        println!("{} {:<4} {}", if c.passed { "pass" } else { "FAIL" }, c.id, c.name);
        for e in &c.evidence {
            println!("          {e}");
        }
        for i in &c.images {
            println!("          image: {} - {}", i.image, i.outcome.name());
        }
    }
    println!("{} {} {}: {}", result.id, result.version, result.abi, if result.passed() { "EmuSen v1 compliant on this image" } else { "not compliant" });
    if let Some(path) = report {
        if let Err(e) = std::fs::write(&path, result.json()) {
            eprintln!("error: {}: {e}", path.display());
            return ExitCode::from(2);
        }
    }
    if result.passed() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
