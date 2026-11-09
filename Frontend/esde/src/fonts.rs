//! ES-DE's own font where the player already has it: the face a theme's text and help take when the theme names none,
//! found in an installed ES-DE or the system's fonts, never shipped with the program (EmuSen_Qt.md §4.8.7).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// ES-DE's default face, the one file looked for.
pub const FILE: &str = "Akrobat-Regular.ttf";
/// Its family as fontconfig names it.
const FAMILY: &str = "Akrobat";

/// The first folder of `roots` holding the face, as a path to the file.
pub fn locate(roots: &[PathBuf]) -> Option<PathBuf> {
    roots.iter().map(|r| r.join(FILE)).find(|p| p.is_file())
}

/// The folders an installed ES-DE keeps its fonts in, as its user guide gives its installation folder (`/usr/share/es-de`
/// for the distribution's package) and its application data folder (`~/ES-DE`), the Flatpak's and a running AppImage's
/// mount, in that order.
pub fn roots(home: &Path) -> Vec<PathBuf> {
    let fonts = |base: PathBuf| base.join("resources/fonts");
    let mut out = vec![
        fonts(PathBuf::from("/usr/share/es-de")),
        fonts(PathBuf::from("/usr/local/share/es-de")),
        fonts(home.join("ES-DE")),
        fonts(PathBuf::from("/var/lib/flatpak/app/org.es_de.frontend/current/active/files/share/es-de")),
        fonts(home.join(".local/share/flatpak/app/org.es_de.frontend/current/active/files/share/es-de")),
    ];
    let mut mounts: Vec<PathBuf> = std::fs::read_dir(std::env::temp_dir())
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with(".mount_ES-DE")))
        .collect();
    mounts.sort();
    out.extend(mounts.into_iter().map(|m| fonts(m.join("usr/share/es-de"))));
    out
}

/// The face as fontconfig knows it: the regular style of the family, or none when the family is not installed.
fn fontconfig() -> Option<PathBuf> {
    let out = std::process::Command::new("fc-list").arg(format!(":family={FAMILY}:style=Regular")).arg("file").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut files: Vec<&str> = text.lines().map(|l| l.trim().trim_end_matches(':')).filter(|l| !l.is_empty()).collect();
    files.sort();
    files.first().map(PathBuf::from).filter(|p| p.is_file())
}

/// ES-DE's default face where the player has it, looked for once: an installed ES-DE's, then fontconfig's; none, and
/// the painter draws such text in the kit's body face.
pub fn default_font() -> Option<&'static Path> {
    static FOUND: OnceLock<Option<PathBuf>> = OnceLock::new();
    FOUND
        .get_or_init(|| {
            let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
            locate(&roots(&home)).or_else(fontconfig)
        })
        .as_deref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_root_holding_es_de_s_face_is_the_one_found() {
        let dir = std::env::temp_dir().join(format!("esde-fonts-{}", std::process::id()));
        let (none, first, second) = (dir.join("a"), dir.join("b"), dir.join("c"));
        for d in [&none, &first, &second] {
            std::fs::create_dir_all(d).unwrap();
        }
        std::fs::write(first.join(FILE), b"face").unwrap();
        std::fs::write(second.join(FILE), b"face").unwrap();
        assert_eq!(locate(&[none.clone(), first.clone(), second]), Some(first.join(FILE)));
        assert_eq!(locate(&[none]), None);
        assert!(roots(Path::new("/home/p")).contains(&PathBuf::from("/home/p/ES-DE/resources/fonts")));
        assert_eq!(roots(Path::new("/home/p"))[0], PathBuf::from("/usr/share/es-de/resources/fonts"));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
