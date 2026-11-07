//! A part of the installation built and laid out for a front end: `out/` for the build output, `publish/` for a publish,
//! each with its cores.manifest. See EmuSen_RustPlatform.md §28.11.

use crate::compile::{self, Options};
use crate::files::{self, sha256_hex};
use crate::libraries::{Library, Role};
use crate::host;
use std::path::{Path, PathBuf};

/// What the platform part's one cargo command builds: the library and the shell's nano (EmuSen_DianaSh.md §7, §13.11).
const PLATFORM_PACKAGES: [&str; 2] = ["emusen-platform", "dianash"];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Cores,
    Platform,
}

/// The build output's cores.manifest and a publish's.
type Manifests = (String, String);

/// One staged file: where it comes from, its path in the folder, and whether a publish carries it.
struct Staged {
    from: PathBuf,
    at: String,
    published: bool,
}

pub fn run(part: Part, libs: &[Library], o: &Options, chdman: Option<PathBuf>, stage: &Path) -> Result<(), String> {
    let rid = host::portable_rid();
    let mine: Vec<&Library> = libs.iter().filter(|l| l.builds_for(&rid) && (l.role == Role::Platform) == (part == Part::Platform)).collect();
    let (staged, manifests) = match part {
        Part::Cores => cores(&mine, o)?,
        Part::Platform => (platform(&mine, o, chdman)?, None),
    };
    for (folder, publish) in [("out", false), ("publish", true)] {
        let root = stage.join(folder);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).map_err(|e| format!("{}: {e}", root.display()))?;
        for s in staged.iter().filter(|s| !publish || s.published) {
            files::stage(&s.from, &root.join(&s.at))?;
        }
        if let Some((out, published)) = &manifests {
            files::write_if_different(&stage.join(format!("{folder}.cores.manifest")), if publish { published } else { out })?;
            files::stage(&stage.join(format!("{folder}.cores.manifest")), &root.join("cores.manifest"))?;
        }
    }
    Ok(())
}

/// Every core, part, program and test library, built in the list's order, then the sidecars; and the two manifests.
fn cores(libs: &[&Library], o: &Options) -> Result<(Vec<Staged>, Option<Manifests>), String> {
    for lib in libs {
        compile::build(lib, o)?;
    }
    compile::sidecars(libs, o)?;
    let (mut staged, mut out, mut published) = (Vec::new(), String::new(), String::new());
    for lib in libs {
        let built = o.built(lib);
        if !built.is_file() {
            continue;
        }
        let name = built.file_name().unwrap().to_string_lossy().to_string();
        // A program is an executable beside the libraries, as the libretro proxy finds its runner - see EmuSen_LibretroHost.md §10d.
        let ships = lib.ships || lib.role == Role::Program;
        staged.push(Staged { from: built.clone(), at: name.clone(), published: ships });
        let sidecar = PathBuf::from(format!("{}.core.json", built.display()));
        if sidecar.is_file() {
            staged.push(Staged { from: sidecar, at: format!("{name}.core.json"), published: ships });
        }
        // A test library is no library of the installation; the build output lists one in development, a publish does not.
        if lib.role != Role::Program && lib.role != Role::Test {
            let line = format!("{}  {name}\n", sha256_hex(&built)?);
            out.push_str(&line);
            if lib.ships {
                published.push_str(&line);
            }
        }
    }
    Ok((staged, Some((out, published))))
}

/// The platform library and nano from one cargo command, nano's manifest, and chdman where the toolchain holds it.
fn platform(libs: &[&Library], o: &Options, chdman: Option<PathBuf>) -> Result<Vec<Staged>, String> {
    let Some(lib) = libs.first() else { return Ok(Vec::new()) };
    let dir = lib.folder(&o.repo);
    let target = o.target_root.join(&lib.name);
    let mut c = compile::command("cargo");
    c.args(["build", "--profile", &o.profile, "--manifest-path"]).arg(dir.join("Cargo.toml"));
    for p in PLATFORM_PACKAGES {
        c.args(["-p", p]);
    }
    c.arg("--target-dir").arg(&target).current_dir(&dir);
    compile::run(&mut c, "cargo build of the platform library")?;
    let rid = host::portable_rid();
    let mut staged = Vec::new();
    let library = target.join(&o.profile).join(host::library_file(&lib.name, &rid));
    if library.is_file() {
        staged.push(Staged { from: library, at: host::library_file(&lib.name, &rid), published: true });
    }
    let nano = target.join(&o.profile).join(host::program_file("nano", &rid));
    if nano.is_file() {
        staged.push(Staged { from: nano, at: format!("dianash/{}", host::program_file("nano", &rid)), published: true });
        staged.push(Staged { from: dir.join("dianash-program/nano.dianash.json"), at: "dianash/nano.dianash.json".into(), published: true });
    }
    // chdman, the one third-party program a published tree carries, built by Platform/library-tool/build-chdman.sh (EmuSen_GamePackages.md §4.9, §12.1).
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let chdman = chdman.unwrap_or_else(|| home.join(".cache/emusen/toolchains/chdman").join(&rid).join(host::program_file("chdman", &rid)));
    if chdman.is_file() {
        staged.push(Staged { at: chdman.file_name().unwrap().to_string_lossy().to_string(), from: chdman, published: true });
    }
    Ok(staged)
}
