//! The card manager of stage 10, moved out of the kernel unchanged in what it shows and does: both cards' saves listed
//! by their titles, one copied to the other card, deleted, or a card formatted (Diamond_OpenKernel_Phase2.md §16).

use crate::cards::{self, Refusal, State};
use crate::{clock, drive, pads};
use diamond_guest::gpu::{self, fill, rgb, text};

const BACKGROUND: u32 = rgb(16, 24, 40);
const HIGHLIGHT: u32 = rgb(40, 64, 104);
const TITLE: u32 = rgb(0x50, 0x70, 0x80);
const NORMAL: u32 = rgb(0x70, 0x70, 0x70);
const DIM: u32 = rgb(0x48, 0x48, 0x48);
const WARN: u32 = rgb(0x80, 0x60, 0x30);

static mut STATUS: [u8; 48] = [0; 48];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ask {
    None,
    Delete,
    Format,
}

fn say(s: &[u8]) {
    let st = unsafe { &mut *(&raw mut STATUS) };
    st.fill(0);
    let n = s.len().min(47);
    st[..n].copy_from_slice(&s[..n]);
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
    for (k, c) in cards::cards().iter().enumerate() {
        let x = 4 + 160 * k as u32;
        let mut head = *b"MEMORY CARD 1";
        head[12] += k as u8;
        text(x + 4, 36, &head, 13, if k == active { TITLE } else { NORMAL });
        match c.state {
            State::Absent => {
                text(x + 4, 52, b"No card", 20, DIM);
            }
            State::Blank => {
                text(x + 4, 52, b"Not formatted", 20, WARN);
            }
            State::Ready => {
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

fn copy(from: usize, chosen: usize) {
    let s = cards::cards()[from].saves[chosen];
    say(b"Copying...");
    let r = cards::copy(from, &s);
    cards::refresh(1 - from);
    say(match r {
        Ok(()) => b"Copied." as &[u8],
        Err(Refusal::NotReady) => b"The other card is not ready.",
        Err(Refusal::SameName) => b"The other card already holds that save.",
        Err(Refusal::TooFull) => b"Not enough free blocks on the other card.",
        Err(Refusal::Unreadable) => b"The save could not be read.",
        Err(Refusal::Unwritable) => b"The copy could not be written.",
    });
}

/// The manager's loop, a frame a step, until the drive holds a disc to boot, asked about every two seconds.
pub fn run() {
    gpu::init(BACKGROUND);
    pads::start();
    clock::start();
    cards::start();
    say(b"");
    cards::refresh(0);
    cards::refresh(1);
    let (mut active, mut chosen, mut ask) = (0usize, [0usize; 2], Ask::None);
    draw(active, chosen);
    loop {
        let ticks = clock::frame();
        let pressed = pads::pressed();
        if pressed != 0 {
            let count = cards::cards()[active].count;
            let was = ask;
            ask = Ask::None;
            match pressed {
                p if p & pads::UP != 0 => chosen[active] = chosen[active].saturating_sub(1),
                p if p & pads::DOWN != 0 => chosen[active] = (chosen[active] + 1).min(count.max(1) - 1),
                p if p & (pads::LEFT | pads::RIGHT) != 0 => active = 1 - active,
                p if p & pads::CROSS != 0 && count > 0 => copy(active, chosen[active]),
                p if p & pads::TRIANGLE != 0 && count > 0 => {
                    if was == Ask::Delete {
                        let ok = cards::erase(active, &cards::cards()[active].saves[chosen[active]].name);
                        cards::refresh(active);
                        chosen[active] = chosen[active].min(cards::cards()[active].count.max(1) - 1);
                        say(if ok { b"Deleted." } else { b"The save could not be deleted." });
                    } else {
                        ask = Ask::Delete;
                        say(b"Press Triangle again to delete the save.");
                    }
                }
                p if p & pads::SQUARE != 0 && cards::cards()[active].state != State::Absent => {
                    if was == Ask::Format {
                        let ok = cards::format(active);
                        cards::refresh(active);
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
            if drive::identify() == drive::Held::Data {
                break;
            }
            for k in 0..2 {
                if cards::changed(k) {
                    cards::refresh(k);
                    chosen[k] = 0;
                    draw(active, chosen);
                }
            }
        }
    }
    cards::stop();
    clock::stop();
    pads::stop();
    diamond_guest::mem::wr(0xBF80_1074, diamond_guest::mem::rd(0xBF80_1074) & !1);
}
