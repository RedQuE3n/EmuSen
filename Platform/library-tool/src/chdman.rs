//! chdman run as a program: its version line, `createcd`, `verify` and `info`, each stopped when the batch is. What
//! each command does is read from MAME's documentation, "chdman", MAME Documentation 0.289; nothing of its source is
//! read. See EmuSen_GamePackages.md §4.1 and §12.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// A pinned chdman, its SHA-256 checked and its version line read.
#[derive(Clone, Debug)]
pub struct Chdman {
    pub path: String,
    pub sha256: String,
    /// What follows "manager " in the line chdman prints first, as `0.289 (unknown)`.
    pub version: String,
}

/// What `chdman info` reports of a CHD that the manifest records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Info {
    pub sha1: String,
    pub data_sha1: String,
    /// The compression line as chdman words it.
    pub compression: String,
}

/// A run's end: its exit status, and the last of what it printed, both streams.
struct Ran {
    ok: bool,
    text: String,
}

/// The last `KEEP` bytes of a stream: chdman redraws its progress on one line, so what it prints is not kept whole.
const KEEP: usize = 64 * 1024;

fn drain(mut from: impl Read + Send + 'static) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let (mut kept, mut buffer) = (Vec::new(), [0u8; 8192]);
        while let Ok(n) = from.read(&mut buffer) {
            if n == 0 {
                break;
            }
            kept.extend_from_slice(&buffer[..n]);
            if kept.len() > 2 * KEEP {
                kept.drain(..kept.len() - KEEP);
            }
        }
        kept
    })
}

fn wait(mut child: Child, stop: &dyn Fn() -> bool) -> Result<Ran, String> {
    let out = drain(child.stdout.take().expect("piped"));
    let err = drain(child.stderr.take().expect("piped"));
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            break Some(status);
        }
        if stop() {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let mut text = String::from_utf8_lossy(&out.join().unwrap_or_default()).into_owned();
    text.push_str(&String::from_utf8_lossy(&err.join().unwrap_or_default()));
    match status {
        None => Err("stopped".into()),
        Some(status) => Ok(Ran { ok: status.success(), text }),
    }
}

/// The last line of what chdman printed that says something, for words that name its failure.
fn last_words(text: &str) -> String {
    text.split(['\n', '\r']).map(str::trim).filter(|l| !l.is_empty()).last().unwrap_or("no output").to_string()
}

impl Chdman {
    /// The binary at `path`, refused unless its SHA-256 is `expected`; then run with no arguments for its version line.
    pub fn open(path: &str, expected: Option<&str>, release: &str) -> Result<Chdman, String> {
        let sha256 = crate::pin::check(path, expected, release)?;
        let output = Command::new(path).stdin(Stdio::null()).output().map_err(|e| format!("chdman at {path} could not be run: {e}"))?;
        let text = String::from_utf8_lossy(&output.stdout).into_owned() + &String::from_utf8_lossy(&output.stderr);
        let first = text.lines().find(|l| l.contains("chdman")).unwrap_or("");
        let version = first.split_once("manager ").map(|(_, v)| v.trim().to_string()).ok_or_else(|| format!("chdman at {path} printed no version line: {}", last_words(&text)))?;
        Ok(Chdman { path: path.to_string(), sha256, version })
    }

    /// The release alone, the version line's first word.
    pub fn release(&self) -> &str {
        self.version.split_whitespace().next().unwrap_or("")
    }

    fn run(&self, args: &[&str], stop: &dyn Fn() -> bool) -> Result<Ran, String> {
        let child = Command::new(&self.path).args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("chdman could not be run: {e}"))?;
        wait(child, stop)
    }

    /// `createcd` from the sheet at `sheet` to `out`, with `codecs` where given (chdman's defaults otherwise) and at
    /// most `threads` compressing.
    pub fn create_cd(&self, sheet: &str, out: &str, codecs: Option<&str>, threads: usize, stop: &dyn Fn() -> bool) -> Result<(), String> {
        let threads = threads.max(1).to_string();
        let mut args = vec!["createcd", "-i", sheet, "-o", out, "-np", &threads];
        if let Some(codecs) = codecs {
            args.extend(["-c", codecs]);
        }
        let ran = self.run(&args, stop)?;
        if ran.ok { Ok(()) } else { Err(format!("chdman createcd failed: {}", last_words(&ran.text))) }
    }

    /// `verify`: every hunk decompressed and the overall SHA-1 compared. Passed only on a clean exit that says so.
    pub fn verify(&self, chd: &str, stop: &dyn Fn() -> bool) -> Result<(), String> {
        let ran = self.run(&["verify", "-i", chd], stop)?;
        if ran.ok && ran.text.contains("Overall SHA1 verification successful") { Ok(()) } else { Err(format!("chdman verify failed: {}", last_words(&ran.text))) }
    }

    /// `info`: the two SHA-1s and the compression.
    pub fn info(&self, chd: &str) -> Result<Info, String> {
        let ran = self.run(&["info", "-i", chd], &|| false)?;
        let field = |name: &str| ran.text.lines().find_map(|l| l.trim().strip_prefix(name)).map(|v| v.trim().to_string());
        match (ran.ok, field("SHA1:"), field("Data SHA1:"), field("Compression:")) {
            (true, Some(sha1), Some(data_sha1), Some(compression)) => Ok(Info { sha1, data_sha1, compression }),
            _ => Err(format!("chdman info failed: {}", last_words(&ran.text))),
        }
    }
}
