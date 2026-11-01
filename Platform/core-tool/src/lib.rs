//! `emusen-core-tool`, Cores' tool (EmuSen_Cores.md §1): libretro's catalogue and index, the installed cores under
//! `/Cores/libretro`, and the jobs that list, describe and fetch them, run on `emusen-tool-job` as the library tool's are
//! (§6). Its words go out as JSON lines; the `cores` command and the Cores window are its faces.

pub mod catalogue;
pub mod index;
pub mod info;
pub mod job;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

/// The systems a core runs, from its `.info`'s `systemid` and `database` list: `emusen_systems::for_libretro`'s shape,
/// handed in by the caller until that function lands (EmuSen_Cores.md §4.2).
pub type Placement = Arc<dyn Fn(&str, &[&str]) -> Vec<String> + Send + Sync>;

/// Where the catalogue and cores come from: the buildbot (EmuSen_Cores.md §5.1), or a test's own server.
#[derive(Clone, Debug)]
pub struct Source {
    /// `https://buildbot.libretro.com`, no trailing slash.
    pub base: String,
    /// The buildbot's folder for this platform, `linux/x86_64`.
    pub platform: &'static str,
    /// A test's own roots and address, as `emusen_net::Request` takes them; empty and none otherwise.
    pub test_roots: Vec<emusen_net::CertificateDer<'static>>,
    pub test_connect: Option<SocketAddr>,
    pub offline: bool,
}

/// The buildbot, the only source (EmuSen_Cores.md §11.3).
pub const BUILDBOT: &str = "https://buildbot.libretro.com";

/// The buildbot's folder for the platform this was built for, where its listing has been read; none elsewhere, so a
/// platform is added only once its listing is (read 2026-10-31: `nightly/linux/x86_64/latest/`).
pub const fn platform() -> Option<&'static str> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) { Some("linux/x86_64") } else { None }
}

impl Source {
    /// The buildbot for this platform, none where its listing is not known.
    pub fn buildbot() -> Option<Source> {
        platform().map(|platform| Source { base: BUILDBOT.into(), platform, test_roots: Vec::new(), test_connect: None, offline: false })
    }

    pub fn info_url(&self) -> String {
        format!("{}/assets/frontend/info.zip", self.base)
    }

    pub fn index_url(&self) -> String {
        format!("{}/nightly/{}/latest/.index-extended", self.base, self.platform)
    }

    /// A core zip's address, the file a line of the index names.
    pub fn zip_url(&self, file: &str) -> String {
        format!("{}/nightly/{}/latest/{file}", self.base, self.platform)
    }
}

/// What a job works on.
#[derive(Clone)]
pub struct Context {
    /// The player's `/Cores` folder (`DataStore.Cores`).
    pub cores: PathBuf,
    pub source: Option<Source>,
    pub placement: Option<Placement>,
}
