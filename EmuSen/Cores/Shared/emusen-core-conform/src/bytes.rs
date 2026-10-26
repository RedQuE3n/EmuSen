//! An image or a media file as a session holds it: read into memory, or mapped through Galaxia as a host lends it, and
//! shared by every case of the session at no copy. For a core that claims `BORROW` the session lends these bytes to each
//! machine it makes (`EMUSEN_CREATE_LENT`, `EMUSEN_MEDIA_LENT`); the loan's promise, that they stay readable, unchanged
//! and in place until the loan ends, is kept by the count every machine's case holds. See EmuSen_CoreAPI.md §6.24 and
//! §28.20.

use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

use emusen_galaxia::mapped::Mapped;

enum Source {
    Read(Vec<u8>),
    Mapped(Mapped),
}

impl Source {
    fn all(&self) -> &[u8] {
        match self {
            Source::Read(bytes) => bytes,
            Source::Mapped(mapped) => mapped.bytes(),
        }
    }
}

/// Bytes held once and shared: a clone is a count, not a copy, and a prefix is a view of the same bytes.
#[derive(Clone)]
pub struct Bytes {
    source: Arc<Source>,
    len: usize,
}

impl Bytes {
    /// The file at `path` mapped read-only; opening reads none of it.
    pub fn map(path: &Path) -> Result<Bytes, String> {
        let mapped = Mapped::open(path).map_err(|e| format!("{} could not be mapped: {e}", path.display()))?;
        let len = mapped.bytes().len();
        Ok(Bytes { source: Arc::new(Source::Mapped(mapped)), len })
    }

    /// The first `len` bytes, the same bytes at the same addresses.
    pub fn prefix(&self, len: usize) -> Bytes {
        Bytes { source: self.source.clone(), len: len.min(self.len) }
    }

    /// Whether the bytes are a file's mapping rather than memory the kit read.
    pub fn is_mapped(&self) -> bool {
        matches!(*self.source, Source::Mapped(_))
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Bytes {
        let len = bytes.len();
        Bytes { source: Arc::new(Source::Read(bytes)), len }
    }
}

impl Default for Bytes {
    fn default() -> Bytes {
        Vec::new().into()
    }
}

impl Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.source.all()[..self.len]
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::Bytes;

    #[test]
    fn a_clone_and_a_prefix_are_the_same_bytes_at_the_same_addresses() {
        let file = std::env::temp_dir().join(format!("emusen-kit-bytes-{}", std::process::id()));
        std::fs::write(&file, [1u8, 2, 3, 4, 5]).unwrap();
        let mapped = Bytes::map(&file).unwrap();
        let (clone, half) = (mapped.clone(), mapped.prefix(2));
        assert!(mapped.is_mapped() && !Bytes::from(vec![1u8]).is_mapped());
        assert_eq!((&clone[..], &half[..], clone.as_ptr(), half.as_ptr()), (&[1u8, 2, 3, 4, 5][..], &[1u8, 2][..], mapped.as_ptr(), mapped.as_ptr()));
        assert_eq!(mapped.prefix(99).len(), 5);
        drop((mapped, clone));
        assert_eq!(&half[..], [1, 2], "a view keeps the mapping it was taken from");
        std::fs::remove_file(&file).unwrap();
    }
}
