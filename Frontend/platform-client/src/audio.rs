//! Endymion's game player: a game's sound to SDL's playback device, steered by a rate control of its own, as the C#
//! front end's `AudioPlayer` plays it (EmuSen_Audio_Sync.md §7), on the system default or a device SDL lists by name.
//! It needs SDL lent first.

use crate::error::Result;
use crate::platform::Platform;
use crate::raw;
use serde_json::Value;
use std::ptr::NonNull;
use std::sync::Arc;

/// How a game's sound is played: the three the base's `AudioConfig` holds, and the device's buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundSettings {
    pub sample_rate: i32,
    pub target_latency_ms: i32,
    pub max_rate_deviation: f64,
    pub buffer_frames: i32,
}

/// The player's own constant for SDL's device buffer hint (frames); no owner in the base holds it (EmuSen_Audio_Sync.md §7.6).
pub const DEVICE_BUFFER_FRAMES: i32 = 4096;

impl SoundSettings {
    /// The settings an `AudioConfig` document holds, as Galaxia binds it (`Platform::model_new` or `model_load` of
    /// `MODEL_AUDIO_CONFIG`); a field missing or of another type is refused with its name.
    pub fn of_audio_config(config: &Value) -> Result<SoundSettings> {
        let field = |name: &str| config.get(name).ok_or_else(|| missing(name));
        let whole = |name: &str| field(name)?.as_i64().and_then(|n| i32::try_from(n).ok()).ok_or_else(|| missing(name));
        Ok(SoundSettings {
            sample_rate: whole("SampleRate")?,
            target_latency_ms: whole("OutputTargetLatencyMs")?,
            max_rate_deviation: field("RateControlMaxDeviation")?.as_f64().ok_or_else(|| missing("RateControlMaxDeviation"))?,
            buffer_frames: DEVICE_BUFFER_FRAMES,
        })
    }
}

fn missing(name: &str) -> crate::error::Error {
    crate::error::status(raw::EMUSEN_PLATFORM_PARSE, format!("AudioConfig has no number {name}"))
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

    /// The rate control's frames it keeps queued and its largest deviation from the nominal ratio, as it holds them.
    pub fn rate_target(&self) -> Result<(i32, f64)> {
        self.rate_state().map(|state| (state.target_queued_frames, state.max_deviation))
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
        self.rate_state().map(|state| state.total_input_frames)
    }

    /// The rate control's state as it holds it.
    fn rate_state(&self) -> Result<raw::emusen_endymion_rate_state> {
        let mut state = raw::emusen_endymion_rate_state { size: std::mem::size_of::<raw::emusen_endymion_rate_state>() as u32, ..Default::default() };
        let n = unsafe { self.platform.lib.emusen_endymion_rate_read(self.rate.as_ptr(), &mut state) };
        self.platform.status(n.into()).map(|_| state)
    }

    /// The device the game plays on, by a name `Platform::audio_devices` gave; none for the system default. An open
    /// device reopens at its rate and the rate control starts over. False where no device has the name and the default plays.
    pub fn set_device(&mut self, name: Option<&str>) -> Result<bool> {
        let (n, nl) = crate::out::text(name);
        let set = unsafe { self.platform.lib.emusen_endymion_audio_device_set(self.audio.as_ptr(), self.rate.as_ptr(), n, nl) };
        match self.platform.status(set.into()) {
            Ok(_) => Ok(true),
            Err(e) if crate::error::absent(&e) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// The volume, 0 to 1, carried across a reopen.
    pub fn set_volume(&self, volume: f32) -> Result<()> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_audio_set_volume(self.audio.as_ptr(), volume) }.into()).map(drop)
    }

    /// The volume the player plays at, as it holds it.
    pub fn volume(&self) -> f32 {
        unsafe { self.platform.lib.emusen_endymion_audio_volume(self.audio.as_ptr()) }
    }

    fn get(&self, which: u32) -> Result<i64> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_audio_get(self.audio.as_ptr(), which) })
    }
}

impl Platform {
    /// SDL's playback devices by name, in SDL's order; none where SDL's audio does not start.
    pub fn audio_devices(&self) -> Result<Vec<String>> {
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_endymion_audio_device_list(o, l) })?.unwrap_or_default();
        Ok(bytes.split(|b| *b == 0).filter(|n| !n.is_empty()).map(|n| String::from_utf8_lossy(n).into_owned()).collect())
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
