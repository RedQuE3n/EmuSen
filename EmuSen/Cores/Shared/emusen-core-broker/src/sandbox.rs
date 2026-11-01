//! The runner's confinement on Linux: `no_new_privs` and a Landlock ruleset of the paths a core may read, run, write
//! and drive, and no network; a kernel or a path that cannot be confined fails the runner, never runs it unconfined
//! (EmuSen_LibretroHost.md §3.3, decided 2026-10-31).

use std::path::PathBuf;

/// What one runner may reach.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Policy {
    pub read_exec: Vec<PathBuf>,
    pub read: Vec<PathBuf>,
    pub read_write: Vec<PathBuf>,
    /// Device nodes read, written and driven by ioctl: the GPU's render nodes.
    pub devices: Vec<PathBuf>,
}

impl Policy {
    /// The policy as runner arguments.
    pub fn to_args(&self) -> Vec<String> {
        let mut a = Vec::new();
        for (flag, list) in [("--read-exec", &self.read_exec), ("--read", &self.read), ("--read-write", &self.read_write), ("--device", &self.devices)] {
            for p in list {
                a.push(flag.to_owned());
                a.push(p.to_string_lossy().into_owned());
            }
        }
        a
    }

    /// The policy from runner arguments, and the arguments it did not take.
    pub fn from_args(args: &[String]) -> (Policy, Vec<String>) {
        let mut p = Policy::default();
        let mut rest = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let list = match args[i].as_str() {
                "--read-exec" => Some(&mut p.read_exec),
                "--read" => Some(&mut p.read),
                "--read-write" => Some(&mut p.read_write),
                "--device" => Some(&mut p.devices),
                _ => None,
            };
            match (list, args.get(i + 1)) {
                (Some(l), Some(v)) => {
                    l.push(PathBuf::from(v));
                    i += 2;
                }
                _ => {
                    rest.push(args[i].clone());
                    i += 1;
                }
            }
        }
        (p, rest)
    }
}

/// The ruleset applied to this process and its threads to come; an error names what could not be confined.
#[cfg(target_os = "linux")]
pub fn confine(p: &Policy) -> Result<String, String> {
    use landlock::{ABI, Access, AccessFs, AccessNet, CompatLevel, Compatible, PathBeneath, PathFd, Ruleset, RulesetAttr, RulesetCreatedAttr, RulesetStatus};
    let abi = ABI::V5;
    let fs_all = AccessFs::from_all(abi);
    let read = AccessFs::from_read(abi);
    let read_exec = read | AccessFs::Execute;
    let write = AccessFs::from_write(abi);
    let device = AccessFs::ReadFile | AccessFs::WriteFile | AccessFs::IoctlDev;
    let mut ruleset = Ruleset::default()
        .set_compatibility(CompatLevel::BestEffort)
        .handle_access(fs_all)
        .map_err(|e| format!("Landlock: {e}"))?
        .handle_access(AccessNet::from_all(abi))
        .map_err(|e| format!("Landlock: {e}"))?
        .create()
        .map_err(|e| format!("Landlock: {e}"))?;
    for (list, access) in [(&p.read_exec, read_exec), (&p.read, read), (&p.read_write, read | write), (&p.devices, device)] {
        for path in list {
            let fd = PathFd::new(path).map_err(|e| format!("{}: {e}", path.display()))?;
            ruleset = ruleset.add_rule(PathBeneath::new(fd, access)).map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    let status = ruleset.no_new_privs(true).restrict_self().map_err(|e| format!("Landlock: {e}"))?;
    match status.ruleset {
        RulesetStatus::NotEnforced => Err("this kernel does not enforce Landlock, and a hosted core never runs unconfined".into()),
        RulesetStatus::PartiallyEnforced => Ok(format!("Landlock partly enforced (ABI {:?}): the rules this kernel knows hold", status.landlock)),
        RulesetStatus::FullyEnforced => Ok("Landlock fully enforced".into()),
    }
}

#[cfg(not(target_os = "linux"))]
pub fn confine(_p: &Policy) -> Result<String, String> {
    Ok("confined by the process boundary only on this platform".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_policy_crosses_as_arguments() {
        let p = Policy { read_exec: vec!["/usr".into()], read: vec!["/etc".into()], read_write: vec!["/tmp/x".into()], devices: vec!["/dev/dri".into()] };
        let mut args = p.to_args();
        args.push("--target".into());
        args.push("lib.so".into());
        let (back, rest) = Policy::from_args(&args);
        assert_eq!(back, p);
        assert_eq!(rest, ["--target", "lib.so"]);
    }
}
