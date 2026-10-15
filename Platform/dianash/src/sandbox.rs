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
            if inside(&as_is, &root) {
                return (true, as_is);
            }
        }
        let base = if looks_rooted { root.clone() } else { dotnet::current_directory() };
        let mut effective = if looks_rooted { requested.trim_start_matches(['/', '\\']) } else { requested };
        if effective.is_empty() {
            effective = ".";
        }
        let candidate = path::full_path_from(Style::HOST, effective, &base);
        let ok = inside(&candidate, &root);
        (ok, candidate)
    }

    /// A real path back to the shell's `/` form; one outside the root is given as it is.
    pub fn display(&self, real: &str) -> String {
        let root = self.root();
        if dotnet::same(real, &root) {
            return "/".to_string();
        }
        if inside(real, &root) {
            let rest: String = real.chars().skip(root.chars().count() + 1).collect();
            return format!("/{}", rest.replace(Style::HOST.separator(), "/"));
        }
        real.to_string()
    }

    /// The words a path above the root is refused with.
    pub fn outside(&self, requested: &str) -> String {
        format!("'{requested}' is outside the project sandbox ({})", self.root())
    }
}

/// `IsInsideRoot`: the root itself or below it, compared ignoring case as the C# does (§16 Q15, kept through S3).
pub fn inside(candidate: &str, root: &str) -> bool {
    let separator = Style::HOST.separator();
    dotnet::same(candidate, root) || dotnet::starts_with_ignore_case(candidate, &format!("{root}{separator}"))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

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
    fn the_comparison_ignores_case_as_the_csharp_does() {
        assert!(inside("/r/HOME/x", "/r/home"));
        assert!(!inside("/r/homex", "/r/home"));
    }
}
