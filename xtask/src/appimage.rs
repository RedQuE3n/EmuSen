//! `appimage`: the kit's gallery program bundled as an AppImage by linuxdeploy, its Qt plugin and appimagetool (with its pinned runtime), each
//! pinned in appimage-tools.tsv and checked against its sha256 before it runs; see EmuSen_Qt.md §9.2.

use std::fs;
use std::path::{Path, PathBuf};

use crate::compile::{command, run as run_command};
use crate::files::sha256_hex;
use crate::say;

/// One pinned tool: its file, the release it is, where it came from and its sha256.
struct Tool {
    file: String,
    release: String,
    url: String,
    sha256: String,
}

fn tools(repo: &Path) -> Result<Vec<Tool>, String> {
    let list = repo.join("xtask/appimage-tools.tsv");
    let text = fs::read_to_string(&list).map_err(|e| format!("{}: {e}", list.display()))?;
    text.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| match l.split('\t').collect::<Vec<_>>()[..] {
            [file, release, url, sha256] => Ok(Tool { file: file.into(), release: release.into(), url: url.into(), sha256: sha256.into() }),
            _ => Err(format!("{}: not four columns: {l}", list.display())),
        })
        .collect()
}

/// The distribution's patchelf, which `$PATCHELF` names: linuxdeploy's own (0.15.0) corrupts the RELR-relocated
/// libraries Fedora 44 ships (EmuSen_Qt.md §9.2), and linuxdeploy takes another from that variable.
const PATCHELF: &str = "patchelf";

/// Where a pinned tool is: patchelf where `$PATCHELF` points, the rest in the tools folder.
fn path_of(tool: &Tool, dir: &Path) -> Result<PathBuf, String> {
    if tool.file == PATCHELF {
        std::env::var_os("PATCHELF").map(PathBuf::from).ok_or_else(|| format!("PATCHELF names no patchelf; set it to {} ({})", tool.release, tool.url))
    } else {
        Ok(dir.join(&tool.file))
    }
}

/// Each tool is the file its pin names, or the bundle is refused before anything runs.
fn verify(tools: &[Tool], dir: &Path) -> Result<(), String> {
    for t in tools {
        let path = path_of(t, dir)?;
        let sum = sha256_hex(&path).map_err(|e| format!("{e}; fetch {} ({}) from {}", t.file, t.release, t.url))?;
        if sum != t.sha256 {
            return Err(format!("{} is not the pinned {} (sha256 {sum}, pinned {}); fetch it again from {}", path.display(), t.release, t.sha256, t.url));
        }
    }
    Ok(())
}

/// Builds the gallery in release, lays out `out/AppDir` with linuxdeploy and its Qt plugin, and packs it into
/// `out/kit-gallery-x86_64.AppImage` with appimagetool.
pub fn run(repo: &Path, tools_dir: &Path, out: &Path) -> Result<PathBuf, String> {
    let tools = tools(repo)?;
    verify(&tools, tools_dir)?;
    say::note(&format!("{} tools match their pins", tools.len()));
    let patchelf = tools.iter().find(|t| t.file == PATCHELF).ok_or("appimage-tools.tsv pins no patchelf")?;
    let patchelf = path_of(patchelf, tools_dir)?;
    let frontend = repo.join("Frontend");
    run_command(
        command("cargo").current_dir(&frontend).args(["build", "--release", "-p", "emusen-kit-gallery", "--bin", "kit-gallery"]),
        "building the gallery",
    )?;
    let appdir = out.join("AppDir");
    if appdir.exists() {
        fs::remove_dir_all(&appdir).map_err(|e| format!("{}: {e}", appdir.display()))?;
    }
    fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let bundle = frontend.join("kit-gallery/appimage");
    let qml = ["kit-gallery/qml", "kit-qt/qml", "kit-qt/style"].map(|d| frontend.join(d).display().to_string()).join(":");
    run_command(
        command(tools_dir.join("linuxdeploy-x86_64.AppImage"))
            .current_dir(out)
            .env("QMAKE", "/usr/bin/qmake6")
            .env("QML_SOURCES_PATHS", qml)
            .env("EXTRA_PLATFORM_PLUGINS", "libqwayland.so;libqoffscreen.so")
            .env("EXTRA_QT_MODULES", "waylandcompositor")
            // linuxdeploy's own strip predates the `.relr.dyn` sections Fedora's libraries carry, which come stripped already.
            .env("NO_STRIP", "1")
            .env("PATCHELF", &patchelf)
            .arg("--appdir")
            .arg(&appdir)
            .arg("--executable")
            .arg(frontend.join("target/release/kit-gallery"))
            .arg("--desktop-file")
            .arg(bundle.join("kit-gallery.desktop"))
            .arg("--icon-file")
            .arg(bundle.join("kit-gallery.svg"))
            .args(["--plugin", "qt"]),
        "linuxdeploy",
    )?;
    let image = out.join("kit-gallery-x86_64.AppImage");
    run_command(
        command(tools_dir.join("appimagetool-x86_64.AppImage"))
            .current_dir(out)
            .arg("--no-appstream")
            // The pinned runtime, so appimagetool downloads nothing (EmuSen_Qt.md §9.2).
            .arg("--runtime-file")
            .arg(tools_dir.join("runtime-x86_64"))
            .arg(&appdir)
            .arg(&image),
        "appimagetool",
    )?;
    Ok(image)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_pins_read_and_a_changed_tool_is_refused() {
        let tools = super::tools(&crate::repo()).unwrap();
        assert_eq!(tools.len(), 5);
        assert!(tools.iter().all(|t| t.sha256.len() == 64 && t.sha256.bytes().all(|b| b.is_ascii_hexdigit())));
        let dir = std::env::temp_dir().join(format!("xtask-appimage-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let tools: Vec<_> = tools.into_iter().filter(|t| t.file != super::PATCHELF).collect();
        for t in &tools {
            std::fs::write(dir.join(&t.file), b"not the release").unwrap();
        }
        let said = super::verify(&tools, &dir).unwrap_err();
        assert!(said.contains("is not the pinned") && said.contains(&tools[0].url), "{said}");
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(super::verify(&tools, &dir).unwrap_err().contains("fetch"), "a missing tool names where it comes from");
    }
}
