//! `SplashSound`: a splash's sound through Endymion's interface player, as an ES-DE theme's sounds are (EmuSen_Qt.md §3.5.2, A splash that enters Home).

use core::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QString, QUrl};
use emusen_platform_client::sounds::InterfacePlayer;
use emusen_qt_endymion::bell_player;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;
        include!("sound_file.h");
        type SoundFile;
        #[rust_name = "sound_file_samples"]
        fn soundFileSamples(file: &SoundFile) -> QByteArray;
        #[rust_name = "sound_file_problem"]
        fn soundFileProblem(file: &SoundFile) -> QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, platform_library)]
        /// The theme's folder as its host was given it; a sound outside it is refused.
        #[qproperty(QString, folder)]
        /// The splash screen's `sound`, as the theme names it.
        #[qproperty(QUrl, source)]
        /// The file read, which is kept once it has been read whole.
        #[qproperty(*mut SoundFile, file)]
        /// AppSettings.NavigationSounds: off plays nothing.
        #[qproperty(bool, on)]
        /// Sound volume, a share of full (SettingsModel.navigationVolume).
        #[qproperty(f64, volume)]
        /// The format the player keeps sounds in, which the file is read into: its rate and its channels of 32-bit floats.
        #[qproperty(i32, rate, READ, CONSTANT)]
        #[qproperty(i32, channels, READ, CONSTANT)]
        /// The samples kept and the player made, or nothing to play: the splash may start.
        #[qproperty(bool, settled, READ, NOTIFY)]
        /// Interleaved frames kept, for the tests; 0 for none.
        #[qproperty(i32, frames, READ, NOTIFY)]
        /// How many times the sound was started and let go, and the volume the player holds (-1 before it is made).
        #[qproperty(i32, starts, READ, NOTIFY)]
        #[qproperty(i32, ends, READ, NOTIFY)]
        #[qproperty(f64, held_volume, cxx_name = "heldVolume", READ, NOTIFY)]
        #[qproperty(QString, problem, READ, NOTIFY)]
        type SplashSound = super::SplashSoundRust;
    }

    extern "RustQt" {
        /// The samples taken from the file once it has been read, checked against the folder, and the player made with them.
        #[qinvokable]
        fn keep(self: Pin<&mut SplashSound>);
        /// Plays the sound from its start, where it is kept and on.
        #[qinvokable]
        fn start(self: Pin<&mut SplashSound>);
        /// The sound let go over `fade` ms from where it plays, falling to silence, so it ends with no click.
        #[qinvokable]
        fn end(self: Pin<&mut SplashSound>, fade: i32);
    }

    impl cxx_qt::Threading for SplashSound {}
    impl cxx_qt::Initialize for SplashSound {}
}

/// The key the player keeps the sound under, its soft end, and the frame of silence that opens its stream before the start.
const SOUND: &str = "theme:splash";
const ENDING: &str = "theme:splash-end";
const SILENCE: &str = "theme:splash-silence";

/// A frame's bytes: the interface player's channels of 32-bit floats.
const FRAME: usize = emusen_platform_client::sounds::CHANNELS as usize * 4;

pub struct SplashSoundRust {
    platform_library: QString,
    folder: QString,
    source: QUrl,
    file: *mut qobject::SoundFile,
    on: bool,
    volume: f64,
    rate: i32,
    channels: i32,
    settled: bool,
    frames: i32,
    starts: i32,
    ends: i32,
    held_volume: f64,
    problem: QString,
    player: Option<InterfacePlayer>,
    samples: Vec<u8>,
    opening: u64,
}

impl Default for SplashSoundRust {
    fn default() -> Self {
        SplashSoundRust {
            platform_library: QString::default(),
            folder: QString::default(),
            source: QUrl::default(),
            file: std::ptr::null_mut(),
            on: true,
            volume: 1.0,
            rate: emusen_platform_client::sounds::RATE as i32,
            channels: emusen_platform_client::sounds::CHANNELS as i32,
            settled: false,
            frames: 0,
            starts: 0,
            ends: 0,
            held_volume: -1.0,
            problem: QString::default(),
            player: None,
            samples: Vec::new(),
            opening: 0,
        }
    }
}

/// Whether a sound's URL is a file of the theme's own folder, as the host's policy keeps a theme to its folder.
pub fn within(folder: &str, source: &str) -> bool {
    if folder.trim_end_matches('/').is_empty() {
        return false;
    }
    let folder = emusen_theme_host::host::folder_url(folder);
    source.strip_prefix(&folder).is_some_and(|rest| rest.starts_with('/') && !rest.split('/').any(|part| part == ".."))
}

/// The samples' last `fade` of a share from `at` bytes on, each frame scaled down a raised cosine to silence.
pub fn faded(samples: &[u8], at: usize, fade_frames: usize) -> Vec<u8> {
    let at = at - at % FRAME;
    let end = samples.len().min(at + fade_frames * FRAME);
    let mut out = Vec::with_capacity(end.saturating_sub(at));
    for (i, frame) in samples.get(at..end).unwrap_or_default().chunks_exact(FRAME).enumerate() {
        let gain = 0.5 + 0.5 * (std::f32::consts::PI * i as f32 / fade_frames.max(1) as f32).cos();
        for sample in frame.chunks_exact(4) {
            let value = f32::from_le_bytes([sample[0], sample[1], sample[2], sample[3]]) * gain;
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
    out
}

impl cxx_qt::Initialize for qobject::SplashSound {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().on_volume_changed(|sound| sound.hold_volume()).release();
    }
}

impl qobject::SplashSound {
    pub fn keep(mut self: Pin<&mut Self>) {
        // Safety: the file is the window's, alive as long as this sound, which reads it on the GUI thread.
        let file = unsafe { self.file.as_ref() };
        let samples = file.map(|f| qobject::sound_file_samples(f).as_slice().to_vec()).unwrap_or_default();
        let unread = file.map(|f| qobject::sound_file_problem(f).to_string()).unwrap_or_default();
        let (library, source, folder) = (self.platform_library.to_string(), self.source.to_string(), self.folder.to_string());
        if samples.is_empty() || library.is_empty() || !self.on {
            self.as_mut().settle(0, unread);
            return;
        }
        if !within(&folder, &source) {
            self.as_mut().settle(0, format!("The theme's sound {source} is not in the theme's folder."));
            return;
        }
        let opening = self.opening + 1;
        self.as_mut().rust_mut().opening = opening;
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            // The stream opened by a frame of silence, so the start plays at once.
            let opened = bell_player::open_player(&library).and_then(|player| {
                player.remember(SOUND, &samples).and_then(|()| player.remember(SILENCE, &[0u8; FRAME])).and_then(|()| player.play(SILENCE)).map(|()| player).map_err(|e| e.to_string())
            });
            let _ = qt.queue(move |mut sound: Pin<&mut qobject::SplashSound>| {
                if sound.opening != opening {
                    return;
                }
                match opened {
                    Ok(player) => {
                        let frames = (samples.len() / FRAME) as i32;
                        sound.as_mut().rust_mut().player = Some(player);
                        sound.as_mut().rust_mut().samples = samples;
                        sound.as_mut().hold_volume();
                        sound.settle(frames, String::new());
                    }
                    Err(words) => sound.settle(0, words),
                }
            });
        });
    }

    pub fn start(mut self: Pin<&mut Self>) {
        if !self.on {
            return;
        }
        let Some(played) = self.player.as_ref().map(|p| p.play(SOUND)) else { return };
        match played {
            Ok(()) => {
                let starts = self.starts + 1;
                self.as_mut().rust_mut().starts = starts;
                self.as_mut().starts_changed();
            }
            Err(e) => self.publish(e.to_string()),
        }
    }

    pub fn end(mut self: Pin<&mut Self>, fade: i32) {
        let Some(player) = self.player.as_ref() else { return };
        let queued = player.queued().unwrap_or(0).max(0) as usize;
        if self.starts == 0 || queued == 0 {
            return;
        }
        let at = self.samples.len().saturating_sub(queued);
        let tail = faded(&self.samples, at, (fade.max(0) as u64 * u64::from(emusen_platform_client::sounds::RATE) / 1000) as usize);
        let ended = player.remember(ENDING, &tail).and_then(|()| if tail.is_empty() { Ok(()) } else { player.play(ENDING) });
        match ended {
            Ok(()) => {
                let ends = self.ends + 1;
                self.as_mut().rust_mut().ends = ends;
                self.as_mut().ends_changed();
            }
            Err(e) => self.publish(e.to_string()),
        }
    }

    /// The player set to the Sound volume, which it then holds; none before it is made.
    fn hold_volume(mut self: Pin<&mut Self>) {
        let asked = self.volume.clamp(0.0, 1.0);
        match self.player.as_ref().map(|p| p.set_volume(asked as f32).map_err(|e| e.to_string())) {
            Some(Err(words)) => self.publish(words),
            Some(Ok(())) => {
                if self.held_volume != asked {
                    self.as_mut().rust_mut().held_volume = asked;
                    self.held_volume_changed();
                }
            }
            None => {}
        }
    }

    fn settle(mut self: Pin<&mut Self>, frames: i32, problem: String) {
        if self.frames != frames {
            self.as_mut().rust_mut().frames = frames;
            self.as_mut().frames_changed();
        }
        self.as_mut().publish(problem);
        if !self.settled {
            self.as_mut().rust_mut().settled = true;
            self.settled_changed();
        }
    }

    fn publish(mut self: Pin<&mut Self>, problem: String) {
        let problem = QString::from(problem.as_str());
        if self.problem != problem {
            self.as_mut().rust_mut().problem = problem;
            self.problem_changed();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sound_is_the_theme_s_only_inside_its_folder() {
        assert!(within("/t/eclipse", "file:///t/eclipse/sounds/splash.wav"));
        assert!(within("qrc:/emusen/themes/eclipse/", "qrc:/emusen/themes/eclipse/sounds/splash.wav"));
        assert!(!within("/t/eclipse", "file:///t/eclipse-other/splash.wav"));
        assert!(!within("/t/eclipse", "file:///t/eclipse/../x.wav"));
        assert!(!within("", "file:///splash.wav"));
    }

    #[test]
    fn the_soft_end_falls_from_where_the_sound_plays_to_silence() {
        let one = 0.5f32.to_le_bytes();
        let samples: Vec<u8> = std::iter::repeat_n(one, 2 * 100).flatten().collect();
        let tail = faded(&samples, 10 * FRAME + 3, 50);
        assert_eq!(tail.len(), 50 * FRAME, "from the frame it plays, the fade's length");
        let first = f32::from_le_bytes(tail[..4].try_into().unwrap());
        let last = f32::from_le_bytes(tail[tail.len() - 4..].try_into().unwrap());
        assert!((first - 0.5).abs() < 1e-6 && last.abs() < 0.001, "{first} to {last}");
        assert_eq!(faded(&samples, samples.len(), 50), Vec::<u8>::new(), "nothing past the end");
    }
}
