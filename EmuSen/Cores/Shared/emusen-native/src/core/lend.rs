//! What a core keeps of an image past the call that passed it (since 1.3, `BORROW`): the host's own bytes where the
//! host lends them, a copy otherwise. A core calls [`keep`] where it would have copied and holds a [`Kept`]; it never
//! tests the flag and never holds a raw pointer. See EmuSen_CoreAPI.md §6.24 and EmuSen_CoreAPI_Extensions.md §10.2.4.
//!
//! This module holds the kit's only `unsafe` on a core's behalf. Its invariant:
//!
//! 1. **A `Kept` that is lent points into bytes a host promised to keep.** One is made only by [`keep`], only while an
//!    export has opened a [`Lend`] for a call whose flags carried `EMUSEN_CREATE_LENT` or `EMUSEN_MEDIA_LENT`, and
//!    only for a slice that lies inside the image or a media file of that call. The host's promise (the header's
//!    words) is that those bytes stay readable, unchanged and in place until the loan ends.
//! 2. **No lent `Kept` outlives its loan.** Each `Lend` makes one [`Loan`], and every `Kept` made under it holds a
//!    reference to it. The export that ends a loan (`free`, a successful `media_insert` on its drive, or the refusal of
//!    the call that made it) calls [`ended`], which panics while a reference is left. A panic in an export aborts the
//!    process, so the host is never told a loan has ended while a core can still read through it.
//! 3. **Nothing is written through it.** A `Kept` gives `&[u8]` only, and the host does not change the bytes, so
//!    sharing one between a machine's threads is sound.

use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;

/// One lending call's promise, alive while any `Kept` made under it is.
#[derive(Debug)]
pub struct Loan(());

/// Bytes kept for the machine's life: lent by the host, or the core's own copy.
pub struct Kept(Bytes);

enum Bytes {
    Owned(Box<[u8]>),
    Lent { at: *const u8, len: usize, _loan: Arc<Loan> },
}

// SAFETY: invariant 3 above; an owned `Kept` is a `Box<[u8]>`.
unsafe impl Send for Kept {}
// SAFETY: as `Send`.
unsafe impl Sync for Kept {}

impl Kept {
    /// Whether these are the host's own bytes, read in place.
    pub fn is_lent(&self) -> bool {
        matches!(self.0, Bytes::Lent { .. })
    }
}

impl Deref for Kept {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match &self.0 {
            Bytes::Owned(b) => b,
            // SAFETY: invariants 1 and 2 above: the bytes are the host's until the loan ends, and this value holds the loan.
            Bytes::Lent { at, len, .. } => unsafe { std::slice::from_raw_parts(*at, *len) },
        }
    }
}

impl AsRef<[u8]> for Kept {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl From<Vec<u8>> for Kept {
    fn from(bytes: Vec<u8>) -> Kept {
        Kept(Bytes::Owned(bytes.into_boxed_slice()))
    }
}

impl Default for Kept {
    fn default() -> Kept {
        Kept(Bytes::Owned(Box::default()))
    }
}

impl std::fmt::Debug for Kept {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Kept({} bytes, {})", self.len(), if self.is_lent() { "lent" } else { "owned" })
    }
}

/// What the call in flight lends: its loan, and the address ranges of its image and media files.
struct Lending {
    loan: Arc<Loan>,
    ranges: Vec<(usize, usize)>,
}

thread_local! {
    static LENDING: RefCell<Option<Lending>> = const { RefCell::new(None) };
}

/// `bytes` to keep past the call: the host's own where the call in flight lends them, a copy otherwise. Called
/// inside `create` or `media_insert` on the image, a media file or any part of one; anywhere else it copies.
pub fn keep(bytes: &[u8]) -> Kept {
    let (at, len) = (bytes.as_ptr() as usize, bytes.len());
    let loan = LENDING.with(|l| l.borrow().as_ref().filter(|l| len > 0 && l.ranges.iter().any(|&(from, n)| at >= from && at + len <= from + n)).map(|l| Arc::clone(&l.loan)));
    match loan {
        Some(loan) => Kept(Bytes::Lent { at: bytes.as_ptr(), len, _loan: loan }),
        None => Kept(Bytes::Owned(bytes.into())),
    }
}

/// A lending call in flight: opened by the export around the core's `create` or `media_insert`, closed when it ends.
pub(crate) struct Lend(Arc<Loan>);

impl Lend {
    /// Lends `ranges`, the image's and each media file's bytes, to the core for the call that follows.
    pub(crate) fn open<'a>(ranges: impl Iterator<Item = &'a [u8]>) -> Lend {
        let loan = Arc::new(Loan(()));
        let ranges = ranges.filter(|r| !r.is_empty()).map(|r| (r.as_ptr() as usize, r.len())).collect();
        LENDING.with(|l| *l.borrow_mut() = Some(Lending { loan: Arc::clone(&loan), ranges }));
        Lend(loan)
    }

    /// The call is over: nothing more is lent, and the loan is the export's to keep until it ends.
    pub(crate) fn close(self) -> Arc<Loan> {
        Arc::clone(&self.0)
    }
}

impl Drop for Lend {
    fn drop(&mut self) {
        LENDING.with(|l| *l.borrow_mut() = None);
    }
}

/// A loan the host is about to be told has ended: a `Kept` still holding it is a defect in the core, and the panic
/// aborts the process before the host can release the bytes (invariant 2).
pub(crate) fn ended(loan: Arc<Loan>, core: &str, when: &str) {
    let held = Arc::strong_count(&loan) - 1;
    assert!(held == 0, "{core} still holds {held} piece(s) of an image its host lent, {when}; a core drops what it kept before the loan ends (EmuSen_CoreAPI.md §6.24)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_are_lent_only_inside_what_a_call_lends_and_copied_otherwise() {
        let (image, track, card) = (vec![1u8; 64], vec![2u8; 64], vec![3u8; 8]);
        assert!(!keep(&image).is_lent(), "no call is in flight");
        let lend = Lend::open([&image[..], &track[..], &[][..]].into_iter());
        let whole = keep(&image);
        let part = keep(&track[8..24]);
        assert!(whole.is_lent() && part.is_lent());
        assert_eq!((whole.as_ptr(), part.as_ptr(), &part[..]), (image.as_ptr(), track[8..].as_ptr(), &track[8..24]));
        let copied = keep(&card);
        assert!(!copied.is_lent() && copied[..] == card[..] && copied.as_ptr() != card.as_ptr(), "a file the call does not lend");
        assert!(!keep(&image[..0]).is_lent() && !keep(&[]).is_lent());
        let loan = lend.close();
        assert!(!keep(&image).is_lent(), "the call is over");
        assert_eq!(Arc::strong_count(&loan), 3);
        drop((whole, part));
        ended(loan, "the test", "after it dropped them");
    }

    #[test]
    #[should_panic(expected = "still holds 1 piece(s) of an image its host lent, at free")]
    fn a_loan_ended_while_a_piece_is_held_stops_the_process() {
        let image = vec![1u8; 16];
        let lend = Lend::open(std::iter::once(&image[..]));
        let kept = keep(&image);
        let loan = lend.close();
        ended(loan, "Leaky", "at free");
        drop(kept);
    }

    #[test]
    fn an_owned_kept_is_its_vector_and_the_default_is_empty() {
        let k = Kept::from(vec![5u8, 6, 7]);
        assert_eq!((&k[..], k.as_ref().len(), k.is_lent(), Kept::default().len()), (&[5u8, 6, 7][..], 3, false, 0));
        assert_eq!(format!("{k:?}"), "Kept(3 bytes, owned)");
    }
}
