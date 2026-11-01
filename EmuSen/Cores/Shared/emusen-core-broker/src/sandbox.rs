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
    /// A `/dev/shm` of the runner's own, in a user and mount namespace, for a core that maps shared memory by name.
    pub private_shm: bool,
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
        if self.private_shm {
            a.push("--private-shm".to_owned());
        }
        a
    }

    /// The policy from runner arguments, and the arguments it did not take.
    pub fn from_args(args: &[String]) -> (Policy, Vec<String>) {
        let mut p = Policy::default();
        let mut rest = Vec::new();
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--private-shm" {
                p.private_shm = true;
                i += 1;
                continue;
            }
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

/// A `/dev/shm` no other process sees: a user namespace mapping the runner's own ids, a mount namespace, and a tmpfs
/// over `/dev/shm`. Called while the process has one thread, as `unshare` requires.
#[cfg(target_os = "linux")]
pub fn private_shm() -> Result<(), String> {
    let (uid, gid) = unsafe { (libc::getuid(), libc::getgid()) };
    if unsafe { libc::unshare(libc::CLONE_NEWUSER | libc::CLONE_NEWNS) } != 0 {
        return Err(format!("a private /dev/shm needs a user namespace: {}", std::io::Error::last_os_error()));
    }
    let write = |path: &str, text: String| std::fs::write(path, text).map_err(|e| format!("{path}: {e}"));
    write("/proc/self/setgroups", "deny".into())?;
    write("/proc/self/uid_map", format!("{uid} {uid} 1"))?;
    write("/proc/self/gid_map", format!("{gid} {gid} 1"))?;
    let none = std::ptr::null::<libc::c_char>();
    if unsafe { libc::mount(none, c"/".as_ptr(), none, libc::MS_REC | libc::MS_PRIVATE, std::ptr::null()) } != 0 {
        return Err(format!("making the mounts private: {}", std::io::Error::last_os_error()));
    }
    if unsafe { libc::mount(c"tmpfs".as_ptr(), c"/dev/shm".as_ptr(), c"tmpfs".as_ptr(), libc::MS_NOSUID | libc::MS_NODEV, c"mode=0700".as_ptr() as *const libc::c_void) } != 0 {
        return Err(format!("a tmpfs over /dev/shm: {}", std::io::Error::last_os_error()));
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn private_shm() -> Result<(), String> {
    Err("a private /dev/shm is Linux's".into())
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
    // ReadDir too: a Vulkan driver lists /dev/dri to find its render node (EmuSen_LibretroHost.md §10f).
    let device = AccessFs::ReadFile | AccessFs::WriteFile | AccessFs::IoctlDev | AccessFs::ReadDir;
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
        let p = Policy { read_exec: vec!["/usr".into()], read: vec!["/etc".into()], read_write: vec!["/tmp/x".into()], devices: vec!["/dev/dri".into()], private_shm: true };
        let mut args = p.to_args();
        args.push("--target".into());
        args.push("lib.so".into());
        let (back, rest) = Policy::from_args(&args);
        assert_eq!(back, p);
        assert_eq!(rest, ["--target", "lib.so"]);
    }
}
