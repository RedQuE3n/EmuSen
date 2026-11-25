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
        /// Test seam: when non-empty in dry-run mode, simulates a suspended-game save failure with this message.
        #[qproperty(QString, fail_save_with, cxx_name = "failSaveWith")]
        /// Ordered log of actions executed in dry-run mode ("save:keep", "save:close", "suspend", "reboot", "poweroff", "desktop", "quit").
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

        /// Completes Quit EmuSen after saving; emits `quitAsked` on success or raises a failure notice on error.
        #[qinvokable]
        #[cxx_name = "finishQuit"]
        fn finish_quit(self: Pin<&mut PowerModel>, error: &QString);

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

        /// Requests that the host exit the application after Quit EmuSen saves cleanly.
        #[qsignal]
        #[cxx_name = "quitAsked"]
        fn quit_asked(self: Pin<&mut PowerModel>);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pending {
    Desktop,
    System(Action),
}

#[derive(Default)]
pub struct PowerModelRust {
    dry_run: bool,
    fail_with: QString,
    fail_save_with: QString,
    recorded: QStringList,
    recorded_log: QString,
    problem: QString,
    pending: Option<Pending>,
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

    fn effective_save_error(&self, error: &QString) -> Option<String> {
        let err = error.to_string();
        if !err.is_empty() {
            return Some(err);
        }
        if self.dry_run {
            let simulated = self.fail_save_with.to_string();
            if !simulated.is_empty() {
                return Some(simulated);
            }
        }
        None
    }

    pub fn trigger(mut self: Pin<&mut Self>, key: &QString, has_game: bool) {
        let key_str = key.to_string();
        if key_str == crate::home_card::POWER_DESKTOP {
            self.as_mut().set_problem_text("");
            self.as_mut().rust_mut().pending = Some(Pending::Desktop);
            if has_game {
                self.save_needed(false);
            } else {
                self.finish_desktop(None);
            }
            return;
        }
        let Some(action) = Action::from_key(&key_str) else { return };
        self.as_mut().set_problem_text("");
        self.as_mut().rust_mut().pending = Some(Pending::System(action));
        if has_game {
            self.save_needed(action.closes_session());
        } else {
            self.finish_system(action, None);
        }
    }

    pub fn saved(mut self: Pin<&mut Self>, error: &QString) {
        let Some(pending) = self.as_mut().rust_mut().pending.take() else { return };
        let close_session = matches!(pending, Pending::System(a) if a.closes_session());
        if self.dry_run {
            self.as_mut().record(if close_session { "save:close" } else { "save:keep" });
        }
        let save_err = self.effective_save_error(error);
        match pending {
            Pending::Desktop => self.finish_desktop(save_err),
            Pending::System(action) => self.finish_system(action, save_err),
        }
    }

    pub fn finish_quit(mut self: Pin<&mut Self>, error: &QString) {
        self.as_mut().set_problem_text("");
        if let Some(why) = self.effective_save_error(error) {
            self.set_problem_text(&crate::power::quit_cancelled(&why));
            return;
        }
        if self.dry_run {
            self.record("quit");
        } else {
            self.quit_asked();
        }
    }

    fn finish_desktop(mut self: Pin<&mut Self>, save_err: Option<String>) {
        self.as_mut().rust_mut().pending = None;
        let mut minimized = false;
        let outcome = crate::power::execute_desktop(
            || match save_err {
                Some(e) => Err(e),
                None => Ok(()),
            },
            || minimized = true,
        );
        if minimized {
            if self.dry_run {
                self.as_mut().record("desktop");
            } else {
                self.as_mut().minimize_asked();
            }
        }
        if let Err(words) = outcome {
            self.set_problem_text(&words);
        }
    }

    fn finish_system(mut self: Pin<&mut Self>, action: Action, save_err: Option<String>) {
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
