//! CHD v5 compact discs through the `chd` crate, an unread dependency (Diamond_Plan.md §11 Q9). The layout a CD CHD
//! holds was measured from chdman 0.289's output on synthetic sheets (EmuSen_Disc.md §5): units of 2,352 bytes and 96
//! of subcode, each track padded to a multiple of four units, CHT2 metadata per track, audio stored big-endian, a
//! cooked mode 1 track's 2,048 bytes at the start of its unit, and a `V` pregap type meaning the pregap is stored.
//! A hunk is checked before the crate decodes it and after (§13): its block within the file, a CD codec's stream
//! length within the block, and the map's CRC of the decoded bytes, so that a damaged hunk is refused and never panics.

use std::io::Cursor;

use crate::cue::Mode;
use crate::disc::Track;
use crate::msf::PROGRAM_START;
use crate::subq::{self, CONTROL_AUDIO, CONTROL_DATA};

/// A CD CHD's unit: the sector and its subcode.
pub const UNIT: usize = 2448;
/// The CD-ROM track metadata tags, CHT2 and the older CHTR.
const CHT2: u32 = u32::from_be_bytes(*b"CHT2");
const CHTR: u32 = u32::from_be_bytes(*b"CHTR");

pub struct ChdImage<B: AsRef<[u8]>> {
    chd: chd::Chd<Cursor<B>>,
    /// The hunk decompressed last, by number, and the compressed bytes' scratch.
    hunk: Vec<u8>,
    at: Option<u32>,
    scratch: Vec<u8>,
    units_per_hunk: u64,
    /// The file's length, which every stored block lies within.
    len: u64,
    /// For each of the header's four codecs, whether its blocks open with the frames' flags and the sector stream's
    /// length: cdlz, cdzl and cdzs, as measured (§13.2).
    framed: [bool; 4],
}

/// The CD codecs whose blocks open with the frames' flags and a length.
const FRAMED: [&[u8; 4]; 3] = [b"cdlz", b"cdzl", b"cdzs"];
/// How many times a self-reference is followed before the hunk is refused.
const REFERENCES: usize = 4;

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

impl<B: AsRef<[u8]>> ChdImage<B> {
    /// The image, its tracks on the disc's time line and its lead-out; the header, the map and the metadata are read
    /// here, and a hunk only when a sector in it is.
    pub fn open(bytes: B) -> Result<(ChdImage<B>, Vec<Track>, u32), String> {
        let len = bytes.as_ref().len() as u64;
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
        let framed = match chd.header() {
            chd::header::Header::V5Header(v) => v.compression.map(|c| FRAMED.contains(&&c.to_be_bytes())),
            _ => [false; 4],
        };
        Ok((ChdImage { hunk: vec![0; hunk_size], at: None, scratch: Vec::new(), units_per_hunk: (hunk_size / UNIT) as u64, len, framed, chd }, tracks, cursor))
    }

    /// The byte of the file where the hunk holding unit `n` is stored, compressed or not, by the hunk map (`chd`'s
    /// `Map::get_entry`): where the system must read before the unit can be decoded. `None` for a hunk that is a copy
    /// of another, here or in a parent, and for a map older than v5, whose entries this crate has not measured.
    pub fn place(&self, n: u64) -> Option<u64> {
        use chd::map::{CompressionTypeV5 as Kind, MapEntry};
        match self.chd.map().get_entry((n / self.units_per_hunk) as usize)? {
            MapEntry::V5Compressed(e) => match e.hunk_type().ok()? {
                Kind::CompressionType0 | Kind::CompressionType1 | Kind::CompressionType2 | Kind::CompressionType3 | Kind::CompressionNone => e.block_offset().ok(),
                _ => None,
            },
            MapEntry::V5Uncompressed(e) => e.block_offset().ok(),
            MapEntry::LegacyEntry(_) => None,
        }
    }

    /// Unit `n`'s 2,448 bytes, or the words for the damaged hunk that holds it.
    pub fn unit(&mut self, n: u64) -> Result<&[u8], String> {
        let h = (n / self.units_per_hunk) as u32;
        if self.at != Some(h) {
            self.at = None;
            let crc = self.check(h).map_err(|w| format!("hunk {h} of the CHD is damaged: {w}"))?;
            let mut hunk = self.chd.hunk(h).map_err(|e| format!("hunk {h} of the CHD does not decode: {e}"))?;
            hunk.read_hunk_in(&mut self.scratch, &mut self.hunk).map_err(|e| format!("hunk {h} of the CHD does not decode: {e}"))?;
            if let Some(want) = crc {
                let got = subq::ccitt(0xFFFF, &self.hunk);
                if got != want {
                    return Err(format!("hunk {h} of the CHD is damaged: it decodes to bytes whose CRC is {got:04X}h, and the map records {want:04X}h"));
                }
            }
            self.at = Some(h);
        }
        let o = (n % self.units_per_hunk) as usize * UNIT;
        Ok(&self.hunk[o..o + UNIT])
    }

    /// Hunk `h` checked before it is decoded: the stored block it decodes (its own, or the one its self-references
    /// lead to) lies within the file, and a framed codec's sector stream within the block. Answers the CRC the map
    /// records for the decoded bytes, where it records one.
    fn check(&mut self, h: u32) -> Result<Option<u16>, String> {
        use chd::map::{CompressionTypeV5 as Kind, MapEntry};
        let (count, hunk_bytes) = (self.chd.header().hunk_count(), self.hunk.len());
        let mut at = h;
        for _ in 0..REFERENCES {
            let entry = self.chd.map().get_entry(at as usize).ok_or_else(|| format!("the map has no entry for hunk {at}"))?;
            let unread = |e: chd::Error| format!("the map's entry for hunk {at} does not read: {e}");
            let (codec, offset, size, crc) = match entry {
                MapEntry::V5Compressed(e) => {
                    let codec = match e.hunk_type().map_err(unread)? {
                        Kind::CompressionType0 => Some(0),
                        Kind::CompressionType1 => Some(1),
                        Kind::CompressionType2 => Some(2),
                        Kind::CompressionType3 => Some(3),
                        Kind::CompressionNone => None,
                        Kind::CompressionSelf => {
                            let to = e.block_offset().map_err(unread)?;
                            if to >= count as u64 || to == at as u64 {
                                return Err(format!("hunk {at} refers to hunk {to}, of {count}"));
                            }
                            at = to as u32;
                            continue;
                        }
                        _ => return Ok(None),
                    };
                    (codec, e.block_offset().map_err(unread)?, e.block_size().map_err(unread)? as u64, Some(e.hunk_crc().map_err(unread)?))
                }
                MapEntry::V5Uncompressed(e) => (None, e.block_offset().map_err(unread)?, e.block_size() as u64, None),
                MapEntry::LegacyEntry(_) => return Ok(None),
            };
            let end = offset.checked_add(size).filter(|&end| end <= self.len).ok_or_else(|| format!("its block is bytes {offset} to {offset}+{size} of a {}-byte file", self.len))?;
            if codec.is_some_and(|k| self.framed[k]) {
                let block = &self.chd.inner().get_ref().as_ref()[offset as usize..end as usize];
                let (flags, width) = ((hunk_bytes / UNIT).div_ceil(8), if hunk_bytes < 0x1_0000 { 2 } else { 3 });
                let rest = block.len().checked_sub(flags + width).ok_or_else(|| format!("its block of {} bytes is shorter than its codec's header", block.len()))?;
                let stream = block[flags..flags + width].iter().fold(0usize, |a, &b| a << 8 | b as usize);
                if stream > rest {
                    return Err(format!("its sector stream is said to be {stream} bytes, and {rest} follow the header"));
                }
            }
            return Ok(crc);
        }
        Err(format!("its self-references run past {REFERENCES} hunks"))
    }
}
