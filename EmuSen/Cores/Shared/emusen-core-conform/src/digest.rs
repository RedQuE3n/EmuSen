//! The kit's digests, defined as Pharaoh's are so that a kit result and a Pharaoh result compare directly
//! (EmuSen_CoreAPI.md §12.1); the SHA-256 a sidecar records is the core host's.

pub const FNV_OFFSET: u64 = 14695981039346656037;
pub const FNV_PRIME: u64 = 1099511628211;

/// `FrameHash.Compute`: FNV-1a over the bytes.
pub fn frame_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET, |h, &b| (h ^ b as u64).wrapping_mul(FNV_PRIME))
}

/// `framesum` and `audiosum` folded as they go: each frame's hash, and each sample as an unsigned 16-bit value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Digests {
    pub frames: u64,
    pub audio: u64,
    pub samples: u64,
}

impl Default for Digests {
    fn default() -> Digests {
        Digests { frames: FNV_OFFSET, audio: FNV_OFFSET, samples: 0 }
    }
}

impl Digests {
    pub fn frame(&mut self, picture: &[u8]) {
        self.frame_hashed(frame_hash(picture));
    }

    /// A frame whose hash is already taken, so that two digests of one run hash each picture once.
    pub fn frame_hashed(&mut self, hash: u64) {
        self.frames = (self.frames ^ hash).wrapping_mul(FNV_PRIME);
    }

    pub fn sound(&mut self, samples: &[i16]) {
        for &s in samples {
            self.audio = (self.audio ^ s as u16 as u64).wrapping_mul(FNV_PRIME);
        }
        self.samples += samples.len() as u64;
    }
}

/// The SHA-256 a sidecar records, the core host's.
pub use emusen_core_host::sha256::sha256_hex;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_the_standards_examples() {
        assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"), "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
    }

    #[test]
    fn the_frame_hash_is_fnv1a() {
        assert_eq!(frame_hash(b""), FNV_OFFSET);
        assert_eq!(frame_hash(b"a"), 0xaf63dc4c8601ec8c);
    }
}
