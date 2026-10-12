//! The memory cards' files on "bu00:" and "bu10:" in the on-card format of psx-spx's "Memory Card Data Format": the
//! header frame, fifteen directory frames with their block chains and checksums, the broken sector list, and the
//! title frames left to the game; block 0's first 36 frames kept per slot. The device functions A(65h)-A(6Fh), the
//! asynchronous jobs of "Asynchronous Access" and the formatting of a blank card. Diamond_OpenKernel_Phase2.md §14.

use crate::card::{self, Kind, Outcome};
use crate::file::{self, dcb, error, fcb};
use crate::mem::{rd, rd8, wr, wr8};
use crate::{event, names};

const FRAME: u32 = 0x80;
const BLOCK: u32 = 0x2000;
const KEPT: usize = 36;
const FIRST: u32 = 0x51;
const MIDDLE: u32 = 0x52;
const LAST: u32 = 0x53;
const FREE: u32 = 0xA0;
const DELETED: u32 = 0xA1;
const NAME: u32 = 0x0A;
const NAME_MAX: u32 = 20;
const NO_BLOCK: u32 = 0xFFFF;
const ASYNC: u32 = 0x8000;
const CREATE: u32 = 0x200;
/// FCB fields psx-spx gives: the file's size and its first block, "logical block number".
const SIZE: u32 = 0x20;
const START: u32 = 0x24;

static mut FRAMES: [[u8; KEPT * 128]; 2] = [[0; KEPT * 128]; 2];
static mut LOADED: [bool; 2] = [false; 2];
static mut FORMATTED: [bool; 2] = [false; 2];
static mut AUTO: u32 = 0;
static mut FIND_MODE: u32 = 0;
static mut SCRATCH: [u8; 128] = [0; 128];
static mut PATTERN: [u8; 24] = [0; 24];
static mut SEARCH: (usize, u32) = (0, 16);
static mut INFO: [u32; 20] = [0; 20];

#[derive(Clone, Copy, PartialEq, Eq)]
enum JobKind {
    None,
    Read,
    Write,
    Load,
}

#[derive(Clone, Copy)]
struct Job {
    kind: JobKind,
    port: u32,
    fcb: u32,
    buffer: u32,
    offset: u32,
    remaining: u32,
    stage: u32,
}

const IDLE: Job = Job { kind: JobKind::None, port: 0, fcb: 0, buffer: 0, offset: 0, remaining: 0, stage: 0 };
static mut JOBS: [Job; 2] = [IDLE; 2];

fn frame(slot: usize, i: u32) -> u32 {
    unsafe { &raw mut FRAMES[slot] as u32 + i * FRAME }
}

fn le32(a: u32) -> u32 {
    rd8(a) | rd8(a + 1) << 8 | rd8(a + 2) << 16 | rd8(a + 3) << 24
}

fn set32(a: u32, v: u32) {
    for i in 0..4 {
        wr8(a + i, v >> (8 * i));
    }
}

fn checksum(a: u32) {
    let mut x = 0;
    for i in 0..0x7F {
        x ^= rd8(a + i);
    }
    wr8(a + 0x7F, x);
}

fn state_of(slot: usize, b: u32) -> u32 {
    le32(frame(slot, b))
}

fn next(slot: usize, b: u32) -> u32 {
    let f = frame(slot, b);
    rd8(f + 8) | rd8(f + 9) << 8
}

fn is_mc(a: u32) -> bool {
    rd8(a) == b'M' as u32 && rd8(a + 1) == b'C' as u32
}

fn port_of(f: u32) -> u32 {
    rd(f + fcb::PORT)
}

/// Block 0's frames 0-35 read into the slot's copy, the change flag cleared by the write test when the card was new.
fn load(port: u32, new: bool) -> Outcome {
    let slot = card::slot_of(port);
    unsafe { LOADED[slot] = false };
    for i in 0..KEPT as u32 {
        let o = card::run(Kind::Read, port, i, frame(slot, i), true);
        if o != Outcome::Ok {
            return o;
        }
        if i == 0 && !is_mc(frame(slot, 0)) {
            unsafe {
                LOADED[slot] = true;
                FORMATTED[slot] = false;
            }
            return Outcome::Ok;
        }
    }
    if new {
        let o = card::write_test(port);
        if o != Outcome::Ok {
            return o;
        }
    }
    unsafe {
        LOADED[slot] = true;
        FORMATTED[slot] = true;
    }
    Outcome::Ok
}

/// The card checked by a read of sector 0, as the original does on each synchronous open, its directory read anew
/// when the card is new or unread; then formatted when auto format is on, or an error.
fn ready(port: u32) -> Result<(), u32> {
    let slot = card::slot_of(port);
    let scratch = &raw mut SCRATCH as u32;
    let o = card::run(Kind::Read, port, 0, scratch, false);
    let o = match o {
        Outcome::New => load(port, true),
        Outcome::Ok if !unsafe { LOADED[slot] } || is_mc(scratch) != unsafe { FORMATTED[slot] } => load(port, false),
        o => o,
    };
    finish_ready(port, o)
}

/// The directory read if it was not, without the extra read of sector 0.
fn known(port: u32) -> Result<(), u32> {
    let slot = card::slot_of(port);
    if unsafe { LOADED[slot] } {
        return finish_ready(port, Outcome::Ok);
    }
    let o = match card::run(Kind::Info, port, 0, 0, false) {
        Outcome::New => load(port, true),
        Outcome::Ok => load(port, false),
        o => o,
    };
    finish_ready(port, o)
}

fn finish_ready(port: u32, o: Outcome) -> Result<(), u32> {
    let slot = card::slot_of(port);
    if o != Outcome::Ok {
        unsafe { LOADED[slot] = false };
        return Err(error::GENERAL);
    }
    if !unsafe { FORMATTED[slot] } && !(unsafe { AUTO } != 0 && format_card(port)) {
        return Err(error::GENERAL);
    }
    Ok(())
}

fn write_frame(port: u32, i: u32) -> bool {
    let slot = card::slot_of(port);
    checksum(frame(slot, i));
    card::run(Kind::Write, port, i, frame(slot, i), true) == Outcome::Ok
}

/// A blank card's block 0: the header, fifteen free directory frames, an empty broken sector list, the rest FFh, and
/// frame 63 as the header; written whole.
fn format_card(port: u32) -> bool {
    let slot = card::slot_of(port);
    for i in 0..KEPT as u32 {
        let f = frame(slot, i);
        for k in 0..FRAME {
            wr8(f + k, 0);
        }
        match i {
            0 => {
                wr8(f, b'M' as u32);
                wr8(f + 1, b'C' as u32);
            }
            1..=15 => {
                set32(f, FREE);
                wr8(f + 8, 0xFF);
                wr8(f + 9, 0xFF);
            }
            _ => set32(f, 0xFFFF_FFFF),
        }
        if !write_frame(port, i) {
            unsafe { LOADED[slot] = false };
            return false;
        }
    }
    let scratch = &raw mut SCRATCH as u32;
    for k in 0..FRAME {
        wr8(scratch + k, 0xFF);
    }
    for i in KEPT as u32..63 {
        if card::run(Kind::Write, port, i, scratch, true) != Outcome::Ok {
            return false;
        }
    }
    if card::write_test(port) != Outcome::Ok {
        return false;
    }
    unsafe {
        LOADED[slot] = true;
        FORMATTED[slot] = true;
    }
    true
}

/// The first block (1-15) whose state is `wanted` and whose name matches the pattern.
fn find(slot: usize, pattern: u32, wanted: u32, star: bool) -> Option<u32> {
    (1..=15).find(|&b| state_of(slot, b) == wanted && names::matches(pattern, frame(slot, b) + NAME, 0, star, false))
}

/// The block holding the file's `index`th 8 KiB, following the chain.
fn block_at(slot: usize, first: u32, index: u32) -> Option<u32> {
    let mut b = first;
    for _ in 0..index {
        let n = next(slot, b);
        if n == NO_BLOCK || n > 14 {
            return None;
        }
        b = n + 1;
    }
    Some(b)
}

/// The card sector of a file's byte offset, through the broken sector list.
fn sector_of(slot: usize, first: u32, offset: u32) -> Option<u32> {
    let s = block_at(slot, first, offset / BLOCK)? * 64 + offset % BLOCK / FRAME;
    Some((0..20).find(|n| le32(frame(slot, 16 + n)) == s).map_or(s, |n| 36 + n))
}

fn create(port: u32, name: u32, blocks: u32) -> Result<u32, u32> {
    let slot = card::slot_of(port);
    if find(slot, name, FIRST, false).is_some() {
        return Err(error::EXISTS);
    }
    let mut chosen = [0u32; 15];
    let mut n = 0;
    for wanted in [FREE, 0xA1, 0xA2, 0xA3] {
        for b in 1..=15 {
            if n < blocks && state_of(slot, b) == wanted {
                chosen[n as usize] = b;
                n += 1;
            }
        }
    }
    if n < blocks {
        return Err(error::NO_BLOCKS);
    }
    chosen[..n as usize].sort_unstable();
    for (i, &b) in chosen[..n as usize].iter().enumerate() {
        let f = frame(slot, b);
        for k in 0..FRAME {
            wr8(f + k, 0);
        }
        let last = i + 1 == n as usize;
        set32(
            f,
            if i == 0 {
                FIRST
            } else if last {
                LAST
            } else {
                MIDDLE
            },
        );
        let link = if last { NO_BLOCK } else { chosen[i + 1] - 1 };
        wr8(f + 8, link);
        wr8(f + 9, link >> 8);
        if i == 0 {
            set32(f + 4, blocks * BLOCK);
            names::copy(f + NAME, name, NAME_MAX, NAME_MAX + 1);
        }
        if !write_frame(port, b) {
            return Err(error::GENERAL);
        }
    }
    Ok(chosen[0])
}

fn fail(f: u32, e: u32) -> u32 {
    file::device_error(f, e);
    e
}

/// A(65h): an existing file, or a new one of accessmode bits 16-31 blocks with bit 9; 0 or an error number.
pub extern "C" fn dev_card_open(f: u32, path: u32, mode: u32) -> u32 {
    let port = port_of(f);
    let slot = card::slot_of(port);
    let checked = if mode & ASYNC != 0 { known(port) } else { ready(port) };
    if let Err(e) = checked {
        return fail(f, e);
    }
    let name = names::strip(path);
    let b = if mode & CREATE != 0 {
        match create(port, name, (mode >> 16).max(1)) {
            Ok(b) => b,
            Err(e) => return fail(f, e),
        }
    } else {
        match find(slot, name, FIRST, false) {
            Some(b) => b,
            None => return fail(f, error::NOT_FOUND),
        }
    };
    wr(f + SIZE, le32(frame(slot, b) + 4));
    wr(f + START, b);
    0
}

/// The bytes moved, or in asynchronous mode 0 once the job is accepted, the position moved past them, as Spyro the
/// Dragon's loops require (Diamond_OpenKernel_Phase2.md §25.2).
fn transfer(f: u32, buffer: u32, length: u32, kind: Kind) -> u32 {
    let (port, position, size) = (port_of(f), rd(f + fcb::POSITION), rd(f + SIZE));
    let slot = card::slot_of(port);
    if length % FRAME != 0 || position % FRAME != 0 || position >= size {
        return file::device_error(f, error::INVALID);
    }
    if !unsafe { LOADED[slot] } {
        return file::device_error(f, error::GENERAL);
    }
    let n = length.min(size - position);
    let first = rd(f + START);
    if rd(f + fcb::STATUS) & ASYNC != 0 {
        wait(slot);
        card::undeliver_sw();
        let job = Job {
            kind: if kind == Kind::Read { JobKind::Read } else { JobKind::Write },
            port,
            fcb: f,
            buffer,
            offset: position,
            remaining: n / FRAME,
            stage: first,
        };
        unsafe { JOBS[slot] = job };
        step(slot);
        wr(f + fcb::POSITION, position + n);
        return 0;
    }
    for k in 0..n / FRAME {
        let Some(s) = sector_of(slot, first, position + k * FRAME) else {
            return file::device_error(f, error::GENERAL);
        };
        let o = card::run(kind, port, s, buffer + k * FRAME, false);
        if o != Outcome::Ok {
            if o == Outcome::New {
                unsafe { LOADED[slot] = false };
            }
            return file::device_error(f, error::GENERAL);
        }
    }
    n
}

/// A(66h).
pub extern "C" fn dev_card_read(f: u32, dst: u32, length: u32) -> u32 {
    transfer(f, dst, length, Kind::Read)
}

/// A(67h).
pub extern "C" fn dev_card_write(f: u32, src: u32, length: u32) -> u32 {
    transfer(f, src, length, Kind::Write)
}

/// A(68h): a job still running on the file finished first.
pub extern "C" fn dev_card_close(f: u32) -> u32 {
    let slot = card::slot_of(port_of(f));
    if unsafe { JOBS[slot].fcb } == f {
        wait(slot);
    }
    0
}

/// The direntry psx-spx gives: the name, the attribute 50h (A0h deleted), the size and the first sector.
fn entry(slot: usize, b: u32, direntry: u32) -> u32 {
    let f = frame(slot, b);
    names::copy(direntry, f + NAME, NAME_MAX, 0x14);
    wr(direntry + 0x14, if state_of(slot, b) == FIRST { 0x50 } else { 0xA0 });
    wr(direntry + 0x18, le32(f + 4));
    wr(direntry + 0x1C, 0);
    wr(direntry + 0x20, b * 64);
    wr(direntry + 0x24, 0);
    direntry
}

fn search(direntry: u32) -> u32 {
    let (slot, from) = unsafe { SEARCH };
    let wanted = if unsafe { FIND_MODE } == 1 { DELETED } else { FIRST };
    let pattern = &raw const PATTERN as u32;
    for b in from..=15 {
        if state_of(slot, b) == wanted && names::matches(pattern, frame(slot, b) + NAME, 0, true, false) {
            unsafe { SEARCH = (slot, b + 1) };
            return entry(slot, b, direntry);
        }
    }
    unsafe { SEARCH = (slot, 16) };
    0
}

/// A(69h).
pub extern "C" fn dev_card_firstfile(f: u32, path: u32, direntry: u32) -> u32 {
    let port = port_of(f);
    if known(port).is_err() {
        return 0;
    }
    names::copy(&raw mut PATTERN as u32, names::strip(path), 23, 24);
    unsafe { SEARCH = (card::slot_of(port), 1) };
    search(direntry)
}

/// A(6Ah).
pub extern "C" fn dev_card_nextfile(_f: u32, direntry: u32) -> u32 {
    search(direntry)
}

/// Each block of the chain from `first` given the state its place calls for, from `states` (first, middle, last).
fn restate(port: u32, first: u32, states: [u32; 3]) -> bool {
    let slot = card::slot_of(port);
    let mut b = first;
    loop {
        let n = next(slot, b);
        let s = if b == first {
            states[0]
        } else if n == NO_BLOCK {
            states[2]
        } else {
            states[1]
        };
        set32(frame(slot, b), s);
        if !write_frame(port, b) {
            return false;
        }
        if n == NO_BLOCK || n > 14 {
            return true;
        }
        b = n + 1;
    }
}

/// A(6Bh): the file's blocks marked deleted, A1h, A2h and A3h; 1, or 0.
pub extern "C" fn dev_card_erase(f: u32, path: u32) -> u32 {
    let port = port_of(f);
    if let Err(e) = ready(port) {
        fail(f, e);
        return 0;
    }
    let Some(b) = find(card::slot_of(port), names::strip(path), FIRST, false) else {
        fail(f, error::NOT_FOUND);
        return 0;
    };
    restate(port, b, [DELETED, 0xA2, 0xA3]) as u32
}

/// A(6Ch): a deleted file's blocks in use again, unless a file of its name exists.
pub extern "C" fn dev_card_undelete(f: u32, path: u32) -> u32 {
    let port = port_of(f);
    let slot = card::slot_of(port);
    if let Err(e) = ready(port) {
        fail(f, e);
        return 0;
    }
    let name = names::strip(path);
    let Some(b) = find(slot, name, DELETED, false) else {
        fail(f, error::NOT_FOUND);
        return 0;
    };
    if find(slot, frame(slot, b) + NAME, FIRST, false).is_some() {
        fail(f, error::EXISTS);
        return 0;
    }
    restate(port, b, [FIRST, MIDDLE, LAST]) as u32
}

/// A(6Dh).
pub extern "C" fn dev_card_format(f: u32) -> u32 {
    let port = port_of(f);
    if format_card(port) {
        1
    } else {
        fail(f, error::GENERAL);
        0
    }
}

/// A(6Eh): the name in the file's first directory frame replaced, unless the new name exists.
pub extern "C" fn dev_card_rename(f: u32, old: u32, _f2: u32, new: u32) -> u32 {
    let port = port_of(f);
    let slot = card::slot_of(port);
    if let Err(e) = ready(port) {
        fail(f, e);
        return 0;
    }
    let (old, new) = (names::strip(old), names::strip(new));
    let Some(b) = find(slot, old, FIRST, false) else {
        fail(f, error::NOT_FOUND);
        return 0;
    };
    if find(slot, new, FIRST, false).is_some() {
        fail(f, error::EXISTS);
        return 0;
    }
    names::copy(frame(slot, b) + NAME, new, NAME_MAX, NAME_MAX + 1);
    write_frame(port, b) as u32
}

/// A(6Fh): the FCB's error cleared, as psx-spx's entry shows.
pub extern "C" fn card_clear_error(f: u32) -> u32 {
    wr(f + fcb::ERROR, 0);
    0
}

/// The device's testdevice, which psx-spx says returns without doing anything.
extern "C" fn test_device(_f: u32, _path: u32) -> u32 {
    0
}

/// The DCB psx-spx gives for "bu": flags 14h, 80h-byte sectors, "MEMORY CARD".
pub fn device() -> u32 {
    let info = &raw mut INFO as u32;
    for o in (0..0x50).step_by(4) {
        wr(info + o, 0);
    }
    wr(info + dcb::NAME, b"bu\0".as_ptr() as u32);
    wr(info + dcb::FLAGS, 0x14);
    wr(info + dcb::SECTOR, 0x80);
    wr(info + dcb::LONG_NAME, b"MEMORY CARD\0".as_ptr() as u32);
    wr(info + dcb::OPEN, dev_card_open as *const () as u32);
    wr(info + dcb::CLOSE, dev_card_close as *const () as u32);
    wr(info + dcb::READ, dev_card_read as *const () as u32);
    wr(info + dcb::WRITE, dev_card_write as *const () as u32);
    wr(info + dcb::ERASE, dev_card_erase as *const () as u32);
    wr(info + dcb::UNDELETE, dev_card_undelete as *const () as u32);
    wr(info + dcb::FIRSTFILE, dev_card_firstfile as *const () as u32);
    wr(info + dcb::NEXTFILE, dev_card_nextfile as *const () as u32);
    wr(info + dcb::FORMAT, dev_card_format as *const () as u32);
    wr(info + dcb::RENAME, dev_card_rename as *const () as u32);
    wr(info + dcb::TESTDEVICE, test_device as *const () as u32);
    info
}

/// A(55h) and A(70h): both directories to be read again, auto format off.
pub extern "C" fn bu_init() -> u32 {
    unsafe {
        LOADED = [false; 2];
        AUTO = 0;
        JOBS = [IDLE; 2];
    }
    0
}

/// A(ADh).
pub extern "C" fn card_auto(flag: u32) -> u32 {
    unsafe { AUTO = flag };
    0
}

/// C(1Ah).
pub extern "C" fn set_card_find_mode(mode: u32) -> u32 {
    unsafe { FIND_MODE = mode };
    0
}

/// C(1Dh).
pub extern "C" fn get_card_find_mode() -> u32 {
    unsafe { FIND_MODE }
}

/// A(ACh): the directory read asynchronously, its end signalled by the higher-level events.
pub extern "C" fn card_load(port: u32) -> u32 {
    let slot = card::slot_of(port);
    wait(slot);
    card::undeliver_sw();
    unsafe {
        LOADED[slot] = false;
        JOBS[slot] = Job { kind: JobKind::Load, port, stage: 0, ..IDLE };
    }
    step(slot);
    1
}

/// Until the slot's job ends.
fn wait(slot: usize) {
    while unsafe { JOBS[slot].kind } != JobKind::None {
        card::card_wait(slot as u32);
    }
}

/// The job's next request queued.
fn step(slot: usize) {
    let j = unsafe { JOBS[slot] };
    match j.kind {
        JobKind::None => {}
        JobKind::Load if j.stage < KEPT as u32 => card::queue(Kind::Read, j.port, j.stage, frame(slot, j.stage), true),
        JobKind::Load => card::queue(Kind::Write, j.port, 0x3F, card::test_frame(), true),
        _ => match sector_of(slot, j.stage, j.offset) {
            Some(s) => {
                card::queue(if j.kind == JobKind::Read { Kind::Read } else { Kind::Write }, j.port, s, j.buffer, false)
            }
            None => end(slot, Outcome::Error),
        },
    }
}

fn end(slot: usize, outcome: Outcome) {
    let j = unsafe { JOBS[slot] };
    unsafe { JOBS[slot] = IDLE };
    if j.fcb != 0 {
        if outcome == Outcome::Ok {
            event::deliver_event(rd(j.fcb + fcb::NUMBER), card::SPEC_OK);
        } else {
            wr(j.fcb + fcb::ERROR, error::GENERAL);
        }
    }
    if outcome == Outcome::New {
        unsafe { LOADED[slot] = false };
    }
    card::notify(outcome);
}

/// From the card layer when a job's request is done: the next one, or the job's end and its events.
pub fn sector_done(slot: usize, outcome: Outcome) {
    let mut j = unsafe { JOBS[slot] };
    if outcome != Outcome::Ok {
        return end(slot, outcome);
    }
    match j.kind {
        JobKind::None => return,
        JobKind::Load => {
            if j.stage == 0 && !is_mc(frame(slot, 0)) {
                unsafe {
                    LOADED[slot] = true;
                    FORMATTED[slot] = false;
                }
                return end(slot, Outcome::New);
            }
            if j.stage == KEPT as u32 {
                unsafe {
                    LOADED[slot] = true;
                    FORMATTED[slot] = true;
                }
                return end(slot, Outcome::Ok);
            }
            j.stage += 1;
        }
        _ => {
            j.remaining -= 1;
            j.offset += FRAME;
            j.buffer += FRAME;
            if j.remaining == 0 {
                return end(slot, Outcome::Ok);
            }
        }
    }
    unsafe { JOBS[slot] = j };
    step(slot);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Absent,
    Blank,
    Ready,
}

/// What the slot holds, by the documented read of sector 0, for the card manager.
pub fn state(port: u32) -> State {
    let saved = unsafe { AUTO };
    unsafe { AUTO = 0 };
    let r = ready(port);
    unsafe { AUTO = saved };
    match r {
        Ok(()) => State::Ready,
        Err(_) if unsafe { LOADED[card::slot_of(port)] } => State::Blank,
        Err(_) => State::Absent,
    }
}

/// The blocks free on a ready card.
pub fn free_blocks(port: u32) -> u32 {
    let slot = card::slot_of(port);
    (1..=15).filter(|&b| (FREE..=0xA3).contains(&state_of(slot, b))).count() as u32
}
