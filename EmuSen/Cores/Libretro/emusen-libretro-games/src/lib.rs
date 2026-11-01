//! A sandbox player home for real libretro cores on the Rust runtime (EmuSen_LibretroHost.md §10l): the installation
//! folder a build would ship, the core tool's context over the home's `/Cores`, its jobs run to their end, and the
//! frame timings a run reports.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use emusen_core_entry::{built_name, sha256_file, HOST_LIBRARY, PROXY_LIBRARY};
use emusen_core_host::trust::MANIFEST;

/// The home's folders: its `/Cores`, the data and config folders the tree is pointed at, and the installation.
pub struct Home {
    pub root: PathBuf,
    pub cores: PathBuf,
    pub data: PathBuf,
    pub config: PathBuf,
    pub installation: PathBuf,
}

impl Home {
    pub fn at(root: &Path) -> Home {
        Home { root: root.to_path_buf(), cores: root.join("Cores"), data: root.join("Home"), config: root.join("Config"), installation: root.join("lib") }
    }

    /// The tree pointed at this home, so the runtime's settings and saves stay inside it.
    pub fn point_tree(&self) {
        use emusen_galaxia::tree::{Override, tree_mut};
        let mut t = tree_mut();
        t.set_override(Override::Data, Some(self.data.to_string_lossy().into_owned()));
        t.set_override(Override::Config, Some(self.config.to_string_lossy().into_owned()));
    }
}

/// The installation as a build lays it: the proxy and libretro host from `built`, the runner, and the manifest vouching for the two libraries.
pub fn install_libraries(home: &Home, built: &Path, runner: &Path) -> Result<String, String> {
    std::fs::create_dir_all(&home.installation).map_err(|e| format!("{}: {e}", home.installation.display()))?;
    let mut manifest = String::new();
    for library in [PROXY_LIBRARY, HOST_LIBRARY] {
        let name = built_name(library);
        let to = home.installation.join(&name);
        std::fs::copy(built.join(&name), &to).map_err(|e| format!("{}: {e}", built.join(&name).display()))?;
        manifest.push_str(&format!("{}  {name}\n", sha256_file(&to).map_err(|e| e.to_string())?));
    }
    std::fs::copy(runner, home.installation.join("emusen-core-runner")).map_err(|e| format!("{}: {e}", runner.display()))?;
    std::fs::write(home.installation.join(MANIFEST), &manifest).map_err(|e| e.to_string())?;
    Ok(manifest)
}

/// The core tool's context as the platform library makes it, over the home's `/Cores` and installation.
pub fn tool_context(home: &Home) -> emusen_core_tool::Context {
    let placement: emusen_core_tool::Placement = Arc::new(|systemid, databases| emusen_systems::for_libretro(systemid, databases).into_iter().map(|s| s.id.clone()).collect());
    let hosted = emusen_systems::libretro_cores::hosted().iter().map(|c| emusen_core_tool::Hosted { stem: c.stem.clone(), systems: c.systems.clone() }).collect();
    emusen_core_tool::Context {
        cores: home.cores.clone(),
        source: emusen_core_tool::Source::buildbot(),
        placement: Some(placement),
        installation: Some(home.installation.clone()),
        notes: emusen_core_tool::notes::parse(emusen_core_tool::notes::SHIPPED),
        hosted: Some(hosted),
    }
}

/// A core request run to its end, each event line handed to `line`; the words of a refused event, if any.
pub fn run_job(request: &str, ctx: emusen_core_tool::Context, line: &mut dyn FnMut(&str)) -> Result<(), String> {
    let job = emusen_core_tool::job::start(request, ctx)?;
    let mut refused = None;
    while !job.finished() {
        if let Some(l) = job.peek(Duration::from_millis(500)) {
            job.take();
            if l.contains("\"event\":\"refused\"") {
                refused = Some(l.clone());
            }
            line(&l);
        }
    }
    refused.map_or(Ok(()), Err)
}

/// A key's quoted value in an `.info` file, as `display_version = "v2.9.138"` writes it.
pub fn info_value(info: &str, key: &str) -> Option<String> {
    info.lines().find_map(|l| {
        let (k, v) = l.split_once('=')?;
        (k.trim() == key).then(|| v.trim().trim_matches('"').to_string())
    })
}

/// Frame times in seconds summarised: frames, frames per second over the whole span, and the median and worst frame in ms.
#[derive(Debug, PartialEq)]
pub struct Timing {
    pub frames: usize,
    pub fps: f64,
    pub median_ms: f64,
    pub worst_ms: f64,
}

pub fn timing(times: &[f64]) -> Option<Timing> {
    if times.is_empty() {
        return None;
    }
    let total: f64 = times.iter().sum();
    let mut sorted = times.to_vec();
    sorted.sort_by(f64::total_cmp);
    Some(Timing { frames: times.len(), fps: times.len() as f64 / total, median_ms: sorted[sorted.len() / 2] * 1e3, worst_ms: sorted[sorted.len() - 1] * 1e3 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_is_frames_over_the_whole_span_with_the_median_and_worst_frame() {
        let t = timing(&[0.010, 0.020, 0.030, 0.040]).unwrap();
        assert_eq!((t.frames, t.fps), (4, 40.0));
        assert!((t.median_ms - 30.0).abs() < 1e-9 && (t.worst_ms - 40.0).abs() < 1e-9);
        assert_eq!(timing(&[]), None);
    }

    #[test]
    fn the_installation_holds_both_libraries_the_runner_and_a_manifest_of_their_hashes() {
        let root = std::env::temp_dir().join(format!("libretro-games-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let built = root.join("built");
        std::fs::create_dir_all(&built).unwrap();
        std::fs::write(built.join(built_name(PROXY_LIBRARY)), b"proxy").unwrap();
        std::fs::write(built.join(built_name(HOST_LIBRARY)), b"host").unwrap();
        std::fs::write(built.join("runner"), b"runner").unwrap();
        let home = Home::at(&root.join("home"));
        let manifest = install_libraries(&home, &built, &built.join("runner")).unwrap();
        let lines: Vec<(&str, &str)> = manifest.lines().map(|l| l.split_once("  ").unwrap()).collect();
        assert_eq!(lines.iter().map(|l| l.1).collect::<Vec<_>>(), [built_name(PROXY_LIBRARY), built_name(HOST_LIBRARY)]);
        for (hash, name) in lines {
            assert_eq!(hash, sha256_file(&home.installation.join(name)).unwrap());
        }
        assert_eq!(std::fs::read_to_string(home.installation.join(MANIFEST)).unwrap(), manifest);
        assert_eq!(std::fs::read(home.installation.join("emusen-core-runner")).unwrap(), b"runner");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_info_value_is_read_by_its_key_alone() {
        let info = "display_name = \"Sony - PlayStation 2 (PCEE2)\"\ndisplay_version = \"v2.9.138\"\nversion = \"x\"\n";
        assert_eq!(info_value(info, "display_version").as_deref(), Some("v2.9.138"));
        assert_eq!(info_value(info, "version").as_deref(), Some("x"));
        assert_eq!(info_value(info, "license"), None);
    }
}
