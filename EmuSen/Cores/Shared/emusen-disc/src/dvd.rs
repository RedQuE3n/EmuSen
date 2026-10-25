//! A DVD as sectors (EmuSen_Disc.md §14): its logical sectors from an image of 2,048-byte sectors, of whole 2,064-byte
//! Data Frames, or a DVD CHD; one or two layers, the second's start taken from where the image's second volume begins;
//! any sector read as its Data Frame (ECMA-267 §16), stored whole or synthesised with its ID, IED, CPR_MAI and EDC,
//! those past the image's end as the Lead-out Zone's. No drive's commands or timing, which are its console's.

use crate::chd::{ChdImage, Opened};
use crate::disc::{CHD_MAGIC, DiscError, Place};
use crate::ecma267::{self, DATA_AT, EDC_AT, FRAME, TrackPath, USER, Zone};

/// The most logical sectors a compact disc's time code can name past its first pause: 100 minutes of 75-frame seconds,
/// 99:59:74 the last, less the 150 sectors before 00:02:00 (ECMA-130 §22.3.3.4). An image of more is a DVD's.
pub const CD_LIMIT: u32 = 100 * 60 * 75 - 150;

/// The ISO 9660 primary volume descriptor's place and identifier (ECMA-119 §8.4), and its Volume Space Size (§8.4.8).
const PVD: u32 = 16;
const PVD_ID: &[u8; 6] = b"\x01CD001";
const VOLUME_SPACE_SIZE: usize = 80;

/// What reading a DVD sector found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DvdSector {
    /// The layer it lies on, 0 or 1.
    pub layer: u8,
    /// Its physical sector number (§25).
    pub psn: u32,
    pub zone: Zone,
    /// Made here rather than read from the image: a sector past its end, a damaged CHD hunk's, or any sector of an
    /// image of 2,048-byte sectors, whose frame is built around the stored data.
    pub synthesised: bool,
}

enum Store<B: AsRef<[u8]>> {
    /// 2,048-byte logical sectors, as an `.iso` holds them.
    User(B),
    /// Whole 2,064-byte Data Frames.
    Frames(B),
    Chd(Box<ChdImage<B>>),
}

/// A DVD over backing `B`, as `Disc` is a compact disc's (EmuSen_Disc.md §9.1).
pub struct Dvd<B: AsRef<[u8]> = Vec<u8>> {
    store: Store<B>,
    sectors: u32,
    layer_break: Option<u32>,
    path: TrackPath,
    damage: Option<String>,
}

/// Whether `bytes` begin with a Data Frame of the Data Zone's first sector: its ID names (030000) and its IED holds.
pub fn is_frames(bytes: &[u8]) -> bool {
    bytes.len() >= FRAME
        && bytes.len().is_multiple_of(FRAME)
        && bytes[1..4] == [0x03, 0x00, 0x00]
        && ecma267::ied([bytes[0], bytes[1], bytes[2], bytes[3]]) == [bytes[4], bytes[5]]
}

impl<B: AsRef<[u8]>> Dvd<B> {
    /// A DVD from one file: a DVD CHD by its magic and metadata, whole Data Frames by the first's ID, or 2,048-byte
    /// sectors holding an ISO 9660 volume descriptor at sector 16. The layer break is found here (§14.2).
    pub fn from_image(bytes: B) -> Result<Dvd<B>, DiscError> {
        let head = bytes.as_ref();
        if head.starts_with(CHD_MAGIC) {
            return match ChdImage::open_either(bytes).map_err(DiscError::Chd)? {
                Opened::Dvd(image, sectors) => Ok(Dvd::new(Store::Chd(Box::new(image)), sectors)),
                Opened::Cd(..) => Err(DiscError::Unknown("the CHD holds a compact disc, not a DVD".into())),
            };
        }
        if is_frames(head) {
            let sectors = Dvd::<B>::count(head.len() / FRAME)?;
            return Ok(Dvd::new(Store::Frames(bytes), sectors));
        }
        if head.get(PVD as usize * USER..PVD as usize * USER + 6) != Some(&PVD_ID[..]) {
            return Err(DiscError::Unknown(format!("{} bytes are neither a DVD CHD, whole 2,064-byte Data Frames nor an ISO 9660 image of 2,048-byte sectors", head.len())));
        }
        Dvd::from_user_image(bytes)
    }

    /// 2,048-byte sectors read as a DVD's logical sectors though they hold no volume descriptor; a partial last sector
    /// is left out.
    pub fn from_user_image(bytes: B) -> Result<Dvd<B>, DiscError> {
        let sectors = Dvd::<B>::count(bytes.as_ref().len() / USER)?;
        Ok(Dvd::new(Store::User(bytes), sectors))
    }

    /// A DVD CHD already opened.
    pub(crate) fn from_chd(image: ChdImage<B>, sectors: u32) -> Dvd<B> {
        Dvd::new(Store::Chd(Box::new(image)), sectors)
    }

    fn count(n: usize) -> Result<u32, DiscError> {
        u32::try_from(n).ok().filter(|&n| n > 0 && n <= ecma267::SECTOR_MASK - ecma267::DATA_ZONE).ok_or_else(|| DiscError::Unknown(format!("{n} sectors are no DVD's")))
    }

    fn new(store: Store<B>, sectors: u32) -> Dvd<B> {
        let mut d = Dvd { store, sectors, layer_break: None, path: TrackPath::Opposite, damage: None };
        d.layer_break = d.find_layer_break();
        d
    }

    /// Where layer 1 begins (EmuSen_Disc.md §14.2): the primary volume descriptor's Volume Space Size V, where the
    /// image holds more than V sectors, V is a multiple of 16 (ECMA-267 §25) and sector V + 16 holds a second
    /// primary volume descriptor. Reads two sectors.
    fn find_layer_break(&mut self) -> Option<u32> {
        let mut s = [0u8; USER];
        self.user_into(PVD, &mut s).then_some(())?;
        if s[..6] != PVD_ID[..] {
            return None;
        }
        let v = u32::from_le_bytes(s[VOLUME_SPACE_SIZE..VOLUME_SPACE_SIZE + 4].try_into().unwrap());
        if v == 0 || !v.is_multiple_of(16) || v.checked_add(PVD)? >= self.sectors {
            return None;
        }
        self.user_into(v + PVD, &mut s).then_some(())?;
        (s[..6] == PVD_ID[..]).then_some(v)
    }

    /// Its logical sectors: the Data Zone over both layers, as many as the image holds.
    pub fn sectors(&self) -> u32 {
        self.sectors
    }

    pub fn layers(&self) -> u8 {
        if self.layer_break.is_some() { 2 } else { 1 }
    }

    /// The first logical sector of layer 1, on a Dual Layer disk.
    pub fn layer_break(&self) -> Option<u32> {
        self.layer_break
    }

    /// How the second layer is numbered: Opposite Track Path unless the console says otherwise, since an image
    /// records no physical format information (ECMA-267 §26.5.1, byte 2).
    pub fn track_path(&self) -> TrackPath {
        self.path
    }

    pub fn set_track_path(&mut self, path: TrackPath) {
        self.path = path;
    }

    /// The layer and physical sector number of logical sector `lba`; past the Data Zone the Lead-out's, numbered on.
    pub fn physical(&self, lba: u32) -> (u8, u32) {
        ecma267::physical(lba, self.layer_break, self.path)
    }

    /// The first stored sector `read` could not read, and why: a CHD's damaged hunk (EmuSen_Disc.md §13).
    pub fn damage(&self) -> Option<&str> {
        self.damage.as_deref()
    }

    /// The byte of the image where sector `lba` is stored: its 2,048 bytes, its frame, or in a CHD the stored hunk that
    /// holds it. `None` past the image's end and for a CHD hunk that copies another. It reads nothing of the image.
    pub fn place(&self, lba: u32) -> Option<Place> {
        if lba >= self.sectors {
            return None;
        }
        let offset = match &self.store {
            Store::User(_) => lba as u64 * USER as u64,
            Store::Frames(_) => lba as u64 * FRAME as u64,
            Store::Chd(c) => c.place(lba as u64)?,
        };
        Some(Place { file: None, offset })
    }

    /// Sector `lba`'s 2,048 bytes of Main Data, if the image holds them readable; a damaged hunk is kept in `damage`.
    fn user_into(&mut self, lba: u32, out: &mut [u8; USER]) -> bool {
        if lba >= self.sectors {
            return false;
        }
        let n = lba as usize;
        match &mut self.store {
            Store::User(b) => out.copy_from_slice(&b.as_ref()[n * USER..(n + 1) * USER]),
            Store::Frames(b) => out.copy_from_slice(&b.as_ref()[n * FRAME + DATA_AT..n * FRAME + EDC_AT]),
            Store::Chd(c) => match c.unit(lba as u64) {
                Ok(u) => out.copy_from_slice(&u[..USER]),
                Err(w) => {
                    self.damage.get_or_insert_with(|| format!("sector {lba}: {w}"));
                    return false;
                }
            },
        }
        true
    }

    fn sector(&self, lba: u32, synthesised: bool) -> DvdSector {
        let (layer, psn) = self.physical(lba);
        DvdSector { layer, psn, zone: if lba < self.sectors { Zone::Data } else { Zone::LeadOut }, synthesised }
    }

    /// Sector `lba`'s Main Data alone: zeros past the image's end (the Lead-out Zone's, §28) and for a damaged hunk.
    pub fn read_user(&mut self, lba: u32, out: &mut [u8; USER]) -> DvdSector {
        let stored = self.user_into(lba, out);
        if !stored {
            out.fill(0);
        }
        self.sector(lba, !stored)
    }

    /// Sector `lba` as its whole Data Frame: stored frames as the image holds them; otherwise built around the Main
    /// Data with the ID (the layer bit, the zone, the reflectivity bit a Dual Layer disk sets), the IED, CPR_MAI zero
    /// (§16.3's default) and the EDC.
    pub fn read(&mut self, lba: u32, out: &mut [u8; FRAME]) -> DvdSector {
        if lba < self.sectors
            && let Store::Frames(b) = &self.store
        {
            let n = lba as usize;
            out.copy_from_slice(&b.as_ref()[n * FRAME..(n + 1) * FRAME]);
            return self.sector(lba, false);
        }
        let mut user = [0u8; USER];
        let stored = self.user_into(lba, &mut user);
        let s = self.sector(lba, !stored || !matches!(self.store, Store::Frames(_)));
        let info = ecma267::sector_information(s.layer, s.zone, self.layer_break.is_some());
        ecma267::frame(info, s.psn, [0; 6], &user, out);
        s
    }

    /// What a state names its disc by, never 0: FNV-1a over the sector count, the layer break and the Main Data of the
    /// sectors the console names; bounded by construction, as `Disc::fingerprint` is.
    pub fn fingerprint(&mut self, sectors: impl IntoIterator<Item = u32>) -> u64 {
        let mut h = 0xCBF2_9CE4_8422_2325u64;
        let mut eat = |bytes: &[u8]| bytes.iter().for_each(|&b| h = (h ^ b as u64).wrapping_mul(0x0100_0000_01B3));
        eat(&self.sectors.to_le_bytes());
        eat(&self.layer_break.unwrap_or(0).to_le_bytes());
        let mut user = [0u8; USER];
        for s in sectors {
            self.read_user(s, &mut user);
            eat(&user);
        }
        h.max(1)
    }
}
