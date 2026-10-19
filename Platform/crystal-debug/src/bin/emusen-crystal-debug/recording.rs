//! The recorder's verbs: a recording started, frames captured as a given picture, stopped, and what it left read back
//! with the local time taken out. `CrystalDebugScript.Recording` runs the same verbs over the C# `FrameRecorder`.

use emusen_recorder::Recorder;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct RecordingScript {
    recorder: Recorder,
    folder: Option<PathBuf>,
}

/// The recording's folder in an answer, by a word that does not hold its time.
fn masked(text: &str, folder: &Path) -> String {
    text.replace(&folder.display().to_string(), "<folder>")
}

impl RecordingScript {
    pub fn run(&mut self, verb: &str, rest: &str) -> Option<String> {
        let words: Vec<&str> = rest.split(' ').filter(|w| !w.is_empty()).collect();
        let w = |i: usize| words.get(i).copied().unwrap_or("");
        Some(match verb {
            "rec.start" => match self.recorder.start(Path::new(w(0)), "fake", w(1).parse().unwrap_or(1)) {
                Ok(folder) => {
                    let name = folder.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    self.folder = Some(folder);
                    format!("ok {}", name.split('_').next().unwrap_or(""))
                }
                Err(e) => format!("! {e}"),
            },
            "rec.recording" => self.recorder.is_recording().to_string(),
            "rec.capture" => {
                let picture = w(1).to_string();
                self.recorder.capture(w(0).parse().unwrap_or(0), "fake", |path| {
                    let _ = std::fs::copy(&picture, path);
                });
                "ok".into()
            }
            "rec.stop" => {
                let mut said = Vec::new();
                match self.recorder.stop(&mut said) {
                    Some(folder) => said.iter().filter(|l| l.starts_with("[RECORD]")).map(|l| masked(l, &folder)).collect::<Vec<_>>().join(" | "),
                    None => "not recording".into(),
                }
            }
            "rec.files" => {
                let Some(folder) = &self.folder else { return Some("none".into()) };
                let mut names: Vec<String> = std::fs::read_dir(folder)
                    .map(|d| {
                        d.filter_map(|e| e.ok())
                            .map(|e| {
                                let name = e.file_name().to_string_lossy().into_owned();
                                let size = e.metadata().map_or(0, |m| m.len());
                                if name.ends_with(".mkv") || name.ends_with(".zip") { format!("{name}:{}", size > 0) } else { format!("{name}:{size}") }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                names.sort();
                names.join(",")
            }
            "rec.ledger" => {
                let Some(folder) = &self.folder else { return Some("none".into()) };
                std::fs::read_to_string(folder.join("frames.log"))
                    .unwrap_or_default()
                    .lines()
                    .map(|l| {
                        let mut f: Vec<&str> = l.split('\t').collect();
                        if f.len() > 1 && f[0] != "Frame" {
                            f[1] = "-";
                        }
                        f.join(" ")
                    })
                    .collect::<Vec<_>>()
                    .join("|")
            }
            _ => return None,
        })
    }
}
