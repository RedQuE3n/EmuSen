//! Step 6's gate for the formats (EmuSen_GamePackages.md §7): each format applied to synthetic images reproduces a
//! target made independently in the test, by building the target first and then the patch from source to target with
//! the test's own encoder; mismatched sources are refused with words naming both checksums.

mod encoders;

use emusen_patch::{apply, apply_list, info, Format, Refusal, Step, MAX_IMAGE};

/// A target built from a source with nothing of the crate: bytes changed here and there, a block of the source moved
/// back and one moved forward, a run of one byte, a repeated phrase, and the length changed by `grow` (negative cuts).
fn target_from(source: &[u8], grow: i64) -> Vec<u8> {
    let mut t = source.to_vec();
    for i in (17..t.len()).step_by(997) {
        t[i] ^= 0xA5;
    }
    let n = t.len();
    t.copy_within(n / 2..n / 2 + 700, n / 8);
    t.copy_within(n / 16..n / 16 + 500, n * 3 / 4);
    t[n / 3..n / 3 + 300].fill(0x5A);
    let phrase = b"EmuSen soft patch ";
    for (k, b) in t[n / 5..n / 5 + 900].iter_mut().enumerate() {
        *b = phrase[k % phrase.len()];
    }
    if grow >= 0 {
        t.extend(encoders::image(grow as usize, 0xBEEF));
    } else {
        t.truncate((n as i64 + grow) as usize);
    }
    t
}

fn sizes() -> [(usize, i64); 3] {
    [(40_000, 0), (40_000, 3_000), (40_000, -5_000)]
}

#[test]
fn ips_reproduces_independent_targets_with_records_runs_and_a_truncation() {
    for (n, grow) in sizes() {
        let source = encoders::image(n, 7);
        let target = target_from(&source, grow);
        for runs in [false, true] {
            let patch = encoders::ips(&source, &target, runs);
            assert_eq!(apply(&source, &patch).unwrap(), target, "IPS, {n} bytes grown by {grow}, runs {runs}");
        }
    }
    let source = encoders::image(1000, 3);
    let mut target = source.clone();
    target[100..400].fill(0x11);
    let patch = encoders::ips(&source, &target, true);
    assert!(patch.windows(3).any(|w| w == [0, 0, 1]), "the run form was written: {patch:?}");
    assert_eq!(apply(&source, &patch).unwrap(), target);
    let cut = &source[..600];
    let patch = encoders::ips(&source, cut, false);
    assert_eq!(&patch[patch.len() - 6..], &[b'E', b'O', b'F', 0, 2, 0x58], "the EOF-offset form names 600");
    assert_eq!(apply(&source, &patch).unwrap(), cut);
}

#[test]
fn ips_refuses_damage_in_words() {
    let source = encoders::image(64, 1);
    assert_eq!(apply(&source, b"PATCH\x00\x00\x10\x00\x04ab").unwrap_err().words(), "the IPS record at byte 5 holds 4 bytes and the patch ends first");
    assert!(apply(&source, b"PATCH").unwrap_err().words().contains("ends before its EOF marker"));
    assert!(apply(&source, b"PATCHEOF\x00\x01").unwrap_err().words().contains("2 bytes follow"));
    assert!(apply(&source, b"PATCHEOF\x00\x01\x00").unwrap_err().words().contains("more than the 64"), "a truncation that would lengthen is refused");
    assert_eq!(apply(&source, b"PATCH\x00\x00\x01\x00\x00\x00\x00\x07EOF").unwrap_err().words(), "the IPS run at byte 5 has a length of zero");
    assert_eq!(apply(&source, b"PATC").unwrap_err(), Refusal::NotAPatch);
}

#[test]
fn ups_reproduces_independent_targets() {
    for (n, grow) in sizes() {
        let source = encoders::image(n, 11);
        let target = target_from(&source, grow);
        let patch = encoders::ups(&source, &target);
        assert_eq!(apply(&source, &patch).unwrap(), target, "UPS, {n} bytes grown by {grow}");
        let i = info(&patch).unwrap();
        assert_eq!((i.format, i.source_size, i.target_size, i.source_crc32, i.target_crc32), (Format::Ups, Some(n as u64), Some(target.len() as u64), Some(encoders::crc32(&source)), Some(encoders::crc32(&target))));
    }
}

#[test]
fn bps_reproduces_independent_targets_with_all_four_actions() {
    let mut used = [0usize; 4];
    for (n, grow) in sizes() {
        let source = encoders::image(n, 13);
        let target = target_from(&source, grow);
        let patch = encoders::bps(&source, &target, b"<patch>made in the test</patch>");
        let actions = encoders::bps_actions(&patch);
        (0..4).for_each(|k| used[k] += actions[k]);
        assert_eq!(apply(&source, &patch).unwrap(), target, "BPS, {n} bytes grown by {grow}");
    }
    assert!(used.iter().all(|&n| n > 0), "SourceRead, TargetRead, SourceCopy and TargetCopy each used: {used:?}");
}

#[test]
fn a_mismatched_source_is_refused_naming_both_checksums() {
    let source = encoders::image(5000, 21);
    let target = target_from(&source, 100);
    let mut other = source.clone();
    other[10] ^= 1;
    let (want, found) = (encoders::crc32(&source), encoders::crc32(&other));
    for patch in [encoders::ups(&source, &target), encoders::bps(&source, &target, b"")] {
        let words = apply(&other, &patch).unwrap_err().words();
        assert!(words.contains(&format!("{want:08x}")) && words.contains(&format!("{found:08x}")) && words.contains("5000 bytes"), "{words}");
        assert!(matches!(apply(&target, &patch).unwrap_err(), Refusal::AlreadyTarget { .. }), "an image patched before is refused");
    }
}

#[test]
fn a_damaged_patch_is_refused_by_its_own_checksum_and_a_bad_action_by_the_rules() {
    let source = encoders::image(3000, 5);
    let target = target_from(&source, 0);
    for mut patch in [encoders::ups(&source, &target), encoders::bps(&source, &target, b"")] {
        let k = patch.len() / 2;
        patch[k] ^= 0x40;
        assert!(apply(&source, &patch).unwrap_err().words().contains("the patch file is damaged"));
    }
    let mut bad = b"BPS1".to_vec();
    encoders::number(&mut bad, source.len() as u64);
    encoders::number(&mut bad, 4);
    encoders::number(&mut bad, 0);
    encoders::number(&mut bad, 3 | (3 << 2));
    encoders::number(&mut bad, 0);
    bad.extend_from_slice(&encoders::crc32(&source).to_le_bytes());
    bad.extend_from_slice(&0u32.to_le_bytes());
    let own = encoders::crc32(&bad);
    bad.extend_from_slice(&own.to_le_bytes());
    assert!(apply(&source, &bad).unwrap_err().words().contains("reads target bytes not yet written"));
}

#[test]
fn a_target_past_64_mib_is_refused_before_anything_is_made() {
    let source = encoders::image(16, 2);
    let mut big = b"BPS1".to_vec();
    encoders::number(&mut big, 16);
    encoders::number(&mut big, MAX_IMAGE + 1);
    encoders::number(&mut big, 0);
    big.extend_from_slice(&encoders::crc32(&source).to_le_bytes());
    big.extend_from_slice(&0u32.to_le_bytes());
    let own = encoders::crc32(&big);
    big.extend_from_slice(&own.to_le_bytes());
    assert_eq!(apply(&source, &big).unwrap_err(), Refusal::TooLarge(MAX_IMAGE + 1));
}

#[test]
fn a_list_applies_in_order_and_pins_ips_to_the_image_it_was_added_for() {
    let a = encoders::image(8000, 31);
    let b = target_from(&a, 200);
    let c = target_from(&b, -100);
    let first = encoders::ips(&a, &b, true);
    let second = encoders::bps(&b, &c, b"");
    let pin = emusen_galaxia::rom_hash::sha1_of(&a);
    let steps = [Step { name: "one.ips", format: Format::Ips, bytes: &first, source_sha1: Some(&pin) }, Step { name: "two.bps", format: Format::Bps, bytes: &second, source_sha1: None }];
    assert_eq!(apply_list(&a, &steps).unwrap(), c);
    let words = apply_list(&b, &steps).unwrap_err();
    assert!(words.starts_with("one.ips: ") && words.contains(&pin) && words.contains(&emusen_galaxia::rom_hash::sha1_of(&b)), "{words}");
    let wrong = [Step { name: "two.bps", format: Format::Ups, bytes: &second, source_sha1: None }];
    assert_eq!(apply_list(&b, &wrong).unwrap_err(), "two.bps: the package records it as UPS, and the file is BPS");
}
