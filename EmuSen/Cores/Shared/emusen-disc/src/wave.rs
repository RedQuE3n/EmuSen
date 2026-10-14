//! A RIFF WAVE file as a cue sheet's audio track: where its samples begin and how many bytes they take. The track's
//! samples must be the disc's own, 16-bit stereo PCM at 44,100 Hz, little-endian, which a WAVE file stores as a raw
//! track does (EmuSen_Disc.md §11).

/// The data chunk's offset and length, or why the file is no CD-DA WAVE file.
pub fn data(bytes: &[u8]) -> Result<(usize, usize), String> {
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("a WAVE file does not begin with RIFF and WAVE".into());
    }
    let u16_at = |a: usize| u16::from_le_bytes([bytes[a], bytes[a + 1]]);
    let u32_at = |a: usize| u32::from_le_bytes(bytes[a..a + 4].try_into().unwrap());
    let mut at = 12;
    let mut format = None;
    while at + 8 <= bytes.len() {
        let (id, len) = (&bytes[at..at + 4], u32_at(at + 4) as usize);
        let body = at + 8;
        if id == b"fmt " {
            if len < 16 || body + 16 > bytes.len() {
                return Err("a WAVE file's fmt chunk is short".into());
            }
            format = Some((u16_at(body), u16_at(body + 2), u32_at(body + 4), u16_at(body + 14)));
        } else if id == b"data" {
            match format {
                Some((1, 2, 44_100, 16)) => return Ok((body, len.min(bytes.len() - body))),
                Some((f, c, r, b)) => return Err(format!("a WAVE file of format {f}, {c} channels, {r} Hz and {b} bits is not CD audio (PCM, 2, 44,100, 16)")),
                None => return Err("a WAVE file's data chunk comes before its fmt chunk".into()),
            }
        }
        at = body + len + (len & 1);
    }
    Err("a WAVE file has no data chunk".into())
}

/// A WAVE file of `samples` stereo frames, each `frame(i)`, for tests.
#[cfg(test)]
pub fn build(frames: usize, frame: impl Fn(usize) -> (i16, i16)) -> Vec<u8> {
    let data = frames * 4;
    let mut w = Vec::with_capacity(44 + data);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + data as u32).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    for v in [1u16, 2] {
        w.extend_from_slice(&v.to_le_bytes());
    }
    w.extend_from_slice(&44_100u32.to_le_bytes());
    w.extend_from_slice(&(44_100u32 * 4).to_le_bytes());
    for v in [4u16, 16] {
        w.extend_from_slice(&v.to_le_bytes());
    }
    w.extend_from_slice(b"data");
    w.extend_from_slice(&(data as u32).to_le_bytes());
    for i in 0..frames {
        let (l, r) = frame(i);
        w.extend_from_slice(&l.to_le_bytes());
        w.extend_from_slice(&r.to_le_bytes());
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wave_files_samples_are_found_and_others_refused() {
        let w = build(588, |i| (i as i16, -(i as i16)));
        assert_eq!(data(&w), Ok((44, 588 * 4)));
        let mut list = w.clone();
        // A LIST chunk of odd length before the data: skipped with its pad byte.
        list.splice(36..36, b"LIST\x03\x00\x00\x00abc\x00".iter().copied());
        assert_eq!(data(&list), Ok((56, 588 * 4)));
        let mut mono = w.clone();
        mono[22] = 1;
        assert!(data(&mono).is_err());
        assert!(data(b"RIFF\x00\x00\x00\x00WAVE").is_err());
        assert!(data(&w[..30]).is_err());
    }
}
