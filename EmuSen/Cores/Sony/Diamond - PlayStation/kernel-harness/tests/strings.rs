//! The memory, string and conversion calls, each against what psx-spx's description of it says, its documented bugs
//! included (Diamond_OpenKernel.md §4.8).

use diamond_kernel_harness::image;
use diamond_kernel_harness::machine::Machine;

const S1: u32 = 0x8010_0000;
const S2: u32 = 0x8010_0100;
const D: u32 = 0x8010_0200;
const END: u32 = 0x8010_0300;

fn booted() -> Machine {
    Machine::boot(&image::committed())
}

fn neg(v: i32) -> u32 {
    v as u32
}

#[test]
fn memcpy_memset_bcopy_bzero_and_their_refusals() {
    let mut m = booted();
    m.text(S1, "ABCDEFGH");
    assert_eq!(m.a(0x2A, &[D, S1, 4]), D);
    assert_eq!(m.rig.bytes(D, 5), b"ABCD\0");
    assert_eq!(m.a(0x2A, &[0, S1, 4]), 0, "dst 0 refused, dst returned");
    assert_eq!(m.a(0x2A, &[D, S1, 0x8000_0000]), D, "len over 7FFFFFFFh refused");
    assert_eq!(m.a(0x2B, &[D, 'x' as u32, 3]), D);
    assert_eq!(m.rig.bytes(D, 4), b"xxxD");
    assert_eq!(m.a(0x2B, &[D, 'y' as u32, 0]), 0, "memset returns 0 for len 0");
    assert_eq!(m.a(0x27, &[S1, D, 2]), S1, "bcopy: src first, src returned");
    assert_eq!(m.rig.bytes(D, 3), b"ABx");
    assert_eq!(m.a(0x27, &[0, D, 2]), 0);
    assert_eq!(m.a(0x28, &[D, 2]), D);
    assert_eq!(m.rig.bytes(D, 3), b"\0\0x");
}

#[test]
fn memmove_copies_len_plus_1_backwards_and_forwards_into_an_overlap() {
    let mut m = booted();
    m.text(S1, "ABCDEFGHX");
    m.text(S1 + 8 + 0x10, "zzzzzzzzZ");
    let dst = S1 + 0x18;
    assert_eq!(m.a(0x2C, &[dst, S1, 8]), dst);
    assert_eq!(m.rig.bytes(dst, 9), b"ABCDEFGHX", "nine bytes: the documented extra one");
    m.text(S2, "ABCDEF");
    m.a(0x2C, &[S2 + 2, S2, 4]);
    assert_eq!(m.rig.bytes(S2, 6), b"ABABAB", "dst < src+len: a forward copy");
}

#[test]
fn memcmp_and_bcmp_return_the_difference_after_the_first_mismatch() {
    let mut m = booted();
    m.text(S1, "ABCD");
    m.text(S2, "ABXD");
    assert_eq!(m.a(0x2D, &[S1, S2, 4]), 0, "C/X differ, D/D follow: 0");
    m.text(S2, "ABXE");
    assert_eq!(m.a(0x2D, &[S1, S2, 4]), neg(-1));
    assert_eq!(m.a(0x29, &[S1, S2, 4]), neg(-1));
    assert_eq!(m.a(0x2D, &[0, S2, 4]), 0);
    assert_eq!(m.a(0x2D, &[S1, S1, 4]), 0);
    assert_eq!(m.a(0x2E, &[S1, 'C' as u32, 4]), S1 + 2);
    assert_eq!(m.a(0x2E, &[S1, 'Z' as u32, 4]), 0);
}

#[test]
fn strcat_strncat_strcpy_strncpy_and_strlen() {
    let mut m = booted();
    m.text(D, "ab");
    m.text(S1, "cde");
    assert_eq!(m.a(0x15, &[D, S1]), D);
    assert_eq!(m.rig.string(D), "abcde");
    assert_eq!(m.a(0x15, &[0, S1]), 0);
    m.text(D, "ab");
    assert_eq!(m.a(0x16, &[D, S1, 2]), D);
    assert_eq!(m.rig.string(D), "abcd");
    m.text(D, "ab");
    m.a(0x16, &[D, S1, 9]);
    assert_eq!(m.rig.string(D), "abcde");
    assert_eq!(m.a(0x19, &[D, S1]), D);
    assert_eq!(m.rig.string(D), "cde");
    assert_eq!(m.a(0x19, &[D, 0]), 0);
    m.rig.put_bytes(D, b"########");
    assert_eq!(m.a(0x1A, &[D, S1, 2]), D);
    assert_eq!(m.rig.bytes(D, 3), b"cd#", "no 00h when src is maxlen or longer");
    m.a(0x1A, &[D, S1, 6]);
    assert_eq!(m.rig.bytes(D, 7), b"cde\0\0\0#", "padded with 00h to maxlen");
    assert_eq!(m.a(0x1B, &[S1]), 3);
    assert_eq!(m.a(0x1B, &[0]), 0);
}

#[test]
fn strcmp_and_strncmp_with_sign_extended_bytes_and_null_pointers() {
    let mut m = booted();
    m.text(S1, "abc");
    m.text(S2, "abd");
    assert_eq!(m.a(0x17, &[S1, S2]), neg(-1));
    assert_eq!(m.a(0x17, &[S1, S1]), 0);
    assert_eq!(m.a(0x18, &[S1, S2, 2]), 0);
    m.rig.put_bytes(S1, &[0x80, 0]);
    m.rig.put_bytes(S2, &[0x01, 0]);
    assert_eq!(m.a(0x17, &[S1, S2]), neg(-129), "80h is -128");
    assert_eq!(m.a(0x17, &[0, 0]), 0);
    assert_eq!(m.a(0x17, &[0, S2]), neg(-1));
    assert_eq!(m.a(0x17, &[S1, 0]), 1);
}

#[test]
fn index_rindex_strchr_strrchr_strpbrk_strspn_strcspn() {
    let mut m = booted();
    m.text(S1, "hello");
    assert_eq!(m.a(0x1C, &[S1, 'l' as u32]), S1 + 2);
    assert_eq!(m.a(0x1E, &[S1, 'l' as u32]), S1 + 2);
    assert_eq!(m.a(0x1D, &[S1, 'l' as u32]), S1 + 3);
    assert_eq!(m.a(0x1F, &[S1, 'l' as u32]), S1 + 3);
    assert_eq!(m.a(0x1C, &[S1, 0]), S1 + 5, "char 00h: the end");
    assert_eq!(m.a(0x1D, &[S1, 0]), S1 + 5);
    assert_eq!(m.a(0x1C, &[S1, 'z' as u32]), 0);
    assert_eq!(m.a(0x1C, &[0, 'h' as u32]), 0);
    m.text(S2, "le");
    assert_eq!(m.a(0x20, &[S1, S2]), S1 + 1);
    m.text(S2, "xyz");
    assert_eq!(m.a(0x20, &[S1, S2]), S1, "no occurrence: src itself, as documented");
    m.text(D, "");
    assert_eq!(m.a(0x20, &[D, S2]), 0, "unless src is empty");
    m.text(S2, "eh");
    assert_eq!(m.a(0x21, &[S1, S2]), 2);
    m.text(S2, "lo");
    assert_eq!(m.a(0x22, &[S1, S2]), 2);
    m.text(S2, "q");
    assert_eq!(m.a(0x22, &[S1, S2]), 5, "none: the length");
}

#[test]
fn strtok_splits_as_psx_spx_s_two_examples() {
    let mut m = booted();
    m.text(S1, ",,TEXT,,,END");
    for (list, want) in [(",", vec!["", "TEXT", "END"]), (",.", vec!["", "", "TEXT", "", "", "END"])] {
        m.text(S2, list);
        let mut got = Vec::new();
        let mut src = S1;
        loop {
            let t = m.a(0x23, &[src, S2]);
            src = 0;
            if t == 0 {
                break;
            }
            got.push(m.rig.string(t));
            assert!(got.len() < 10);
        }
        assert_eq!(got, want, "list {list:?}");
    }
}

#[test]
fn strstr_resumes_after_the_mismatch_so_aab_is_not_in_aaab() {
    let mut m = booted();
    m.text(S1, "aaab");
    m.text(S2, "aab");
    assert_eq!(m.a(0x24, &[S1, S2]), 0);
    m.text(S1, "xxabc");
    m.text(S2, "abc");
    assert_eq!(m.a(0x24, &[S1, S2]), S1 + 2);
    m.text(S2, "abd");
    assert_eq!(m.a(0x24, &[S1, S2]), 0);
}

#[test]
fn characters_and_digits() {
    let mut m = booted();
    assert_eq!(m.a(0x25, &['q' as u32]), 'Q' as u32);
    assert_eq!(m.a(0x25, &[0x141]), 'A' as u32, "ANDed with FFh");
    assert_eq!(m.a(0x26, &['Q' as u32]), 'q' as u32);
    assert_eq!(m.a(0x0A, &['7' as u32]), 7);
    assert_eq!(m.a(0x0A, &['z' as u32]), 35);
    assert_eq!(m.a(0x0A, &['!' as u32]), 9_999_999);
    assert_eq!(m.a(0x0E, &[neg(-5)]), 5);
    assert_eq!(m.a(0x0F, &[5]), 5);
}

#[test]
fn strtol_strtoul_atoi_atol_and_atob_with_their_prefixes() {
    let mut m = booted();
    let cases: [(&str, u32, i32, usize); 8] = [
        ("  -0x1F", 10, -31, 7),
        ("0b11", 16, 3, 4),
        ("o55", 36, 45, 3),
        ("b11", 16, 0xB11, 3),
        ("+5", 10, 0, 0),
        ("12", 1, 12, 2),
        ("789z", 8, 7, 1),
        ("\t42!", 10, 42, 3),
    ];
    for (s, base, want, end) in cases {
        m.text(S1, s);
        assert_eq!(m.a(0x0D, &[S1, END, base]), want as u32, "strtol({s:?}, {base})");
        assert_eq!(m.rig.word(END), S1 + end as u32, "strtol({s:?}) end");
    }
    m.text(S1, "-5");
    assert_eq!(m.a(0x0C, &[S1, END, 10]), 0, "strtoul takes no sign");
    m.rig.put(END, 0x1234);
    assert_eq!(m.a(0x0D, &[0, END, 10]), 0);
    assert_eq!(m.rig.word(END), 0x1234, "a null src leaves [src_end]");
    for (s, want) in [("010", 8), ("0x10", 16), ("-12", -12), ("0b101", 5), ("7", 7)] {
        m.text(S1, s);
        assert_eq!(m.a(0x10, &[S1]), want as u32, "atoi({s:?})");
        assert_eq!(m.a(0x11, &[S1]), want as u32, "atol({s:?})");
    }
    m.text(S1, "123x");
    assert_eq!(m.a(0x12, &[S1, END]), S1 + 3, "atob returns the end");
    assert_eq!(m.rig.word(END), 123);
}

#[test]
fn rand_is_the_documented_generator() {
    let mut m = booted();
    m.a(0x30, &[1]);
    let mut x: u32 = 1;
    for _ in 0..4 {
        x = x.wrapping_mul(0x41C6_4E6D).wrapping_add(0x3039);
        assert_eq!(m.a(0x2F, &[]), x >> 16 & 0x7FFF);
    }
}

#[test]
fn setjmp_records_the_saved_registers_and_longjmp_returns_through_them() {
    let mut m = booted();
    m.cpu.regs.r[16] = 0x1616;
    m.cpu.regs.r[28] = 0x2828;
    m.cpu.regs.r[30] = 0x3030;
    assert_eq!(m.a(0x13, &[D]), 0);
    let w: Vec<u32> = (0..12).map(|i| m.rig.word(D + 4 * i)).collect();
    assert_eq!(
        &w[..4],
        &[diamond_kernel_harness::machine::SENTINEL, diamond_kernel_harness::machine::STACK, 0x3030, 0x1616]
    );
    assert_eq!(w[11], 0x2828);
    m.cpu.regs.r[16] = 0;
    assert_eq!(m.a(0x14, &[D, 7]), 7, "longjmp's value, unclamped");
    assert_eq!(m.cpu.regs.r[16], 0x1616);
}
