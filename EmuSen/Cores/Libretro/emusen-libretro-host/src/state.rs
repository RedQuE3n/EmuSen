//! A state's header: whose it is, which versions, a digest of the payload, the disc in the drive, where the core
//! asks the platform, and since header version 2 the SHA-256 of the libretro core's build (EmuSen_LibretroHost.md §4.6,
//! §10h).

use sha2::{Digest, Sha256};

pub const MAGIC: &[u8; 4] = b"ESLR";
pub const HEADER_VERSION: u16 = 2;
/// The adapter's own state version, which machine info's `state.version` gives.
pub const STATE_VERSION: u32 = 1;
pub const HEADER: usize = 112;
const FLAG_PLATFORM: u16 = 1;
/// A snapshot kept in memory: the digest's place holds the process's mark, and no digest is made (EmuSen_Backend.md §37).
const FLAG_SNAPSHOT: u16 = 2;

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
    /// Saved over another disc than the one in the drive.
    WrongMedia,
    /// Saved by another build of the same core and version: the build's SHA-256 the state records.
    OtherBuild([u8; 32]),
    /// A snapshot another run of the core made, which only the run that made it reads.
    OtherRun,
}

/// A disc's identity: its length and the digest of its first MiB, or zero for an empty drive.
pub fn disc(image: Option<&[u8]>) -> [u8; 16] {
    let Some(image) = image else { return [0; 16] };
    let d = Sha256::new().chain_update((image.len() as u64).to_le_bytes()).chain_update(&image[..image.len().min(1 << 20)]).finalize();
    d[..16].try_into().unwrap()
}

/// The libretro core's name and version, as eight bytes of their digest.
pub fn identity(name: &str, version: &str) -> u64 {
    let d = Sha256::new().chain_update(name.as_bytes()).chain_update([0]).chain_update(version.as_bytes()).finalize();
    u64::from_le_bytes(d[..8].try_into().unwrap())
}

/// A build's SHA-256 from its hex form as the library reports it; zero where the text is not one.
pub fn build(sha256: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    if sha256.len() == 64 && sha256.is_ascii() {
        for (i, o) in out.iter_mut().enumerate() {
            *o = u8::from_str_radix(&sha256[i * 2..i * 2 + 2], 16).unwrap_or(0);
        }
    }
    out
}

/// A SHA-256 written as lowercase hex.
pub fn hex(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
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
pub fn seal(out: &mut [u8], len: usize, identity: u64, build: &[u8; 32], platform: Option<u32>, disc: [u8; 16]) {
    let digest: [u8; 32] = Sha256::digest(&out[HEADER..HEADER + len]).into();
    head(out, len, identity, build, platform, disc, 0, &digest);
}

/// A snapshot's header: as `seal`'s, with the process's mark where the digest goes, so none is computed.
pub fn mark(out: &mut [u8], len: usize, identity: u64, build: &[u8; 32], platform: Option<u32>, disc: [u8; 16]) {
    head(out, len, identity, build, platform, disc, FLAG_SNAPSHOT, run_mark());
}

/// Random bytes made once a process, which mark its snapshots as its own.
fn run_mark() -> &'static [u8; 32] {
    static MARK: std::sync::OnceLock<[u8; 32]> = std::sync::OnceLock::new();
    MARK.get_or_init(|| {
        let mut m = [0u8; 32];
        let got = unsafe { libc::getrandom(m.as_mut_ptr() as *mut libc::c_void, m.len(), 0) };
        if got != m.len() as isize {
            let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
            m = Sha256::digest([t.to_le_bytes().as_slice(), &std::process::id().to_le_bytes()].concat()).into();
        }
        m
    })
}

#[allow(clippy::too_many_arguments)]
fn head(out: &mut [u8], len: usize, identity: u64, build: &[u8; 32], platform: Option<u32>, disc: [u8; 16], flags: u16, digest: &[u8; 32]) {
    let h = &mut out[..HEADER];
    h.fill(0);
    h[..4].copy_from_slice(MAGIC);
    h[4..6].copy_from_slice(&HEADER_VERSION.to_le_bytes());
    h[6..8].copy_from_slice(&(flags | if platform.is_some() { FLAG_PLATFORM } else { 0 }).to_le_bytes());
    h[8..16].copy_from_slice(&identity.to_le_bytes());
    h[16..20].copy_from_slice(&STATE_VERSION.to_le_bytes());
    h[20..24].copy_from_slice(&platform.unwrap_or(0).to_le_bytes());
    h[24..32].copy_from_slice(&(len as u64).to_le_bytes());
    h[32..64].copy_from_slice(digest);
    h[64..80].copy_from_slice(&disc);
    h[80..112].copy_from_slice(build);
}

/// The payload of a state this machine may load, or why not.
pub fn open<'a>(data: &'a [u8], identity: u64, build: &[u8; 32], platform: Option<u32>, disc: [u8; 16]) -> Result<&'a [u8], Refusal> {
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
    if &data[80..112] != build {
        return Err(Refusal::OtherBuild(data[80..112].try_into().unwrap()));
    }
    let recorded = (u16_at(6) & FLAG_PLATFORM != 0).then(|| u32_at(20));
    if let (Some(here), Some(there)) = (platform, recorded)
        && here != there
    {
        return Err(Refusal::Platform);
    }
    let len = u64::from_le_bytes(data[24..32].try_into().unwrap()) as usize;
    let payload = data.get(HEADER..HEADER.checked_add(len).ok_or(Refusal::Damaged)?).ok_or(Refusal::Damaged)?;
    if u16_at(6) & FLAG_SNAPSHOT != 0 {
        if &data[32..64] != run_mark() {
            return Err(Refusal::OtherRun);
        }
    } else if Sha256::digest(payload).as_slice() != &data[32..64] {
        return Err(Refusal::Damaged);
    }
    if data[64..80] != disc {
        return Err(Refusal::WrongMedia);
    }
    Ok(payload)
}

/// One line per field of the header, then the payload, as `state_layout` gives them.
pub fn layout(len: usize) -> String {
    format!(
        "0 4 bytes header.magic\n4 2 u16 header.version\n6 2 u16 header.flags\n8 8 u64 header.core\n16 4 u32 header.state_version\n20 4 u32 header.platform\n24 8 u64 header.length\n32 32 bytes header.sha256\n64 16 bytes header.disc\n80 32 bytes header.build\n{HEADER} {len} bytes payload\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUILD: [u8; 32] = [7; 32];

    fn sealed(payload: &[u8], id: u64, platform: Option<u32>) -> Vec<u8> {
        let mut out = vec![0u8; HEADER + payload.len()];
        out[HEADER..].copy_from_slice(payload);
        seal(&mut out, payload.len(), id, &BUILD, platform, disc(Some(b"a disc")));
        out
    }

    #[test]
    fn a_sealed_state_opens_to_its_payload() {
        let id = identity("Core", "1.0");
        let s = sealed(b"payload", id, None);
        assert_eq!(open(&s, id, &BUILD, None, disc(Some(b"a disc"))), Ok(&b"payload"[..]));
        assert_eq!(open(&s, id, &BUILD, None, disc(None)), Err(Refusal::WrongMedia));
    }

    #[test]
    fn each_refusal_is_its_own() {
        let here = disc(Some(b"a disc"));
        let id = identity("Core", "1.0");
        let s = sealed(b"payload", id, Some(0x0101));
        assert_eq!(open(b"nothing", id, &BUILD, None, here), Err(Refusal::Foreign));
        assert_eq!(open(&s, identity("Core", "1.1"), &BUILD, None, here), Err(Refusal::OtherCore));
        assert_eq!(open(&s, id, &[8; 32], None, here), Err(Refusal::OtherBuild(BUILD)));
        assert_eq!(hex(&build(&hex(&BUILD))), hex(&BUILD));
        assert_eq!(open(&s, id, &BUILD, Some(0x0201), here), Err(Refusal::Platform));
        let mut damaged = s.clone();
        *damaged.last_mut().unwrap() ^= 1;
        assert_eq!(open(&damaged, id, &BUILD, Some(0x0101), here), Err(Refusal::Damaged));
        let mut later = s.clone();
        later[16] = 2;
        assert_eq!(open(&later, id, &BUILD, None, here), Err(Refusal::Version));
        assert_eq!(open(&s[..HEADER + 3], id, &BUILD, None, here), Err(Refusal::Damaged));
    }

    #[test]
    fn a_snapshot_carries_this_runs_mark_not_a_digest_and_another_runs_is_refused() {
        let here = disc(Some(b"a disc"));
        let id = identity("Core", "1.0");
        let mut m = vec![0u8; HEADER + 7];
        m[HEADER..].copy_from_slice(b"payload");
        mark(&mut m, 7, id, &BUILD, None, here);
        assert_ne!(&m[32..64], Sha256::digest(b"payload").as_slice());
        assert_eq!(open(&m, id, &BUILD, None, here), Ok(&b"payload"[..]));
        let mut other = m.clone();
        other[40] ^= 1;
        assert_eq!(open(&other, id, &BUILD, None, here), Err(Refusal::OtherRun));
        let mut unmarked = sealed(b"payload", id, None);
        unmarked[6] |= 2;
        assert_eq!(open(&unmarked, id, &BUILD, None, here), Err(Refusal::OtherRun));
    }
}
