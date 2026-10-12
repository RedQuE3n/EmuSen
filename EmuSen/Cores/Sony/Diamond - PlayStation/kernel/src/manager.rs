//! The minimal card manager of the screen with no disc: both cards' saves listed by their titles, one copied to the
//! other card, deleted, or a card formatted, through the kernel's own file calls and pad buffers. Its words and
//! layout are the kernel's own. Diamond_OpenKernel_Phase2.md §16.

use crate::file;
use crate::mem::{rd8, wr8};
use crate::screen::{fill, pause, rgb, text};
use crate::{bu, card, pad};

const PORTS: [u32; 2] = [0x00, 0x10];
const ROWS: usize = 15;
const BUFFER: u32 = 0x8010_0000;
const BACKGROUND: u32 = rgb(16, 24, 40);
const HIGHLIGHT: u32 = rgb(40, 64, 104);
const TITLE: u32 = rgb(0x50, 0x70, 0x80);
const NORMAL: u32 = rgb(0x70, 0x70, 0x70);
const DIM: u32 = rgb(0x48, 0x48, 0x48);
const WARN: u32 = rgb(0x80, 0x60, 0x30);

const SELECT_UP: u32 = 1 << 4;
const RIGHT: u32 = 1 << 5;
const DOWN: u32 = 1 << 6;
const LEFT: u32 = 1 << 7;
const TRIANGLE: u32 = 1 << 12;
const CROSS: u32 = 1 << 14;
const SQUARE: u32 = 1 << 15;

#[derive(Clone, Copy)]
struct Save {
    name: [u8; 21],
    title: [u8; 33],
    blocks: u32,
}

const EMPTY: Save = Save { name: [0; 21], title: [0; 33], blocks: 0 };

struct Card {
    state: bu::State,
    saves: [Save; ROWS],
    count: usize,
    free: u32,
}

static mut CARDS: [Card; 2] = [
    Card { state: bu::State::Absent, saves: [EMPTY; ROWS], count: 0, free: 0 },
    Card { state: bu::State::Absent, saves: [EMPTY; ROWS], count: 0, free: 0 },
];
static mut PADS: [[u8; 0x22]; 2] = [[0; 0x22]; 2];
static mut PATH: [u8; 32] = [0; 32];
static mut DIRENTRY: [u32; 10] = [0; 10];
static mut FRAME: [u8; 128] = [0; 128];
static mut STATUS: [u8; 48] = [0; 48];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ask {
    None,
    Delete,
    Format,
}

fn cards() -> &'static mut [Card; 2] {
    unsafe { &mut *(&raw mut CARDS) }
}

fn say(s: &[u8]) {
    let st = unsafe { &mut *(&raw mut STATUS) };
    st.fill(0);
    let n = s.len().min(47);
    st[..n].copy_from_slice(&s[..n]);
}

/// "bu00:" or "bu10:" and a name, as a C string.
fn path(card: usize, name: &[u8]) -> u32 {
    let p = unsafe { &mut *(&raw mut PATH) };
    p.fill(0);
    p[..5].copy_from_slice(if card == 0 { b"bu00:" } else { b"bu10:" });
    for (i, &c) in name.iter().take_while(|&&c| c != 0).take(21).enumerate() {
        p[5 + i] = c;
    }
    p.as_ptr() as u32
}

/// A title frame's Shift-JIS title as ASCII: the full-width letters, digits and punctuation of JIS X 0208's first rows
/// as their ASCII forms, single bytes 20h-7Eh as they are, anything else "?".
fn ascii(title: u32, out: &mut [u8; 33]) {
    out.fill(0);
    let (mut i, mut n) = (0, 0);
    while i < 64 && n < 32 {
        let a = rd8(title + i);
        if a == 0 {
            break;
        }
        let c = if (0x81..=0x9F).contains(&a) || (0xE0..=0xEF).contains(&a) {
            let b = rd8(title + i + 1);
            i += 2;
            match (a, b) {
                (0x82, 0x4F..=0x58) => b - 0x4F + b'0' as u32,
                (0x82, 0x60..=0x79) => b - 0x60 + b'A' as u32,
                (0x82, 0x81..=0x9A) => b - 0x81 + b'a' as u32,
                (0x81, _) => (match b {
                    0x40 => b' ',
                    0x43 => b',',
                    0x44 => b'.',
                    0x46 => b':',
                    0x47 => b';',
                    0x48 => b'?',
                    0x49 => b'!',
                    0x5B..=0x5D | 0x7C => b'-',
                    0x5E => b'/',
                    0x60 => b'~',
                    0x62 => b'|',
                    0x65 | 0x66 => b'\'',
                    0x67 | 0x68 => b'"',
                    0x69 => b'(',
                    0x6A => b')',
                    0x6D => b'[',
                    0x6E => b']',
                    0x7B => b'+',
                    0x81 => b'=',
                    0x83 => b'<',
                    0x84 => b'>',
                    0x90 => b'$',
                    0x93 => b'%',
                    0x94 => b'#',
                    0x95 => b'&',
                    0x96 => b'*',
                    0x97 => b'@',
                    _ => b'?',
                }) as u32,
                _ => b'?' as u32,
            }
        } else {
            i += 1;
            if (0x20..0x7F).contains(&a) { a } else { b'?' as u32 }
        };
        out[n] = c as u8;
        n += 1;
    }
    while n > 0 && out[n - 1] == b' ' {
        n -= 1;
        out[n] = 0;
    }
}

/// A card's saves read again: the names by firstfile2 and nextfile, then each one's title frame.
fn refresh(card: usize) {
    let c = &mut cards()[card];
    c.count = 0;
    c.state = bu::state(PORTS[card]);
    if c.state != bu::State::Ready {
        return;
    }
    c.free = bu::free_blocks(PORTS[card]);
    let d = &raw mut DIRENTRY as u32;
    let mut r = file::firstfile2(path(card, b"*"), d);
    while r != 0 && c.count < ROWS {
        let s = &mut c.saves[c.count];
        for (i, b) in s.name.iter_mut().enumerate() {
            *b = if i < 20 { rd8(d + i as u32) as u8 } else { 0 };
        }
        s.blocks = crate::mem::rd(d + 0x18) / 0x2000;
        c.count += 1;
        r = file::nextfile(d);
    }
    let frame = &raw mut FRAME as u32;
    for k in 0..c.count {
        let s = &mut c.saves[k];
        s.title = [0; 33];
        let fd = file::open(path(card, &s.name), 1);
        if fd != u32::MAX {
            if file::read(fd, frame, 128) == 128 && rd8(frame) == b'S' as u32 && rd8(frame + 1) == b'C' as u32 {
                ascii(frame + 4, &mut s.title);
            }
            file::close(fd);
        }
        if s.title[0] == 0 {
            s.title[..20].copy_from_slice(&s.name[..20]);
        }
    }
}

fn number(mut v: u32, out: &mut [u8; 4]) -> &[u8] {
    let mut i = 4;
    loop {
        i -= 1;
        out[i] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 || i == 0 {
            break;
        }
    }
    &out[i..]
}

fn draw(active: usize, chosen: [usize; 2]) {
    fill(0, 0, 320, 240, BACKGROUND);
    text(8, 6, b"EmuSen  Diamond open kernel", 40, TITLE);
    text(8, 18, b"No disc. Put a PlayStation disc in to start it.", 53, DIM);
    for (k, c) in cards().iter().enumerate() {
        let x = 4 + 160 * k as u32;
        let mut head = *b"MEMORY CARD 1";
        head[12] += k as u8;
        text(x + 4, 36, &head, 13, if k == active { TITLE } else { NORMAL });
        match c.state {
            bu::State::Absent => {
                text(x + 4, 52, b"No card", 20, DIM);
            }
            bu::State::Blank => {
                text(x + 4, 52, b"Not formatted", 20, WARN);
            }
            bu::State::Ready => {
                let mut n = [0u8; 4];
                let end = text(x + 4, 196, number(c.free, &mut n), 4, DIM);
                text(end + 6, 196, b"blocks free", 12, DIM);
                if c.count == 0 {
                    text(x + 4, 52, b"No saves", 20, DIM);
                }
                for (i, s) in c.saves[..c.count].iter().enumerate() {
                    let y = 50 + 10 * i as u32;
                    if k == active && i == chosen[k] {
                        fill(x, y - 1, 156, 10, HIGHLIGHT);
                    }
                    text(x + 4, y, &s.title, 21, NORMAL);
                    let mut n = [0u8; 4];
                    text(x + 140, y, number(s.blocks, &mut n), 2, DIM);
                }
            }
        }
    }
    text(8, 210, unsafe { &*(&raw const STATUS) }, 47, WARN);
    text(8, 222, b"Up/Down save  Left/Right card  Cross copy", 53, DIM);
    text(8, 231, b"Triangle delete  Square format", 53, DIM);
}

/// The chosen save copied to the other card: read whole into user RAM, a file of its blocks made there, written.
fn copy(from: usize, s: &Save) {
    let to = 1 - from;
    if cards()[to].state != bu::State::Ready {
        return say(b"The other card is not ready.");
    }
    if cards()[to].saves[..cards()[to].count].iter().any(|t| t.name == s.name) {
        return say(b"The other card already holds that save.");
    }
    if cards()[to].free < s.blocks {
        return say(b"Not enough free blocks on the other card.");
    }
    say(b"Copying...");
    let size = s.blocks * 0x2000;
    let fd = file::open(path(from, &s.name), 1);
    let ok = fd != u32::MAX && file::read(fd, BUFFER, size) == size;
    if fd != u32::MAX {
        file::close(fd);
    }
    if !ok {
        return say(b"The save could not be read.");
    }
    let fd = file::open(path(to, &s.name), 0x200 | 2 | s.blocks << 16);
    let ok = fd != u32::MAX && file::write(fd, BUFFER, size) == size;
    if fd != u32::MAX {
        file::close(fd);
    }
    refresh(to);
    say(if ok { b"Copied." } else { b"The copy could not be written." });
}

fn buttons() -> u32 {
    let b = unsafe { &raw const PADS[0] } as u32;
    if rd8(b) != 0 { 0 } else { !(rd8(b + 2) | rd8(b + 3) << 8) & 0xFFFF }
}

/// The manager's loop, a frame a step, until `disc` finds a disc to boot, checked about every two seconds.
pub fn run(disc: impl Fn() -> bool) {
    let (p1, p2) = unsafe { (&raw mut PADS[0] as u32, &raw mut PADS[1] as u32) };
    pad::init_pad2(p1, 0x22, p2, 0x22);
    pad::start_pad2();
    card::init_card2(1);
    card::start_card2();
    bu::bu_init();
    for i in 0..0x22 {
        wr8(p1 + i, 0xFF);
    }
    say(b"");
    refresh(0);
    refresh(1);
    let (mut active, mut chosen, mut ask) = (0usize, [0usize; 2], Ask::None);
    let mut held = 0;
    let mut ticks = 0u32;
    draw(active, chosen);
    loop {
        pause(1);
        ticks += 1;
        let now = buttons();
        let pressed = now & !held;
        held = now;
        if pressed != 0 {
            let count = cards()[active].count;
            let was = ask;
            ask = Ask::None;
            match pressed {
                p if p & SELECT_UP != 0 => chosen[active] = chosen[active].saturating_sub(1),
                p if p & DOWN != 0 => chosen[active] = (chosen[active] + 1).min(count.max(1) - 1),
                p if p & (LEFT | RIGHT) != 0 => active = 1 - active,
                p if p & CROSS != 0 && count > 0 => {
                    let s = cards()[active].saves[chosen[active]];
                    copy(active, &s);
                }
                p if p & TRIANGLE != 0 && count > 0 => {
                    if was == Ask::Delete {
                        let ok = file::erase(path(active, &cards()[active].saves[chosen[active]].name)) == 1;
                        refresh(active);
                        chosen[active] = chosen[active].min(cards()[active].count.max(1) - 1);
                        say(if ok { b"Deleted." } else { b"The save could not be deleted." });
                    } else {
                        ask = Ask::Delete;
                        say(b"Press Triangle again to delete the save.");
                    }
                }
                p if p & SQUARE != 0 && cards()[active].state != bu::State::Absent => {
                    if was == Ask::Format {
                        let ok = file::format(path(active, b"")) == 1;
                        refresh(active);
                        say(if ok { b"Formatted." } else { b"The card could not be formatted." });
                    } else {
                        ask = Ask::Format;
                        say(b"Press Square again to erase the whole card.");
                    }
                }
                _ => {}
            }
            draw(active, chosen);
        }
        if ticks % 120 == 0 {
            if disc() {
                break;
            }
            let before = [cards()[0].state, cards()[1].state];
            for k in 0..2 {
                if bu::state(PORTS[k]) != before[k] {
                    refresh(k);
                    chosen[k] = 0;
                    draw(active, chosen);
                }
            }
        }
    }
    pad::stop_pad2();
    card::stop_card2();
    crate::mem::wr(crate::layout::io::I_MASK, crate::mem::rd(crate::layout::io::I_MASK) & !1);
}
