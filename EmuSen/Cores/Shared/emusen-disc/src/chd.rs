//! CHD v5 compact discs through the `chd` crate, an unread dependency (Diamond_Plan.md §11 Q9). The layout a CD CHD
//! holds was measured from chdman 0.289's output on synthetic sheets (EmuSen_Disc.md §5): units of 2,352 bytes and 96
//! of subcode, each track padded to a multiple of four units, CHT2 metadata per track, audio stored big-endian, a
//! cooked mode 1 track's 2,048 bytes at the start of its unit, and a `V` pregap type meaning the pregap is stored.

use std::io::Cursor;

use crate::cue::Mode;
use crate::disc::Track;
use crate::msf::PROGRAM_START;
use crate::subq::{CONTROL_AUDIO, CONTROL_DATA};

/// A CD CHD's unit: the sector and its subcode.
pub const UNIT: usize = 2448;
/// The CD-ROM track metadata tags, CHT2 and the older CHTR.
const CHT2: u32 = u32::from_be_bytes(*b"CHT2");
const CHTR: u32 = u32::from_be_bytes(*b"CHTR");

pub struct ChdImage {
    chd: chd::Chd<Cursor<Vec<u8>>>,
    /// The hunk decompressed last, by number, and the compressed bytes' scratch.
    hunk: Vec<u8>,
    at: Option<u32>,
    scratch: Vec<u8>,
    units_per_hunk: u64,
}

/// One CHT2 entry's fields.
struct Entry {
    track: u8,
    mode: Mode,
    frames: u32,
    pregap: u32,
    pregap_stored: bool,
    postgap: u32,
}

fn mode_of(kind: &str) -> Option<Mode> {
    Some(match kind {
        "AUDIO" => Mode::Audio,
        "MODE1" | "MODE1/2048" | "MODE2_FORM1" => Mode::Mode1User,
        "MODE1_RAW" | "MODE1/2352" => Mode::Mode1Raw,
        "MODE2" | "MODE2/2336" | "MODE2_FORM_MIX" => Mode::Mode2Xa,
        "MODE2_RAW" | "MODE2/2352" | "CDI/2352" => Mode::Mode2Raw,
        _ => return None,
    })
}

fn entry(text: &str) -> Option<Entry> {
    let field = |key: &str| text.split_whitespace().find_map(|w| w.strip_prefix(key)).map(|v| v.trim_end_matches('\0'));
    let number = |key: &str| field(key).and_then(|v| v.parse::<u32>().ok());
    let pgtype = field("PGTYPE:").unwrap_or("");
    Some(Entry {
        track: number("TRACK:")? as u8,
        mode: mode_of(field("TYPE:")?)?,
        frames: number("FRAMES:")?,
        pregap: number("PREGAP:").unwrap_or(0),
        pregap_stored: pgtype.starts_with('V'),
        postgap: number("POSTGAP:").unwrap_or(0),
    })
}

impl ChdImage {
    /// The image, its tracks on the disc's time line and its lead-out.
    pub fn open(bytes: Vec<u8>) -> Result<(ChdImage, Vec<Track>, u32), String> {
        let mut chd = chd::Chd::open(Cursor::new(bytes), None).map_err(|e| format!("the CHD could not be opened: {e}"))?;
        let hunk_size = chd.header().hunk_size() as usize;
        if chd.header().unit_bytes() as usize != UNIT || hunk_size % UNIT != 0 {
            return Err(format!("the CHD's units are {} bytes, not a compact disc's {UNIT}", chd.header().unit_bytes()));
        }
        let refs: Vec<_> = chd.metadata_refs().collect();
        let mut entries = Vec::new();
        for r in refs {
            let m = r.read(chd.inner()).map_err(|e| format!("the CHD's metadata could not be read: {e}"))?;
            if m.metatag == CHT2 || m.metatag == CHTR {
                let text = String::from_utf8_lossy(&m.value).into_owned();
                entries.push(entry(&text).ok_or_else(|| format!("the CHD's track entry is not understood: {}", text.trim_end_matches('\0')))?);
            }
        }
        if entries.is_empty() {
            return Err("the CHD holds no CD-ROM track metadata: not a compact disc".into());
        }
        entries.sort_by_key(|e| e.track);
        let mut tracks = Vec::new();
        let (mut unit, mut cursor) = (0u64, 0u32);
        for (k, e) in entries.iter().enumerate() {
            let unstored = if e.pregap_stored { 0 } else { e.pregap };
            let from = if k == 0 { PROGRAM_START - if e.pregap_stored { e.pregap } else { 0 } } else { cursor + unstored };
            let to = from + e.frames;
            let start = if e.pregap_stored { from + e.pregap } else { from };
            let end = to + e.postgap;
            tracks.push(Track::stored(e.track, e.mode, if e.mode.audio() { CONTROL_AUDIO } else { CONTROL_DATA }, if k == 0 { 0 } else { cursor }, start, end, 0, from, to, unit));
            unit += e.frames.div_ceil(4) as u64 * 4;
            cursor = end;
        }
        Ok((ChdImage { hunk: vec![0; hunk_size], at: None, scratch: Vec::new(), units_per_hunk: (hunk_size / UNIT) as u64, chd }, tracks, cursor))
    }

    /// Unit `n`'s 2,448 bytes.
    pub fn unit(&mut self, n: u64) -> Result<&[u8], String> {
        let h = (n / self.units_per_hunk) as u32;
        if self.at != Some(h) {
            self.at = None;
            let mut hunk = self.chd.hunk(h).map_err(|e| e.to_string())?;
            hunk.read_hunk_in(&mut self.scratch, &mut self.hunk).map_err(|e| e.to_string())?;
            self.at = Some(h);
        }
        let o = (n % self.units_per_hunk) as usize * UNIT;
        Ok(&self.hunk[o..o + UNIT])
    }
}
