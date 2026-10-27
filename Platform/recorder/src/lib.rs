//! The recorder, the C# `FrameRecorder` (EmuSen_Debugging_Tools_Reference_v5.md §3.8): a folder named for the engine
//! and the local time, a ledger of the frames kept, every Nth drawn frame's picture written by the caller into the path
//! the recorder names, and on stopping the pictures encoded by ffmpeg, losslessly, and archived. Every line the C#
//! wrote to the console is said, for the runtime's log (EmuSen_Backend.md §29.11 Q2, §29.36).

mod zip;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A convenience encode, not a timing-accurate export.
const ASSUMED_FPS: f64 = 60.0;

/// Lossless either way: libx264rgb first, ffv1 where the build lacks it.
const CODECS: [&[&str]; 2] = [&["-c:v", "libx264rgb", "-crf", "0", "-preset", "slow"], &["-c:v", "ffv1"]];

/// The local time's fields: year, month, day, hour, minute, second, millisecond.
pub fn local_now() -> (i64, u32, u32, u32, u32, u32, u32) {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    local(now.as_secs() as i64, now.subsec_millis())
}

fn local(seconds: i64, millis: u32) -> (i64, u32, u32, u32, u32, u32, u32) {
    let (y, mo, d, h, mi, s) = emusen_dotnet::date::local_fields(seconds);
    (y, mo, d, h, mi, s, millis)
}

/// The fields of a file's time in the local zone.
fn local_of(time: std::time::SystemTime) -> (i64, u32, u32, u32, u32, u32, u32) {
    let at = time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    local(at.as_secs() as i64, at.subsec_millis())
}

/// A recording in progress, or none.
#[derive(Default)]
pub struct Recorder {
    folder: Option<PathBuf>,
    ledger: Option<File>,
    stride: u64,
    seen: u64,
}

impl Recorder {
    pub fn new() -> Recorder {
        Recorder::default()
    }

    pub fn is_recording(&self) -> bool {
        self.folder.is_some()
    }

    pub fn folder(&self) -> Option<&Path> {
        self.folder.as_deref()
    }

    /// `Start`: a folder `<core>_<yyyyMMdd_HHmmss>` under `base`, its ledger begun; every `stride`th frame kept.
    pub fn start(&mut self, base: &Path, core_name: &str, stride: u32) -> Result<PathBuf, String> {
        if self.is_recording() {
            return Err("Already recording - call Stop() first.".to_string());
        }
        self.stride = stride.max(1) as u64;
        self.seen = 0;
        let (y, mo, d, h, mi, s, _) = local_now();
        let folder = base.join(format!("{core_name}_{y:04}{mo:02}{d:02}_{h:02}{mi:02}{s:02}"));
        std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
        let mut ledger = File::create(folder.join("frames.log")).map_err(|e| e.to_string())?;
        ledger.write_all(b"Frame\tWallClock\tCore\tImageFile\n").map_err(|e| e.to_string())?;
        self.ledger = Some(ledger);
        self.folder = Some(folder.clone());
        Ok(folder)
    }

    /// `CaptureFrame`: called each drawn frame; every `stride`th has its picture written by `capture` into the path named.
    pub fn capture(&mut self, frame: i64, core_name: &str, capture: impl FnOnce(&Path)) {
        let Some(folder) = &self.folder else { return };
        let keep = self.seen % self.stride == 0;
        self.seen += 1;
        if !keep {
            return;
        }
        let image = format!("frame_{frame:08}.png");
        capture(&folder.join(&image));
        let (y, mo, d, h, mi, s, ms) = local_now();
        if let Some(ledger) = &mut self.ledger {
            let _ = writeln!(ledger, "{frame}\t{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}.{ms:03}\t{core_name}\t{image}");
        }
    }

    /// `Stop`: the ledger closed, the video encoded and the pictures archived; each line said. The folder, or none.
    pub fn stop(&mut self, said: &mut Vec<String>) -> Option<PathBuf> {
        self.ledger = None;
        let folder = self.folder.take()?;
        encode(&folder, self.stride, said);
        Some(folder)
    }
}

/// `TryEncodeVideo`: ffmpeg in the folder over its PNGs, the fallback codec when the first fails.
fn encode(folder: &Path, stride: u64, said: &mut Vec<String>) {
    let output = folder.join("recording.mkv");
    let fps = ASSUMED_FPS / stride.max(1) as f64;
    for (attempt, codec) in CODECS.iter().enumerate() {
        let mut command = Command::new("ffmpeg");
        command.current_dir(folder).args(["-y", "-pattern_type", "glob", "-framerate", &fps.to_string(), "-i", "frame_*.png"]).args(*codec).arg("recording.mkv");
        command.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());
        let run = match command.output() {
            Ok(run) => run,
            Err(_) => {
                said.push(format!(
                    "[RECORD] ffmpeg not found on PATH - leaving the PNG sequence + frames.log as-is in {}. Install ffmpeg for automatic video output.",
                    folder.display()
                ));
                return;
            }
        };
        if run.status.success() {
            said.push(format!("[RECORD] Video encoded ({}) -> {}", codec[1], output.display()));
            archive(folder, said);
            return;
        }
        let more = attempt < CODECS.len() - 1;
        let code = run.status.code().unwrap_or(-1);
        said.push(format!("[RECORD] ffmpeg -c:v {} exited with code {code}{}", codec[1], if more { " - trying fallback codec" } else { "" }));
        if !more {
            said.push(String::from_utf8_lossy(&run.stderr).into_owned());
            said.push(format!("[RECORD] PNG sequence + frames.log are still in {}", folder.display()));
        }
    }
}

/// `ZipAndCleanupFrames`: the pictures, redundant after a lossless encode, archived rather than deleted.
fn archive(folder: &Path, said: &mut Vec<String>) {
    let pictures: Vec<PathBuf> = match std::fs::read_dir(folder) {
        Ok(entries) => entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("frame_") && n.ends_with(".png")))
            .collect(),
        Err(_) => return,
    };
    if pictures.is_empty() {
        return;
    }
    let zip_path = folder.join("frames.zip");
    let written = (|| -> std::io::Result<()> {
        let mut entries = Vec::with_capacity(pictures.len());
        for p in &pictures {
            let bytes = std::fs::read(p)?;
            let time = local_of(std::fs::metadata(p)?.modified()?);
            entries.push((p.file_name().unwrap().to_string_lossy().into_owned(), bytes, time));
        }
        std::fs::write(&zip_path, zip::write(&entries)?)?;
        for p in &pictures {
            std::fs::remove_file(p)?;
        }
        Ok(())
    })();
    match written {
        Ok(()) => said.push(format!("[RECORD] Archived {} frame PNGs -> {}", pictures.len(), zip_path.display())),
        Err(e) => said.push(format!("[RECORD] Zipping frame PNGs failed ({e}); leaving them as loose files in {}", folder.display())),
    }
}
