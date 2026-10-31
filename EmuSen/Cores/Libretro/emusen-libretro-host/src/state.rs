//! A state's header: whose it is, which versions, a digest of the payload and, where the core asks, the platform
//! (EmuSen_LibretroHost.md §4.6).

use sha2::{Digest, Sha256};

pub const MAGIC: &[u8; 4] = b"ESLR";
pub const HEADER_VERSION: u16 = 1;
/// The adapter's own state version, which machine info's `state.version` gives.
pub const STATE_VERSION: u32 = 1;
pub const HEADER: usize = 64;
const FLAG_PLATFORM: u16 = 1;

/// Why a state is not this machine's, each answered with its own status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Too short, or another format altogether.
    Foreign,
    /// Another libretro core's.
    OtherCore,
    /// Another header or adapter version.
    Version,
    /// Saved on another platform by a core that declared its states platform-dependent.
    Platform,
    /// The payload does not match its digest.
    Damaged,
}

/// The libretro core's name and version, as eight bytes of their digest.
pub fn identity(name: &str, version: &str) -> u64 {
    let d = Sha256::new().chain_update(name.as_bytes()).chain_update([0]).chain_update(version.as_bytes()).finalize();
    u64::from_le_bytes(d[..8].try_into().unwrap())
}

/// This process's architecture and operating system, for a core whose states depend on them.
pub fn platform() -> u32 {
    let arch: u32 = match std::env::consts::ARCH {
        "x86_64" => 1,
        "aarch64" => 2,
        "x86" => 3,
        "arm" => 4,
        _ => 0xFF,
    };
    let os: u32 = match std::env::consts::OS {
        "linux" => 1,
        "windows" => 2,
        "macos" => 3,
        "android" => 4,
        _ => 0xFF,
    };
    arch << 8 | os
}

/// The header written in front of a payload already in `out[HEADER..HEADER + len]`.
pub fn seal(out: &mut [u8], len: usize, identity: u64, platform: Option<u32>) {
    let digest = Sha256::digest(&out[HEADER..HEADER + len]);
    let h = &mut out[..HEADER];
    h.fill(0);
    h[..4].copy_from_slice(MAGIC);
    h[4..6].copy_from_slice(&HEADER_VERSION.to_le_bytes());
    h[6..8].copy_from_slice(&(if platform.is_some() { FLAG_PLATFORM } else { 0 }).to_le_bytes());
    h[8..16].copy_from_slice(&identity.to_le_bytes());
    h[16..20].copy_from_slice(&STATE_VERSION.to_le_bytes());
    h[20..24].copy_from_slice(&platform.unwrap_or(0).to_le_bytes());
    h[24..32].copy_from_slice(&(len as u64).to_le_bytes());
    h[32..64].copy_from_slice(&digest);
}

/// The payload of a state this machine may load, or why not.
pub fn open(data: &[u8], identity: u64, platform: Option<u32>) -> Result<&[u8], Refusal> {
    if data.len() < HEADER || &data[..4] != MAGIC {
        return Err(Refusal::Foreign);
    }
    let u16_at = |o: usize| u16::from_le_bytes(data[o..o + 2].try_into().unwrap());
    let u32_at = |o: usize| u32::from_le_bytes(data[o..o + 4].try_into().unwrap());
    if u16_at(4) != HEADER_VERSION || u32_at(16) != STATE_VERSION {
        return Err(Refusal::Version);
    }
    if u64::from_le_bytes(data[8..16].try_into().unwrap()) != identity {
        return Err(Refusal::OtherCore);
    }
    let recorded = (u16_at(6) & FLAG_PLATFORM != 0).then(|| u32_at(20));
    if let (Some(here), Some(there)) = (platform, recorded)
        && here != there
    {
        return Err(Refusal::Platform);
    }
    let len = u64::from_le_bytes(data[24..32].try_into().unwrap()) as usize;
    let payload = data.get(HEADER..HEADER.checked_add(len).ok_or(Refusal::Damaged)?).ok_or(Refusal::Damaged)?;
    if Sha256::digest(payload).as_slice() != &data[32..64] {
        return Err(Refusal::Damaged);
    }
    Ok(payload)
}

/// One line per field of the header, then the payload, as `state_layout` gives them.
pub fn layout(len: usize) -> String {
    format!(
        "0 4 bytes header.magic\n4 2 u16 header.version\n6 2 u16 header.flags\n8 8 u64 header.core\n16 4 u32 header.state_version\n20 4 u32 header.platform\n24 8 u64 header.length\n32 32 bytes header.sha256\n{HEADER} {len} bytes payload\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sealed(payload: &[u8], id: u64, platform: Option<u32>) -> Vec<u8> {
        let mut out = vec![0u8; HEADER + payload.len()];
        out[HEADER..].copy_from_slice(payload);
        seal(&mut out, payload.len(), id, platform);
        out
    }

    #[test]
    fn a_sealed_state_opens_to_its_payload() {
        let id = identity("Core", "1.0");
        let s = sealed(b"payload", id, None);
        assert_eq!(open(&s, id, None), Ok(&b"payload"[..]));
    }

    #[test]
    fn each_refusal_is_its_own() {
        let id = identity("Core", "1.0");
        let s = sealed(b"payload", id, Some(0x0101));
        assert_eq!(open(b"nothing", id, None), Err(Refusal::Foreign));
        assert_eq!(open(&s, identity("Core", "1.1"), None), Err(Refusal::OtherCore));
        assert_eq!(open(&s, id, Some(0x0201)), Err(Refusal::Platform));
        let mut damaged = s.clone();
        *damaged.last_mut().unwrap() ^= 1;
        assert_eq!(open(&damaged, id, Some(0x0101)), Err(Refusal::Damaged));
        let mut later = s.clone();
        later[16] = 2;
        assert_eq!(open(&later, id, None), Err(Refusal::Version));
        assert_eq!(open(&s[..HEADER + 3], id, None), Err(Refusal::Damaged));
    }
}
