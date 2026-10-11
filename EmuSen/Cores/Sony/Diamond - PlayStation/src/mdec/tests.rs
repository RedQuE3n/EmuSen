//! The MDEC's register interface on words written by the CPU: the commands, the status, the FIFOs and the order.

use super::*;

/// psx-spx's standard scale table ("set_scale_table").
const SCALE: [u16; 64] = [
    0x5A82, 0x5A82, 0x5A82, 0x5A82, 0x5A82, 0x5A82, 0x5A82, 0x5A82, 0x7D8A, 0x6A6D, 0x471C, 0x18F8, 0xE707, 0xB8E3, 0x9592, 0x8275, 0x7641, 0x30FB, 0xCF04, 0x89BE, 0x89BE, 0xCF04, 0x30FB, 0x7641, 0x6A6D, 0xE707, 0x8275,
    0xB8E3, 0x471C, 0x7D8A, 0x18F8, 0x9592, 0x5A82, 0xA57D, 0xA57D, 0x5A82, 0x5A82, 0xA57D, 0xA57D, 0x5A82, 0x471C, 0x8275, 0x18F8, 0x6A6D, 0x9592, 0xE707, 0x7D8A, 0xB8E3, 0x30FB, 0x89BE, 0x7641, 0xCF04, 0xCF04, 0x7641,
    0x89BE, 0x30FB, 0x18F8, 0xB8E3, 0x6A6D, 0x8275, 0x7D8A, 0x9592, 0x471C, 0xE707,
];

fn tables(m: &mut Mdec) {
    m.write(0x4000_0001, 0);
    for _ in 0..32 {
        m.write(0x0202_0202, 0);
    }
    m.write(0x6000_0000, 0);
    for i in 0..32 {
        m.write(SCALE[2 * i] as u32 | (SCALE[2 * i + 1] as u32) << 16, 0);
    }
}

fn read(m: &mut Mdec, n: usize, now: u64) -> Vec<u32> {
    (0..n).map(|_| m.read(now).0).collect()
}

#[test]
fn a_reset_and_the_no_function_commands_show_in_the_status() {
    let mut m = Mdec::new();
    assert_eq!(m.status(), 0x8004_0000);
    m.write(0x1E00_1234, 0);
    assert_eq!(m.status(), 0x8784_1234, "MDEC(0): bits 25-28 to 23-26 and bits 0-15 as written");
    m.write(0x6000_0000, 0);
    assert_eq!(m.status() & 0xFFFF, 31, "MDEC(3) expects 32 words, shown less one");
    m.control(0x8000_0000, 0);
    assert_eq!(m.status(), 0x8004_0000);
}

#[test]
fn the_decoder_waits_for_32_words_or_the_last_and_holds_a_block_behind_a_full_output() {
    let mut m = Mdec::new();
    tables(&mut m);
    m.write(0x3800_0000 | 64, 0);
    for i in 0..31 {
        m.write(0xFE00_0400, i);
    }
    assert_eq!(m.status() >> 31, 1, "31 words: nothing taken yet");
    m.write(0xFE00_0400, 100);
    m.run(100_000);
    assert_eq!(m.status() >> 16 & 7, 1, "Y1 out, Y2 the block arriving");
    assert_eq!(m.queue_len, OUT_WORDS);
    let first = read(&mut m, 32, 100_000);
    assert!(first.iter().all(|&w| w == first[0]), "a flat block");
}

#[test]
fn blocks_of_a_few_words_decode_one_after_another_at_four_bits() {
    let mut m = Mdec::new();
    tables(&mut m);
    for words in [[0xFC00_035F, 0x0000_FE00], [0x0032_0360, 0xFE00_F800]] {
        m.write(0x2000_0002, 0);
        m.write(words[0], 0);
        m.write(words[1], 0);
        let out = read(&mut m, 8, 0);
        assert_ne!(out, [0; 8], "{words:08X?}: {out:08X?}");
        assert_eq!(m.status() & 0x8000_0000, 0x8000_0000, "{words:08X?}: the output read whole");
    }
}

#[test]
fn dma1_writes_a_colour_macroblocks_words_row_by_row() {
    assert_eq!(reorder(Depth::Fifteen, 0, 0), 0);
    assert_eq!(reorder(Depth::Fifteen, 0, 4), 4, "Y1's second row starts 8 words on");
    assert_eq!(reorder(Depth::Fifteen, 1, 0), 4 - 32, "Y2 beside Y1");
    assert_eq!(reorder(Depth::Fifteen, 3, 31), 127 - 127);
    assert_eq!(reorder(Depth::TwentyFour, 2, 6), 9 * 12 - (96 + 6));
    assert_eq!(reorder(Depth::Eight, 4, 9), 0);
}
