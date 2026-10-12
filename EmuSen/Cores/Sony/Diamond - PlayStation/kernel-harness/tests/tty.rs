//! The TTY at the DUART's transmit register, printf, puts and putchar, and the file layer around them; and a PS-EXE
//! that calls the three tables and prints (Diamond_OpenKernel.md §4.9, §4.10).

use diamond_kernel_harness::image;
use diamond_kernel_harness::machine::Machine;

const FMT: u32 = 0x8010_0000;
const S: u32 = 0x8010_0100;

fn booted() -> Machine {
    Machine::boot(&image::committed())
}

#[test]
fn printf_resolves_the_documented_conversions_and_prefixes() {
    let mut m = booted();
    m.text(FMT, "%d|%5s|%-4x|%04X|%c|%+i|%o|%#x|%%|%s|%.2s|%3c|%hu\n");
    m.text(S, "abc");
    let n = m.a(0x3F, &[FMT, (-42i32) as u32, S, 0x1F, 0xBEEF, 'Z' as u32, 7, 8, 255, 0, S, 'q' as u32, 0xFFFF]);
    let want = "-42|  abc|1f  |BEEF|Z|+7|10|0xff|%|<NULL>|ab|  q|4294967295\r\n";
    assert_eq!(m.rig.tty_text(), want);
    assert_eq!(n as usize, want.len() - 1, "the characters printed before LF's expansion");
}

#[test]
fn printf_writes_the_count_for_n_and_takes_widths_from_arguments() {
    let mut m = booted();
    m.text(FMT, "ab%n|%*d|%-*d|%.*d");
    m.a(0x3F, &[FMT, 0x8010_0200, 4, 7, -3i32 as u32, 8, 3, 5]);
    assert_eq!(m.rig.word(0x8010_0200), 2);
    assert_eq!(m.rig.tty_text(), "ab|   7|8  |005");
}

/// ps1-tests' cpu/access-time prints "%2d.%2-d", and its console log shows "5.3 " for 5 and 3: a flag after the width
/// still applies, as psx-spx lists the prefixes as a set between "%" and the conversion (Diamond_OpenKernel.md §4.10).
#[test]
fn printf_takes_a_flag_after_the_width_as_the_console_s_log_shows() {
    let mut m = booted();
    m.text(FMT, "%2d.%2-d|%2-d|%-2d|%5+d|%4-x|%3 d");
    m.a(0x3F, &[FMT, 5, 3, 21, 7, 9, 0xAB, 4]);
    assert_eq!(m.rig.tty_text(), " 5.3 |21|7 |   +9|ab  |  4");
}

#[test]
fn putchar_expands_tab_and_lf_and_puts_adds_nothing_and_prints_null_as_documented() {
    let mut m = booted();
    for c in "a\tb\n".chars() {
        m.a(0x3C, &[c as u32]);
    }
    m.b(0x3D, &['c' as u32]);
    m.text(S, "x\ny");
    m.a(0x3E, &[S]);
    m.b(0x3F, &[0]);
    assert_eq!(m.rig.tty_text(), "a       b\r\ncx\r\ny<NULL>");
}

#[test]
fn the_c_table_reads_on_into_the_b_table() {
    let mut m = booted();
    m.c(0x80 + 0x3D, &['!' as u32]);
    assert_eq!(m.rig.tty_text(), "!");
}

#[test]
fn files_on_the_tty_and_the_errors_of_the_file_layer() {
    let mut m = booted();
    m.text(S, "hi");
    assert_eq!(m.a(0x03, &[1, S, 2]), 2);
    assert_eq!(m.a(0x09, &['!' as u32, 1]), 1, "putc: the handle second");
    assert_eq!(m.rig.tty_text(), "hi!");
    assert_eq!(m.a(0x07, &[1]), 1, "isatty: the DUART TTY's flags have bit 1");
    assert_eq!(m.a(0x01, &[1, 5, 0]), 5, "lseek from the start");
    assert_eq!(m.a(0x01, &[1, 0, 2]), u32::MAX, "seeking from the end refused");
    assert_eq!(m.b(0x54, &[]), 0x16);
    assert_eq!(m.b(0x55, &[1]), 0x16, "the FCB's own error");
    assert_eq!(m.a(0x02, &[0, S, 1]), 0, "no input: nothing read");
    assert_eq!(m.a(0x3B, &[]), 0, "getchar");
    m.text(S, "pcdrv:\\SYSTEM.CNF;1");
    assert_eq!(m.a(0x00, &[S, 1]), u32::MAX, "no PC file server's device");
    assert_eq!(m.b(0x54, &[]), 0x13, "unknown device");
    m.text(S, "bu00:");
    assert_eq!(m.b(0x42, &[S, 0x8010_0400]), 0, "firstfile2 with no card answering");
    m.text(S, "tty2:");
    assert_eq!(m.a(0x00, &[S, 2]), u32::MAX);
    assert_eq!(m.b(0x54, &[]), 0x06, "bad device port");
    assert_eq!(m.a(0x04, &[9]), u32::MAX);
    assert_eq!(m.b(0x54, &[]), 0x09, "invalid handle");
    assert_eq!(m.b(0x55, &[9]), u32::MAX);
    m.text(S, "tty:");
    let fd = m.b(0x32, &[S, 2]);
    assert_eq!(fd, 2, "the first free handle after std_in and std_out");
    assert_eq!(m.a(0x04, &[fd]), fd);
}

#[test]
fn kernel_redirect_mounts_the_dummy_tty_and_back() {
    let mut m = booted();
    m.c(0x1B, &[0]);
    m.a(0x3C, &['x' as u32]);
    assert_eq!(m.a(0x07, &[1]), 0, "the dummy's flags lack bit 1");
    assert_eq!(m.rig.tty_text(), "");
    m.c(0x1B, &[1]);
    m.a(0x3C, &['y' as u32]);
    assert_eq!(m.rig.tty_text(), "y");
    m.b(0x49, &[]);
    assert_eq!(
        m.rig.tty_text(),
        "y       tty:    CONSOLE\r\n        cdrom:  CD-ROM\r\n        bu:     MEMORY CARD\r\n",
        "PrintInstalledDevices, its tabs expanded"
    );
}

#[test]
fn a_ps_exe_calls_the_three_tables_and_prints_through_the_tty() {
    let mut m = booted();
    m.text(0x8011_0000, "%s %d\n");
    m.text(0x8011_0100, "printf");
    m.text(0x8011_0200, "strlen(\"printf\")=");
    m.run_program(
        "addiu sp, sp, -32\n\
         li a0, 0x80110100\naddiu t2, zero, 0xA0\njalr t2\naddiu t1, zero, 0x1B\n\
         or s0, v0, zero\n\
         li a0, 0x80110200\naddiu t2, zero, 0xB0\njalr t2\naddiu t1, zero, 0x3F\n\
         addiu a0, s0, 0x30\naddiu t2, zero, 0xA0\njalr t2\naddiu t1, zero, 0x3C\n\
         addiu a0, zero, 0x0A\naddiu t2, zero, 0xC0\njalr t2\naddiu t1, zero, 0xBD\n\
         li a0, 0x80110000\nli a1, 0x80110100\naddiu a2, zero, 1997\naddiu t2, zero, 0xA0\njalr t2\naddiu t1, zero, 0x3F\n\
         done: j done\nnop",
        200_000,
    );
    assert_eq!(m.rig.tty_text(), "strlen(\"printf\")=6\r\nprintf 1997\r\n");
}
