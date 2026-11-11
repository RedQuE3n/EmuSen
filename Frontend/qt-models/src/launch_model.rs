//! `LaunchModel`: the questions before a game opens, on Qt (launch.rs; EmuSen_Qt.md §7.7): `prepare` reads the route,
//! the cores and the firmware off the interface thread, a question is put up where one is due, and `ready` names the
//! game once every question is answered; B on the firmware question gives the game up.

use crate::help::Screen;
use crate::launch::{self, Awaiting, FirmwareAnswer};
use crate::question::{back_index, Question};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QList, QString, QStringList};
use emusen_crystal_client::shapes::FirmwareNeed;
use emusen_crystal_client::{Api, Runtime};
use emusen_platform_client::galaxia::consts::DIR_FIRMWARE;
use emusen_platform_client::Platform;

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
        #[qproperty(QString, platform_library, cxx_name = "platformLibrary")]
        /// The runtime's cores folder, as the game's; empty for the installation's own.
        #[qproperty(QString, cores)]
        #[qproperty(bool, asking, READ, NOTIFY)]
        #[qproperty(QString, title, READ, NOTIFY)]
        #[qproperty(QStringList, sentences, READ, NOTIFY)]
        #[qproperty(QString, caption, READ, NOTIFY)]
        #[qproperty(QStringList, choices, READ, NOTIFY)]
        #[qproperty(i32, back_index, cxx_name = "backIndex", READ, NOTIFY)]
        /// The help bar's screen (HelpModel's): B answers Not Now, or goes back.
        #[qproperty(i32, help, READ, NOTIFY)]
        /// What happened, said once the game is on its way: approved, opened on another engine, or what could not be checked.
        #[qproperty(QString, notice, READ, NOTIFY)]
        /// Whether that notice says something was not done.
        #[qproperty(bool, notice_attention, cxx_name = "noticeAttention", READ, NOTIFY)]
        type LaunchModel = super::LaunchModelRust;
    }

    extern "RustQt" {
        /// The game by its path, asked about before it opens.
        #[qinvokable]
        fn prepare(self: Pin<&mut LaunchModel>, path: &QString);
        /// The question answered by the choice's index; -1 for B with no answer.
        #[qinvokable]
        fn answer(self: Pin<&mut LaunchModel>, index: i32);
        /// Every question answered: the game opens.
        #[qsignal]
        fn ready(self: Pin<&mut LaunchModel>, path: QString);
        /// The game given up on a question: nothing opens.
        #[qsignal]
        fn dropped(self: Pin<&mut LaunchModel>, path: QString);
    }

    impl cxx_qt::Threading for LaunchModel {}
}

/// What is asked now.
enum Asked {
    Approval(Box<Awaiting>),
    Firmware(Vec<FirmwareNeed>),
}

#[derive(Default)]
pub struct LaunchModelRust {
    platform_library: QString,
    cores: QString,
    asking: bool,
    title: QString,
    sentences: QStringList,
    caption: QString,
    choices: QStringList,
    back_index: i32,
    help: i32,
    notice: QString,
    notice_attention: bool,
    runtime: Option<Runtime>,
    folder: String,
    path: String,
    asked: Option<Asked>,
    /// Counts the games prepared, so an answer that comes back after another game was chosen is dropped.
    serial: u64,
}

fn list(words: &[String]) -> QStringList {
    let items: QList<QString> = words.iter().map(|w| QString::from(w.as_str())).collect();
    QStringList::from(&items)
}

impl qobject::LaunchModel {
    /// The runtime, made once on the platform library with the game's configuration, and the firmware folder's path.
    fn runtime(mut self: Pin<&mut Self>) -> Result<Runtime, String> {
        if let Some(runtime) = &self.runtime {
            return Ok(runtime.clone());
        }
        let library = self.platform_library.to_string();
        let platform = Platform::load(&library).map_err(|e| e.to_string())?;
        let config = crate::clients::runtime_config(&platform, &self.cores.to_string())?;
        let runtime = Api::load(&library).map_err(|e| e.to_string()).and_then(|api| crate::clients::runtime(&api, &config))?.0;
        // With no folder set the question says where in general words; a folder that cannot be read is said too (§7.9),
        // though Galaxia fails the call only for a folder number it lacks (`emusen_galaxia_directory`): unreachable.
        let folder = match platform.directory(DIR_FIRMWARE) {
            Ok(folder) => folder.map(|d| d.display().to_string()),
            Err(e) => {
                self.as_mut().set_problem(format!("The firmware folder could not be read: {e}"));
                None
            }
        };
        let folder = folder.unwrap_or_else(|| "the firmware folder".to_owned());
        self.as_mut().rust_mut().folder = folder;
        self.as_mut().rust_mut().runtime = Some(runtime.clone());
        Ok(runtime)
    }

    pub fn prepare(mut self: Pin<&mut Self>, path: &QString) {
        let serial = self.serial + 1;
        self.as_mut().rust_mut().serial = serial;
        self.as_mut().rust_mut().path = path.to_string();
        self.as_mut().set_notice(String::new());
        self.as_mut().put_up(None);
        let runtime = match self.as_mut().runtime() {
            Ok(runtime) => runtime,
            // No runtime to ask: the game's own open says why, on the screen (§7.9).
            Err(_) => return self.go(),
        };
        let path = path.to_string();
        let queue = self.qt_thread();
        std::thread::spawn(move || {
            let found = launch::approval_of(&runtime, &path);
            let _ = queue.queue(move |model| model.approval_read(serial, found));
        });
    }

    fn approval_read(mut self: Pin<&mut Self>, serial: u64, found: Option<Awaiting>) {
        if serial != self.serial {
            return;
        }
        match found {
            Some(found) if found.held.is_some() => {
                let words = launch::held_notice(&found.asking.name, found.instead.as_deref().unwrap_or_default(), found.held.as_deref().unwrap_or_default());
                self.as_mut().set_notice(words);
                self.firmware();
            }
            Some(found) => {
                let question = launch::approval_question(&found);
                self.as_mut().rust_mut().help = crate::help_model::shown_screen(Screen::Question);
                self.as_mut().help_changed();
                self.put_up(Some((question, Asked::Approval(Box::new(found)))));
            }
            None => self.firmware(),
        }
    }

    /// The firmware the game's engine requires and the library lacks, read off the interface thread.
    fn firmware(self: Pin<&mut Self>) {
        let Some(runtime) = self.runtime.clone() else { return self.go() };
        let (path, serial) = (self.path.clone(), self.serial);
        let queue = self.qt_thread();
        std::thread::spawn(move || {
            let engine = launch::engine_of(&runtime, &path);
            // The platform fails this only for an empty path or a null runtime (`emusen_crystal_firmware_needed`), which
            // no launch passes: the failure's words are said, but no player's launch makes them (EmuSen_Qt.md §7.9).
            let needs = runtime.firmware_needed(&path, engine.as_deref()).map(|n| n.into_iter().filter(|n| n.required).collect()).map_err(|e| e.to_string());
            let _ = queue.queue(move |model| model.firmware_checked(serial, needs));
        });
    }

    /// A failed check is never read as "no firmware needed": the player is told, and the game's own open says the rest.
    fn firmware_checked(mut self: Pin<&mut Self>, serial: u64, needs: Result<Vec<FirmwareNeed>, String>) {
        match needs {
            Ok(needs) => self.firmware_read(serial, needs),
            Err(e) if serial == self.serial => {
                self.as_mut().set_problem(launch::firmware_unchecked(&e));
                self.go();
            }
            Err(_) => {}
        }
    }

    fn firmware_read(mut self: Pin<&mut Self>, serial: u64, needs: Vec<FirmwareNeed>) {
        if serial != self.serial {
            return;
        }
        let Some(first) = needs.first() else { return self.go() };
        let question = launch::firmware_question(first, &self.folder);
        self.as_mut().rust_mut().help = crate::help_model::shown_screen(Screen::QuestionBack);
        self.as_mut().help_changed();
        self.put_up(Some((question, Asked::Firmware(needs))));
    }

    pub fn answer(mut self: Pin<&mut Self>, index: i32) {
        let Some(asked) = self.as_mut().rust_mut().asked.take() else { return };
        self.as_mut().put_up(None);
        match asked {
            Asked::Approval(found) => {
                let yes = index == 1;
                if yes {
                    let refused = self.runtime.as_ref().and_then(|r| r.core_approve(&found.asking.sha256).err()).map(|e| e.to_string());
                    self.as_mut().set_notice(launch::approved_notice(&found, refused.as_deref()));
                } else if let Some(instead) = &found.instead {
                    self.as_mut().set_notice(launch::fallback_notice(&found.asking.name, instead));
                }
                self.firmware();
            }
            Asked::Firmware(mut needs) => match launch::firmware_answer(index) {
                FirmwareAnswer::PlayWithout => {
                    needs.remove(0);
                    let serial = self.serial;
                    self.firmware_read(serial, needs);
                }
                FirmwareAnswer::CheckAgain => self.firmware(),
                FirmwareAnswer::Back => {
                    let path = QString::from(self.path.as_str());
                    self.dropped(path);
                }
            },
        }
    }

    fn go(mut self: Pin<&mut Self>) {
        let path = QString::from(self.path.as_str());
        self.as_mut().put_up(None);
        self.ready(path);
    }

    fn set_notice(self: Pin<&mut Self>, words: String) {
        self.said(words, false);
    }

    /// A notice that says something was not done.
    fn set_problem(self: Pin<&mut Self>, words: String) {
        self.said(words, true);
    }

    fn said(mut self: Pin<&mut Self>, words: String, attention: bool) {
        let words = QString::from(words.as_str());
        if self.notice_attention != attention {
            self.as_mut().rust_mut().notice_attention = attention;
            self.as_mut().notice_attention_changed();
        }
        if self.notice != words {
            self.as_mut().rust_mut().notice = words;
            self.notice_changed();
        }
    }

    /// The question shown, or taken down with none.
    fn put_up(mut self: Pin<&mut Self>, asked: Option<(Question, Asked)>) {
        let Some((question, asked)) = asked else {
            self.as_mut().rust_mut().asked = None;
            if self.asking {
                self.as_mut().rust_mut().asking = false;
                self.asking_changed();
            }
            return;
        };
        self.as_mut().rust_mut().title = QString::from(question.title.as_str());
        self.as_mut().title_changed();
        self.as_mut().rust_mut().sentences = list(&question.sentences);
        self.as_mut().sentences_changed();
        self.as_mut().rust_mut().caption = QString::from(question.caption.as_str());
        self.as_mut().caption_changed();
        self.as_mut().rust_mut().choices = list(&question.choices);
        self.as_mut().choices_changed();
        self.as_mut().rust_mut().back_index = back_index(&question);
        self.as_mut().back_index_changed();
        self.as_mut().rust_mut().asked = Some(asked);
        self.as_mut().rust_mut().asking = true;
        self.asking_changed();
    }
}
