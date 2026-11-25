//! `PowerModel`: bridges Eclipse Power dialog actions to session save and `power::Runner`, with a test dry-run seam.

use crate::power::{Action, Runner, SystemctlRunner};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString, QStringList};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        /// Test seam: records actions without invoking systemctl or minimizing the window.
        #[qproperty(bool, dry_run, cxx_name = "dryRun")]
        /// Test seam: when non-empty in dry-run mode, simulates runner failure with this message.
        #[qproperty(QString, fail_with, cxx_name = "failWith")]
        /// Ordered log of actions executed in dry-run mode ("save:keep", "save:close", "suspend", "reboot", "poweroff", "desktop").
        #[qproperty(QStringList, recorded, READ, NOTIFY)]
        /// Comma-joined dry-run log for single-string property reads in harness tests.
        #[qproperty(QString, recorded_log, cxx_name = "recordedLog", READ, NOTIFY)]
        /// Failure notice raised when saving or running a power action fails.
        #[qproperty(QString, problem, READ, NOTIFY)]
        type PowerModel = super::PowerModelRust;
    }

    extern "RustQt" {
        /// Starts a power action by its HomeCard key, saving first when a game is running.
        #[qinvokable]
        fn trigger(self: Pin<&mut PowerModel>, key: &QString, has_game: bool);

        /// Resumes the pending power action once the host finishes saving (empty error on success).
        #[qinvokable]
        fn saved(self: Pin<&mut PowerModel>, error: &QString);

        /// Clears recorded dry-run entries between test checks.
        #[qinvokable]
        #[cxx_name = "clearRecorded"]
        fn clear_recorded(self: Pin<&mut PowerModel>);

        /// Requests that the host save (and close when `close_session` is true) the active game, then call `saved(error)`.
        #[qsignal]
        #[cxx_name = "saveNeeded"]
        fn save_needed(self: Pin<&mut PowerModel>, close_session: bool);

        /// Requests that the host window minimize for "Back to the desktop".
        #[qsignal]
        #[cxx_name = "minimizeAsked"]
        fn minimize_asked(self: Pin<&mut PowerModel>);
    }
}

#[derive(Default)]
pub struct PowerModelRust {
    dry_run: bool,
    fail_with: QString,
    recorded: QStringList,
    recorded_log: QString,
    problem: QString,
    pending: Option<Action>,
}

struct DryRunner<'a> {
    fail_with: &'a str,
}

impl Runner for DryRunner<'_> {
    fn run(&self, _verb: &str) -> Result<(), String> {
        if self.fail_with.is_empty() { Ok(()) } else { Err(self.fail_with.to_owned()) }
    }
}

impl qobject::PowerModel {
    fn record(mut self: Pin<&mut Self>, entry: &str) {
        let mut items: QList<QString> = self.recorded.iter().cloned().collect();
        items.append(QString::from(entry));
        let joined = items.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(",");
        self.as_mut().rust_mut().recorded = QStringList::from(&items);
        self.as_mut().rust_mut().recorded_log = QString::from(joined.as_str());
        self.as_mut().recorded_changed();
        self.recorded_log_changed();
    }

    fn set_problem_text(mut self: Pin<&mut Self>, text: &str) {
        let value = QString::from(text);
        if self.problem != value {
            self.as_mut().rust_mut().problem = value;
            self.problem_changed();
        }
    }

    pub fn clear_recorded(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().recorded = QStringList::default();
        self.as_mut().rust_mut().recorded_log = QString::default();
        self.as_mut().recorded_changed();
        self.recorded_log_changed();
    }

    pub fn trigger(mut self: Pin<&mut Self>, key: &QString, has_game: bool) {
        let key_str = key.to_string();
        if key_str == crate::home_card::POWER_DESKTOP {
            if self.dry_run {
                self.record("desktop");
            } else {
                self.minimize_asked();
            }
            return;
        }
        let Some(action) = Action::from_key(&key_str) else { return };
        self.as_mut().set_problem_text("");
        self.as_mut().rust_mut().pending = Some(action);
        if has_game {
            self.save_needed(action.closes_session());
        } else {
            self.finish(action, None);
        }
    }

    pub fn saved(mut self: Pin<&mut Self>, error: &QString) {
        let Some(action) = self.as_mut().rust_mut().pending.take() else { return };
        if self.dry_run {
            self.as_mut().record(if action.closes_session() { "save:close" } else { "save:keep" });
        }
        let err = error.to_string();
        let save_err = (!err.is_empty()).then_some(err);
        self.finish(action, save_err);
    }

    fn finish(mut self: Pin<&mut Self>, action: Action, save_err: Option<String>) {
        self.as_mut().rust_mut().pending = None;
        let save_ok = save_err.is_none();
        let fail_with = self.fail_with.to_string();
        let dry = self.dry_run;
        let outcome = if dry {
            let runner = DryRunner { fail_with: &fail_with };
            crate::power::execute(action, &runner, |_| match save_err {
                Some(e) => Err(e),
                None => Ok(()),
            })
        } else {
            crate::power::execute(action, &SystemctlRunner, |_| match save_err {
                Some(e) => Err(e),
                None => Ok(()),
            })
        };
        if dry && save_ok {
            self.as_mut().record(action.verb());
        }
        if let Err(words) = outcome {
            self.set_problem_text(&words);
        }
    }
}
