//! The bridge to the sandbox's C++ shells (cpp/sandbox.h): a theme's engine with its interceptor and its refusing network, and
//! the policy they ask, which is Rust's (`policy.rs`).

use crate::policy::{Kind, Policy};
use cxx_qt_lib::QString;

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
        /// A sandboxed engine whose interceptor asks `policy`, which must outlive it.
        fn sandbox_new(policy: &Policy) -> UniquePtr<Sandbox>;
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
        /// Whether the interceptor must read `url`'s text for `judge`.
        fn policy_reads(policy: &Policy, url: &QString, local: &QString, kind: i32) -> bool;
        /// Empty when the engine may fetch `url`; else the words of the refusal.
        fn policy_judge(policy: &Policy, url: &QString, local: &QString, kind: i32, text: &QString) -> QString;
    }
}

fn policy_reads(policy: &Policy, url: &QString, local: &QString, kind: i32) -> bool {
    policy.reads(&url.to_string(), &local.to_string(), Kind::from_qt(kind))
}

fn policy_judge(policy: &Policy, url: &QString, local: &QString, kind: i32, text: &QString) -> QString {
    QString::from(policy.judge(&url.to_string(), &local.to_string(), Kind::from_qt(kind), &text.to_string()).as_str())
}
