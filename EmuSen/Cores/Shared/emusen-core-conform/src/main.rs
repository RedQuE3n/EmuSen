//! `emusen-core-conform --core LIB --image FILE [--self-delimiting] [--frames N] [--input SCRIPT] [--settings KEY=VALUE]... [--file WHICH=PATH]...
//! [--report REPORT.json]` runs the core suite and exits 0 when every case passes; with `--images DIR|LIST [--from DIR]
//! [--jobs N] [--report-dir DIR]` it runs a corpus, the cases that do not depend on the image once and the rest for
//! every image; `emusen-core-conform --sidecar LIB [--development]` writes the library's sidecar. See
//! EmuSen_CoreAPI.md §12.3, §21 and §28.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use emusen_core_conform::{Case, Options, Script, corpus, run, sidecar};

fn usage() -> ExitCode {
    eprintln!("usage: emusen-core-conform --core LIB --image FILE [--self-delimiting] [--frames N] [--input SCRIPT] [--settings KEY=VALUE]... [--file WHICH=PATH]... [--report REPORT.json]");
    eprintln!("       emusen-core-conform --core LIB --images DIR|LIST [--from DIR] [--image FILE] [--jobs N] [--report-dir DIR] [the options above]");
    eprintln!("       emusen-core-conform --sidecar LIB [--development]");
    ExitCode::from(2)
}

fn print_cases(cases: &[Case]) {
    for c in cases {
        println!("{} {:<4} {}", if c.passed { "pass" } else { "FAIL" }, c.id, c.name);
        for e in &c.evidence {
            println!("          {e}");
        }
        for i in &c.images {
            println!("          image: {} - {}", i.image, i.outcome.name());
        }
    }
}

/// A report's file for the image numbered `n`: its place in the corpus, and its name with nothing a file system minds.
fn report_file(n: usize, name: &str) -> String {
    let stem: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') { c } else { '_' }).collect();
    format!("{:04}-{stem}.json", n + 1)
}

/// The corpus form: the core's cases on the reference image, then a line for each image as it ends and the count.
fn run_corpus(core: &Path, opts: Options, reference: Option<PathBuf>, images: &Path, from: Option<&Path>, jobs: Option<usize>, report: Option<&Path>, report_dir: Option<&Path>) -> ExitCode {
    let list = match corpus::image_list(images, from) {
        Ok(list) if !list.is_empty() => list,
        Ok(_) => {
            eprintln!("error: {} names no image", images.display());
            return ExitCode::from(2);
        }
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    let (reference_name, reference_path) = match reference {
        Some(p) => (p.display().to_string(), p),
        None => list[0].clone(),
    };
    let image = match std::fs::read(&reference_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: {}: {e}", reference_path.display());
            return ExitCode::from(2);
        }
    };
    if let Some(dir) = report_dir {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("error: {}: {e}", dir.display());
            return ExitCode::from(2);
        }
    }
    let jobs = jobs.unwrap_or_else(corpus::default_jobs);
    let mut done = 0;
    let total = list.len();
    let result = corpus::run(core, &Options { image, ..opts }, &reference_name, &list, jobs, Duration::from_secs(corpus::IMAGE_DEADLINE_SECONDS), &mut |i| {
        done += 1;
        let failed = i.failed();
        println!("{} [{done}/{total}] {}{}", if failed.is_empty() { "pass" } else { "FAIL" }, i.name, if failed.is_empty() { String::new() } else { format!(": {}", failed.join(" ")) });
        for c in i.report.cases.iter().filter(|c| !c.passed) {
            for e in c.evidence.iter().take(2) {
                println!("          {} {e}", c.id);
            }
        }
    });
    println!("the core's cases, on {}:", result.reference);
    print_cases(&result.core.cases);
    let passing = result.images.iter().filter(|i| i.report.passed()).count();
    println!(
        "{} {} {}: the core's cases {}; {passing} of {} images pass; {} at a time, {:.1} s",
        result.core.id,
        result.core.version,
        result.core.abi,
        if result.core.passed() { "pass" } else { "FAIL" },
        result.images.len(),
        result.jobs,
        result.seconds
    );
    let mut files = Vec::new();
    if let Some(dir) = report_dir {
        for (n, i) in result.images.iter().enumerate() {
            let file = report_file(n, &i.name);
            if let Err(e) = std::fs::write(dir.join(&file), i.report.json()) {
                eprintln!("error: {}: {e}", dir.join(&file).display());
                return ExitCode::from(2);
            }
            files.push(file);
        }
    }
    let summary = result.json(&files);
    for path in report.map(Path::to_owned).into_iter().chain(report_dir.map(|d| d.join("summary.json"))) {
        if let Err(e) = std::fs::write(&path, &summary) {
            eprintln!("error: {}: {e}", path.display());
            return ExitCode::from(2);
        }
    }
    if result.passed() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
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
    let (mut images, mut from, mut jobs, mut report_dir) = (None, None, None, None);
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
                let development = args[i + 2..].iter().any(|a| a == "--development");
                return match sidecar::write(&PathBuf::from(&v), development) {
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
            "--images" => images = Some(PathBuf::from(v)),
            "--from" => from = Some(PathBuf::from(v)),
            "--report-dir" => report_dir = Some(PathBuf::from(v)),
            "--jobs" => match v.parse::<usize>() {
                Ok(n) if n > 0 => jobs = Some(n),
                _ => return usage(),
            },
            _ => return usage(),
        }
        i += 2;
    }
    if let (Some(core), Some(images)) = (&core, &images) {
        let opts = Options { self_delimiting, image: Vec::new(), frames, settings, files, script };
        return run_corpus(core, opts, image, images, from.as_deref(), jobs, report.as_deref(), report_dir.as_deref());
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
    print_cases(&result.cases);
    println!("{} {} {}: {}", result.id, result.version, result.abi, if result.passed() { "EmuSen v1 compliant on this image" } else { "not compliant" });
    if let Some(path) = report {
        if let Err(e) = std::fs::write(&path, result.json()) {
            eprintln!("error: {}: {e}", path.display());
            return ExitCode::from(2);
        }
    }
    if result.passed() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
