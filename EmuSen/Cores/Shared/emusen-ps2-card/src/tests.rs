//! The crate's own round trips, and damaged cards refused in words without a panic (Hardware/Shared/EmuSen_MediumFormats.md §6.5).

extern crate std;

use std::vec;
use std::vec::Vec;

use super::*;

const T: Time = Time { sec: 1, min: 2, hour: 3, day: 25, month: 10, year: 2026 };

fn blank() -> Vec<u8> {
    let mut c = vec![0u8; CARD_LENGTH];
    format(&mut c, T).unwrap();
    c
}

#[test]
fn a_formatted_card_has_the_standard_layout_and_8000_clusters_free() {
    let c = blank();
    let card = Card::open(&c).unwrap();
    let sb = card.sb;
    assert_eq!((sb.alloc_offset, sb.alloc_end, sb.backup_block1, sb.backup_block2, sb.ifc_list[0]), (41, 8135, 1023, 1022, 8), "Ridge's defaults");
    assert_eq!((sb.card_type, sb.card_flags), (2, 0x52));
    assert_eq!(card.free_clusters().unwrap(), 7999, "Ridge: rounded down to the thousand, the root's cluster in use");
    let root = card.root().unwrap();
    assert_eq!(root.entries, 2);
    assert_eq!(card.entry(root, 0).unwrap().name(), b".");
    assert_eq!(card.entry(root, 1).unwrap().name(), b"..");
    assert_eq!(saves(&card).count(), 0);
    for p in [0usize, 16, 17, 18, 81, 82, 83] {
        let mut d = [0u8; PAGE];
        d.copy_from_slice(page(&c, p));
        assert_eq!(&spare(&c, p)[..12], &ecc::page(&d), "page {p}'s code");
        assert_eq!(&spare(&c, p)[12..], &[0xFF; 4]);
    }
    assert!(erased(&c, 1) && erased(&c, 82 + 2) && erased(&c, 1022 * 16), "the rest erased");
}

#[test]
fn an_erased_chunks_code_reads_77h_7fh_7fh_and_one_wrong_bit_is_put_right() {
    assert_eq!(ecc::chunk(&[0xFF; 128]), [0x77, 0x7F, 0x7F], "D-226's witness");
    assert_eq!(ecc::chunk(&[0x00; 128]), [0x77, 0x7F, 0x7F], "even parity everywhere for zeros too");
    let mut d = [0u8; 128];
    for (i, b) in d.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(37).wrapping_add(11);
    }
    let code = ecc::chunk(&d);
    for byte in [0usize, 1, 64, 127] {
        for bit in 0..8u8 {
            let mut bad = d;
            bad[byte] ^= 1 << bit;
            assert_eq!(ecc::correct(&mut bad, &code), ecc::Check::Corrected { byte, bit });
            assert_eq!(bad, d);
        }
    }
    let mut two = d;
    two[3] ^= 1;
    two[90] ^= 4;
    assert_eq!(ecc::correct(&mut two, &code), ecc::Check::Uncorrectable);
    let mut code_bit = code;
    code_bit[1] ^= 0x10;
    assert_eq!(ecc::correct(&mut d.clone(), &code_bit), ecc::Check::CodeBit);
}

#[test]
fn saves_made_read_back_listed_and_deleted() {
    let mut c = blank();
    mkdir(&mut c, b"", b"BASLUS-20000GAME", T).unwrap();
    let data: Vec<u8> = (0..5000u32).map(|i| (i * 7 + 1) as u8).collect();
    create(&mut c, b"BASLUS-20000GAME", b"icon.sys", &data[..964], T).unwrap();
    create(&mut c, b"BASLUS-20000GAME", b"data", &data, T).unwrap();
    create(&mut c, b"BASLUS-20000GAME", b"empty", &[], T).unwrap();
    for n in 0..5 {
        let name = [b'f', b'0' + n];
        create(&mut c, b"BASLUS-20000GAME", &name, &data[..100], T).unwrap();
    }
    let card = Card::open(&c).unwrap();
    let list: Vec<_> = saves(&card).map(|s| s.unwrap()).collect();
    assert_eq!(list.len(), 1);
    let (_, save) = list[0];
    assert_eq!(save.name(), b"BASLUS-20000GAME");
    let d = card.dir(&save).unwrap();
    assert_eq!(d.entries, 2 + 8, "eight files after . and ..; the directory grew past its first cluster");
    let (_, f) = card.find(d, b"data").unwrap();
    let mut back = vec![0u8; 6000];
    assert_eq!(card.read(&f, 0, &mut back).unwrap(), 5000);
    assert_eq!(&back[..5000], &data[..]);
    assert_eq!(card.read(&f, 4096, &mut back[..10]).unwrap(), 10);
    assert_eq!(&back[..10], &data[4096..4106]);
    // 5 clusters of data, 1 of icon.sys, 5 of the f files, the directory's 5.
    assert_eq!(card.clusters_of(&save).unwrap(), 5 + 1 + 5 + 5);
    assert_eq!(card.free_clusters().unwrap(), 7999 - 1 - 16, "the root grown by a cluster for its third entry");
    assert_eq!(mkdir(&mut c, b"", b"BASLUS-20000GAME", T), Err(Refusal::Exists));
    assert_eq!(create(&mut c, b"BASLUS-20000GAME", b"a/b", &[], T), Err(Refusal::Name));
    assert_eq!(create(&mut c, b"NOPE", b"x", &[], T), Err(Refusal::Missing));
    assert_eq!(create(&mut c, b"BASLUS-20000GAME/data", b"x", &[], T), Err(Refusal::Kind));
    delete(&mut c, b"BASLUS-20000GAME/data").unwrap();
    assert_eq!(Card::open(&c).unwrap().free_clusters().unwrap(), 7999 - 1 - 11);
    delete(&mut c, b"BASLUS-20000GAME").unwrap();
    let card = Card::open(&c).unwrap();
    assert_eq!(card.free_clusters().unwrap(), 7998, "the root keeps its second cluster");
    assert_eq!(saves(&card).count(), 0);
}

#[test]
fn a_save_copied_to_another_card_reads_the_same_and_a_full_or_clashing_copy_is_refused() {
    let mut a = blank();
    mkdir(&mut a, b"", b"BESLES-50000", T).unwrap();
    let data: Vec<u8> = (0..3000u32).map(|i| (i % 251) as u8).collect();
    create(&mut a, b"BESLES-50000", b"save", &data, T).unwrap();
    let mut b = blank();
    copy_save(&a, b"BESLES-50000", &mut b).unwrap();
    let (ca, cb) = (Card::open(&a).unwrap(), Card::open(&b).unwrap());
    let (_, sa) = ca.find(ca.root().unwrap(), b"BESLES-50000").unwrap();
    let (_, sb) = cb.find(cb.root().unwrap(), b"BESLES-50000").unwrap();
    assert_eq!((sa.mode, sa.created, sa.modified), (sb.mode, sb.created, sb.modified));
    let (_, fb) = cb.find(cb.dir(&sb).unwrap(), b"save").unwrap();
    let mut back = vec![0u8; 3000];
    cb.read(&fb, 0, &mut back).unwrap();
    assert_eq!(back, data);
    assert_eq!(ca.clusters_of(&sa).unwrap(), cb.clusters_of(&sb).unwrap());
    let before = b.clone();
    assert_eq!(copy_save(&a, b"BESLES-50000", &mut b), Err(Refusal::Exists));
    assert_eq!(b, before, "a refusal leaves the card as it was");
    let mut full = blank();
    let big = vec![0x5Au8; 900 * CLUSTER];
    for n in 0..20u8 {
        let free = Card::open(&full).unwrap().free_clusters().unwrap();
        if free <= 4 {
            break;
        }
        mkdir(&mut full, b"", &[b'D', b'A' + n], T).unwrap();
        let room = Card::open(&full).unwrap().free_clusters().unwrap();
        create(&mut full, &[b'D', b'A' + n], b"x", &big[..(room - 1).min(900) * CLUSTER], T).unwrap();
    }
    let free = Card::open(&full).unwrap().free_clusters().unwrap();
    assert!(free <= 4, "{free} clusters left");
    assert_eq!(create(&mut full, b"DA", b"y", &big[..(free + 1) * CLUSTER], T), Err(Refusal::Full));
    let before = full.clone();
    assert_eq!(copy_save(&a, b"BESLES-50000", &mut full), Err(Refusal::Full));
    assert_eq!(full, before);
}

#[test]
fn damaged_cards_are_refused_in_words_never_a_panic() {
    assert_eq!(Card::open(&[0u8; 100]).err(), Some(Damage::Length));
    // What PCSX2 2.8.2 makes for a new card, every byte FFh (its Mcd001.ps2's MD5 914e26ce1e2661e14d39d87310d3cdd3, measured 2026-10-25).
    let ff = vec![0xFFu8; CARD_LENGTH];
    assert_eq!(Card::open(&ff).err(), Some(Damage::Unformatted));
    assert_eq!(Damage::Unformatted.words(), "the card is not formatted");
    let good = blank();
    let mut bad = good.clone();
    bad[0x150] = 1;
    assert_eq!(Card::open(&bad).err(), Some(Damage::Superblock));
    // Two bits wrong in the FAT's first page: beyond what the code repairs.
    let mut bad = good.clone();
    bad[18 * RAW_PAGE] ^= 0x03;
    assert_eq!(Card::open(&bad).err(), Some(Damage::Ecc), "the FAT's first page read through its code");
    // One bit wrong is repaired.
    let mut one = good.clone();
    one[82 * RAW_PAGE + 5] ^= 0x10;
    assert!(Card::open(&one).is_ok());
    // A FAT entry pointing at itself: a loop.
    let mut lp = good.clone();
    mkdir(&mut lp, b"", b"LOOP", T).unwrap();
    create(&mut lp, b"LOOP", b"f", &[1u8; 3000], T).unwrap();
    let sb = Superblock::read(&lp).unwrap();
    let (_, f) = { let c = Card::open(&lp).unwrap(); let d = c.dir(&c.find(c.root().unwrap(), b"LOOP").unwrap().1).unwrap(); c.find(d, b"f").unwrap() };
    set_fat(&mut lp, &sb, f.cluster, FAT_USED | f.cluster).unwrap();
    let card = Card::open(&lp).unwrap();
    let (_, save) = card.find(card.root().unwrap(), b"LOOP").unwrap();
    assert_eq!(card.clusters_of(&save), Err(Damage::Chain));
    assert_eq!(copy_save(&lp, b"LOOP", &mut blank()), Err(Refusal::Damaged(Damage::Chain)));
    // Every byte of the superblock, the FAT's first page and the root, each flipped in turn: some answer, never a panic.
    for at in (0..0x160).chain(18 * RAW_PAGE..18 * RAW_PAGE + 64).chain(82 * RAW_PAGE..82 * RAW_PAGE + 0x60) {
        let mut x = good.clone();
        x[at] ^= 0xA5;
        if let Ok(card) = Card::open(&x) {
            let _ = card.free_clusters();
            let _ = saves(&card).count();
        }
        let _ = mkdir(&mut x, b"", b"NEW", T);
        let _ = delete(&mut x, b"NEW");
    }
}
