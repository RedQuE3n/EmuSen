//! A temporary directory for a test, removed when the test ends.

use crate::dotnet_path::Style;
use std::sync::atomic::{AtomicU32, Ordering};

static NEXT: AtomicU32 = AtomicU32::new(0);

pub struct TempDir(String);

impl TempDir {
    pub fn new(label: &str) -> TempDir {
        let name = format!("emusen-galaxia-{label}-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed));
        let dir = std::env::temp_dir().join(name);
        std::fs::create_dir_all(&dir).unwrap();
        // The canonical spelling, so a symbolic link in the temporary directory's own path cannot differ between two answers.
        TempDir(std::fs::canonicalize(&dir).unwrap().to_string_lossy().into_owned())
    }

    pub fn path(&self) -> &str {
        &self.0
    }

    /// A path under the directory, given with `/` between its parts.
    pub fn join(&self, relative: &str) -> String {
        let separator = Style::HOST.separator().to_string();
        format!("{}{separator}{}", self.0, relative.replace('/', &separator))
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

static ERROR_LOG: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Held by a test that depends on the error log's once-in-a-process pruning.
pub fn error_log_tests() -> std::sync::MutexGuard<'static, ()> {
    ERROR_LOG.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
