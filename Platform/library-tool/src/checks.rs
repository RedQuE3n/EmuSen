//! Checks 2 and 3 of EmuSen_GamePackages.md §4.3, both through `emusen-disc` and the `chd` crate, an implementation
//! other than chdman's: each original track's bytes rebuilt from the CHD and hashed against the hashes taken of the
//! original before chdman ran, and the table of contents against the one the original disc gave. §12.2 records why
//! the rebuilt range is found from the sheet and the CHD's own track starts, and not from the original files.

use emusen_disc::cue::Sheet;
use emusen_disc::{Disc, IsoForm};
use emusen_galaxia::json::{Text, Value};
use emusen_galaxia::mapped::Mapped;
use emusen_galaxia::rom_hash::{Hasher, Hashes};

/// A mapped file as the bytes a disc reads.
pub struct Bytes(pub Mapped);

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.0.bytes()
    }
}

/// Every disc is opened with the same framing for a cooked sector, so that the two tables of contents compare.
pub const ISO_FORM: IsoForm = IsoForm::Mode1;

/// One track's place in the table of contents, as the original disc and the CHD both give it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TocTrack {
    pub number: u8,
    pub audio: bool,
    pub pregap: u32,
    pub start: u32,
    pub end: u32,
}

/// Check 3's record: each track, the lead-out and point A0h's disc type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toc {
    pub tracks: Vec<TocTrack>,
    pub lead_out: u32,
    pub disc_type: u8,
}

pub fn toc_of<B: AsRef<[u8]>>(disc: &Disc<B>) -> Toc {
    Toc {
        tracks: disc.tracks().iter().map(|t| TocTrack { number: t.number, audio: t.audio, pregap: t.pregap, start: t.start, end: t.end }).collect(),
        lead_out: disc.lead_out(),
        disc_type: disc.toc().disc_type,
    }
}

fn n(v: u64) -> Value {
    Value::number(v)
}

fn member(name: &str, value: Value) -> (Text, Value) {
    (Text::new(name), value)
}

impl Toc {
    pub fn to_json(&self) -> Value {
        let tracks = self
            .tracks
            .iter()
            .map(|t| Value::Object(vec![member("track", n(t.number as u64)), member("audio", Value::Bool(t.audio)), member("pregap", n(t.pregap as u64)), member("start", n(t.start as u64)), member("end", n(t.end as u64))]))
            .collect();
        Value::Object(vec![member("tracks", Value::Array(tracks)), member("lead_out", n(self.lead_out as u64)), member("disc_type", n(self.disc_type as u64))])
    }

    pub fn from_json(v: &Value) -> Option<Toc> {
        let int = |v: &Value, name: &str| -> Option<u64> {
            match v.get(name)? {
                Value::Number(t) => t.parse().ok(),
                _ => None,
            }
        };
        let Value::Array(items) = v.get("tracks")? else { return None };
        let tracks = items
            .iter()
            .map(|t| Some(TocTrack { number: int(t, "track")? as u8, audio: t.get("audio")?.as_bool()?, pregap: int(t, "pregap")? as u32, start: int(t, "start")? as u32, end: int(t, "end")? as u32 }))
            .collect::<Option<Vec<_>>>()?;
        Some(Toc { tracks, lead_out: int(v, "lead_out")? as u32, disc_type: int(v, "disc_type")? as u8 })
    }
}

/// The original disc: the sheet over its files mapped, read only where the table of contents needs their lengths.
pub fn original(sheet: &str, files: &[String]) -> Result<Disc<Bytes>, String> {
    let parsed = Sheet::parse(sheet).map_err(|e| e.to_string())?;
    let mapped = files.iter().map(|f| Mapped::open(f).map(Bytes).map_err(|e| format!("{f}: {e}"))).collect::<Result<Vec<_>, _>>()?;
    Disc::from_sheet(&parsed, mapped, ISO_FORM).map_err(|e| e.to_string())
}

/// A CHD as a disc, through `emusen-disc` and the `chd` crate.
pub fn chd(at: &str) -> Result<Disc<Bytes>, String> {
    let mapped = Mapped::open(at).map_err(|e| format!("{at}: {e}"))?;
    Disc::from_image(Bytes(mapped), ISO_FORM).map_err(|e| format!("the CHD does not read through EmuSen's reader: {e}"))
}

/// Check 3: the CHD's table of contents equals the original's, track for track.
pub fn toc(chd: &Disc<Bytes>, expected: &Toc) -> Result<(), String> {
    let got = toc_of(chd);
    if got.tracks.len() != expected.tracks.len() {
        return Err(format!("the CHD has {} tracks and the original {}", got.tracks.len(), expected.tracks.len()));
    }
    for (g, e) in got.tracks.iter().zip(&expected.tracks) {
        if g != e {
            return Err(format!("track {} differs: the CHD's {:?}, the original's {:?}", e.number, g, e));
        }
    }
    if got.lead_out != expected.lead_out {
        return Err(format!("the lead-out differs: the CHD's {}, the original's {}", got.lead_out, expected.lead_out));
    }
    if got.disc_type != expected.disc_type {
        return Err(format!("the disc type differs: the CHD's {:02X}h, the original's {:02X}h", got.disc_type, expected.disc_type));
    }
    Ok(())
}

/// Check 2: each track of `sheet` rebuilt from the CHD as its file held it and hashed, against `expected`, the
/// original's hashes in the sheet's order. Run after check 3, whose equal track starts place each track.
pub fn tracks(chd: &mut Disc<Bytes>, sheet: &str, expected: &[Hashes], stop: &dyn Fn() -> bool) -> Result<(), String> {
    let parsed = Sheet::parse(sheet).map_err(|e| e.to_string())?;
    if parsed.tracks.len() != expected.len() || chd.tracks().len() != expected.len() {
        return Err(format!("the sheet has {} tracks, the record {} and the CHD {}", parsed.tracks.len(), expected.len(), chd.tracks().len()));
    }
    let mut raw = [0u8; 2352];
    for (k, (t, want)) in parsed.tracks.iter().zip(expected).enumerate() {
        let bytes = t.mode.bytes() as u64;
        if want.size % bytes != 0 {
            return Err(format!("track {}'s file ends in part of a {bytes}-byte sector, which a CHD does not keep", t.number));
        }
        let gap = t.index(1).unwrap_or(0).checked_sub(t.first_frame()).ok_or_else(|| format!("track {}'s indexes run backward", t.number))?;
        let from = chd.tracks()[k].start.checked_sub(gap).ok_or_else(|| format!("track {} starts before the disc", t.number))?;
        let mut hasher = Hasher::new(true);
        for at in from..from + (want.size / bytes) as u32 {
            if at % 4096 == 0 && stop() {
                return Err("stopped".into());
            }
            chd.read(at, &mut raw);
            hasher.update(crate::discs::file_form(t.mode, &raw));
        }
        if let Some(w) = chd.damage() {
            return Err(format!("track {} could not be rebuilt from the CHD: {w}", t.number));
        }
        let got = hasher.finish();
        if got.crc32 != want.crc32 || got.md5 != want.md5 || got.sha1 != want.sha1 {
            return Err(format!("track {} rebuilt from the CHD has SHA-1 {} and the original {}", t.number, got.sha1.unwrap_or_default(), want.sha1.clone().unwrap_or_default()));
        }
    }
    Ok(())
}
