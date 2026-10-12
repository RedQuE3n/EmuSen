//! The CD-ROM controller driven by the kernel through its host interface (psx-spx "CDROM Controller I/O Ports" and
//! the command chapters): one operation at a time, a short list of commands each waited for by its responses, sectors
//! moved by DMA channel 3 on each INT1; serviced from the priority 0 element while the kernel owns the drive, or by the
//! waiting caller. The BIOS CDROM Functions over it. Diamond_OpenKernel_Phase2.md §12.

use crate::exception::{self, acknowledge, pending};
use crate::layout::io;
use crate::mem::{rd, rd8, wr, wr8};
use crate::{entry, event};

const HSTS: u32 = 0xBF80_1800;
const REG1: u32 = 0xBF80_1801;
const REG2: u32 = 0xBF80_1802;
const REG3: u32 = 0xBF80_1803;
const RSLRRDY: u32 = 0x20;
const DRQSTS: u32 = 0x40;
const BUSYSTS: u32 = 0x80;
const D3_MADR: u32 = 0xBF80_10B0;
const D3_BCR: u32 = 0xBF80_10B4;
const D3_CHCR: u32 = 0xBF80_10B8;

/// The CD's events ("CDROM Events"), the five the kernel opens for itself.
const CLASS: u32 = 0xF000_0003;
const DONE_DMA: u32 = 0x10;
const DONE: u32 = 0x20;
const DEAD: u32 = 0x40;
const END: u32 = 0x80;
const ERROR: u32 = 0x8000;

/// Mode A0h: double speed, the whole sector but its sync, so that each sector's header can be checked.
const KERNEL_MODE: u32 = 0xA0;
const WHOLE: u32 = 0x924;
const USER: u32 = 0x800;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    /// A command, its parameters and whether a second response follows the first.
    Command(u32, [u8; 3], u32, bool),
    /// Sectors taken on INT1 until the count is met.
    Data,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    First,
    Second,
    Data,
}

const NOP: Step = Step::Command(0x01, [0; 3], 0, false);

/// How sectors reach memory: the kernel's checked window of user data, or an asynchronous read's whole sectors.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sink {
    Window,
    Raw(u32),
}

struct Read {
    sink: Sink,
    lba: u32,
    left: u32,
    dst: u32,
    skip: u32,
    total: u32,
    done: u32,
    restarts: u32,
}

static mut STEPS: [Step; 6] = [NOP; 6];
static mut COUNT: usize = 0;
static mut AT: usize = 0;
static mut PHASE: Phase = Phase::First;
static mut BUSY: bool = false;
static mut OK: bool = false;
static mut ASYNC: bool = false;
static mut ENDED: bool = false;
static mut OWNED: bool = false;
static mut MASKED: u32 = 0;
static mut RESULT: [u8; 16] = [0; 16];
static mut STAT: u32 = 0;
static mut INT5: [u32; 2] = [0; 2];
static mut READ: Read = Read { sink: Sink::Window, lba: 0, left: 0, dst: 0, skip: 0, total: 0, done: 0, restarts: 0 };
static mut WHOLE_SECTOR: [u32; WHOLE as usize / 4] = [0; WHOLE as usize / 4];
static mut ELEMENT: [u32; 4] = [0; 4];
static mut QUEUED: bool = false;
static mut EVENTS: bool = false;
static mut AUTO_ABORT: [u32; 2] = [0; 2];
static mut STATUS_DST: u32 = 0;

fn bank(b: u32) {
    wr8(HSTS, b);
}

fn bcd(v: u32) -> u8 {
    (v / 10 << 4 | v % 10) as u8
}

fn from_bcd(v: u32) -> u32 {
    (v >> 4) * 10 + (v & 0xF)
}

/// A logical sector's MSF, two seconds past the disc's start, in BCD.
fn msf(lba: u32) -> [u8; 3] {
    let s = lba + 150;
    [bcd(s / 4500), bcd(s / 75 % 60), bcd(s % 75)]
}

fn acknowledge_controller() {
    bank(1);
    wr8(REG3, 0x1F);
    bank(0);
}

/// The next command sent once BUSYSTS clears and no response is pending, its parameters first.
fn send(cmd: u32, params: [u8; 3], n: u32) {
    let mut polls = 0;
    while rd8(HSTS) & BUSYSTS != 0 && polls < 100_000 {
        polls += 1;
    }
    bank(1);
    if rd8(REG3) & 7 != 0 {
        wr8(REG3, 0x1F);
    }
    wr8(REG3, 0x40);
    bank(0);
    for p in params.iter().take(n as usize) {
        wr8(REG2, *p as u32);
    }
    unsafe { PHASE = Phase::First };
    wr8(REG1, cmd);
}

fn issue() {
    unsafe {
        match STEPS[AT] {
            Step::Command(c, p, n, _) => send(c, p, n),
            Step::Data => PHASE = Phase::Data,
        }
    }
}

fn advance() {
    unsafe {
        AT += 1;
        if AT >= COUNT {
            finish(true);
        } else {
            issue();
        }
    }
}

fn finish(ok: bool) {
    unsafe {
        let was_read = STEPS[..COUNT].contains(&Step::Data);
        BUSY = false;
        OK = ok;
        release();
        if STATUS_DST != 0 {
            wr8(STATUS_DST, STAT);
            STATUS_DST = 0;
        }
        if ASYNC {
            if !ok {
                event::deliver_event(CLASS, if ENDED { END } else { ERROR });
            } else {
                if was_read {
                    event::deliver_event(CLASS, DONE_DMA);
                }
                event::deliver_event(CLASS, DONE);
            }
        }
    }
}

/// The drive taken: the kernel's element answers IRQ2, which I_MASK lets through while it does.
fn own() {
    unsafe {
        if !OWNED {
            OWNED = true;
            MASKED = rd(io::I_MASK) & 4;
            wr(io::I_MASK, rd(io::I_MASK) | 4);
        }
    }
    bank(1);
    wr8(REG2, 0x1F);
    bank(0);
}

fn release() {
    unsafe {
        if OWNED {
            OWNED = false;
            if MASKED == 0 {
                wr(io::I_MASK, rd(io::I_MASK) & !4);
            }
        }
    }
}

fn start(steps: &[Step], asynchronous: bool) {
    wait_idle();
    unsafe {
        for (i, s) in steps.iter().enumerate() {
            STEPS[i] = *s;
        }
        COUNT = steps.len();
        AT = 0;
        BUSY = true;
        OK = false;
        ENDED = false;
        ASYNC = asynchronous;
        STATUS_DST = 0;
        if asynchronous {
            for spec in [DONE_DMA, DONE, END, ERROR] {
                event::undeliver_event(CLASS, spec);
            }
        }
    }
    entry::critical(|| {
        own();
        issue();
    });
}

/// DMA channel 3 enabled in DPCR, its priority kept, as the documented kernel's DMA-driven reads need it ("CDROM
/// Events": "cdrom DMA finished"; A(91h), A(93h)).
fn enable_dma() {
    wr(io::DPCR, rd(io::DPCR) | 0x8000);
}

/// `words` words of the data FIFO moved to `to` by DMA channel 3 in its usual mode, 11000000h ("DMA Channels").
fn dma(to: u32, words: u32) {
    enable_dma();
    wr(D3_MADR, to & 0x00FF_FFFF);
    wr(D3_BCR, 0x0001_0000 | words);
    wr(D3_CHCR, 0x1100_0000);
    let mut polls = 0;
    while rd(D3_CHCR) & 1 << 24 != 0 && polls < 100_000 {
        polls += 1;
    }
}

/// One sector taken once BFRD is set and DRQSTS rises: the kernel's whole sector into its buffer, its header checked
/// against the sector wanted and its user data copied into the window [skip, skip+total) of the stream; or an
/// asynchronous read's sector of the mode's size straight to its destination.
fn sector() {
    bank(0);
    wr8(REG3, 0);
    wr8(REG3, 0x80);
    bank(1);
    wr8(REG3, 0x1F);
    bank(0);
    let mut polls = 0;
    while rd8(HSTS) & DRQSTS == 0 && polls < 10_000 {
        polls += 1;
    }
    let r = unsafe { &mut *(&raw mut READ) };
    match r.sink {
        Sink::Raw(size) => {
            dma(r.dst, size / 4);
            r.dst += size;
            r.left -= 1;
        }
        Sink::Window => {
            let buffer = &raw mut WHOLE_SECTOR as u32;
            dma(buffer, WHOLE / 4);
            let at = (from_bcd(rd8(buffer)) * 60 + from_bcd(rd8(buffer + 1))) * 75 + from_bcd(rd8(buffer + 2));
            let at = at.wrapping_sub(150);
            if at == r.lba {
                let base = r.done * USER;
                let from = r.skip.max(base);
                let to = (r.skip + r.total).min(base + USER);
                let data = buffer + 12;
                let mut g = from;
                if (r.dst + g - r.skip) % 4 == 0 && (g - base) % 4 == 0 {
                    while g + 4 <= to {
                        wr(r.dst + g - r.skip, rd(data + g - base));
                        g += 4;
                    }
                }
                while g < to {
                    wr8(r.dst + g - r.skip, rd8(data + g - base));
                    g += 1;
                }
                r.lba += 1;
                r.done += 1;
                r.left -= 1;
            } else if at > r.lba && at < r.lba + 0x10000 {
                r.restarts += 1;
                bank(0);
                wr8(REG3, 0);
                if r.restarts > 8 {
                    return finish(false);
                }
                unsafe {
                    STEPS[0] = Step::Command(0x02, msf(r.lba), 3, false);
                    AT = 0;
                }
                return issue();
            }
        }
    }
    bank(0);
    wr8(REG3, 0);
    if r.left == 0 {
        advance();
    }
}

/// The controller's response, if one is pending, read, acknowledged and acted on; called with interrupts off.
fn service() {
    bank(1);
    let t = rd8(REG3) & 7;
    if t == 0 {
        bank(0);
        acknowledge(2);
        return;
    }
    let result = &raw mut RESULT as u32;
    let mut n = 0;
    while rd8(HSTS) & RSLRRDY != 0 && n < 16 {
        wr8(result + n, rd8(REG1));
        n += 1;
    }
    acknowledge(2);
    if t == 1 {
        if unsafe { BUSY && PHASE == Phase::Data } {
            return sector();
        }
        return acknowledge_controller();
    }
    acknowledge_controller();
    if !unsafe { BUSY } {
        return;
    }
    let r0 = rd8(result);
    unsafe { STAT = r0 };
    match (t, unsafe { PHASE }) {
        (5, _) => unsafe {
            INT5 = [r0, rd8(result + 1)];
            finish(false)
        },
        (4, _) => unsafe {
            ENDED = true;
            finish(false)
        },
        (3, Phase::First) => unsafe {
            match STEPS[AT] {
                Step::Command(_, _, _, true) => PHASE = Phase::Second,
                _ => advance(),
            }
        },
        (2, Phase::Second) => advance(),
        _ => {}
    }
}

/// Until the operation ends, servicing the controller itself between the interrupts it may take; false on a failure.
fn wait() -> bool {
    let mut polls = 0u32;
    while unsafe { BUSY } {
        entry::critical(service);
        polls += 1;
        if polls > 4_000_000 {
            entry::critical(|| finish(false));
        }
    }
    unsafe { OK }
}

fn wait_idle() {
    if unsafe { BUSY } {
        wait();
    }
}

fn run(steps: &[Step]) -> bool {
    start(steps, false);
    wait()
}

/// A(90h): the priority 0 element's first function; it acts only on an operation of the kernel's own.
pub extern "C" fn cdrom_io_irq_func1() -> u32 {
    if unsafe { OWNED } && pending() & 4 != 0 {
        service();
    }
    0
}

/// A(92h): no second function is needed by the first, which handles the response itself.
pub extern "C" fn cdrom_io_irq_func2() -> u32 {
    0
}

/// A(91h) and A(93h): the kernel waits for each transfer on channel 3 itself, so nothing waits on its interrupt.
pub extern "C" fn cdrom_dma_irq_func() -> u32 {
    0
}

/// A(A2h): the element at priority 0, as psx-spx gives it.
pub extern "C" fn enqueue_cd_intr() -> u32 {
    unsafe {
        if !QUEUED {
            let e = &raw mut ELEMENT as u32;
            wr(e + 4, cdrom_io_irq_func2 as *const () as u32);
            wr(e + 8, cdrom_io_irq_func1 as *const () as u32);
            exception::sys_enq_int_rp(0, e);
            QUEUED = true;
        }
    }
    0
}

/// A(A3h).
pub extern "C" fn dequeue_cd_intr() -> u32 {
    unsafe {
        if QUEUED {
            exception::sys_deq_int_rp(0, &raw const ELEMENT as u32);
            QUEUED = false;
        }
    }
    0
}

/// SetConf's reset loses the element and the events with the chains and the EvCBs.
pub fn forget() {
    unsafe {
        QUEUED = false;
        EVENTS = false;
    }
}

/// A(95h): psx-spx's "To init the CD": the interrupts flushed, HCHPCTL cleared, COM_DELAY 1325h, two Nops, Init and
/// Demute.
pub extern "C" fn cd_init_sub_func() -> u32 {
    entry::critical(|| {
        acknowledge_controller();
        acknowledge(2);
        wr8(REG3, 0);
    });
    wr(io::COM_DELAY, 0x1325);
    enable_dma();
    let ok = run(&[NOP, NOP, Step::Command(0x0A, [0; 3], 0, true), Step::Command(0x0C, [0; 3], 0, false)]);
    ok as u32
}

/// A(54h) and A(71h): the element, the kernel's five events, the device and the controller; 1.
pub extern "C" fn init_96() -> u32 {
    enqueue_cd_intr();
    unsafe {
        if !EVENTS {
            for spec in [DONE_DMA, DONE, DEAD, END, ERROR] {
                let e = event::open_event(CLASS, spec, event::MODE_READY, 0);
                event::enable_event(e);
            }
            EVENTS = true;
        }
    }
    crate::cdfs::unmount();
    cd_init_sub_func();
    1
}

/// A(56h) and A(72h): the element removed (the original's SysDeqIntRP defect is not reproduced, §4.2).
pub extern "C" fn remove_96() -> u32 {
    dequeue_cd_intr();
    0
}

/// A(9Eh): the flags kept; the kernel's reads end by their own count.
pub extern "C" fn set_cdrom_irq_auto_abort(kind: u32, flag: u32) -> u32 {
    unsafe { AUTO_ABORT[(kind & 1) as usize] = flag };
    0
}

/// A(94h): the first two bytes of the latest INT5.
pub extern "C" fn cdrom_get_int5_err_code(dst1: u32, dst2: u32) -> u32 {
    unsafe {
        wr8(dst1, INT5[0]);
        wr8(dst2, INT5[1]);
    }
    1
}

/// `count` sectors from `lba`, the window [skip, skip+total) of their user data stored at `dst`; false on a failure.
pub fn read(lba: u32, count: u32, dst: u32, skip: u32, total: u32) -> bool {
    wait_idle();
    unsafe {
        READ = Read { sink: Sink::Window, lba, left: count, dst, skip, total, done: 0, restarts: 0 };
    }
    run(&[
        Step::Command(0x02, msf(lba), 3, false),
        Step::Command(0x0E, [KERNEL_MODE as u8, 0, 0], 1, false),
        Step::Command(0x06, [0; 3], 0, false),
        Step::Data,
        Step::Command(0x09, [0; 3], 0, true),
    ])
}

/// The status byte, by Nop (Getstat); None when the command failed.
pub fn status() -> Option<u32> {
    run(&[NOP]).then(|| unsafe { STAT })
}

/// GetID's eight bytes, or the INT5 that refused it.
pub fn get_id() -> Result<[u8; 8], [u32; 2]> {
    if run(&[Step::Command(0x1A, [0; 3], 0, true)]) {
        let mut id = [0u8; 8];
        for (i, b) in id.iter_mut().enumerate() {
            *b = unsafe { RESULT[i] };
        }
        Ok(id)
    } else {
        Err(unsafe { INT5 })
    }
}

/// A(A5h): in the kernel's mode, each sector's header checked; the count, or -1.
pub extern "C" fn cd_read_sector(count: u32, sector: u32, buffer: u32) -> u32 {
    if read(sector, count, buffer, 0, count * USER) { count } else { u32::MAX }
}

/// A(A6h).
pub extern "C" fn cd_get_status() -> u32 {
    status().unwrap_or(u32::MAX)
}

/// A(78h): Setloc and SeekL; 1, the end signalled by the events.
pub extern "C" fn cd_async_seek_l(src: u32) -> u32 {
    let p = [rd8(src) as u8, rd8(src + 1) as u8, rd8(src + 2) as u8];
    start(&[Step::Command(0x02, p, 3, false), Step::Command(0x15, [0; 3], 0, true)], true);
    1
}

/// A(7Ch): Getstat, its byte stored once it arrives.
pub extern "C" fn cd_async_get_status(dst: u32) -> u32 {
    start(&[NOP], true);
    unsafe { STATUS_DST = dst };
    1
}

/// A(7Eh): Setmode and ReadN, or ReadS with bit 8, `count` sectors of the mode's size to `dst`, then Pause.
pub extern "C" fn cd_async_read_sector(count: u32, dst: u32, mode: u32) -> u32 {
    let size = if mode & 0x10 != 0 {
        0x918
    } else if mode & 0x20 != 0 {
        WHOLE
    } else {
        USER
    };
    unsafe {
        READ = Read { sink: Sink::Raw(size), lba: 0, left: count, dst, skip: 0, total: 0, done: 0, restarts: 0 };
    }
    let read = if mode & 0x100 != 0 { 0x1B } else { 0x06 };
    start(
        &[
            Step::Command(0x0E, [mode as u8, 0, 0], 1, false),
            Step::Command(read, [0; 3], 0, false),
            Step::Data,
            Step::Command(0x09, [0; 3], 0, true),
        ],
        true,
    );
    1
}

/// A(81h).
pub extern "C" fn cd_async_set_mode(mode: u32) -> u32 {
    start(&[Step::Command(0x0E, [mode as u8, 0, 0], 1, false)], true);
    1
}

/// Whether the status shows the shell opened since it was last read.
pub fn shell_was_opened() -> bool {
    unsafe { STAT & 0x10 != 0 }
}
