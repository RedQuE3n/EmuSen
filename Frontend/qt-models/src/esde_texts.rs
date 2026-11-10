//! The ES-DE view's four texts made QStrings off the GUI thread (EmuSen_Qt.md §4.9): the shelves' text runs to megabytes
//! at a player's library's scale, and its UTF-16 copy took 5 ms of the GUI thread's refresh at 9,000 games.

use cxx_qt_lib::QString;

use crate::esde_shelves::{Esde, EsdeSettings};

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
