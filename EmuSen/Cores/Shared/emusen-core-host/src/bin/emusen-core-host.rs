//! The core host's tool: what discovery lists in the given cores directories, as JSON, and with `--open` what opening
//! each core as a game's open would decide (EmuSen_Backend.md §27.2); the player's directories and approvals' file as
//! §29.40 adds them.

use std::path::PathBuf;
use std::process::ExitCode;

use emusen_core_host::discovery::DEVELOPMENT_VARIABLE;
use emusen_core_host::trust::read_approvals;
use emusen_core_host::{Discovery, Trust, listing};

const USAGE: &str = "usage: emusen-core-host list [--development] [--open] [--approve SHA256]... [--approvals FILE] [--player DIR]... DIR...";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("list") {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let (mut development, mut open) = (std::env::var(DEVELOPMENT_VARIABLE).as_deref() == Ok("1"), false);
    let (mut approvals, mut dirs, mut player) = (Vec::new(), Vec::new(), Vec::new());
    while let Some(a) = args.next() {
        match a.as_str() {
            "--development" => development = true,
            "--open" => open = true,
            "--approve" | "--approvals" | "--player" => {
                let Some(v) = args.next() else {
                    eprintln!("{USAGE}");
                    return ExitCode::from(2);
                };
                match a.as_str() {
                    "--approve" => approvals.push(v),
                    "--player" => player.push(PathBuf::from(v)),
                    _ => match read_approvals(std::path::Path::new(&v)) {
                        Ok(read) => approvals.extend(read),
                        Err(why) => eprintln!("{why}"),
                    },
                }
            }
            _ => dirs.push(PathBuf::from(a)),
        }
    }
    if dirs.is_empty() {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let discovery = Discovery::scan_with_player(&dirs, &player, development);
    let trust = Trust::new(&dirs, &approvals).with_player(&player);
    if open {
        for c in discovery.found() {
            let _ = c.open(&trust);
        }
    }
    println!("{}", listing::both(&discovery, &trust));
    ExitCode::SUCCESS
}
