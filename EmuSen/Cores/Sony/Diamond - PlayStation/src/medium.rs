//! What of a disc is the PlayStation's (Diamond_CDROM.md §8): a disc emusen-disc opens is taken only where the licence
//! text in sector 4 of its first track names a market, read as `media.rs` reads a raw image's; its 2,048-byte sectors
//! are framed as CD-XA; and the sectors a state's fingerprint covers. Opening an image with its companions, the
//! formats and the fingerprint's arithmetic are emusen-disc's (EmuSen_Disc.md §9).

use emusen_disc::{IsoForm, Lone, ecma130};
use emusen_native::core::Kept;

use crate::media::{Kind, Media, Refusal, licensed};

/// The disc in a machine's drive: its bytes the host's where the host lends them, a copy otherwise (BORROW).
pub type Disc = emusen_disc::Disc<Kept>;

pub use emusen_disc::is_sheet;

/// What opening found: no disc (a PS-EXE), or the disc and what it is.
pub enum Opened<B: AsRef<[u8]>> {
    Program,
    Disc(Box<emusen_disc::Disc<B>>, Media),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DiscRefusal {
    Media(Refusal),
    /// A sheet whose track files were not all given.
    Missing { needs: usize, has: usize },
    /// A sheet or an image emusen-disc refuses, in its words.
    Bad(String),
}

/// The sectors a state's fingerprint covers, from the first track's index 01: the licence, and the volume
/// descriptors with the root directory that usually follows them. Nine sectors whatever the disc's size.
pub const PRINTED: [u32; 9] = [4, 16, 17, 18, 19, 20, 21, 22, 23];

/// Opens `image` with its companions over the caller's backing: a core's kept bytes at create, bytes borrowed for
/// the call by a question about the image. A PS-EXE is left to `media.rs`.
pub fn open<B: AsRef<[u8]>>(image: B, companions: Vec<B>) -> Result<Opened<B>, DiscRefusal> {
    if image.as_ref().starts_with(crate::media::EXE_MAGIC) {
        return Media::read(image.as_ref()).map(|_| Opened::Program).map_err(DiscRefusal::Media);
    }
    // Headerless 2,048-byte sectors are a disc only where the licence says so; anything else is told by its own bytes.
    let lone = match Media::read(image.as_ref()) {
        Ok(Media { kind: Kind::Disc { raw: false }, .. }) => Lone::User,
        _ => Lone::Detect,
    };
    let sheet = is_sheet(image.as_ref());
    let mut disc = match emusen_disc::open(image, companions, IsoForm::Mode2Form1, lone) {
        Ok(d) => d,
        Err(emusen_disc::Refusal::Missing { needs, has }) => return Err(DiscRefusal::Missing { needs, has }),
        Err(emusen_disc::Refusal::Image(_)) if !sheet => return Err(DiscRefusal::Media(Refusal::NotPlayStation)),
        Err(e) => return Err(DiscRefusal::Bad(e.to_string())),
    };
    let mut raw = [0u8; ecma130::RAW];
    let start = disc.tracks()[0].start;
    disc.read(start + 4, &mut raw);
    let user = if raw[15] == 1 { &raw[16..] } else { &raw[24..] };
    match licensed(user) {
        Some(market) => Ok(Opened::Disc(Box::new(disc), Media { kind: Kind::Disc { raw: true }, market })),
        None => Err(DiscRefusal::Media(Refusal::NotPlayStation)),
    }
}

/// A disc's fingerprint, never 0: emusen-disc's over the sectors `PRINTED` names, as every state since DMND 9 holds it.
pub fn fingerprint<B: AsRef<[u8]>>(disc: &mut emusen_disc::Disc<B>) -> u64 {
    disc.fingerprint(PRINTED)
}
