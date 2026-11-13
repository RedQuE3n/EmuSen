//! The recorded-response ScreenScraper the platform's scraping test answers from, built from this commit's `Platform/`
//! as its `recorded_scraper` example and run beside the test: the scraper's test seam to give it, the paths it was
//! asked, and the made-up developer file a run needs (EmuSen_Qt.md §1.1.3).

use crate::{platform_dir, PROFILE};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};

/// The developer file's name in Galaxia's config directory, as the scraper's credentials read it.
pub const DEVELOPER_FILE: &str = "screenscraper-developer.json";

/// The server, running while this is held; it ends when this is dropped.
pub struct RecordedScraper {
    child: Child,
    /// The scraper's `test` settings member, `{"connect", "roots"}`, as JSON.
    pub test: String,
    asked: Arc<Mutex<Vec<String>>>,
}

fn built() -> &'static Path {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT.get_or_init(|| {
        let dir = platform_dir();
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let status = Command::new(cargo)
            .current_dir(&dir)
            .args(["build", "--quiet", "--profile", PROFILE, "-p", "emusen-platform", "--example", "recorded_scraper"])
            .env_remove("CARGO_TARGET_DIR")
            .status()
            .expect("cargo runs");
        assert!(status.success(), "the recorded scraper did not build: {status}");
        dir.join("target").join(PROFILE).join("examples").join("recorded_scraper")
    })
}

impl RecordedScraper {
    /// The server started, its first line read.
    pub fn start() -> RecordedScraper {
        let mut child = Command::new(built()).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("the recorded scraper starts");
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let test = lines.next().and_then(Result::ok).expect("the recorded scraper's settings line");
        let asked = Arc::new(Mutex::new(Vec::new()));
        let kept = asked.clone();
        std::thread::spawn(move || {
            for line in lines.map_while(Result::ok) {
                if let Some(path) = line.strip_prefix("asked ") {
                    kept.lock().unwrap_or_else(|p| p.into_inner()).push(path.to_owned());
                }
            }
        });
        RecordedScraper { child, test, asked }
    }

    /// The paths asked so far.
    pub fn asked(&self) -> Vec<String> {
        self.asked.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }

    /// A made-up developer's file written into `config`, Galaxia's config directory, so a run is allowed to start.
    pub fn developer(config: &Path) {
        std::fs::create_dir_all(config).unwrap();
        std::fs::write(config.join(DEVELOPER_FILE), br#"{"devid":"dummydev","devpassword":"dummydevpw","softname":"EmuSenTest"}"#).unwrap();
    }
}

impl Drop for RecordedScraper {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
