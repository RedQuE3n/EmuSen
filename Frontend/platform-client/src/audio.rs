//! Endymion's game player: a game's sound to SDL's playback device, steered by a rate control of its own, as the C#
//! front end's `AudioPlayer` plays it (EmuSen_Audio_Sync.md §7). It needs SDL lent first.

use crate::error::Result;
use crate::platform::Platform;
use crate::raw;
use std::ptr::NonNull;
use std::sync::Arc;

/// How a game's sound is played: audio.json's three, and the device's buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundSettings {
    pub sample_rate: i32,
    pub target_latency_ms: i32,
    pub max_rate_deviation: f64,
    pub buffer_frames: i32,
}

impl Default for SoundSettings {
    /// `AudioPlayer`'s own defaults, which audio.json's are.
    fn default() -> SoundSettings {
        SoundSettings { sample_rate: 32000, target_latency_ms: 256, max_rate_deviation: 0.005, buffer_frames: 4096 }
    }
}

/// The game's sound player and the rate control it is lent at each call.
pub struct GameSound {
    platform: Arc<Platform>,
    rate: NonNull<raw::emusen_endymion_rate>,
    audio: NonNull<raw::emusen_endymion_audio>,
}

// The header: a player and its rate control are used by one thread at a time.
unsafe impl Send for GameSound {}

impl GameSound {
    /// A player on the system's default device, opened at the first submit; refused with words where SDL is not lent.
    pub fn new(platform: &Arc<Platform>, settings: SoundSettings) -> Result<GameSound> {
        let lib = &platform.lib;
        let rate = NonNull::new(unsafe { lib.emusen_endymion_rate_new(settings.target_latency_ms * settings.sample_rate / 1000) })
            .ok_or_else(|| crate::error::status(raw::EMUSEN_PLATFORM_BAD_ARGUMENT, platform.last_error()))?;
        let set = unsafe { lib.emusen_endymion_rate_set(rate.as_ptr(), raw::EMUSEN_ENDYMION_RATE_MAX_DEVIATION, settings.max_rate_deviation) };
        let made = platform.status(set.into()).map(|_| unsafe {
            lib.emusen_endymion_audio_new(rate.as_ptr(), settings.sample_rate, settings.target_latency_ms, settings.buffer_frames)
        });
        match made.map(NonNull::new) {
            Ok(Some(audio)) => Ok(GameSound { platform: platform.clone(), rate, audio }),
            refused => {
                unsafe { lib.emusen_endymion_rate_free(rate.as_ptr()) };
                Err(refused.err().unwrap_or_else(|| crate::error::status(raw::EMUSEN_PLATFORM_NOT_SUPPORTED, platform.last_error())))
            }
        }
    }

    /// Interleaved stereo at `sample_rate` played; a new rate reopens the device.
    pub fn submit(&mut self, samples: &[i16], sample_rate: u32) -> Result<()> {
        let n = unsafe {
            self.platform.lib.emusen_endymion_audio_submit(
                self.audio.as_ptr(),
                self.rate.as_ptr(),
                samples.as_ptr(),
                samples.len(),
                sample_rate as i32,
            )
        };
        self.platform.status(n.into()).map(drop)
    }

    /// The rate control started over, as a game that stops playing for a while needs it.
    pub fn reset(&mut self) -> Result<()> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_rate_reset(self.rate.as_ptr()) }.into()).map(drop)
    }

    /// The stereo frames queued on the device and not yet played: what the rate control steers.
    pub fn queued_frames(&self) -> Result<i64> {
        self.get(raw::EMUSEN_ENDYMION_AUDIO_QUEUED_FRAMES)
    }

    /// Whether a device is open.
    pub fn available(&self) -> Result<bool> {
        self.get(raw::EMUSEN_ENDYMION_AUDIO_AVAILABLE).map(|n| n != 0)
    }

    /// The rate the device is open at.
    pub fn sample_rate(&self) -> Result<i64> {
        self.get(raw::EMUSEN_ENDYMION_AUDIO_SAMPLE_RATE)
    }

    /// The stereo frames the player has taken in, by its rate control's count.
    pub fn taken_frames(&self) -> Result<i64> {
        let mut state = raw::emusen_endymion_rate_state { size: std::mem::size_of::<raw::emusen_endymion_rate_state>() as u32, ..Default::default() };
        let n = unsafe { self.platform.lib.emusen_endymion_rate_read(self.rate.as_ptr(), &mut state) };
        self.platform.status(n.into()).map(|_| state.total_input_frames)
    }

    /// The volume, 0 to 1, carried across a reopen.
    pub fn set_volume(&self, volume: f32) -> Result<()> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_audio_set_volume(self.audio.as_ptr(), volume) }.into()).map(drop)
    }

    fn get(&self, which: u32) -> Result<i64> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_audio_get(self.audio.as_ptr(), which) })
    }
}

impl Drop for GameSound {
    fn drop(&mut self) {
        unsafe {
            self.platform.lib.emusen_endymion_audio_dispose(self.audio.as_ptr());
            self.platform.lib.emusen_endymion_audio_free(self.audio.as_ptr());
            self.platform.lib.emusen_endymion_rate_free(self.rate.as_ptr());
        }
    }
}
