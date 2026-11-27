//! A compositor of the test's own, KWin's virtual one, for a scene Qt draws with Vulkan in a real window that nothing on the
//! desk shows (EmuSen_Qt.md §8.1, tier 2; the picture's measurements of §1.8.1 ran in the same).

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Once;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DEFAULT_WATCHDOG_SECONDS: u64 = 300;
static WATCHDOG_PID: AtomicU32 = AtomicU32::new(0);
static WATCHDOG_DEADLINE_SECS: AtomicU64 = AtomicU64::new(u64::MAX);
static WATCHDOG_ONCE: Once = Once::new();

#[cfg(target_os = "linux")]
const PR_SET_PDEATHSIG: std::ffi::c_int = 1;
#[cfg(target_os = "linux")]
const LINUX_SIGTERM: std::ffi::c_ulong = 15;

#[cfg(target_os = "linux")]
extern "C" {
    fn prctl(option: std::ffi::c_int, arg2: std::ffi::c_ulong, arg3: std::ffi::c_ulong, arg4: std::ffi::c_ulong, arg5: std::ffi::c_ulong) -> std::ffi::c_int;
}

fn now_epoch_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn compute_deadline_secs(now_secs: u64, seconds: u64) -> u64 {
    now_secs.saturating_add(seconds.max(1))
}

fn stop_pid(pid: u32) {
    if pid != 0 {
        let _ = Command::new("kill").args(["-TERM", &pid.to_string()]).status();
    }
}

fn arm_watchdog(pid: u32, seconds: u64) {
    WATCHDOG_DEADLINE_SECS.store(compute_deadline_secs(now_epoch_secs(), seconds), Ordering::Release);
    WATCHDOG_PID.store(pid, Ordering::Release);
    WATCHDOG_ONCE.call_once(|| {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let p = WATCHDOG_PID.swap(0, Ordering::AcqRel);
            stop_pid(p);
            prev(info);
            std::process::abort();
        }));
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(500));
            let p = WATCHDOG_PID.load(Ordering::Acquire);
            if p != 0 && now_epoch_secs() >= WATCHDOG_DEADLINE_SECS.load(Ordering::Acquire) {
                use std::io::Write;
                let _ = writeln!(std::io::stderr(), "the run exceeded its compositor deadline; stopping kwin_wayland (pid {p})");
                stop_pid(p);
                std::process::abort();
            }
        });
    });
}

/// The running compositor, stopped when dropped; Qt's Wayland platform points at it from `start` on.
pub struct Compositor {
    child: Child,
    socket: PathBuf,
}

impl Compositor {
    /// `kwin_wayland --virtual` at a size, on a socket named for this process, and Qt's platform set to it; one per process.
    pub fn start(width: i32, height: i32) -> Compositor {
        let socket = format!("emusen-test-{}", std::process::id());
        let runtime = PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR, where Wayland's sockets are"));
        let at = runtime.join(&socket);
        let _ = std::fs::remove_file(&at);
        let mut cmd = Command::new("kwin_wayland");
        cmd.args(["--virtual", "--no-lockscreen", "--no-global-shortcuts", "--socket", &socket])
            .args(["--width", &width.to_string(), "--height", &height.to_string()])
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .env("KWIN_COMPOSE", "O2")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(target_os = "linux")]
        unsafe {
            use std::os::unix::process::CommandExt;
            cmd.pre_exec(|| {
                // Bind kwin_wayland lifetime to this thread so SIGKILL/OOM cannot orphan the compositor.
                if prctl(PR_SET_PDEATHSIG, LINUX_SIGTERM, 0, 0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = cmd.spawn().expect("kwin_wayland");
        arm_watchdog(child.id(), DEFAULT_WATCHDOG_SECONDS);
        let mut compositor = Compositor { child, socket: at.clone() };
        let start = Instant::now();
        while !at.exists() {
            assert!(compositor.child.try_wait().ok().flatten().is_none(), "kwin_wayland stopped before its socket was made");
            assert!(start.elapsed() < Duration::from_secs(10), "kwin_wayland made no socket in 10 s");
            std::thread::sleep(Duration::from_millis(20));
        }
        std::env::set_var("WAYLAND_DISPLAY", &socket);
        std::env::set_var("QT_QPA_PLATFORM", "wayland");
        compositor
    }
}

impl Compositor {
    /// The compositor's process, for a watchdog that must stop it when the test cannot unwind.
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Updates the default watchdog deadline to `seconds` from now without spawning extra threads.
    pub fn watch(&self, seconds: u64) {
        arm_watchdog(self.child.id(), seconds);
    }
}

impl Drop for Compositor {
    fn drop(&mut self) {
        let _ = WATCHDOG_PID.compare_exchange(self.child.id(), 0, Ordering::AcqRel, Ordering::Acquire);
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.socket);
        let _ = std::fs::remove_file(self.socket.with_extension("lock"));
    }
}

#[cfg(test)]
mod tests {
    use super::compute_deadline_secs;

    #[test]
    fn watchdog_deadline_clamps_zero_and_saturates_overflow() {
        assert_eq!(compute_deadline_secs(100, 0), 101);
        assert_eq!(compute_deadline_secs(100, 300), 400);
        assert_eq!(compute_deadline_secs(u64::MAX - 5, 60), u64::MAX);
    }
}
