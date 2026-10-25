//! A disc as sectors: its tracks placed on the absolute time line from a cue sheet, an `.iso`, a lone raw track or a
//! CHD; any sector read as its raw 2,352 bytes, those the image does not hold synthesised (ECMA-130); its subchannel Q
//! synthesised (§22.3), `.sbi` patches applied; the TOC; CD-DA as samples. EmuSen_Disc.md §3 records each rule, and §9
//! the backing: the bytes behind a disc are any `AsRef<[u8]>`, owned, borrowed for a call or lent by a host.

use crate::chd::ChdImage;
use crate::cue::{CueError, Mode, Sheet, flags};
use crate::ecma130::{self, RAW, submode};
use crate::msf::{Msf, PROGRAM_START, bcd};
use crate::sbi::{Patch, SbiError};
use crate::subq::{self, CONTROL_AUDIO, CONTROL_DATA, LEAD_OUT, SubQ};

/// How a 2,048-byte `.iso` sector is framed when read raw: mode 1, or mode 2 form 1 with a data subheader, which a
/// CD-XA console's discs are. The console's choice, given at open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsoForm {
    Mode1,
    Mode2Form1,
}

/// What reading a sector found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sector {
    /// A data track's sector, stored or synthesised; `mode` from its header.
    Data { mode: u8, synthesised: bool },
    /// An audio track's sector: 588 stereo frames, silence where the image holds none.
    Audio { synthesised: bool },
}

impl Sector {
    pub fn audio(self) -> bool {
        matches!(self, Sector::Audio { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Track {
    pub number: u8,
    pub audio: bool,
    /// Q's control nibble: data or audio, with the sheet's flags.
    pub control: u8,
    /// Its first sector, index 00 when it has a pause, and its index 01; `end` is the next track's first sector.
    pub pregap: u32,
    pub start: u32,
    pub end: u32,
    /// Index 01 and up, by absolute sector.
    pub indexes: Vec<(u8, u32)>,
    store: Store,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Store {
    file: usize,
    /// The stored sectors [from, to), the first at byte `offset` of its file (or frame `offset` of a CHD).
    from: u32,
    to: u32,
    offset: u64,
    mode: Mode,
}

impl Track {
    /// A track whose stored sectors [from, to) begin at `offset` of source `file`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn stored(number: u8, mode: Mode, control: u8, pregap: u32, start: u32, end: u32, file: usize, from: u32, to: u32, offset: u64) -> Track {
        Track { number, audio: mode.audio(), control, pregap, start, end, indexes: vec![(1, start)], store: Store { file, from, to, offset, mode } }
    }
}

/// Where a stored sector's bytes lie among the files a disc was opened from: what a host that reads ahead of a drive
/// needs of a sector (EmuSen_Disc.md §10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    /// The file's position among those a sheet names, or `None` for the image itself: a lone track, an `.iso`, a CHD.
    pub file: Option<usize>,
    /// The byte of that file where the sector begins; in a CHD, where the stored hunk holding it does.
    pub offset: u64,
}

/// The table of contents (§22.3.4) as a controller reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toc {
    pub first: u8,
    pub last: u8,
    /// Each track's number, control nibble and index 01.
    pub tracks: Vec<(u8, u8, u32)>,
    pub lead_out: u32,
    /// Point A0h's disc type: 00h audio or mode 1, 20h CD-ROM XA.
    pub disc_type: u8,
}

enum Source<B: AsRef<[u8]>> {
    Bytes(B),
    Chd(Box<ChdImage<B>>),
}

/// A disc over backing `B`: `Vec<u8>` where it owns its bytes, `&[u8]` or a host's lent bytes where it does not; one
/// path reads them all (EmuSen_Disc.md §9.1).
pub struct Disc<B: AsRef<[u8]> = Vec<u8>> {
    tracks: Vec<Track>,
    lead_out: u32,
    sources: Vec<Source<B>>,
    patches: Vec<Patch>,
    iso: IsoForm,
    /// One file that is the image itself, no sheet naming it: what `place` answers a sector's file by.
    lone: bool,
    /// A zero block for synthesised sectors' user data.
    zeros: Box<[u8; RAW]>,
    /// The first stored sector that could not be read, in words: a CHD's damaged hunk (§13).
    damage: Option<String>,
}

#[derive(Debug)]
pub enum DiscError {
    Cue(CueError),
    Sbi(SbiError),
    /// A sheet names more files than were given, or a file too short for its tracks.
    Files(String),
    /// Bytes that are no image this crate reads.
    Unknown(String),
    Chd(String),
}

impl std::fmt::Display for DiscError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DiscError::Cue(e) => e.fmt(f),
            DiscError::Sbi(e) => e.fmt(f),
            DiscError::Files(s) | DiscError::Unknown(s) | DiscError::Chd(s) => f.write_str(s),
        }
    }
}

impl From<CueError> for DiscError {
    fn from(e: CueError) -> Self {
        DiscError::Cue(e)
    }
}

/// The CHD container's identifier at its start.
pub const CHD_MAGIC: &[u8; 8] = b"MComprHD";

fn control_of(audio: bool, cue_flags: u8) -> u8 {
    if audio { CONTROL_AUDIO | cue_flags & (flags::PRE | flags::DCP | flags::FOUR_CHANNEL) } else { CONTROL_DATA | cue_flags & flags::DCP }
}

impl<B: AsRef<[u8]>> Disc<B> {
    /// A disc from a cue sheet and its files' bytes, in the sheet's order.
    pub fn from_sheet(sheet: &Sheet, files: Vec<B>, iso: IsoForm) -> Result<Disc<B>, DiscError> {
        if files.len() < sheet.files.len() {
            return Err(DiscError::Files(format!("the sheet names {} files and {} were given", sheet.files.len(), files.len())));
        }
        // Each file's samples or sectors: all its bytes, or a WAVE file's data chunk.
        let spans = files
            .iter()
            .enumerate()
            .map(|(i, f)| match sheet.kinds.get(i) {
                Some(crate::cue::FileKind::Wave) => crate::wave::data(f.as_ref()).map_err(|e| DiscError::Files(format!("{}: {e}", sheet.files[i]))),
                _ => Ok((0, f.as_ref().len())),
            })
            .collect::<Result<Vec<(usize, usize)>, DiscError>>()?;
        let mut tracks: Vec<Track> = Vec::new();
        let mut base: i64 = 0;
        let mut byte: u64 = 0;
        let mut cursor: u32 = 0;
        for (k, t) in sheet.tracks.iter().enumerate() {
            let first = t.first_frame() as i64;
            let new_file = k == 0 || t.file != sheet.tracks[k - 1].file;
            if new_file {
                base = if k == 0 { PROGRAM_START as i64 - t.index(1).unwrap_or(0) as i64 } else { (cursor + t.pregap) as i64 - first };
                byte = first as u64 * t.mode.bytes() as u64;
                if base < 0 {
                    return Err(DiscError::Files(format!("track {}'s file holds more than the two-second pause before it", t.number)));
                }
            } else {
                let p = &sheet.tracks[k - 1];
                byte += (t.first_frame() - p.first_frame()) as u64 * p.mode.bytes() as u64;
                base += t.pregap as i64;
            }
            let from = (base + first) as u32;
            let (head, size) = (spans[t.file].0 as u64, spans[t.file].1 as u64);
            let to = match sheet.tracks.get(k + 1) {
                Some(n) if n.file == t.file => (base + n.first_frame() as i64) as u32,
                _ => from + (size.saturating_sub(byte) / t.mode.bytes() as u64) as u32,
            };
            if byte > size || to < (base + t.index(1).unwrap_or(0) as i64) as u32 {
                return Err(DiscError::Files(format!("track {}'s file is too short for its indexes", t.number)));
            }
            let end = to + t.postgap;
            let indexes = t.indexes.iter().filter(|i| i.0 >= 1).map(|&(n, f)| (n, (base + f as i64) as u32)).collect();
            tracks.push(Track {
                number: t.number,
                audio: t.mode.audio(),
                control: control_of(t.mode.audio(), t.flags),
                pregap: if k == 0 { 0 } else { from - t.pregap },
                start: (base + t.index(1).unwrap_or(0) as i64) as u32,
                end,
                indexes,
                store: Store { file: t.file, from, to, offset: head + byte, mode: t.mode },
            });
            if sheet.tracks.get(k + 1).is_some_and(|n| n.file == t.file) {
                base += t.postgap as i64;
            }
            cursor = end;
        }
        Ok(Disc::new(tracks, cursor, files.into_iter().map(Source::Bytes).collect(), iso))
    }

    fn new(tracks: Vec<Track>, lead_out: u32, sources: Vec<Source<B>>, iso: IsoForm) -> Disc<B> {
        Disc { tracks, lead_out, sources, patches: Vec::new(), iso, lone: false, zeros: Box::new([0; RAW]), damage: None }
    }

    /// A disc from one file's bytes: a CHD, a lone raw track (data by its sync, else audio), or an `.iso`'s user
    /// sectors, each a sheet of one track synthesised here (EmuSen_Disc.md §4).
    pub fn from_image(bytes: B, iso: IsoForm) -> Result<Disc<B>, DiscError> {
        let head = bytes.as_ref();
        if head.starts_with(CHD_MAGIC) {
            let (image, tracks, lead_out) = ChdImage::open(bytes).map_err(DiscError::Chd)?;
            return Ok(Disc::from_chd(image, tracks, lead_out, iso));
        }
        let iso_volume = head.get(16 * 2048 + 1..16 * 2048 + 6) == Some(b"CD001");
        let mode = if head.starts_with(&ecma130::SYNC) && head.len() >= RAW {
            if head[15] == 1 { "MODE1/2352" } else { "MODE2/2352" }
        } else if iso_volume {
            "MODE1/2048"
        } else if head.len() % RAW == 0 && !head.is_empty() {
            "AUDIO"
        } else {
            return Err(DiscError::Unknown(format!("{} bytes are neither a CHD, a raw track of 2,352-byte sectors nor an ISO 9660 image of 2,048-byte ones", head.len())));
        };
        let sheet = Sheet::parse(&format!("FILE \"image\" BINARY\nTRACK 01 {mode}\nINDEX 01 00:00:00\n"))?;
        Ok(Disc { lone: true, ..Disc::from_sheet(&sheet, vec![bytes], iso)? })
    }

    /// A CD CHD already opened, the image itself.
    pub(crate) fn from_chd(image: ChdImage<B>, tracks: Vec<Track>, lead_out: u32, iso: IsoForm) -> Disc<B> {
        Disc { lone: true, ..Disc::new(tracks, lead_out, vec![Source::Chd(Box::new(image))], iso) }
    }

    /// An image of 2,048-byte user sectors read as one data track, for a caller that knows it is one though it holds
    /// no ISO 9660 volume descriptor; a partial last sector is left out.
    pub fn from_user_image(bytes: B, iso: IsoForm) -> Result<Disc<B>, DiscError> {
        let sheet = Sheet::parse("FILE \"image\" BINARY\nTRACK 01 MODE1/2048\nINDEX 01 00:00:00\n")?;
        Ok(Disc { lone: true, ..Disc::from_sheet(&sheet, vec![bytes], iso)? })
    }

    /// The `.sbi` beside the image: each listed sector's Q is the recorded one, its CRC failing.
    pub fn apply_sbi(&mut self, data: &[u8]) -> Result<(), DiscError> {
        self.patches = crate::sbi::parse(data).map_err(DiscError::Sbi)?;
        Ok(())
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    /// The lead-out's first sector, past every track.
    pub fn lead_out(&self) -> u32 {
        self.lead_out
    }

    pub fn toc(&self) -> Toc {
        Toc {
            first: self.tracks[0].number,
            last: self.tracks[self.tracks.len() - 1].number,
            tracks: self.tracks.iter().map(|t| (t.number, t.control, t.start)).collect(),
            lead_out: self.lead_out,
            disc_type: if self.tracks.iter().any(|t| !t.audio && t.store.mode != Mode::Mode1Raw && !(t.store.mode == Mode::Mode1User && self.iso == IsoForm::Mode1)) { 0x20 } else { 0 },
        }
    }

    /// The track a sector lies in, its pause included; none in the lead-out.
    pub fn track_at(&self, at: u32) -> Option<&Track> {
        if at >= self.lead_out {
            return None;
        }
        let i = self.tracks.partition_point(|t| t.end <= at);
        self.tracks.get(i)
    }

    /// The first stored sector `read` could not read, and why: a CHD's hunk that is damaged. `read` gave it, and every
    /// other sector of the hunk, as a sector the image does not hold.
    pub fn damage(&self) -> Option<&str> {
        self.damage.as_deref()
    }

    /// The file and byte where sector `at` is stored: the first byte `read` takes for it, or in a CHD the first of
    /// the stored hunk it decodes. `None` for a sector the image does not hold (§3.2) and for a CHD's hunk that is a
    /// copy of another, which has no bytes of its own. It reads nothing of the image.
    pub fn place(&self, at: u32) -> Option<Place> {
        let s = self.track_at(at)?.store;
        if at < s.from || at >= s.to {
            return None;
        }
        let n = (at - s.from) as u64;
        let offset = match &self.sources[s.file] {
            Source::Chd(c) => c.place(s.offset + n)?,
            Source::Bytes(b) => {
                let size = s.mode.bytes() as u64;
                let o = s.offset + n * size;
                (o + size <= B::as_ref(b).len() as u64).then_some(o)?
            }
        };
        Some(Place { file: (!self.lone).then_some(s.file), offset })
    }

    /// Sector `at`'s raw bytes and what it is. The lead-out continues the last track's kind, synthesised.
    pub fn read(&mut self, at: u32, out: &mut [u8; RAW]) -> Sector {
        let Some(t) = self.track_at(at).or(self.tracks.last()) else { return Sector::Audio { synthesised: true } };
        let (audio, s) = (t.audio, t.store);
        let stored = at >= s.from && at < s.to && at < self.lead_out;
        if !stored {
            return self.synthesise(at, audio, s.mode, out);
        }
        let n = (at - s.from) as u64;
        let iso = self.iso;
        let size = s.mode.bytes();
        let (src, swapped): (&[u8], bool) = match &mut self.sources[s.file] {
            Source::Chd(c) => match c.unit(s.offset + n) {
                Ok(u) => (&u[..size], true),
                Err(w) => {
                    self.damage.get_or_insert_with(|| format!("sector {at}: {w}"));
                    return self.synthesise(at, audio, s.mode, out);
                }
            },
            Source::Bytes(b) => {
                let o = (s.offset + n * size as u64) as usize;
                match B::as_ref(b).get(o..o + size) {
                    Some(src) => (src, false),
                    None => return self.synthesise(at, audio, s.mode, out),
                }
            }
        };
        match s.mode {
            Mode::Audio if swapped => {
                for (d, w) in out.chunks_exact_mut(2).zip(src.chunks_exact(2)) {
                    d.copy_from_slice(&[w[1], w[0]]);
                }
            }
            Mode::Audio | Mode::Mode1Raw | Mode::Mode2Raw => out.copy_from_slice(src),
            Mode::Mode2Xa => {
                ecma130::header(at, 2, out);
                out[16..].copy_from_slice(src);
            }
            Mode::Mode1User => match iso {
                IsoForm::Mode1 => ecma130::mode1(at, src, out),
                IsoForm::Mode2Form1 => ecma130::mode2_form1(at, [0, 0, submode::DATA, 0], src, out),
            },
        }
        if audio { Sector::Audio { synthesised: false } } else { Sector::Data { mode: out[15], synthesised: false } }
    }

    /// A sector the image does not hold: silence on an audio track, an empty sector of the track's mode on a data one.
    fn synthesise(&self, at: u32, audio: bool, mode: Mode, out: &mut [u8; RAW]) -> Sector {
        if audio {
            out.fill(0);
            return Sector::Audio { synthesised: true };
        }
        let mode1 = mode == Mode::Mode1Raw || (mode == Mode::Mode1User && self.iso == IsoForm::Mode1);
        if mode1 {
            ecma130::mode1(at, &self.zeros[..], out);
        } else {
            ecma130::mode2_form1(at, [0; 4], &self.zeros[..], out);
        }
        Sector::Data { mode: out[15], synthesised: true }
    }

    /// Sector `at`'s subchannel Q in q-mode 1: the track, the index, the time within the track (counting down to the
    /// last sector of a pause, ECMA-130 §22.3.3.3), the absolute time; patched and failing its CRC where an `.sbi`
    /// lists the sector.
    pub fn subq(&self, at: u32) -> SubQ {
        let q = match self.track_at(at) {
            None => {
                let last = self.tracks.last().map_or(CONTROL_DATA, |t| t.control);
                SubQ::position(last, LEAD_OUT, 1, Msf::of(at.saturating_sub(self.lead_out)), Msf::of(at))
            }
            Some(t) if at < t.start => SubQ::position(t.control, bcd(t.number), 0, Msf::of(t.start - at - 1), Msf::of(at)),
            Some(t) => {
                let index = t.indexes.iter().rev().find(|i| i.1 <= at).map_or(1, |i| i.0);
                SubQ::position(t.control, bcd(t.number), bcd(index), Msf::of(at - t.start), Msf::of(at))
            }
        };
        match self.patches.binary_search_by_key(&at, |p| p.at) {
            Ok(i) => {
                let mut b = q.0;
                for (k, v) in self.patches[i].q.iter().enumerate() {
                    if let Some(v) = v {
                        b[k] = *v;
                    }
                }
                let crc = subq::crc16(&b[..10]) ^ 0x0080;
                b[10..].copy_from_slice(&crc.to_be_bytes());
                SubQ(b)
            }
            Err(_) => q,
        }
    }

    /// What a state names its disc by, never 0: FNV-1a over each track's index 01 and kind, the lead-out, and the raw
    /// sectors the console names, counted from the first track's index 01. It reads those sectors and no other, so
    /// its cost does not grow with the disc (EmuSen_Disc.md §9.3).
    pub fn fingerprint(&mut self, sectors: impl IntoIterator<Item = u32>) -> u64 {
        let mut h = 0xCBF2_9CE4_8422_2325u64;
        let mut eat = |bytes: &[u8]| bytes.iter().for_each(|&b| h = (h ^ b as u64).wrapping_mul(0x0100_0000_01B3));
        for t in &self.tracks {
            eat(&t.start.to_le_bytes());
            eat(&[t.audio as u8]);
        }
        eat(&self.lead_out.to_le_bytes());
        let first = self.tracks.first().map_or(0, |t| t.start);
        let mut raw = [0u8; RAW];
        for sector in sectors {
            self.read(first + sector, &mut raw);
            eat(&raw);
        }
        h.max(1)
    }

    /// The lead-in's item for a TOC pointer (01h–99h a track, A0h, A1h, A2h), as GetQ reads it.
    pub fn lead_in(&self, pointer: u8) -> Option<SubQ> {
        let toc = self.toc();
        let first_control = self.tracks[0].control;
        let last_control = self.tracks[self.tracks.len() - 1].control;
        let item = match pointer {
            0xA0 => subq::toc_item(first_control, 0xA0, [bcd(toc.first), toc.disc_type, 0], Msf::default()),
            0xA1 => subq::toc_item(last_control, 0xA1, [bcd(toc.last), 0, 0], Msf::default()),
            0xA2 => subq::toc_item(last_control, 0xA2, Msf::of(self.lead_out).bcd(), Msf::default()),
            p => {
                let n = crate::msf::from_bcd(p)?;
                let t = self.tracks.iter().find(|t| t.number == n)?;
                subq::toc_item(t.control, p, Msf::of(t.start).bcd(), Msf::default())
            }
        };
        Some(item)
    }
}

/// An audio sector's 588 stereo frames, little-endian 16-bit, left first.
pub fn samples(raw: &[u8; RAW]) -> impl Iterator<Item = (i16, i16)> + '_ {
    raw.chunks_exact(4).map(|c| (i16::from_le_bytes([c[0], c[1]]), i16::from_le_bytes([c[2], c[3]])))
}
