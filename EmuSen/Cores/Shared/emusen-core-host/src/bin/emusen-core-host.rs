//! The core host's tool: what discovery lists in the given cores directories, as JSON, and with `--open` what opening
//! each core as a game's open would decide (EmuSen_Backend.md §27.2).

use std::path::PathBuf;
use std::process::ExitCode;

use emusen_core_host::discovery::DEVELOPMENT_VARIABLE;
use emusen_core_host::{Discovery, Trust, listing};

const USAGE: &str = "usage: emusen-core-host list [--development] [--open] [--approve SHA256]... DIR...";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("list") {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let (mut development, mut open) = (std::env::var(DEVELOPMENT_VARIABLE).as_deref() == Ok("1"), false);
    let (mut approvals, mut dirs) = (Vec::new(), Vec::new());
    while let Some(a) = args.next() {
        match a.as_str() {
            "--development" => development = true,
            "--open" => open = true,
            "--approve" => match args.next() {
                Some(h) => approvals.push(h),
                None => {
                    eprintln!("{USAGE}");
                    return ExitCode::from(2);
                }
            },
            _ => dirs.push(PathBuf::from(a)),
        }
    }
    if dirs.is_empty() {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let discovery = Discovery::scan(&dirs, development);
    if open {
        let trust = Trust::new(&dirs, &approvals);
        for c in discovery.found() {
            let _ = c.open(&trust);
        }
    }
    println!("{}", listing::both(&discovery));
    ExitCode::SUCCESS
}
