//! The medium an image holds, chosen when it is opened (EmuSen_Disc.md §14.1): a compact disc as `Disc`, or a DVD as
//! `Dvd`. What a container's bytes say decides it; where they cannot (2,048-byte sectors, which either disc may hold),
//! the console's word does, as `IsoForm` and `Lone` are its words.

use crate::disc::Disc;
use crate::dvd::Dvd;

/// A disc's kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Cd,
    Dvd,
}

/// A disc of either kind, over backing `B`.
pub enum Medium<B: AsRef<[u8]> = Vec<u8>> {
    Cd(Disc<B>),
    Dvd(Dvd<B>),
}

impl<B: AsRef<[u8]>> Medium<B> {
    pub fn kind(&self) -> Kind {
        match self {
            Medium::Cd(_) => Kind::Cd,
            Medium::Dvd(_) => Kind::Dvd,
        }
    }

    pub fn cd(&mut self) -> Option<&mut Disc<B>> {
        match self {
            Medium::Cd(d) => Some(d),
            Medium::Dvd(_) => None,
        }
    }

    pub fn dvd(&mut self) -> Option<&mut Dvd<B>> {
        match self {
            Medium::Dvd(d) => Some(d),
            Medium::Cd(_) => None,
        }
    }

    /// The first stored sector that could not be read, either kind's.
    pub fn damage(&self) -> Option<&str> {
        match self {
            Medium::Cd(d) => d.damage(),
            Medium::Dvd(d) => d.damage(),
        }
    }
}
