//! The two cards through the kernel's file calls on "bu00:" and "bu10:", so the on-card format keeps its one owner in
//! the console, the kernel (Diamond_Shell.md §1.4): a card's state by _card_load and its events (psx-spx "Memory Card -
//! Higher Level File/Device Events", class F4000001h), the saves by firstfile2 and nextfile with each title frame read
//! from the file, a copy by read, create and write, a delete by erase, a format by format.

use crate::{bios, clock};
use diamond_guest::mem::{rd, rd8};
use diamond_guest::sjis;

pub const PORTS: [u32; 2] = [0x00, 0x10];
pub const ROWS: usize = 15;
pub const BLOCK: u32 = 0x2000;
/// User RAM the copy reads a save into, above the shell.
const BUFFER: u32 = 0x8010_0000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Absent,
    Blank,
    Ready,
}

#[derive(Clone, Copy)]
pub struct Save {
    pub name: [u8; 21],
    pub title: [u8; 33],
    pub blocks: u32,
    /// The title frame's first four frames: the frame itself and up to three icon frames.
    pub frames: [u8; 512],
}

pub const EMPTY: Save = Save { name: [0; 21], title: [0; 33], blocks: 0, frames: [0; 512] };

pub struct Card {
    pub state: State,
    pub saves: [Save; ROWS],
    pub count: usize,
    pub free: u32,
}

pub static mut CARDS: [Card; 2] = [
    Card { state: State::Absent, saves: [EMPTY; ROWS], count: 0, free: 0 },
    Card { state: State::Absent, saves: [EMPTY; ROWS], count: 0, free: 0 },
];

pub fn cards() -> &'static mut [Card; 2] {
    unsafe { &mut *(&raw mut CARDS) }
}

static mut EVENTS: [u32; 4] = [0; 4];
const SPECS: [u32; 4] = [0x0004, 0x0100, 0x2000, 0x8000];
static mut PATH: [u8; 32] = [0; 32];
static mut DIRENTRY: [u32; 10] = [0; 10];

pub fn start() {
    unsafe {
        bios::init_card2(1);
        bios::start_card2();
        bios::bu_init();
        for (e, &spec) in (*(&raw mut EVENTS)).iter_mut().zip(&SPECS) {
            *e = bios::open_event(0xF400_0001, spec, bios::EV_READY, 0);
            bios::enable_event(*e);
        }
    }
}

pub fn stop() {
    unsafe {
        for &e in (*(&raw const EVENTS)).iter() {
            bios::close_event(e);
        }
        bios::stop_card2();
    }
}

/// "bu00:" or "bu10:" and a name, as a C string.
pub fn path(card: usize, name: &[u8]) -> u32 {
    let p = unsafe { &mut *(&raw mut PATH) };
    p.fill(0);
    p[..5].copy_from_slice(if card == 0 { b"bu00:" } else { b"bu10:" });
    for (i, &c) in name.iter().take_while(|&&c| c != 0).take(21).enumerate() {
        p[5 + i] = c;
    }
    p.as_ptr() as u32
}

/// What the slot holds, by _card_load's events: 4 ready, 100h no card answering, 2000h unformatted, 8000h an error.
pub fn state(card: usize) -> State {
    let events = unsafe { *(&raw const EVENTS) };
    for &e in &events {
        unsafe { bios::test_event(e) };
    }
    unsafe { bios::card_load(PORTS[card]) };
    for _ in 0..600 {
        clock::frame();
        for (k, &e) in events.iter().enumerate() {
            if unsafe { bios::test_event(e) } == 1 {
                return match k {
                    0 => State::Ready,
                    2 => State::Blank,
                    _ => State::Absent,
                };
            }
        }
    }
    State::Absent
}

/// Whether the slot's card is the one read last: _card_info's events, 4 the same card, 100h none, 2000h a card newly
/// put in (psx-spx "FLAG Byte"); a change to read it again.
pub fn changed(card: usize) -> bool {
    let events = unsafe { *(&raw const EVENTS) };
    for &e in &events {
        unsafe { bios::test_event(e) };
    }
    unsafe { bios::card_info(PORTS[card]) };
    let was = cards()[card].state;
    for _ in 0..60 {
        clock::frame();
        for (k, &e) in events.iter().enumerate() {
            if unsafe { bios::test_event(e) } == 1 {
                return match k {
                    0 => was == State::Absent,
                    1 => was != State::Absent,
                    _ => true,
                };
            }
        }
    }
    was != State::Absent
}

/// The font's box, drawn for a character it does not carry (Diamond_Shell.md §3.1, §9 Q12).
const BOX: u8 = 0x7F;

/// A title frame's Shift-JIS title as ASCII: the full-width letters, digits and punctuation as their ASCII forms,
/// single bytes 20h-7Eh as they are, anything else the box (psx-spx "Shift-JIS Character Set").
fn ascii(frame: &[u8], out: &mut [u8; 33]) {
    out.fill(0);
    let (mut i, mut n) = (4, 0);
    while i < 0x44 && n < 32 && frame[i] != 0 {
        let a = frame[i];
        let c = if sjis::lead(a) && i + 1 < 0x44 {
            let b = frame[i + 1];
            i += 2;
            sjis::ascii((a as u32) << 8 | b as u32).unwrap_or(BOX)
        } else {
            i += 1;
            if (0x20..0x7F).contains(&a) { a } else { BOX }
        };
        out[n] = c;
        n += 1;
    }
    while n > 0 && out[n - 1] == b' ' {
        n -= 1;
        out[n] = 0;
    }
}

/// A card's saves read again: its state, the names by firstfile2 and nextfile, then each one's first frames.
pub fn refresh(card: usize) {
    let c = &mut cards()[card];
    c.count = 0;
    c.state = state(card);
    if c.state != State::Ready {
        return;
    }
    let d = &raw mut DIRENTRY as u32;
    let mut used = 0;
    let mut r = unsafe { bios::firstfile2(path(card, b"*"), d) };
    while r != 0 && c.count < ROWS {
        let s = &mut c.saves[c.count];
        for (i, b) in s.name.iter_mut().enumerate() {
            *b = if i < 20 { rd8(d + i as u32) as u8 } else { 0 };
        }
        s.blocks = rd(d + 0x18) / BLOCK;
        used += s.blocks;
        c.count += 1;
        r = unsafe { bios::nextfile(d) };
    }
    c.free = 15u32.saturating_sub(used);
    for k in 0..c.count {
        let s = &mut c.saves[k];
        s.title = [0; 33];
        s.frames = [0; 512];
        let fd = unsafe { bios::open(path(card, &s.name), 1) };
        if fd != u32::MAX {
            let frames = s.frames.as_mut_ptr() as u32;
            if unsafe { bios::read(fd, frames, 512) } == 512 && s.frames[0] == b'S' && s.frames[1] == b'C' {
                let frame = s.frames;
                ascii(&frame, &mut s.title);
            }
            unsafe { bios::close(fd) };
        }
        if s.title[0] == 0 {
            s.title[..20].copy_from_slice(&s.name[..20]);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    NotReady,
    SameName,
    TooFull,
    Unreadable,
    Unwritable,
}

/// The save copied to the other card a block at a time through user RAM into a file of its blocks created there,
/// `progress` told each block done.
pub fn copy(from: usize, s: &Save, mut progress: impl FnMut(u32, u32)) -> Result<(), Refusal> {
    let to = 1 - from;
    let target = &cards()[to];
    if target.state != State::Ready {
        return Err(Refusal::NotReady);
    }
    if target.saves[..target.count].iter().any(|t| t.name == s.name) {
        return Err(Refusal::SameName);
    }
    if target.free < s.blocks {
        return Err(Refusal::TooFull);
    }
    let source = unsafe { bios::open(path(from, &s.name), 1) };
    if source == u32::MAX {
        return Err(Refusal::Unreadable);
    }
    let target = unsafe { bios::open(path(to, &s.name), 0x200 | 2 | s.blocks << 16) };
    if target == u32::MAX {
        unsafe { bios::close(source) };
        return Err(Refusal::Unwritable);
    }
    let mut r = Ok(());
    progress(0, s.blocks);
    for b in 0..s.blocks {
        if unsafe { bios::read(source, BUFFER, BLOCK) } != BLOCK {
            r = Err(Refusal::Unreadable);
            break;
        }
        if unsafe { bios::write(target, BUFFER, BLOCK) } != BLOCK {
            r = Err(Refusal::Unwritable);
            break;
        }
        progress(b + 1, s.blocks);
    }
    unsafe {
        bios::close(source);
        bios::close(target);
    }
    r
}

pub fn erase(card: usize, name: &[u8]) -> bool {
    unsafe { bios::erase(path(card, name)) == 1 }
}

pub fn format(card: usize) -> bool {
    unsafe { bios::format(path(card, b"")) == 1 }
}
