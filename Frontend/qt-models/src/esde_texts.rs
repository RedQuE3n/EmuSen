//! The ES-DE view's four texts made QStrings off the GUI thread (EmuSen_Qt.md §4.9): the shelves' text runs to megabytes
//! at a player's library's scale, and its UTF-16 copy took 5 ms of the GUI thread's refresh at 9,000 games.

use cxx_qt_lib::QString;

use crate::esde_shelves::{Esde, EsdeSettings};
use crate::library::{LibraryView, Refreshed};

/// The settings', shelves', media index's and presence's texts as QStrings, in that order.
pub struct EsdeTexts(pub [QString; 4]);

// Sound: each QString is made here and moved whole, no copy kept, and Qt's implicit sharing counts atomically (EmuSen_Qt.md §4.9).
unsafe impl Send for EsdeTexts {}

impl EsdeTexts {
    /// The four texts of a read, made where the read was.
    pub fn of(settings: &EsdeSettings, esde: &Esde) -> EsdeTexts {
        EsdeTexts([&settings.text, &esde.shelves, &esde.media, &esde.presence].map(|t| QString::from(t.as_str())))
    }
}

/// The GUI thread's part of a refresh: which texts changed, compared as Rust text; the read moved into the view; and the
/// changed texts, already QStrings, moved into the properties `shown`. What they replace, megabytes at a library's scale,
/// is dropped on a thread of its own.
pub fn take(view: &mut LibraryView, shown: [&mut QString; 4], refreshed: Refreshed, texts: Option<EsdeTexts>) -> [bool; 4] {
    let changed = match &refreshed {
        Ok(read) => {
            let (old, new) = ((&view.esde_settings, &view.esde), (&read.esde_settings, &read.esde));
            [old.0.text != new.0.text, old.1.shelves != new.1.shelves, old.1.media != new.1.media, old.1.presence != new.1.presence]
        }
        Err(_) => [false; 4],
    };
    let replaced = view.swap_in(refreshed);
    let mut old: [QString; 4] = Default::default();
    if let Some(EsdeTexts(texts)) = texts {
        for (((slot, text), c), old) in shown.into_iter().zip(texts).zip(changed).zip(&mut old) {
            if c {
                *old = std::mem::replace(slot, text);
            }
        }
    }
    let old = EsdeTexts(old);
    std::thread::spawn(move || drop((replaced, old)));
    changed
}
