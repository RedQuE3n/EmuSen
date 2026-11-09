//! `theme-audit`, theme-check's loaded step (EmuSen_Qt.md §3.7, man theme-check): a theme package's screens placed offscreen
//! through the theme host on a synthetic library and held to the kit's fit and reach audits; one `file:line kind words` a finding.

mod audit;
mod catalogue;
mod crowd;
mod shape;
mod states;
mod world;

use std::path::{Path, PathBuf};

// The two QML modules the scene and every theme resolve, linked in by name.
extern crate emusen_theme_host;
extern crate emusen_theme_qt;

const USAGE: &str = "Usage: theme-audit [--gpu] [--state DIR] [--states LIST] [--catalogue FILE] [--section KEY] <folder>\n       theme-audit --list-states\n  Exit 1 for a finding, 0 for none, 2 when it could not run. See man theme-check.";

/// What the command line asks for.
struct Options {
    folder: String,
    state: Option<PathBuf>,
    catalogue: Option<PathBuf>,
    section: Option<String>,
    /// The states a run records, by name, `all` for every one; the default alone when not given.
    states: String,
    gpu: bool,
}

/// The options, or the words for the usage.
fn options(args: &[String]) -> Result<Options, String> {
    let (mut folder, mut state, mut catalogue, mut section, mut gpu) = (None, None, None, None, false);
    let mut states = states::DEFAULT.to_owned();
    let mut words = args.iter().skip(1);
    while let Some(word) = words.next() {
        match word.as_str() {
            "--state" => state = Some(PathBuf::from(words.next().ok_or(format!("theme-audit: --state needs a folder\n{USAGE}"))?)),
            "--catalogue" => catalogue = Some(PathBuf::from(words.next().ok_or(format!("theme-audit: --catalogue needs a file\n{USAGE}"))?)),
            "--gpu" => gpu = true,
            "--states" => states = words.next().ok_or(format!("theme-audit: --states needs a list of states, or all\n{USAGE}"))?.to_owned(),
            "--section" => section = Some(words.next().ok_or(format!("theme-audit: --section needs a key\n{USAGE}"))?.to_owned()),
            other if other.starts_with("--") => return Err(format!("theme-audit: {other} is not an option\n{USAGE}")),
            other if folder.is_none() => folder = Some(other.to_owned()),
            _ => return Err(format!("theme-audit: one folder at a time\n{USAGE}")),
        }
    }
    Ok(Options { folder: folder.ok_or(format!("theme-audit: a folder is needed\n{USAGE}"))?, state, catalogue, section, states, gpu })
}

/// The catalogue's name (its file's stem, which labels its states) and its entries.
fn catalogue(file: &Path) -> Result<(String, Vec<catalogue::Entry>), String> {
    let text = std::fs::read_to_string(file).map_err(|e| format!("theme-audit: {}: {e}", file.display()))?;
    let entries = catalogue::parse(&text).map_err(|e| format!("theme-audit: {}: {e}", file.display()))?;
    Ok((file.file_stem().map_or(String::new(), |s| s.to_string_lossy().into_owned()), entries))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--list-states") {
        print!("{}", states::listed());
        return;
    }
    let options = options(&args);
    // The held clock replaces Qt's animation driver, which a threaded render loop installs as its own (EmuSen_Kit.md §7.36).
    std::env::set_var("QSG_RENDER_LOOP", "basic");
    // On the GPU, Vulkan in a compositor of the run's own, its scale 1 whatever the desktop's (man theme-check).
    let compositor = options.as_ref().is_ok_and(|o| o.gpu).then(|| {
        std::env::set_var("QT_QUICK_BACKEND", "rhi");
        std::env::set_var("QSG_RHI_BACKEND", "vulkan");
        emusen_qt_test::compositor::Compositor::start(3840, 2160)
    });
    let run = |o: Options| {
        let listed = o.catalogue.as_deref().map(catalogue).transpose()?;
        let asked = audit::Asked {
            state: o.state.as_deref(),
            catalogue: listed.as_ref().map(|(name, e)| (name.as_str(), e.as_slice())),
            section: o.section.as_deref(),
            states: states::chosen(&o.states)?,
        };
        audit::run(&PathBuf::from(&o.folder), &o.folder, &asked)
    };
    let status = match options.and_then(run) {
        Ok(lines) => {
            lines.iter().for_each(|l| println!("{l}"));
            i32::from(!lines.is_empty())
        }
        Err(words) => {
            eprintln!("{words}");
            2
        }
    };
    drop(compositor);
    std::process::exit(status);
}
