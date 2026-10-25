//! A request of the library tool run over a published tree, outside any frontend, for measuring a batch as
//! EmuSen_GamePackages.md §12.5 did: the tree found from <program-dir> as the program finds it, the library read from
//! that tree's settings, the paths printed and checked before anything is written, then every event printed as a JSON
//! line. `EMUSEN_CHDMAN` names chdman where the tree carries none beside it; the pin is checked either way.
//!
//! Usage: cargo run --release --example run_tool -- <program-dir> <ls|convert|check> [<path>...] [--fast] [-j <n>]

use emusen_galaxia::models::{APP_SETTINGS_ID, MODELS};
use emusen_galaxia::{config, setting_folders, tree};
use emusen_library_tool::{job, Context};
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(base), Some(op)) = (args.first(), args.get(1)) else {
        eprintln!("usage: run_tool <program-dir> <ls|convert|check> [<path>...] [--fast] [-j <n>]");
        std::process::exit(2);
    };
    let (mut paths, mut fast, mut threads, mut rest) = (Vec::new(), false, 0usize, args[2..].iter());
    while let Some(a) = rest.next() {
        match a.as_str() {
            "--fast" => fast = true,
            "-j" => threads = rest.next().and_then(|n| n.parse().ok()).unwrap_or(0),
            p => paths.push(format!("\"{}\"", p.replace('\\', "\\\\").replace('"', "\\\""))),
        }
    }
    tree::init(base, None);
    let tree = tree::tree().clone();
    let (settings, said) = config::load_or_new(&tree, &MODELS[APP_SETTINGS_ID as usize], None, "appsettings.json");
    if let Some(words) = said {
        eprintln!("{words}");
    }
    let library = setting_folders::checked(settings.get("RomDirectory").and_then(|v| v.as_str()), &tree::user_home()).path;
    let chdman = std::env::var("EMUSEN_CHDMAN").unwrap_or_else(|_| format!("{base}/chdman"));
    let ctx = Context::pinned(tree, library, chdman);
    eprintln!("library    {:?}\ncompressed {}\npackages   {}\nunpacked   {}\nlogs       {}\nchdman     {}", ctx.library, ctx.compressed(), ctx.packages(), ctx.unpacked(), ctx.logs(), ctx.chdman);
    if let Err(words) = ctx.checked_library() {
        eprintln!("refused: {words}");
        std::process::exit(1);
    }
    let request = format!("{{\"op\": \"{op}\", \"paths\": [{}], \"fast\": {fast}, \"threads\": {threads}}}", paths.join(", "));
    let j = job::start(&request, ctx).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(2)
    });
    while !j.finished() {
        if let Some(line) = j.peek(Duration::from_millis(500)) {
            j.take();
            println!("{line}");
        }
    }
}
