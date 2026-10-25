//! Encoders of the three formats written for the tests alone, from the same published descriptions and sharing no code
//! with the crate: a target is built first, then its patch from the source by these, so that applying the patch is held
//! to bytes the crate never made (EmuSen_GamePackages.md §7, step 6's gate). The CRC-32 is computed bit by bit here,
//! not by RomHash's table.

#![allow(dead_code)]

/// CRC-32, reflected polynomial 0xEDB88320, bit at a time.
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in bytes {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 == 1 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
        }
    }
    !c
}

/// A reproducible image of `n` bytes from a seed: a linear congruential generator's high bytes.
pub fn image(n: usize, seed: u32) -> Vec<u8> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            (x >> 16) as u8
        })
        .collect()
}

/// The specifications' number: seven bits a byte, low first, the last byte flagged, one taken away per continuation.
pub fn number(out: &mut Vec<u8>, mut n: u64) {
    loop {
        let x = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(0x80 | x);
            return;
        }
        out.push(x);
        n -= 1;
    }
}

fn be(out: &mut Vec<u8>, v: u64, n: usize) {
    for i in (0..n).rev() {
        out.push((v >> (8 * i)) as u8);
    }
}

/// IPS: a record for each run of differing bytes (split at 65,535, moved off the offset that reads as `EOF`), a run
/// record where eight or more equal bytes differ, the whole tail past the source written, and a truncation where the
/// target is shorter.
pub fn ips(source: &[u8], target: &[u8], runs: bool) -> Vec<u8> {
    let mut out = b"PATCH".to_vec();
    let differs = |i: usize| i >= source.len() || source[i] != target[i];
    let mut i = 0;
    while i < target.len() {
        if !differs(i) {
            i += 1;
            continue;
        }
        let mut start = i;
        if start == 0x454F46 {
            start -= 1;
        }
        let mut end = i;
        while end < target.len() && (differs(end) || end >= source.len()) && end - start < 0xFFFF {
            end += 1;
        }
        let same = runs && end - start >= 8 && target[start..end].iter().all(|&b| b == target[start]);
        be(&mut out, start as u64, 3);
        if same {
            be(&mut out, 0, 2);
            be(&mut out, (end - start) as u64, 2);
            out.push(target[start]);
        } else {
            be(&mut out, (end - start) as u64, 2);
            out.extend_from_slice(&target[start..end]);
        }
        i = end;
    }
    out.extend_from_slice(b"EOF");
    if target.len() < source.len() {
        be(&mut out, target.len() as u64, 3);
    }
    out
}

/// UPS: each block the offset from the last block's end to the next differing byte, the XORs to the first equal byte,
/// and the zero that stands for that byte; the input read as zero past its end.
pub fn ups(source: &[u8], target: &[u8]) -> Vec<u8> {
    let mut out = b"UPS1".to_vec();
    number(&mut out, source.len() as u64);
    number(&mut out, target.len() as u64);
    let s = |i: usize| source.get(i).copied().unwrap_or(0);
    let mut last = 0usize;
    let mut i = 0;
    while i < target.len() {
        if s(i) == target[i] {
            i += 1;
            continue;
        }
        number(&mut out, (i - last) as u64);
        while i < target.len() && s(i) != target[i] {
            out.push(s(i) ^ target[i]);
            i += 1;
        }
        out.push(0);
        i += 1;
        last = i;
    }
    out.extend_from_slice(&crc32(source).to_le_bytes());
    out.extend_from_slice(&crc32(target).to_le_bytes());
    let own = crc32(&out);
    out.extend_from_slice(&own.to_le_bytes());
    out
}

/// BPS, greedily: SourceRead where at least four bytes agree in place; else the longest match of six or more already
/// written (TargetCopy) or in the source (SourceCopy), found through a table of four-byte prefixes; else TargetRead
/// of literal bytes. Every action is used on the gate's images.
pub fn bps(source: &[u8], target: &[u8], metadata: &[u8]) -> Vec<u8> {
    use std::collections::HashMap;
    let mut out = b"BPS1".to_vec();
    number(&mut out, source.len() as u64);
    number(&mut out, target.len() as u64);
    number(&mut out, metadata.len() as u64);
    out.extend_from_slice(metadata);
    let mut prefixes: HashMap<[u8; 4], Vec<usize>> = HashMap::new();
    for p in 0..source.len().saturating_sub(3) {
        let e = prefixes.entry([source[p], source[p + 1], source[p + 2], source[p + 3]]).or_default();
        if e.len() < 16 {
            e.push(p);
        }
    }
    let action = |out: &mut Vec<u8>, kind: u64, length: usize| number(out, kind | ((length as u64 - 1) << 2));
    let signed = |out: &mut Vec<u8>, from: usize, to: usize| {
        if to >= from { number(out, ((to - from) as u64) << 1) } else { number(out, (((from - to) as u64) << 1) | 1) }
    };
    let (mut source_rel, mut target_rel) = (0usize, 0usize);
    let mut literal: Vec<u8> = Vec::new();
    let flush = |out: &mut Vec<u8>, literal: &mut Vec<u8>| {
        if !literal.is_empty() {
            action(out, 1, literal.len());
            out.extend_from_slice(literal);
            literal.clear();
        }
    };
    let mut i = 0;
    while i < target.len() {
        let in_place = (i..target.len()).take_while(|&j| j < source.len() && source[j] == target[j]).count();
        if in_place >= 4 {
            flush(&mut out, &mut literal);
            action(&mut out, 0, in_place);
            i += in_place;
            continue;
        }
        let mut best = (0usize, 0usize, false);
        for j in i.saturating_sub(1024)..i {
            let n = (0..target.len() - i).take_while(|&k| target[j + k] == target[i + k]).count();
            if n > best.0 {
                best = (n, j, true);
            }
        }
        if i + 4 <= target.len()
            && let Some(cands) = prefixes.get(&[target[i], target[i + 1], target[i + 2], target[i + 3]])
        {
            for &p in cands {
                let n = (0..target.len() - i).take_while(|&k| p + k < source.len() && source[p + k] == target[i + k]).count();
                if n > best.0 {
                    best = (n, p, false);
                }
            }
        }
        if best.0 >= 6 {
            flush(&mut out, &mut literal);
            let (n, at, from_target) = best;
            if from_target {
                action(&mut out, 3, n);
                signed(&mut out, target_rel, at);
                target_rel = at + n;
            } else {
                action(&mut out, 2, n);
                signed(&mut out, source_rel, at);
                source_rel = at + n;
            }
            i += n;
            continue;
        }
        literal.push(target[i]);
        i += 1;
    }
    flush(&mut out, &mut literal);
    out.extend_from_slice(&crc32(source).to_le_bytes());
    out.extend_from_slice(&crc32(target).to_le_bytes());
    let own = crc32(&out);
    out.extend_from_slice(&own.to_le_bytes());
    out
}

/// Which BPS actions a patch uses, read back by the description's grammar, so the gate can say all four were used.
pub fn bps_actions(patch: &[u8]) -> [usize; 4] {
    let read = |at: &mut usize| -> u64 {
        let (mut data, mut shift) = (0u64, 1u64);
        loop {
            let x = patch[*at];
            *at += 1;
            data += (x & 0x7f) as u64 * shift;
            if x & 0x80 != 0 {
                return data;
            }
            shift <<= 7;
            data += shift;
        }
    };
    let mut at = 4;
    read(&mut at);
    read(&mut at);
    let meta = read(&mut at) as usize;
    at += meta;
    let mut counts = [0usize; 4];
    while at < patch.len() - 12 {
        let d = read(&mut at);
        let kind = (d & 3) as usize;
        let length = (d >> 2) as usize + 1;
        counts[kind] += 1;
        match kind {
            1 => at += length,
            2 | 3 => {
                read(&mut at);
            }
            _ => {}
        }
    }
    counts
}
