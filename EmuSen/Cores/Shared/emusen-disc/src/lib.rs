//! A disc as sectors (Diamond_Plan.md §9.2): geometry, ECMA-130 sector synthesis, subchannel Q, the TOC, CD-DA as
//! samples, the cue sheet and .m3u, .sbi patches and CHD v5. No console's drive controller, seek or read timing, XA
//! decoding or filesystem. See Hardware/Shared/EmuSen_Disc.md.

pub mod chd;
pub mod cue;
pub mod disc;
pub mod ecma130;
pub mod msf;
pub mod sbi;
pub mod set;
pub mod subq;

pub use disc::{Disc, DiscError, IsoForm, Sector, Toc, Track, samples};
pub use msf::Msf;
pub use subq::SubQ;
