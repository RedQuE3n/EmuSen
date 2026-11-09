//! `Folders`, the program's folder actions (EmuSen_Qt.md §7.7.E): a folder shown in the system's file manager, a
//! `file:` URL opened by Qt's desktop services, offered everywhere, Steam's Game Mode included, since the player can
//! switch to another window there; and a folder chosen in the system's own folder dialog, the program's QML
//! `FolderDialog`, whose answer goes back as a local path for the key that asked. A test sets `catching`: the `file`
//! scheme is handed to this object and a choice is kept for the test to answer, so no file manager or dialog opens.

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QStringList, QUrl};

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
        include!("folders.h");

        /// A local folder's `file:` URL, as Qt writes one.
        #[cxx_name = "folderUrl"]
        fn url_of(path: &QString) -> QString;

        /// The folder opened by the desktop's handler for `file:` URLs; false where none took it.
        #[cxx_name = "folderOpen"]
        fn open_url(path: &QString) -> bool;

        /// A `file:` URL's local path, empty for any other URL.
        #[cxx_name = "folderPath"]
        fn path_of_url(url: &QUrl) -> QString;

        /// `file:` URLs handed to `method` of `receiver` from now on, in place of the desktop's handler.
        #[cxx_name = "folderUrlsTo"]
        fn urls_to(receiver: &Folders, method: &QString) -> bool;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        /// The URLs taken while catching, oldest first.
        #[qproperty(QStringList, caught, READ, NOTIFY)]
        /// Whether `file:` URLs and folder requests come here rather than to the desktop: a test's, never the program's.
        #[qproperty(bool, catching)]
        /// The folder a choice starts at, and whether one waits for its answer.
        #[qproperty(QString, asked, READ, NOTIFY)]
        #[qproperty(bool, waiting, READ, NOTIFY)]
        /// Where the folder dialog starts.
        #[qproperty(QUrl, dialog_folder, cxx_name = "dialogFolder", READ, NOTIFY)]
        /// A path chosen while catching; `cancel` answers as the dialog's Cancel.
        #[qproperty(QString, answer)]
        #[qproperty(bool, cancel)]
        type Folders = super::FoldersRust;
    }

    extern "RustQt" {
        /// The folder at `path` shown in the file manager.
        #[qinvokable]
        fn open(self: Pin<&mut Folders>, path: &QString) -> bool;

        /// A local folder's `file:` URL, as a folder dialog starts at it.
        #[qinvokable]
        #[cxx_name = "urlOf"]
        fn url_of_path(self: &Folders, path: &QString) -> QUrl;

        /// A `file:` URL's local path, as a folder dialog answers it.
        #[qinvokable]
        #[cxx_name = "pathOf"]
        fn path_of(self: &Folders, url: &QUrl) -> QString;

        /// A folder to choose for `key`, starting at `start`: the dialog asked for, or the request kept while catching.
        #[qinvokable]
        fn choose(self: Pin<&mut Folders>, key: &QString, start: &QString);

        /// The dialog's folder accepted.
        #[qinvokable]
        fn accepted(self: Pin<&mut Folders>, url: &QUrl);

        /// The dialog cancelled: nothing is chosen.
        #[qinvokable]
        fn rejected(self: Pin<&mut Folders>);

        /// The folder dialog to open.
        #[qsignal]
        #[cxx_name = "dialogAsked"]
        fn dialog_asked(self: Pin<&mut Folders>);

        /// A folder chosen for a key.
        #[qsignal]
        fn chosen(self: Pin<&mut Folders>, key: QString, path: QString);

        /// A URL taken in place of the desktop's handler.
        #[qinvokable]
        fn take(self: Pin<&mut Folders>, url: &QUrl);
    }

    impl cxx_qt::Initialize for Folders {}
}

#[derive(Default)]
pub struct FoldersRust {
    caught: QStringList,
    catching: bool,
    asked: QString,
    waiting: bool,
    dialog_folder: QUrl,
    answer: QString,
    cancel: bool,
    /// The key the choice waiting is for.
    key: QString,
}

impl cxx_qt::Initialize for ffi::Folders {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_catching_changed(|model| {
                if model.catching {
                    ffi::urls_to(model.as_ref().get_ref(), &QString::from("take"));
                }
            })
            .release();
        self.as_mut()
            .on_answer_changed(|model| {
                if model.catching && model.waiting && !model.answer.is_empty() {
                    let answer = model.answer.clone();
                    model.done(&answer);
                }
            })
            .release();
        self.on_cancel_changed(|model| {
            if model.catching && model.waiting && model.cancel {
                model.done(&QString::default());
            }
        })
        .release();
    }
}

impl ffi::Folders {
    pub fn open(self: Pin<&mut Self>, path: &QString) -> bool {
        if path.is_empty() {
            return false;
        }
        ffi::open_url(path)
    }

    pub fn choose(mut self: Pin<&mut Self>, key: &QString, start: &QString) {
        self.as_mut().rust_mut().key = key.clone();
        self.as_mut().set_asked_now(start.clone());
        self.as_mut().set_waiting_now(true);
        if !self.catching {
            let url = if start.is_empty() { QUrl::default() } else { QUrl::from(&ffi::url_of(start)) };
            self.as_mut().rust_mut().dialog_folder = url;
            self.as_mut().dialog_folder_changed();
            self.dialog_asked();
        }
    }

    pub fn accepted(self: Pin<&mut Self>, url: &QUrl) {
        let path = ffi::path_of_url(url);
        self.done(&path);
    }

    pub fn rejected(self: Pin<&mut Self>) {
        self.done(&QString::default());
    }

    /// The choice answered: a path chosen, or none for Cancel, which chooses nothing.
    fn done(mut self: Pin<&mut Self>, path: &QString) {
        self.as_mut().set_waiting_now(false);
        if !path.is_empty() {
            let key = self.key.clone();
            self.chosen(key, path.clone());
        }
    }

    fn set_asked_now(mut self: Pin<&mut Self>, asked: QString) {
        self.as_mut().rust_mut().asked = asked;
        self.asked_changed();
    }

    fn set_waiting_now(mut self: Pin<&mut Self>, waiting: bool) {
        if self.waiting != waiting {
            self.as_mut().rust_mut().waiting = waiting;
            self.waiting_changed();
        }
    }

    pub fn url_of_path(&self, path: &QString) -> QUrl {
        QUrl::from(&ffi::url_of(path))
    }

    pub fn path_of(&self, url: &QUrl) -> QString {
        ffi::path_of_url(url)
    }

    pub fn take(mut self: Pin<&mut Self>, url: &QUrl) {
        let mut list = self.caught.clone();
        list.append(url.to_qstring());
        self.as_mut().rust_mut().caught = list;
        self.caught_changed();
    }
}

/// A local folder's `file:` URL, as the action hands it to the desktop.
pub fn url_of(path: &str) -> String {
    ffi::url_of(&QString::from(path)).to_string()
}
