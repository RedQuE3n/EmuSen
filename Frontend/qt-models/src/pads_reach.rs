//! What Settings ▸ Controllers hands the running game's pads (EmuSen_Qt.md §7.7.H8): the binding files a change touched,
//! saved and bound again, and the pad settings read from the stored file. No Qt: SettingsModel and the tests call these.

use crate::bindings::{Bindings, File};
use crate::settings::SettingsView;
use crate::settings_source::ClientSettings;
use emusen_qt_endymion::game::GamePads;

/// The binding files a change touched saved through Endymion, then the hotkeys and a running game's buttons and keys
/// bound again from the files (EmuSen_Interface.md §20.3), saved or not, so the game hears what the files hold.
pub fn bindings_saved(source: &ClientSettings, bindings: &Bindings, touched: &[File], pads: Option<&GamePads>) -> Result<(), String> {
    let saved = source.save_bindings(bindings, touched);
    if let Some(pads) = pads {
        pads.rebind();
    }
    saved
}

/// The settings the pads act on, as the view read them, settled on the pads (Mistress's `SaveApp`); none until the
/// settings are read. The words of a stored value that could not be read, if any.
pub fn settle(view: &SettingsView, pads: Option<&GamePads>) -> Option<String> {
    let (settings, unread) = view.pad_settings()?;
    if let Some(pads) = pads {
        pads.settle(settings);
    }
    unread
}
