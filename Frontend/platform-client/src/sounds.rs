//! Endymion's interface player: the interface's short sounds on a stream of their own. It needs SDL lent first.

use crate::error::Result;
use crate::platform::Platform;
use crate::raw;
use std::ptr::NonNull;
use std::sync::Arc;

/// The interface's sound player.
pub struct InterfacePlayer {
    platform: Arc<Platform>,
    ui: NonNull<raw::emusen_endymion_ui>,
}

// The header: an interface player is used by one thread at a time.
unsafe impl Send for InterfacePlayer {}

impl InterfacePlayer {
    /// A player, its stream opened by the first sound; refused with words where SDL is not lent.
    pub fn new(platform: &Arc<Platform>) -> Result<InterfacePlayer> {
        let ui = unsafe { platform.lib.emusen_endymion_ui_new() };
        let ui = NonNull::new(ui).ok_or_else(|| crate::error::status(raw::EMUSEN_PLATFORM_NOT_SUPPORTED, platform.last_error()))?;
        Ok(InterfacePlayer { platform: platform.clone(), ui })
    }

    /// Samples kept under `key`: 48 kHz stereo 32-bit float, as `decode` gives them.
    pub fn remember(&self, key: &str, samples: &[u8]) -> Result<()> {
        let n = unsafe {
            self.platform.lib.emusen_endymion_ui_remember(self.ui.as_ptr(), key.as_ptr(), key.len(), samples.as_ptr(), samples.len())
        };
        self.platform.status(n.into()).map(drop)
    }

    /// The sound kept under `key` played, replacing the one still playing.
    pub fn play(&self, key: &str) -> Result<()> {
        let n =
            unsafe { self.platform.lib.emusen_endymion_ui_sound(self.ui.as_ptr(), raw::EMUSEN_ENDYMION_UI_PLAY, key.as_ptr(), key.len()) };
        self.platform.status(n.into()).map(drop)
    }

    /// The device the sounds play on, by a name the device list gave; none for the system default. False where no
    /// device has the name and the default plays.
    pub fn set_device(&self, name: Option<&str>) -> Result<bool> {
        let (p, pl) = crate::out::text(name);
        match self.platform.status(unsafe { self.platform.lib.emusen_endymion_ui_set_device(self.ui.as_ptr(), p, pl) }.into()) {
            Ok(_) => Ok(true),
            Err(e) if crate::error::absent(&e) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Whether the stream is open.
    pub fn is_open(&self) -> Result<bool> {
        self.platform
            .status(unsafe { self.platform.lib.emusen_endymion_ui_get(self.ui.as_ptr(), raw::EMUSEN_ENDYMION_UI_IS_OPEN) })
            .map(|n| n != 0)
    }

    /// The bytes queued on the stream and not yet played.
    pub fn queued(&self) -> Result<i64> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_ui_get(self.ui.as_ptr(), raw::EMUSEN_ENDYMION_UI_QUEUED) })
    }

    /// The volume, 0 to 1.
    pub fn set_volume(&self, volume: f32) -> Result<()> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_ui_set_volume(self.ui.as_ptr(), volume) }.into()).map(drop)
    }
}

/// A WAV file as `remember` keeps sounds, 48 kHz stereo 32-bit float; none where SDL cannot read it. SDL must be lent.
/// The header leaves what does not fit waiting on this thread, so the first call asks the count and the second takes it.
pub fn decode(platform: &Platform, path: &str) -> Result<Option<Vec<u8>>> {
    let mut asked = false;
    platform.read(|out, cap| {
        if asked {
            unsafe { platform.lib.emusen_endymion_ui_decode_take(out, cap) }
        } else {
            asked = true;
            unsafe { platform.lib.emusen_endymion_ui_decode(path.as_ptr(), path.len(), out, cap) }
        }
    })
}

impl Drop for InterfacePlayer {
    fn drop(&mut self) {
        unsafe {
            self.platform.lib.emusen_endymion_ui_dispose(self.ui.as_ptr());
            self.platform.lib.emusen_endymion_ui_free(self.ui.as_ptr());
        }
    }
}
