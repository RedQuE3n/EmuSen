//! A compositor of the test's own, KWin's virtual one, for a scene Qt draws with Vulkan in a real window that nothing on the
//! desk shows (EmuSen_Qt.md §8.1, tier 2; the picture's measurements of §1.8.1 ran in the same).

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

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
        let child = Command::new("kwin_wayland")
            .args(["--virtual", "--no-lockscreen", "--no-global-shortcuts", "--socket", &socket])
            .args(["--width", &width.to_string(), "--height", &height.to_string()])
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .env("KWIN_COMPOSE", "O2")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("kwin_wayland");
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
}

impl Drop for Compositor {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.socket);
        let _ = std::fs::remove_file(self.socket.with_extension("lock"));
    }
}
