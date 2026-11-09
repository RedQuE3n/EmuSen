//! `Folders`, the program's one action that shows a folder in the system's file manager (EmuSen_Qt.md §7.7.E): a
//! `file:` URL of the folder, opened by Qt's desktop services, offered everywhere, Steam's Game Mode included, since the
//! player can switch to another window there. A test hands the scheme to this object instead (`catching`), so the URL
//! is read and no file manager starts.

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

        /// `file:` URLs handed to `method` of `receiver` from now on, in place of the desktop's handler.
        #[cxx_name = "folderUrlsTo"]
        fn urls_to(receiver: &Folders, method: &QString) -> bool;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        /// The URLs taken while catching, oldest first.
        #[qproperty(QStringList, caught, READ, NOTIFY)]
        /// Whether `file:` URLs come here rather than to the desktop: a test's, never the program's.
        #[qproperty(bool, catching)]
        type Folders = super::FoldersRust;
    }

    extern "RustQt" {
        /// The folder at `path` shown in the file manager.
        #[qinvokable]
        fn open(self: Pin<&mut Folders>, path: &QString) -> bool;

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
}

impl cxx_qt::Initialize for ffi::Folders {
    fn initialize(self: Pin<&mut Self>) {
        self.on_catching_changed(|model| {
            if model.catching {
                ffi::urls_to(model.as_ref().get_ref(), &QString::from("take"));
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
