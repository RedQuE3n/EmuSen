// `emusen-core-runner --target LIB [--require-abi N] [policy...]`: one v1 library in a confined process of its own,
// served on descriptor 3 (EmuSen_LibretroHost.md §3).

use std::os::fd::FromRawFd;
use std::os::unix::net::UnixStream;
use std::path::Path;

use emusen_core_broker::runner::{Served, serve};
use emusen_core_broker::sandbox::{Policy, confine, private_shm};
use emusen_core_broker::wire;

/// The kernel's Landlock ABI, or 0 where it has none.
fn landlock_abi() -> i64 {
    const LANDLOCK_CREATE_RULESET_VERSION: libc::c_ulong = 1;
    let v = unsafe { libc::syscall(libc::SYS_landlock_create_ruleset, std::ptr::null::<libc::c_void>(), 0usize, LANDLOCK_CREATE_RULESET_VERSION) };
    v.max(0)
}

/// The process ends when the proxy's end of the socket closes, whatever the core is doing.
fn watch_parent() {
    std::thread::spawn(|| loop {
        let mut p = libc::pollfd { fd: 3, events: libc::POLLRDHUP, revents: 0 };
        let n = unsafe { libc::poll(&mut p, 1, -1) };
        if n > 0 && p.revents & (libc::POLLRDHUP | libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0 {
            unsafe { libc::_exit(0) };
        }
    });
}

fn value<'a>(rest: &'a [String], flag: &str) -> Option<&'a str> {
    rest.iter().position(|a| a == flag).and_then(|i| rest.get(i + 1)).map(String::as_str)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (policy, rest) = Policy::from_args(&args);
    let mut io = unsafe { UnixStream::from_raw_fd(3) };
    let shm = if policy.private_shm { private_shm() } else { Ok(()) };
    watch_parent();
    let refuse = |io: &mut UnixStream, why: String| {
        let _ = wire::send_reply(io, -1, 0, why.as_bytes());
        std::process::exit(1);
    };
    let Some(target) = value(&rest, "--target") else { return refuse(&mut io, "the runner was given no library".into()) };
    let need: i64 = value(&rest, "--require-abi").and_then(|v| v.parse().ok()).unwrap_or(1);
    let abi = landlock_abi();
    if abi < need {
        return refuse(&mut io, format!("this kernel's Landlock ABI is {abi} and the runner needs {need}; a hosted core never runs unconfined"));
    }
    if let Err(why) = shm {
        return refuse(&mut io, why);
    }
    let confined = match confine(&policy) {
        Ok(words) => words,
        Err(why) => return refuse(&mut io, why),
    };
    let lib = match emusen_core_host::library::Library::load(Path::new(target)) {
        Ok(lib) => lib,
        Err(e) => return refuse(&mut io, format!("{target} does not load: {e:?}")),
    };
    if wire::send_reply(&mut io, 0, abi, confined.as_bytes()).is_err() {
        return;
    }
    let mut served = Served { x: lib.exports(), m: std::ptr::null_mut(), images: Vec::new(), done: false };
    let _ = serve(&mut served, &mut io);
}
