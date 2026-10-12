//! The memory cards' sectors (psx-spx "BIOS Memory Card Functions" and "Memory Card Read/Write Commands"): one request
//! a slot, carried out on vblank by the element the pads share, slot 1 on even frames and slot 2 on odd ones as psx-spx
//! describes the original's pace, or at once when nothing drives the element; the low-level calls and their events.
//! Diamond_OpenKernel_Phase2.md §13.

use crate::layout::{A_TABLE, io};
use crate::mem::{call, rd, rd8, wr8};
use crate::{bu, entry, event, pad, sio};

pub const READY: u32 = 0x01;
const BUSY_READ: u32 = 0x02;
const BUSY_WRITE: u32 = 0x04;
const BUSY_INFO: u32 = 0x08;
const FAILED: u32 = 0x11;
const ERROR: u32 = 0x21;

/// The lower-level hardware events ("Memory Card - Lower Level Hardware I/O Events").
const HW: u32 = 0xF000_0011;
pub const SPEC_OK: u32 = 0x0004;
pub const SPEC_TIMEOUT: u32 = 0x0100;
pub const SPEC_NEW: u32 = 0x2000;
pub const SPEC_ERROR: u32 = 0x8000;

/// A sector's last frame of block 0, written by the write test ("Write Test Frame").
const TEST_SECTOR: u32 = 0x3F;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Ok,
    Timeout,
    New,
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    None,
    Read,
    Write,
    Info,
}

/// Who asked: a low-level call (events of class F0000011h), the file layer's asynchronous job, or a caller waiting.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    Call,
    Job,
    Wait,
}

#[derive(Clone, Copy)]
struct Request {
    kind: Kind,
    owner: Owner,
    port: u32,
    sector: u32,
    buffer: u32,
    ignore_new: bool,
}

const NONE: Request =
    Request { kind: Kind::None, owner: Owner::Call, port: 0, sector: 0, buffer: 0, ignore_new: false };

static mut REQUESTS: [Request; 2] = [NONE; 2];
static mut STATUS: [u32; 2] = [READY; 2];
static mut WAITED: [Option<Outcome>; 2] = [None; 2];
static mut STARTED: bool = false;
static mut IGNORE_NEW: bool = false;
static mut CHANNEL: u32 = 0;
static mut VBLANKS: u32 = 0;
static mut TEST_FRAME: [u8; 128] = [0; 128];

pub fn slot_of(port: u32) -> usize {
    (port >> 4 & 1) as usize
}

/// The vblanks the shared element has seen.
pub fn vblanks() -> u32 {
    unsafe { core::ptr::read_volatile(&raw const VBLANKS) }
}

pub fn started() -> bool {
    unsafe { STARTED }
}

/// SetConf's reset loses the element; the cards stop with it.
pub fn forget() {
    unsafe {
        STARTED = false;
        REQUESTS = [NONE; 2];
        STATUS = [READY; 2];
    }
}

/// The 128 bytes of a frame read or written as psx-spx's command tables give them: the reply, or how it failed.
fn transfer(r: Request) -> Outcome {
    let control = sio::select(r.port >> 4);
    let outcome = exchange_all(control, r);
    sio::deselect();
    outcome
}

fn exchange_all(control: u32, r: Request) -> Outcome {
    let x = |byte: u32, wait: u32| sio::exchange(control, byte, wait);
    if !x(0x81 + (r.port & 0xF), sio::TIMEOUT).1 {
        return Outcome::Timeout;
    }
    let command = if r.kind == Kind::Write { 0x57 } else { 0x52 };
    let (flag, acked) = x(command, sio::TIMEOUT);
    if !acked {
        return Outcome::Timeout;
    }
    if r.kind == Kind::Info {
        return if flag & 4 != 0 {
            Outcome::Error
        } else if flag & 8 != 0 {
            Outcome::New
        } else {
            Outcome::Ok
        };
    }
    if flag & 8 != 0 && !r.ignore_new {
        return Outcome::New;
    }
    if x(0, sio::TIMEOUT).0 != 0x5A || x(0, sio::TIMEOUT).0 != 0x5D {
        return Outcome::Error;
    }
    let (msb, lsb) = (r.sector >> 8 & 0xFF, r.sector & 0xFF);
    x(msb, sio::TIMEOUT);
    x(lsb, sio::TIMEOUT);
    let mut sum = msb ^ lsb;
    if r.kind == Kind::Read {
        x(0, sio::LATE);
        x(0, sio::TIMEOUT);
        let confirmed = x(0, sio::TIMEOUT).0 << 8 | x(0, sio::TIMEOUT).0;
        if confirmed != r.sector {
            return Outcome::Error;
        }
        for i in 0..128 {
            let v = x(0, sio::TIMEOUT).0;
            wr8(r.buffer + i, v);
            sum ^= v;
        }
        let check = x(0, sio::TIMEOUT).0;
        let end = x(0, 0).0;
        if check != sum || end != 0x47 { Outcome::Error } else { Outcome::Ok }
    } else {
        for i in 0..128 {
            let v = rd8(r.buffer + i);
            sum ^= v;
            x(v, sio::TIMEOUT);
        }
        x(sum, sio::TIMEOUT);
        x(0, sio::TIMEOUT);
        x(0, sio::TIMEOUT);
        if x(0, 0).0 == 0x47 { Outcome::Ok } else { Outcome::Error }
    }
}

fn busy(kind: Kind) -> u32 {
    match kind {
        Kind::Read => BUSY_READ,
        Kind::Write => BUSY_WRITE,
        _ => BUSY_INFO,
    }
}

/// The slot's request, if any, carried out now; called with interrupts off.
fn perform(slot: usize) {
    let r = unsafe { REQUESTS[slot] };
    if r.kind == Kind::None {
        return;
    }
    unsafe { REQUESTS[slot] = NONE };
    let outcome = transfer(r);
    unsafe {
        STATUS[slot] = match outcome {
            Outcome::Ok => READY,
            Outcome::Timeout => FAILED,
            _ => ERROR,
        };
        CHANNEL = r.port;
    }
    match r.owner {
        Owner::Call => {
            let spec = match outcome {
                Outcome::Ok => SPEC_OK,
                Outcome::Timeout => SPEC_TIMEOUT,
                Outcome::New => SPEC_NEW,
                Outcome::Error => SPEC_ERROR,
            };
            event::deliver_event(HW, spec);
            notify(outcome);
        }
        Owner::Wait => unsafe { WAITED[slot] = Some(outcome) },
        Owner::Job => bu::sector_done(slot, outcome),
    }
}

/// From the shared element on each vblank: the slot whose turn it is.
pub fn on_vblank() {
    unsafe {
        VBLANKS = VBLANKS.wrapping_add(1);
        if STARTED {
            perform((VBLANKS & 1) as usize);
        }
    }
}

/// Whether vblank will carry the request: the element queued, vblank enabled and interrupts on.
fn driven() -> bool {
    (unsafe { STARTED }) && pad::queued() && rd(io::I_MASK) & 1 != 0 && entry::interrupts_on()
}

/// Polls of a wait before it decides vblank is not coming, about three frames.
const STALL: u32 = 200_000;

/// Waits until the slot has no request, carrying it out itself when nothing else will.
fn drain(slot: usize) {
    let start = unsafe { VBLANKS };
    let mut polls = 0u32;
    while unsafe { REQUESTS[slot].kind } != Kind::None {
        polls += 1;
        if !driven() || (polls > STALL && unsafe { VBLANKS } == start) {
            entry::critical(|| perform(slot));
        }
    }
}

/// The request stored whole with interrupts off, once the slot is free, so that the vblank element never takes a
/// request half stored (Diamond_OpenKernel_Phase2.md §23).
fn submit(slot: usize, r: Request) {
    loop {
        drain(slot);
        let stored = entry::critical(|| unsafe {
            if REQUESTS[slot].kind != Kind::None {
                return false;
            }
            STATUS[slot] = busy(r.kind);
            REQUESTS[slot] = r;
            true
        });
        if stored {
            return;
        }
    }
}

/// A request queued for the file layer's job, carried out on the slot's vblank.
pub fn queue(kind: Kind, port: u32, sector: u32, buffer: u32, ignore_new: bool) {
    submit(slot_of(port), Request { kind, owner: Owner::Job, port, sector, buffer, ignore_new });
}

/// A sector read or written, or the card's flag taken, waiting for the outcome.
pub fn run(kind: Kind, port: u32, sector: u32, buffer: u32, ignore_new: bool) -> Outcome {
    let slot = slot_of(port);
    unsafe { WAITED[slot] = None };
    submit(slot, Request { kind, owner: Owner::Wait, port, sector, buffer, ignore_new });
    drain(slot);
    unsafe { WAITED[slot].unwrap_or(Outcome::Error) }
}

fn undeliver() {
    for spec in [SPEC_OK, SPEC_TIMEOUT, SPEC_NEW, SPEC_ERROR] {
        event::undeliver_event(HW, spec);
    }
}

fn call_request(kind: Kind, port: u32, sector: u32, buffer: u32) -> u32 {
    let slot = slot_of(port);
    if unsafe { REQUESTS[slot].kind } != Kind::None {
        return 0;
    }
    undeliver();
    undeliver_sw();
    let ignore_new = unsafe { IGNORE_NEW };
    unsafe { IGNORE_NEW = false };
    submit(slot, Request { kind, owner: Owner::Call, port, sector, buffer, ignore_new });
    1
}

/// B(4Ah): the pads' flag, SIO0 reset, the slots idle.
pub extern "C" fn init_card2(pad_enable: u32) -> u32 {
    pad::enable(pad_enable);
    sio::reset();
    unsafe {
        REQUESTS = [NONE; 2];
        STATUS = [READY; 2];
    }
    0
}

/// B(4Bh).
pub extern "C" fn start_card2() -> u32 {
    unsafe { STARTED = true };
    pad::queue();
    0
}

/// B(4Ch).
pub extern "C" fn stop_card2() -> u32 {
    unsafe { STARTED = false };
    pad::unqueue();
    0
}

/// B(4Fh): 1, or 0 for a sector past 400h, which psx-spx says the original accepts.
pub extern "C" fn card_read(port: u32, sector: u32, dst: u32) -> u32 {
    if sector > 0x400 { 0 } else { call_request(Kind::Read, port, sector, dst) }
}

/// B(4Eh).
pub extern "C" fn card_write(port: u32, sector: u32, src: u32) -> u32 {
    if sector > 0x400 { 0 } else { call_request(Kind::Write, port, sector, src) }
}

/// A(ABh) and B(4Dh): a read stopped at the card's flag.
pub extern "C" fn card_info(port: u32) -> u32 {
    call_request(Kind::Info, port, 0, 0)
}

/// The header frame's usual contents, "MC", zeros and the checksum 0Eh, which the write test repeats at frame 63.
pub fn test_frame() -> u32 {
    let f = &raw mut TEST_FRAME as u32;
    for i in 0..128 {
        wr8(f + i, 0);
    }
    wr8(f, b'M' as u32);
    wr8(f + 1, b'C' as u32);
    wr8(f + 0x7F, b'M' as u32 ^ b'C' as u32);
    f
}

/// A(AFh): the change flag cleared by a write of sector 3Fh.
pub extern "C" fn card_write_test(port: u32) -> u32 {
    unsafe { IGNORE_NEW = true };
    call_request(Kind::Write, port, TEST_SECTOR, test_frame())
}

pub fn write_test(port: u32) -> Outcome {
    run(Kind::Write, port, TEST_SECTOR, test_frame(), true)
}

/// B(50h): the next request ignores the change flag.
pub extern "C" fn new_card() -> u32 {
    unsafe { IGNORE_NEW = true };
    0
}

/// B(5Ch).
pub extern "C" fn card_status(slot: u32) -> u32 {
    unsafe { STATUS[(slot & 1) as usize] }
}

/// B(5Dh): the status once the slot is no longer busy.
pub extern "C" fn card_wait(slot: u32) -> u32 {
    drain((slot & 1) as usize);
    card_status(slot)
}

/// B(58h): the port of the request completed last.
pub extern "C" fn card_chan() -> u32 {
    unsafe { CHANNEL }
}

/// The higher-level events ("Memory Card - Higher Level File/Device Events"), each delivered by its callback.
const SW: u32 = 0xF400_0001;

/// A(A7h): done.
pub extern "C" fn bufs_cb_0() -> u32 {
    event::deliver_event(SW, SPEC_OK)
}

/// A(A8h): a write error.
pub extern "C" fn bufs_cb_1() -> u32 {
    event::deliver_event(SW, SPEC_ERROR)
}

/// A(A9h): busy.
pub extern "C" fn bufs_cb_2() -> u32 {
    event::deliver_event(SW, SPEC_TIMEOUT)
}

/// A(AAh): ejected, or unformatted.
pub extern "C" fn bufs_cb_3() -> u32 {
    event::deliver_event(SW, SPEC_NEW)
}

/// A(AEh): a general error.
pub extern "C" fn bufs_cb_4() -> u32 {
    event::deliver_event(SW, SPEC_ERROR)
}

/// The callback for an outcome, called through the A table so that a hook there sees it.
pub fn notify(outcome: Outcome) {
    let n = match outcome {
        Outcome::Ok => 0xA7,
        Outcome::Timeout => 0xA9,
        Outcome::New => 0xAA,
        Outcome::Error => 0xA8,
    };
    call(rd(A_TABLE + 4 * n), 0, 0, 0, 0);
}

/// The higher-level events left from an earlier job made busy again before a new one.
pub fn undeliver_sw() {
    for spec in [SPEC_OK, SPEC_TIMEOUT, SPEC_NEW, SPEC_ERROR] {
        event::undeliver_event(SW, spec);
    }
}
