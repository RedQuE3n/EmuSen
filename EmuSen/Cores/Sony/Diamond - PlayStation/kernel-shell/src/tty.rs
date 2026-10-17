//! One line a shell action, on the kernel's TTY (B(3Dh) putchar): the plain-text record tests read
//! (Diamond_Shell.md §3.3).

use crate::bios;

pub fn line(parts: &[&[u8]]) {
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            put(b' ');
        }
        for &c in part.iter().take_while(|&&c| c != 0) {
            put(c);
        }
    }
    put(b'\n');
}

fn put(c: u8) {
    unsafe { bios::putchar(c as u32) };
}
