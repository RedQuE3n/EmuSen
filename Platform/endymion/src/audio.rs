//! The game's sound to SDL's default playback device through the rate control, and the interface's short sounds on a
//! stream of their own: the C# `AudioPlayer` and `UiSoundPlayer`. See EmuSen_Audio_Sync.md §7 and
//! EmuSen_Settings_Reference.md §4.10 and §4.52.
//!
//! Each owns its device until `dispose`, which may be called more than once, as the C#'s `Dispose` may; dropping one
//! disposes it, and `abandon` lets the device go to the process's end, as a C# player never disposed did.

use crate::rate_control::DynamicRateControl;
use crate::resampler::RatioError;
use crate::sdl::{self, AudioSpec, Sdl};
use std::collections::HashMap;
use std::ffi::{CString, c_void};
use std::sync::Arc;

/// The game's sink: samples and a rate in, nothing about cores. The rate control is the caller's, lent to each call.
pub struct AudioPlayer {
    sdl: Arc<Sdl>,
    subsystem: bool,
    stream: *mut c_void,
    device_open: bool,
    open_sample_rate: i32,
    target_latency_ms: i32,
    buffer_frames: i32,
    volume: f32,
    scratch: Vec<i16>,
}

unsafe impl Send for AudioPlayer {}

/// `targetLatencyMs * sampleRate / 1000` in C#'s unchecked 32-bit arithmetic.
pub fn target_frames(target_latency_ms: i32, sample_rate: i32) -> i32 {
    target_latency_ms.wrapping_mul(sample_rate) / 1000
}

impl AudioPlayer {
    /// Opens the default device up front, so `is_available` means something before any samples exist.
    pub fn new(sdl: Arc<Sdl>, rate: &mut DynamicRateControl, sample_rate: i32, target_latency_ms: i32, buffer_frames: i32) -> AudioPlayer {
        let mut player = AudioPlayer {
            subsystem: unsafe { (sdl.f.init_sub_system)(sdl::INIT_AUDIO) },
            sdl,
            stream: std::ptr::null_mut(),
            device_open: false,
            open_sample_rate: 0,
            target_latency_ms,
            buffer_frames,
            volume: 1.0,
            scratch: Vec::new(),
        };
        if player.subsystem {
            player.open_device(rate, sample_rate);
        }
        player
    }

    fn open_device(&mut self, rate: &mut DynamicRateControl, sample_rate: i32) {
        self.sdl.set_hint(sdl::HINT_AUDIO_DEVICE_SAMPLE_FRAMES, &self.buffer_frames.to_string());
        let format = if cfg!(target_endian = "little") { sdl::AUDIO_S16LE } else { sdl::AUDIO_S16BE };
        let desired = AudioSpec { format, channels: 2, freq: sample_rate };
        self.stream = unsafe { (self.sdl.f.open_audio_device_stream)(sdl::AUDIO_DEVICE_DEFAULT_PLAYBACK, &desired, std::ptr::null(), std::ptr::null_mut()) };
        self.device_open = !self.stream.is_null();
        self.open_sample_rate = if self.device_open { sample_rate } else { 0 };
        if !self.device_open {
            return;
        }
        rate.target_queued_frames = target_frames(self.target_latency_ms, sample_rate);
        unsafe {
            (self.sdl.f.set_audio_stream_gain)(self.stream, self.volume);
            (self.sdl.f.resume_audio_stream_device)(self.stream);
        }
    }

    fn close_device(&mut self) {
        if !self.device_open {
            return;
        }
        unsafe { (self.sdl.f.destroy_audio_stream)(self.stream) };
        self.stream = std::ptr::null_mut();
        self.device_open = false;
        self.open_sample_rate = 0;
    }

    /// Frames waiting in the device's stream: what the rate control steers.
    pub fn queued_frames(&self) -> i32 {
        if self.device_open { unsafe { (self.sdl.f.get_audio_stream_queued)(self.stream) }.max(0) / 4 } else { 0 }
    }

    pub fn is_available(&self) -> bool {
        self.device_open
    }

    pub fn sample_rate(&self) -> i32 {
        self.open_sample_rate
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// 0 to 1, the stream's gain, carried across a reopen.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if self.device_open {
            unsafe { (self.sdl.f.set_audio_stream_gain)(self.stream, self.volume) };
        }
    }

    /// L,R,L,R 16-bit PCM at `sample_rate`; a new rate moves the device. None is C#'s null, which does nothing.
    pub fn submit(&mut self, rate: &mut DynamicRateControl, samples: Option<&[i16]>, sample_rate: i32) -> Result<(), RatioError> {
        let Some(samples) = samples else { return Ok(()) };
        if !self.subsystem {
            return Ok(());
        }
        if self.device_open && sample_rate > 0 && sample_rate != self.open_sample_rate {
            self.close_device();
            self.open_device(rate, sample_rate);
            rate.reset();
        }
        if !self.device_open || samples.is_empty() {
            return Ok(());
        }
        let queued = self.queued_frames();
        let mut data = std::mem::take(&mut self.scratch);
        data.clear();
        let result = rate.process(samples, queued, &mut data);
        if result.is_ok() && !data.is_empty() {
            unsafe { (self.sdl.f.put_audio_stream_data)(self.stream, data.as_ptr().cast(), (data.len() * 2) as i32) };
        }
        self.scratch = data;
        result
    }

    pub fn dispose(&mut self) {
        self.close_device();
        if self.subsystem {
            unsafe { (self.sdl.f.quit_sub_system)(sdl::INIT_AUDIO) };
        }
    }

    /// Forgets the device without closing it.
    pub fn abandon(mut self) {
        self.subsystem = false;
        self.device_open = false;
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.dispose();
    }
}

/// The interface sounds' format: 48 kHz stereo 32-bit float.
pub const UI_FORMAT: AudioSpec = AudioSpec { format: sdl::AUDIO_F32LE, channels: 2, freq: 48000 };

/// The interface's short sounds, decoded once each and played on a stream opened by the first of them.
pub struct UiSounds {
    sdl: Arc<Sdl>,
    decoded: HashMap<String, Option<Vec<u8>>>,
    subsystem: bool,
    tried: bool,
    stream: *mut c_void,
    volume: f32,
}

unsafe impl Send for UiSounds {}

/// A path as SDL's C# binding hands it over: the text up to its first NUL.
fn c_path(path: &str) -> CString {
    CString::new(path.split('\0').next().unwrap_or_default()).expect("no NUL is left")
}

/// A WAV file converted once to the interface's format; None when SDL cannot read it.
pub fn decode(sdl: &Sdl, path: &str) -> Option<Vec<u8>> {
    let path = c_path(path);
    let mut spec = AudioSpec::default();
    let mut data: *mut u8 = std::ptr::null_mut();
    let mut length = 0u32;
    if !unsafe { (sdl.f.load_wav)(path.as_ptr(), &mut spec, &mut data, &mut length) } {
        return None;
    }
    let mut converted: *mut u8 = std::ptr::null_mut();
    let mut converted_length = 0;
    let ok = unsafe { (sdl.f.convert_audio_samples)(&spec, data, length as i32, &UI_FORMAT, &mut converted, &mut converted_length) };
    let bytes = ok.then(|| unsafe { std::slice::from_raw_parts(converted, converted_length.max(0) as usize) }.to_vec());
    unsafe {
        if ok {
            (sdl.f.free)(converted.cast());
        }
        (sdl.f.free)(data.cast());
    }
    bytes
}

impl UiSounds {
    pub fn new(sdl: Arc<Sdl>) -> UiSounds {
        UiSounds { sdl, decoded: HashMap::new(), subsystem: false, tried: false, stream: std::ptr::null_mut(), volume: 0.7 }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if !self.stream.is_null() {
            unsafe { (self.sdl.f.set_audio_stream_gain)(self.stream, self.volume) };
        }
    }

    pub fn is_open(&self) -> bool {
        !self.stream.is_null()
    }

    /// Bytes waiting in the stream.
    pub fn queued(&self) -> i32 {
        if self.stream.is_null() { 0 } else { unsafe { (self.sdl.f.get_audio_stream_queued)(self.stream) }.max(0) }
    }

    fn samples(&mut self, path: &str) -> Option<&Vec<u8>> {
        if !self.decoded.contains_key(path) {
            let samples = decode(&self.sdl, path);
            self.decoded.insert(path.to_string(), samples);
        }
        self.decoded[path].as_ref()
    }

    pub fn preload(&mut self, path: &str) {
        self.samples(path);
    }

    /// Samples already in the stream's format, kept under a name `play` then takes as it takes a file's path.
    pub fn remember(&mut self, key: &str, samples: Vec<u8>) {
        self.decoded.insert(key.to_string(), Some(samples));
    }

    /// A new sound replaces the one still playing.
    pub fn play(&mut self, path: &str) {
        let has = self.samples(path).is_some_and(|s| !s.is_empty());
        if !has || !self.open() {
            return;
        }
        let samples = self.decoded[path].as_ref().expect("decoded above");
        unsafe {
            (self.sdl.f.clear_audio_stream)(self.stream);
            (self.sdl.f.put_audio_stream_data)(self.stream, samples.as_ptr().cast(), samples.len() as i32);
        }
    }

    /// Opened on the first sound, so a session that never plays one never touches the audio device.
    fn open(&mut self) -> bool {
        if !self.stream.is_null() {
            return true;
        }
        if self.tried {
            return false;
        }
        self.tried = true;
        self.subsystem = unsafe { (self.sdl.f.init_sub_system)(sdl::INIT_AUDIO) };
        if !self.subsystem {
            return false;
        }
        self.stream = unsafe { (self.sdl.f.open_audio_device_stream)(sdl::AUDIO_DEVICE_DEFAULT_PLAYBACK, &UI_FORMAT, std::ptr::null(), std::ptr::null_mut()) };
        if self.stream.is_null() {
            return false;
        }
        unsafe {
            (self.sdl.f.set_audio_stream_gain)(self.stream, self.volume);
            (self.sdl.f.resume_audio_stream_device)(self.stream);
        }
        true
    }

    pub fn dispose(&mut self) {
        if !self.stream.is_null() {
            unsafe { (self.sdl.f.destroy_audio_stream)(self.stream) };
        }
        self.stream = std::ptr::null_mut();
        if self.subsystem {
            unsafe { (self.sdl.f.quit_sub_system)(sdl::INIT_AUDIO) };
        }
        self.subsystem = false;
    }

    /// Forgets the stream without destroying it.
    pub fn abandon(mut self) {
        self.stream = std::ptr::null_mut();
        self.subsystem = false;
    }
}

impl Drop for UiSounds {
    fn drop(&mut self) {
        self.dispose();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_latency_target_is_csharps_unchecked_arithmetic() {
        assert_eq!(target_frames(256, 32000), 8192);
        assert_eq!(target_frames(256, 44100), 11289);
        assert_eq!(target_frames(i32::MAX, 2), -2 / 1000);
        assert_eq!(target_frames(100_000, 48_000), 100_000i32.wrapping_mul(48_000) / 1000);
    }

    #[test]
    fn a_path_is_cut_at_its_first_nul_as_the_binding_cuts_it() {
        assert_eq!(c_path("a.wav\0b").as_bytes(), b"a.wav");
        assert_eq!(c_path("plain.wav").as_bytes(), b"plain.wav");
    }
}
