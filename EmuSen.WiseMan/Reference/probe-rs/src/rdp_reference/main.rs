// Replays an RDPDUMP2 command stream through angrylion-rdp-plus and writes what it
// drew, so Mars's display processor can be graded against the reference rasterizer.
// See Mars_RdpDifferential.md.
//
//   rdp-reference <dump> <out>
//
// The replay is the one parallel-rdp's own angrylion driver performs, so this tool
// and rdp-validate-dump put the same stream through the same library: RDRAM uploads
// are gathered and applied at each flush, each command's words go to rdp_cmd, and
// each SignalComplete record is a comparison point. It links the libalp-core.a that
// rdp-validate-dump's build produced rather than compiling angrylion again, so the
// two instruments cannot differ by compiler flags.
//
// <out> is RDPREF01, little-endian: the eight-byte magic, then for each sync a
// record (tag 1, index from 1), every 4 KiB page of RDRAM (tag 2) or hidden RDRAM
// (tag 3) that differs from what the last flush uploaded as offset and bytes, the
// RDP's 4 KiB of texture memory as angrylion stores it (tag 5, bytes), and every
// message angrylion printed since the sync before (tag 4, length and bytes).
// A tag of 0 ends the file.
use std::ffi::{CStr, c_char, c_void};
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};
use std::process::ExitCode;
use std::sync::Mutex;

const PAGE: usize = 4096;
const HIDDEN_SIZE: usize = 0x40_0000;
const VI_REGISTERS: usize = 14;
const DP_REGISTERS: usize = 8;

// n64video.h's enums, by the values the upstream driver chose.
const VI_MODE_NORMAL: u32 = 0;
const VI_INTERP_LINEAR: u32 = 1;
const DP_COMPAT_HIGH: u32 = 2;

#[repr(C)]
pub struct Gfx {
    rdram: *mut u8,
    rdram_size: u32,
    dmem: *mut u8,
    vi_reg: *mut *mut u32,
    dp_reg: *mut *mut u32,
    mi_intr_reg: *mut u32,
    mi_intr_cb: Option<unsafe extern "C" fn()>,
}

#[repr(C)]
pub struct Vi {
    mode: u32,
    interp: u32,
    widescreen: bool,
    hide_overscan: bool,
    vsync: bool,
    exclusive: bool,
}

#[repr(C)]
pub struct Dp {
    compat: u32,
}

// vdac.h's frame_buffer: a window onto angrylion's persistent prescale buffer.
#[repr(C)]
pub struct FrameBuffer {
    pixels: *const u8,
    width: u32,
    height: u32,
    height_out: u32,
    pitch: u32,
}

#[repr(C)]
pub struct Config {
    gfx: Gfx,
    vi: Vi,
    dp: Dp,
    parallel: bool,
    num_workers: u32,
}

unsafe extern "C" {
    fn n64video_init(config: *mut Config);
    fn n64video_update_screen();
    fn n64video_close();
    fn rdp_cmd(wid: u32, args: *const u32);
    fn get_tmem() -> *mut u8;
    static mut rdram_hidden: [u8; HIDDEN_SIZE];
}

static MESSAGES: Mutex<Vec<String>> = Mutex::new(Vec::new());
static FRAMES: Mutex<Vec<(u32, u32, Vec<u8>)>> = Mutex::new(Vec::new());

fn remember(kind: &str, format: *const c_char) {
    if format.is_null() {
        return;
    }
    // SAFETY: angrylion passes a string literal as the format.
    let text = unsafe { CStr::from_ptr(format) }.to_string_lossy();
    MESSAGES.lock().unwrap().push(format!("{kind}: {}", text.trim_end()));
}

// The host half of angrylion's interface. Nothing is displayed: the RDRAM and the
// scanned-out frames are the output.
#[unsafe(no_mangle)]
pub extern "C" fn vdac_init(_config: *mut Config) {}

// Copied row by row, because the pointer is into a buffer angrylion keeps between frames.
#[unsafe(no_mangle)]
pub extern "C" fn vdac_write(frame: *mut c_void) {
    if frame.is_null() {
        return;
    }
    // SAFETY: angrylion passes a frame_buffer it owns, valid until this call returns.
    let frame = unsafe { &*frame.cast::<FrameBuffer>() };
    if frame.pixels.is_null() || frame.width == 0 || frame.height == 0 {
        return;
    }

    let width = frame.width as usize;
    let mut pixels = Vec::with_capacity(width * frame.height as usize * 4);
    for row in 0..frame.height as usize {
        // SAFETY: the row lies inside the prescale buffer the frame describes.
        let start = unsafe { frame.pixels.add((row * frame.pitch as usize) * 4) };
        pixels.extend_from_slice(unsafe { std::slice::from_raw_parts(start, width * 4) });
    }

    FRAMES.lock().unwrap().push((frame.width, frame.height, pixels));
}

#[unsafe(no_mangle)]
pub extern "C" fn vdac_sync(_invalid: bool) {}

#[unsafe(no_mangle)]
pub extern "C" fn vdac_close() {}

// msg.h declares these variadic, and stable Rust cannot define a variadic function.
// Only the format is read, and a variadic call passes its first argument where a
// fixed one goes on every calling convention this tool is built for, so the
// arguments are left unread and the recorded message keeps its placeholders.
#[unsafe(no_mangle)]
pub extern "C" fn msg_error(format: *const c_char) {
    remember("error", format);
}

#[unsafe(no_mangle)]
pub extern "C" fn msg_warning(format: *const c_char) {
    remember("warning", format);
}

unsafe extern "C" fn no_interrupt() {}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: rdp-reference <dump> <out>");
        return ExitCode::from(2);
    }

    match replay(&args[1], &args[2]) {
        Ok(syncs) => {
            println!("{syncs} syncs");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("rdp-reference: {error}");
            ExitCode::FAILURE
        }
    }
}

fn replay(dump: &str, out: &str) -> Result<u32, String> {
    let file = File::open(dump).map_err(|e| format!("{dump}: {e}"))?;
    let mut input = BufReader::new(file);

    let mut magic = [0u8; 8];
    input.read_exact(&mut magic).map_err(|e| format!("{dump}: {e}"))?;
    if &magic != b"RDPDUMP2" {
        return Err(format!("{dump} is not an RDPDUMP2 dump"));
    }

    let rdram_size = word(&mut input)? as usize;
    let hidden_size = word(&mut input)? as usize;
    if rdram_size != 0x40_0000 && rdram_size != 0x80_0000 {
        return Err(format!("RDRAM of {rdram_size} bytes is neither 4 nor 8 MiB"));
    }
    if hidden_size != HIDDEN_SIZE {
        return Err(format!("hidden RDRAM of {hidden_size} bytes is not 4 MiB"));
    }

    // Owned through raw pointers alone, because angrylion writes through them between our reads.
    let rdram: *mut u8 = Box::into_raw(vec![0u8; rdram_size].into_boxed_slice()).cast();
    let mut cache = vec![0u8; rdram_size];
    let mut baseline = vec![0u8; rdram_size];
    let mut hidden_cache = vec![0u8; HIDDEN_SIZE];
    let mut hidden_baseline = vec![0u8; HIDDEN_SIZE];

    let registers: *mut u32 = Box::into_raw(vec![0u32; VI_REGISTERS + DP_REGISTERS + 1].into_boxed_slice()).cast();
    // SAFETY: each offset is inside the allocation just made.
    let mut vi_pointers: Vec<*mut u32> = (0..VI_REGISTERS).map(|i| unsafe { registers.add(i) }).collect();
    let mut dp_pointers: Vec<*mut u32> = (0..DP_REGISTERS).map(|i| unsafe { registers.add(VI_REGISTERS + i) }).collect();
    // SAFETY: the last slot of the same allocation.
    let interrupts = unsafe { registers.add(VI_REGISTERS + DP_REGISTERS) };

    let mut config = Config {
        gfx: Gfx {
            rdram,
            rdram_size: rdram_size as u32,
            dmem: std::ptr::null_mut(),
            vi_reg: vi_pointers.as_mut_ptr(),
            dp_reg: dp_pointers.as_mut_ptr(),
            mi_intr_reg: interrupts,
            mi_intr_cb: Some(no_interrupt),
        },
        vi: Vi {
            mode: VI_MODE_NORMAL,
            interp: VI_INTERP_LINEAR,
            widescreen: false,
            hide_overscan: false,
            vsync: false,
            exclusive: false,
        },
        dp: Dp { compat: DP_COMPAT_HIGH },
        parallel: false,
        num_workers: 0,
    };

    // SAFETY: every pointer in config outlives the replay, and angrylion is closed before they drop.
    unsafe { n64video_init(&mut config) };

    let written = File::create(out).map_err(|e| format!("{out}: {e}"))?;
    let mut output = BufWriter::new(written);
    output.write_all(b"RDPREF01").map_err(|e| format!("{out}: {e}"))?;
    let result = records(
        &mut input,
        &mut output,
        rdram,
        &mut cache,
        &mut baseline,
        &mut hidden_cache,
        &mut hidden_baseline,
        &vi_pointers,
    );

    // SAFETY: paired with the init above; after it nothing holds either pointer, so both allocations are released.
    unsafe {
        n64video_close();
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(rdram, rdram_size)));
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(registers, VI_REGISTERS + DP_REGISTERS + 1)));
    }

    let syncs = result?;
    output.write_all(&0u32.to_le_bytes()).map_err(|e| format!("{out}: {e}"))?;
    output.flush().map_err(|e| format!("{out}: {e}"))?;
    Ok(syncs)
}

#[allow(clippy::too_many_arguments)]
fn records(
    input: &mut impl Read,
    output: &mut impl Write,
    rdram: *mut u8,
    cache: &mut [u8],
    baseline: &mut [u8],
    hidden_cache: &mut [u8],
    hidden_baseline: &mut [u8],
    vi: &[*mut u32],
) -> Result<u32, String> {
    let mut syncs = 0u32;
    let mut words: Vec<u32> = Vec::new();

    // rdp_dump.cpp's record numbers.
    while let Some(record) = next_word(input)? {
        match record {
            1 => upload(input, cache)?,
            2 => {
                let _id = word(input)?;
                let count = word(input)? as usize;
                words.resize(count, 0);
                for slot in words.iter_mut() {
                    *slot = word(input)?;
                }
                if count > 0 {
                    // SAFETY: rdp_cmd reads only as many words as the command's own length, which the dump carries.
                    unsafe { rdp_cmd(0, words.as_ptr()) };
                }
            }
            3 => {
                let index = word(input)? as usize;
                let value = word(input)?;
                if let Some(&register) = vi.get(index) {
                    // SAFETY: one of angrylion's own register slots, written while it is not running.
                    unsafe { *register = value };
                }
            }
            // SAFETY: the configuration it reads is the one initialised in replay.
            4 => unsafe { n64video_update_screen() },
            5 => {
                syncs += 1;
                // SAFETY: a borrow that ends before angrylion runs again.
                let drawn = unsafe { std::slice::from_raw_parts(rdram, baseline.len()) };
                snapshot(output, syncs, drawn, baseline, hidden_baseline)?;
            }
            6 => return Ok(syncs),
            7 => {
                // SAFETY: as above.
                unsafe { std::slice::from_raw_parts_mut(rdram, cache.len()) }.copy_from_slice(cache);
                baseline.copy_from_slice(cache);
            }
            8 => upload(input, hidden_cache)?,
            9 => {
                // SAFETY: rdram_hidden is angrylion's own 4 MiB array, and nothing else runs.
                unsafe { (*(&raw mut rdram_hidden)).copy_from_slice(hidden_cache) };
                hidden_baseline.copy_from_slice(hidden_cache);
            }
            other => return Err(format!("unknown dump record {other}")),
        }
    }

    Ok(syncs)
}

fn upload(input: &mut impl Read, cache: &mut [u8]) -> Result<(), String> {
    let offset = word(input)? as usize;
    let size = word(input)? as usize;
    let end = offset.checked_add(size).filter(|&end| end <= cache.len());
    let Some(end) = end else {
        return Err(format!("upload of {size} bytes at {offset:#x} is past the end of memory"));
    };
    input.read_exact(&mut cache[offset..end]).map_err(|e| e.to_string())
}

fn snapshot(
    output: &mut impl Write,
    index: u32,
    rdram: &[u8],
    baseline: &[u8],
    hidden_baseline: &[u8],
) -> Result<(), String> {
    let mut put = |bytes: &[u8]| output.write_all(bytes).map_err(|e| e.to_string());

    put(&1u32.to_le_bytes())?;
    put(&index.to_le_bytes())?;

    for (page, (now, before)) in rdram.chunks(PAGE).zip(baseline.chunks(PAGE)).enumerate() {
        if now != before {
            put(&2u32.to_le_bytes())?;
            put(&((page * PAGE) as u32).to_le_bytes())?;
            put(now)?;
        }
    }

    // SAFETY: read only, between commands.
    let hidden: &[u8] = unsafe { &*(&raw const rdram_hidden) };
    for (page, (now, before)) in hidden.chunks(PAGE).zip(hidden_baseline.chunks(PAGE)).enumerate() {
        if now != before {
            put(&3u32.to_le_bytes())?;
            put(&((page * PAGE) as u32).to_le_bytes())?;
            put(now)?;
        }
    }

    // SAFETY: angrylion's own 4 KiB texture memory, read between commands.
    let texture_memory: &[u8] = unsafe { std::slice::from_raw_parts(get_tmem(), 0x1000) };
    put(&5u32.to_le_bytes())?;
    put(texture_memory)?;

    for message in MESSAGES.lock().unwrap().drain(..) {
        put(&4u32.to_le_bytes())?;
        put(&(message.len() as u32).to_le_bytes())?;
        put(message.as_bytes())?;
    }

    for (width, height, pixels) in FRAMES.lock().unwrap().drain(..) {
        put(&6u32.to_le_bytes())?;
        put(&width.to_le_bytes())?;
        put(&height.to_le_bytes())?;
        put(&pixels)?;
    }

    Ok(())
}

fn word(input: &mut impl Read) -> Result<u32, String> {
    next_word(input)?.ok_or_else(|| "the dump ends inside a record".to_string())
}

fn next_word(input: &mut impl Read) -> Result<Option<u32>, String> {
    let mut bytes = [0u8; 4];
    match input.read_exact(&mut bytes) {
        Ok(()) => Ok(Some(u32::from_le_bytes(bytes))),
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
