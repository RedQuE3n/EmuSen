// Audio capture format - see EmuSen_Debugging_Tools_Reference_v5.md §3.51.
//
// A run always lands on WAV first and is re-encoded afterwards. Which side writes
// that WAV depends on the backend, and both paths now exist: a backend that drives
// an emulator with its own recorder hands it a path, and a backend that is *handed
// samples* writes the container here with write_wav(). The second case is what
// libretro needs, and until 2026-08-09 its absence was misread as libretro being
// unable to supply audio at all - see Mercury_Gameplan.md §3.1.
//
// FLAC is the default because a 20-second gameplay capture is 3.8 MB of WAV and
// 553 KB of FLAC - measured, 14.4% - and nothing downstream reads the file as
// anything but sample data.
//
// WAV survives as a deliberate fallback, on two paths. `--wav` asks for it
// outright, which is what deep troubleshooting wants: every language reads it
// with no dependency, and `xxd` on a WAV is a real debugging step. And an encode
// that fails *or does not verify* leaves the WAV in place rather than replacing
// it with something unproven.
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AudioFormat {
    Flac,
    Wav,
}

impl AudioFormat {
    // The extension the caller asked for wins, so `--audio out.wav` is honoured
    // without needing --wav as well.
    pub fn from_path(path: &str) -> AudioFormat {
        match Path::new(path).extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase) {
            Some(ext) if ext == "wav" => AudioFormat::Wav,
            _ => AudioFormat::Flac,
        }
    }
}

// Interleaved stereo i16 to a RIFF/WAVE file. A backend that receives samples
// rather than writing files needs this; see EmuSen_Debugging_Tools_Reference_v5.md §3.51.
pub fn write_wav(path: &Path, samples: &[i16], channels: u16, sample_rate: u32) -> std::result::Result<(), String> {
    let bits = 16u16;
    let block_align = channels * (bits / 8);
    let byte_rate = sample_rate * u32::from(block_align);
    let data_bytes = (samples.len() * 2) as u32;

    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&bits.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_bytes.to_le_bytes());
    for sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }

    std::fs::write(path, &out).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

// Where the backend is told to record. Always a WAV, because that is the only
// container the probe and the reference emulator agree on; the requested path is
// what the run ends up with.
pub fn capture_path(requested: &str, format: AudioFormat) -> PathBuf {
    match format {
        AudioFormat::Wav => PathBuf::from(requested),
        AudioFormat::Flac => PathBuf::from(format!("{requested}.capture.wav")),
    }
}

pub struct EncodeReport {
    pub encoded: bool,
    pub wav_bytes: u64,
    pub out_bytes: u64,
}

// Runs after the backend has stopped recording, so the WAV is complete and
// closed. Returns Ok(None) when nothing needed doing.
pub fn finish(requested: &str, format: AudioFormat) -> Option<EncodeReport> {
    if format == AudioFormat::Wav {
        return None;
    }

    let wav = capture_path(requested, format);
    let wav_bytes = std::fs::metadata(&wav).ok()?.len();

    let encoder = match find_encoder() {
        Some(encoder) => encoder,
        None => {
            keep_wav(&wav, requested, "no flac or ffmpeg on PATH");
            return Some(EncodeReport { encoded: false, wav_bytes, out_bytes: wav_bytes });
        }
    };

    if let Err(reason) = encoder.encode(&wav, Path::new(requested)) {
        keep_wav(&wav, requested, &reason);
        return Some(EncodeReport { encoded: false, wav_bytes, out_bytes: wav_bytes });
    }

    // Lossless is a claim, not an observation, until the samples come back out
    // the same. A codec that quietly resampled or clipped would otherwise be
    // discovered as an emulator bug months later.
    if let Err(reason) = verify(&encoder, Path::new(requested), &wav) {
        let _ = std::fs::remove_file(requested);
        keep_wav(&wav, requested, &reason);
        return Some(EncodeReport { encoded: false, wav_bytes, out_bytes: wav_bytes });
    }

    let out_bytes = std::fs::metadata(requested).map(|m| m.len()).unwrap_or(0);
    let _ = std::fs::remove_file(&wav);
    Some(EncodeReport { encoded: true, wav_bytes, out_bytes })
}

// The WAV is moved to the requested name so a run always leaves exactly one
// audio file where the caller asked for it, whatever happened.
fn keep_wav(wav: &Path, requested: &str, reason: &str) {
    println!("[WARN] keeping uncompressed audio ({reason})");
    if wav != Path::new(requested) {
        let fallback = format!("{}.wav", strip_extension(requested));
        if std::fs::rename(wav, &fallback).is_ok() {
            println!("[INFO] audio -> {fallback}");
        }
    }
}

fn strip_extension(path: &str) -> String {
    match path.rsplit_once('.') {
        Some((stem, _)) => stem.to_string(),
        None => path.to_string(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Encoder {
    Flac,
    Ffmpeg,
}

pub fn find_encoder() -> Option<Encoder> {
    // flac(1) first: it is the reference implementation and its -8 is smaller
    // than ffmpeg's default. ffmpeg is the fallback because the project already
    // depends on it for frame recording.
    for (name, encoder) in [("flac", Encoder::Flac), ("ffmpeg", Encoder::Ffmpeg)] {
        if Command::new(name).arg("-version").output().is_ok_and(|o| o.status.success()) {
            return Some(encoder);
        }
    }
    None
}

impl Encoder {
    fn encode(self, wav: &Path, out: &Path) -> std::result::Result<(), String> {
        let status = match self {
            Encoder::Flac => Command::new("flac")
                .args(["-8", "-s", "-f", "-o"])
                .arg(out)
                .arg(wav)
                .status(),
            Encoder::Ffmpeg => Command::new("ffmpeg")
                .args(["-y", "-loglevel", "error", "-i"])
                .arg(wav)
                .args(["-c:a", "flac", "-compression_level", "12"])
                .arg(out)
                .status(),
        };
        match status {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(format!("encoder exited {status}")),
            Err(error) => Err(format!("could not run encoder: {error}")),
        }
    }

    fn decode_to(self, input: &Path, out: &Path) -> std::result::Result<(), String> {
        let status = match self {
            Encoder::Flac => Command::new("flac").args(["-d", "-s", "-f", "-o"]).arg(out).arg(input).status(),
            Encoder::Ffmpeg => Command::new("ffmpeg")
                .args(["-y", "-loglevel", "error", "-i"])
                .arg(input)
                .arg(out)
                .status(),
        };
        match status {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(format!("decoder exited {status}")),
            Err(error) => Err(format!("could not run decoder: {error}")),
        }
    }
}

// Compares the decoded PCM against the original, not the container: a WAV
// written by a different tool carries different chunk padding and would fail a
// whole-file comparison while being sample-identical.
fn verify(encoder: &Encoder, encoded: &Path, original: &Path) -> std::result::Result<(), String> {
    let round_trip = original.with_extension("verify.wav");
    encoder.decode_to(encoded, &round_trip)?;

    let result = match (read_pcm(original), read_pcm(&round_trip)) {
        (Ok(a), Ok(b)) if a == b => Ok(()),
        (Ok(a), Ok(b)) => Err(format!("round-trip differs: {} vs {} sample bytes", a.len(), b.len())),
        (Err(e), _) | (_, Err(e)) => Err(e),
    };
    let _ = std::fs::remove_file(&round_trip);
    result
}

// The data chunk of a RIFF/WAVE file, found by walking the chunk list rather
// than assuming it starts at byte 44 - which is true of Mesen's writer and not
// of every decoder's.
pub fn read_pcm(path: &Path) -> std::result::Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(format!("{} is not a RIFF/WAVE file", path.display()));
    }

    let mut at = 12;
    while at + 8 <= bytes.len() {
        let id = &bytes[at..at + 4];
        let size = u32::from_le_bytes([bytes[at + 4], bytes[at + 5], bytes[at + 6], bytes[at + 7]]) as usize;
        let body = at + 8;
        if id == b"data" {
            let end = body.saturating_add(size).min(bytes.len());
            return Ok(bytes[body..end].to_vec());
        }
        at = body + size + (size & 1);
    }
    Err(format!("{} has no data chunk", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_follows_the_requested_extension() {
        assert_eq!(AudioFormat::from_path("out.flac"), AudioFormat::Flac);
        assert_eq!(AudioFormat::from_path("out.wav"), AudioFormat::Wav);
        assert_eq!(AudioFormat::from_path("out.WAV"), AudioFormat::Wav);
        // No extension, or an unfamiliar one, takes the default rather than
        // silently writing a WAV under a name that says otherwise.
        assert_eq!(AudioFormat::from_path("out"), AudioFormat::Flac);
        assert_eq!(AudioFormat::from_path("out.oga"), AudioFormat::Flac);
    }

    #[test]
    fn capture_is_a_wav_whatever_was_asked_for() {
        assert_eq!(capture_path("a/b.flac", AudioFormat::Flac), PathBuf::from("a/b.flac.capture.wav"));
        assert_eq!(capture_path("a/b.wav", AudioFormat::Wav), PathBuf::from("a/b.wav"));
    }

    #[test]
    fn strip_extension_keeps_directories() {
        assert_eq!(strip_extension("/tmp/run/out.flac"), "/tmp/run/out");
        assert_eq!(strip_extension("noext"), "noext");
    }

    // A data chunk that does not start at byte 44, which is what a LIST-carrying
    // decoder emits and what a naive reader gets wrong.
    #[test]
    fn read_pcm_walks_the_chunk_list() {
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&0u32.to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&[0u8; 16]);
        wav.extend_from_slice(b"LIST");
        wav.extend_from_slice(&5u32.to_le_bytes());
        wav.extend_from_slice(b"hello");
        wav.push(0); // odd-sized chunks are padded
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&4u32.to_le_bytes());
        wav.extend_from_slice(&[1, 2, 3, 4]);

        let path = std::env::temp_dir().join("emusen-probe-chunkwalk.wav");
        std::fs::write(&path, &wav).unwrap();
        assert_eq!(read_pcm(&path).unwrap(), vec![1, 2, 3, 4]);
        let _ = std::fs::remove_file(&path);
    }

    // The probe now writes the container it used to only read, so the two must agree.
    #[test]
    fn a_written_wav_reads_back_through_the_chunk_walker() {
        let samples: Vec<i16> = vec![0, -1, 32767, -32768, 1234, -1234];
        let path = std::env::temp_dir().join("emusen-probe-writewav.wav");

        write_wav(&path, &samples, 2, 32768).unwrap();

        let pcm = read_pcm(&path).unwrap();
        assert_eq!(pcm.len(), samples.len() * 2);

        let decoded: Vec<i16> =
            pcm.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();
        assert_eq!(decoded, samples);

        // The header must describe what was actually written, or a decoder resamples it.
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(u16::from_le_bytes([bytes[22], bytes[23]]), 2);
        assert_eq!(u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]), 32768);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn read_pcm_rejects_a_non_wave_file() {
        let path = std::env::temp_dir().join("emusen-probe-notwave.bin");
        std::fs::write(&path, b"this is not a wave file at all").unwrap();
        assert!(read_pcm(&path).is_err());
        let _ = std::fs::remove_file(&path);
    }
}
