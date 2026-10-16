//! The shell's `/`: Galaxia's home, and paths resolved chroot-style inside it, as `DianaShSandbox.TryResolve`
//! resolves them. A walled garden, not a security boundary: see `man hier` and EmuSen_DianaSh.md §12.1.

use crate::dotnet;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::tree;

/// Where `/` is: the host's root when it named one, else Galaxia's home, read on every call so that a moved override takes effect at once.
#[derive(Clone, Debug, Default)]
pub struct Sandbox {
    pub root: Option<String>,
}

impl Sandbox {
    pub fn root(&self) -> String {
        self.root.clone().unwrap_or_else(|| tree::tree().home())
    }

    /// A path as the shell takes it: a leading `/` is the sandbox's root unless the path is already a real one inside it.
    pub fn resolve(&self, requested: &str) -> (bool, String) {
        let root = self.root();
        let looks_rooted = requested.starts_with('/') || requested.starts_with('\\');
        if looks_rooted {
            let as_is = path::full_path(requested);
            if self.within(&as_is, &root).is_some() {
                return (true, as_is);
            }
        }
        let base = if looks_rooted { root.clone() } else { dotnet::current_directory() };
        let mut effective = if looks_rooted { requested.trim_start_matches(['/', '\\']) } else { requested };
        if effective.is_empty() {
            effective = ".";
        }
        let candidate = path::full_path_from(Style::HOST, effective, &base);
        let ok = self.within(&candidate, &root).is_some();
        (ok, candidate)
    }

    /// The root with every link along it followed, where that differs from the root as spelled (§13.11).
    fn resolved(root: &str) -> Option<String> {
        let real = std::fs::canonicalize(root).ok()?.to_string_lossy().into_owned();
        // Windows answers in the extended form, which no path the shell compares carries.
        let real = match real.strip_prefix(r"\\?\") {
            Some(rest) if rest.as_bytes().get(1) == Some(&b':') => rest.to_string(),
            _ => real,
        };
        (real != root).then_some(real)
    }

    /// Whether a real path lies under the root as spelled, or as the system resolves its links: a working directory
    /// reached through a link reads resolved (`/var` is `/private/var` on macOS), and is the same directory.
    fn within(&self, candidate: &str, root: &str) -> Option<String> {
        if inside(candidate, root) {
            return Some(root.to_string());
        }
        Self::resolved(root).filter(|real| inside(candidate, real))
    }

    /// A real path back to the shell's `/` form; one outside the root is given as it is.
    pub fn display(&self, real: &str) -> String {
        let root = self.root();
        let Some(root) = self.within(real, &root) else { return real.to_string() };
        if dotnet::same(real, &root) {
            return "/".to_string();
        }
        let rest: String = real.chars().skip(root.chars().count() + 1).collect();
        format!("/{}", rest.replace(Style::HOST.separator(), "/"))
    }

    /// The words a path above the root is refused with.
    pub fn outside(&self, requested: &str) -> String {
        format!("'{requested}' is outside the project sandbox ({})", self.root())
    }
}

/// Whether paths that differ only by case name one file: the file systems Windows and macOS make by default; not Linux's.
pub const IGNORES_CASE: bool = cfg!(any(windows, target_os = "macos"));

/// `IsInsideRoot`: the root itself or below it, ignoring case only where the file system does (§16 Q15).
pub fn inside(candidate: &str, root: &str) -> bool {
    let below = format!("{root}{}", Style::HOST.separator());
    if IGNORES_CASE {
        dotnet::same(candidate, root) || dotnet::starts_with_ignore_case(candidate, &below)
    } else {
        candidate == root || candidate.starts_with(&below)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn a_working_directory_reached_through_a_link_to_the_root_is_inside_it() {
        let temp = std::env::temp_dir().join(format!("dianash-link-root-{}", std::process::id()));
        let real = temp.join("real");
        std::fs::create_dir_all(real.join("home/sub")).unwrap();
        std::os::unix::fs::symlink(&real, temp.join("link")).unwrap();
        let spelled = temp.join("link/home").to_string_lossy().into_owned();
        let resolved = std::fs::canonicalize(real.join("home")).unwrap().to_string_lossy().into_owned();
        let sandbox = Sandbox { root: Some(spelled.clone()) };
        // The system names a directory entered through the link by its resolved path, as getcwd does.
        assert_eq!(sandbox.resolve(&format!("{resolved}/sub/a.txt")), (true, format!("{resolved}/sub/a.txt")));
        assert_eq!(sandbox.display(&format!("{resolved}/sub")), "/sub");
        assert_eq!(sandbox.display(&resolved), "/");
        assert_eq!(sandbox.resolve(&format!("{spelled}/sub")), (true, format!("{spelled}/sub")));
        // A sibling of the resolved root is still outside.
        let sibling = std::fs::canonicalize(&real).unwrap().join("other").to_string_lossy().into_owned();
        assert!(!sandbox.within(&sibling, &spelled).is_some());
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn a_rooted_path_is_the_sandbox_root_unless_it_is_already_inside() {
        let sandbox = Sandbox { root: Some("/r/home".to_string()) };
        assert_eq!(sandbox.resolve("/Logs/a.txt"), (true, "/r/home/Logs/a.txt".to_string()));
        assert_eq!(sandbox.resolve("/r/home/Logs"), (true, "/r/home/Logs".to_string()));
        assert_eq!(sandbox.resolve("/"), (true, "/r/home".to_string()));
        assert_eq!(sandbox.resolve("/../x"), (false, "/r/x".to_string()));
        assert_eq!(sandbox.display("/r/home/Logs/a.txt"), "/Logs/a.txt");
        assert_eq!(sandbox.display("/r/home"), "/");
    }

    #[test]
    fn a_sibling_differing_from_the_root_only_by_case_is_outside_it() {
        assert_eq!(inside("/r/HOME/x", "/r/home"), IGNORES_CASE);
        assert_eq!(inside("/r/HOME", "/r/home"), IGNORES_CASE);
        assert!(inside("/r/home/x", "/r/home"));
        assert!(!inside("/r/homex", "/r/home"));
        // Refused as it is, the path is read chroot-style, inside the root, and never reaches the sibling.
        let sandbox = Sandbox { root: Some("/r/home".to_string()) };
        let expected = if IGNORES_CASE { "/r/HOME/secret" } else { "/r/home/r/HOME/secret" };
        assert_eq!(sandbox.resolve("/r/HOME/secret"), (true, expected.to_string()));
    }
}
