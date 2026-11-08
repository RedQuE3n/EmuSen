//! The recorder, the C# `FrameRecorder` (EmuSen_Debugging_Tools_Reference_v5.md §3.8): a folder named for the engine
//! and the local time, a ledger of the frames kept, every Nth drawn frame's picture written by the caller into the path
//! the recorder names, and on stopping the pictures encoded by ffmpeg, losslessly, and archived. Every line the C#
//! wrote to the console is said, for the runtime's log (EmuSen_Backend.md §29.11 Q2, §29.36).

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
        let mut out = zip::ZipWriter::new(File::create(&zip_path)?);
        for p in &pictures {
            let (y, mo, d, h, mi, s, _) = local_of(std::fs::metadata(p)?.modified()?);
            let time = u16::try_from(y).ok().and_then(|y| zip::DateTime::from_date_and_time(y, mo as u8, d as u8, h as u8, mi as u8, s as u8).ok()).unwrap_or_default();
            let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated).compression_level(Some(9)).last_modified_time(time);
            out.start_file(p.file_name().unwrap().to_string_lossy(), options)?;
            out.write_all(&std::fs::read(p)?)?;
        }
        out.finish()?.sync_all()?;
        for p in &pictures {
            std::fs::remove_file(p)?;
        }
        Ok(())
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(&zip_path);
    }
    match written {
        Ok(()) => said.push(format!("[RECORD] Archived {} frame PNGs -> {}", pictures.len(), zip_path.display())),
        Err(e) => said.push(format!("[RECORD] Zipping frame PNGs failed ({e}); leaving them as loose files in {}", folder.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pictures_are_archived_with_their_local_times_and_removed() {
        let folder = std::env::temp_dir().join(format!("emusen-recorder-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        let frames = [("frame_00000.png", vec![1u8; 1500]), ("frame_00001.png", (0..1500u32).map(|i| i as u8).collect())];
        for (name, bytes) in &frames {
            std::fs::write(folder.join(name), bytes).unwrap();
        }
        std::fs::write(folder.join("frames.log"), "kept").unwrap();
        let (y, mo, d, h, mi, s, _) = local_of(std::fs::metadata(folder.join(frames[0].0)).unwrap().modified().unwrap());
        let mut said = Vec::new();
        archive(&folder, &mut said);
        assert!(said[0].starts_with("[RECORD] Archived 2 frame PNGs"), "{said:?}");
        let mut zip = zip::ZipArchive::new(File::open(folder.join("frames.zip")).unwrap()).unwrap();
        let mut names: Vec<String> = zip.file_names().map(|n| n.unwrap().into_owned()).collect();
        names.sort();
        assert_eq!(names, ["frame_00000.png", "frame_00001.png"]);
        for (name, bytes) in &frames {
            let mut entry = zip.by_name(name).unwrap();
            let mut read = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut read).unwrap();
            assert_eq!(&read, bytes);
            assert!(!folder.join(name).exists());
        }
        let t = zip.by_name(frames[0].0).unwrap().last_modified().unwrap();
        assert_eq!((t.year() as i64, t.month() as u32, t.day() as u32, t.hour() as u32, t.minute() as u32, t.second() as u32), (y, mo, d, h, mi, s / 2 * 2));
        assert!(folder.join("frames.log").exists());
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
