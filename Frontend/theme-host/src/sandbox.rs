//! The bridge to the sandbox's C++ shells (cpp/sandbox.h): a theme's engine with its interceptor and its refusing network, and
//! the policy they ask, which is Rust's (`policy.rs`).

use crate::policy::{Kind, Policy};
use cxx_qt_lib::QString;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

const THUMB_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "webp", "gif", "bmp"];

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ThumbKey {
    path: PathBuf,
    len: u64,
    modified_ns: u128,
}

fn thumb_key(path: &Path) -> Option<ThumbKey> {
    let meta = std::fs::metadata(path).ok()?;
    let modified_ns = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_nanos());
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    Some(ThumbKey { path: canonical, len: meta.len(), modified_ns })
}

#[derive(Default)]
struct ThumbStore {
    next: u64,
    by_key: HashMap<ThumbKey, String>,
    by_id: HashMap<String, PathBuf>,
}

/// Thread-safe registry of host-approved thumbnail image files served via `image://thumb/<id>`.
#[derive(Clone, Default)]
pub struct ThumbHandle(Arc<Mutex<ThumbStore>>);

impl ThumbHandle {
    /// Registers an existing regular image file and returns its `image://thumb/<id>` URL, rotating on change.
    pub fn allow(&self, path: &Path) -> Option<String> {
        let ext = path.extension().and_then(|e| e.to_str())?;
        if !THUMB_EXTENSIONS.iter().any(|allowed| ext.eq_ignore_ascii_case(allowed)) || !path.is_file() {
            return None;
        }
        let key = thumb_key(path)?;
        let mut store = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(id) = store.by_key.get(&key) {
            return Some(format!("image://thumb/{id}"));
        }
        let obsolete: Vec<(ThumbKey, String)> = store.by_key.iter().filter(|(k, _)| k.path == key.path).map(|(k, v)| (k.clone(), v.clone())).collect();
        for (old_key, old_id) in obsolete {
            store.by_key.remove(&old_key);
            store.by_id.remove(&old_id);
        }
        store.next += 1;
        let id = store.next.to_string();
        store.by_id.insert(id.clone(), key.path.clone());
        store.by_key.insert(key, id.clone());
        Some(format!("image://thumb/{id}"))
    }

    /// Resolves a registered thumbnail id to its approved filesystem path.
    pub fn resolve(&self, id: &str) -> Option<PathBuf> {
        let clean = id.split(['?', '#']).next().unwrap_or(id);
        self.0.lock().unwrap_or_else(PoisonError::into_inner).by_id.get(clean).cloned()
    }
}

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!(<QtQuick/QQuickItem>);
        type QQuickItem;
        include!("cxx-qt-lib/qobject.h");
        type QObject = cxx_qt::QObject;

        include!("sandbox.h");
        /// One theme's engine, its interceptor and its network factory, and the screens created in it.
        type Sandbox;
        /// A sandboxed engine whose interceptor asks `policy`, which must outlive it, and serves approved thumbnails.
        fn sandbox_new(policy: &Policy, thumbs: Box<ThumbHandle>) -> UniquePtr<Sandbox>;
        /// Creates the screen at `url` in the theme's engine as a child of `slot`, filling it; false with the reasons in the log.
        unsafe fn sandbox_place(sandbox: Pin<&mut Sandbox>, url: &QString, slot: *mut QQuickItem) -> bool;
        /// The engine's warnings and the sandbox's refusals since the last call, a line each.
        fn sandbox_take_log(sandbox: Pin<&mut Sandbox>) -> QString;
        /// The engine, for a check that it is gone once the sandbox is.
        fn sandbox_engine(sandbox: &Sandbox) -> *mut QObject;
        /// A file's text through Qt, so a folder in the resources reads as one on disk; false where it cannot be read.
        fn sandbox_read(url: &QString, text: &mut QString) -> bool;
        /// Qt's own QML folder, where its modules are installed.
        fn sandbox_qt_qml() -> QString;
        /// A watch on an object: whether it still lives.
        type Watch;
        unsafe fn watch_new(object: *mut QObject) -> UniquePtr<Watch>;
        fn watch_alive(watch: &Watch) -> bool;
    }

    extern "Rust" {
        type Policy;
        type ThumbHandle;
        /// Whether the interceptor must read `url`'s text for `judge`.
        fn policy_reads(policy: &Policy, url: &QString, local: &QString, kind: i32) -> bool;
        /// Empty when the engine may fetch `url`; else the words of the refusal.
        fn policy_judge(policy: &Policy, url: &QString, local: &QString, kind: i32, text: &QString) -> QString;
        /// Resolves an approved thumbnail id to its disk path for `ThumbImageProvider`.
        #[cxx_name = "thumbPath"]
        fn thumb_path(thumbs: &ThumbHandle, id: &QString) -> QString;
    }
}

fn thumb_path(thumbs: &ThumbHandle, id: &QString) -> QString {
    thumbs.resolve(&id.to_string()).map_or_else(QString::default, |p| QString::from(p.to_string_lossy().as_ref()))
}

fn policy_reads(policy: &Policy, url: &QString, local: &QString, kind: i32) -> bool {
    policy.reads(&url.to_string(), &local.to_string(), Kind::from_qt(kind))
}

fn policy_judge(policy: &Policy, url: &QString, local: &QString, kind: i32, text: &QString) -> QString {
    QString::from(policy.judge(&url.to_string(), &local.to_string(), Kind::from_qt(kind), &text.to_string()).as_str())
}
