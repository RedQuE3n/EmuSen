//! The runner's side: one v1 library loaded in a process of its own, each request answered by the export it names, with
//! the caller's buffer sizes and the export's exact return (EmuSen_LibretroHost.md §3.2).

use std::ffi::CString;
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::net::UnixStream;

use emusen_core_host::exports::Exports;
use emusen_native::core::sys::{CreateParams, Event, FileEntry, FrameInfo, Machine as RawMachine, Media, RingFrame, RingLend, status};

use crate::wire::{self, Head, Op, Reader};

/// The most a caller's buffer may ask the runner to allocate.
const MAX_OUT: u64 = 1 << 30;

/// A file mapped read-only for the machine's life, so a disc image is never copied.
pub struct Mapped {
    ptr: *mut u8,
    len: usize,
}

impl Mapped {
    pub fn open(path: &str) -> std::io::Result<Mapped> {
        let f = std::fs::File::open(path)?;
        let len = f.metadata()?.len() as usize;
        if len == 0 {
            return Ok(Mapped { ptr: std::ptr::null_mut(), len: 0 });
        }
        use std::os::fd::AsRawFd;
        let p = unsafe { libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ, libc::MAP_PRIVATE, f.as_raw_fd(), 0) };
        if p == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Mapped { ptr: p as *mut u8, len })
    }

    pub fn bytes(&self) -> &[u8] {
        if self.ptr.is_null() { &[] } else { unsafe { std::slice::from_raw_parts(self.ptr, self.len) } }
    }
}

impl Drop for Mapped {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { libc::munmap(self.ptr as *mut libc::c_void, self.len) };
        }
    }
}

/// The served library and its one machine.
pub struct Served {
    pub x: &'static Exports,
    pub m: *mut RawMachine,
    pub images: Vec<Mapped>,
    pub done: bool,
}

/// The bytes an export wrote into a buffer of `len`, by its return.
fn filled(ret: i64, len: usize, unit: usize) -> usize {
    if ret < 0 { 0 } else { (ret as usize).saturating_mul(unit).min(len) }
}

/// A buffer the caller's size asks for, and the pointer the export is given: null where the caller's was.
fn out(h: &Head) -> Result<(Vec<u8>, bool), i64> {
    if h.out_len > MAX_OUT {
        return Err(status::BUFFER_TOO_SMALL as i64);
    }
    Ok((vec![0u8; h.out_len as usize], h.out_null))
}

fn ptr(buf: &mut [u8], null: bool) -> *mut u8 {
    if null { std::ptr::null_mut() } else { buf.as_mut_ptr() }
}

/// Files of a create or an insert: (which, name, data), with their names kept alive.
struct Files {
    entries: Vec<FileEntry>,
    _names: Vec<CString>,
}

fn files(r: &mut Reader<'_>) -> Files {
    let n = r.u32() as usize;
    let mut entries = Vec::with_capacity(n);
    let mut names = Vec::new();
    for _ in 0..n.min(1 << 16) {
        let which = r.u32();
        let name = r.text().map(|t| CString::new(t).unwrap_or_default());
        let data = r.bytes();
        entries.push(FileEntry { size: size_of::<FileEntry>() as u32, which, data: data.as_ptr(), len: data.len(), name: name.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()) });
        names.extend(name);
    }
    Files { entries, _names: names }
}

impl Served {
    /// One request's reply: (return, word beside it, bytes written).
    pub fn answer(&mut self, h: &Head, args: &[u8], fds: Vec<OwnedFd>) -> (i64, i64, Vec<u8>) {
        let Some(op) = Op::from_u16(h.op) else { return (status::NOT_SUPPORTED as i64, 0, Vec::new()) };
        let mut r = Reader::new(args);
        let x = self.x;
        let m = self.m;
        let none = (status::NOT_SUPPORTED as i64, 0, Vec::new());
        macro_rules! text_out {
            ($call:expr) => {{
                let (mut b, null) = match out(h) {
                    Ok(o) => o,
                    Err(e) => return (e, 0, Vec::new()),
                };
                let p = ptr(&mut b, null);
                let len = b.len();
                let ret = unsafe { $call(p, len) };
                b.truncate(if null { 0 } else { filled(ret, len, 1) });
                (ret, 0, b)
            }};
        }
        match op {
            Op::Ping => (0, 0, Vec::new()),
            Op::Capabilities => (unsafe { (x.capabilities)() } as i64, unsafe { (x.abi_version)() } as i64, Vec::new()),
            Op::Info => text_out!(|p, l| (x.info)(p, l)),
            Op::SettingsSchema => text_out!(|p, l| (x.settings_schema)(p, l)),
            Op::FirmwareFor => {
                let image = r.bytes().to_vec();
                text_out!(|p, l| (x.firmware_for)(image.as_ptr(), image.len(), p, l))
            }
            Op::StatusText => {
                let code = r.u32() as i32;
                text_out!(|p, l| (x.status_text)(code, p, l))
            }
            Op::SetCrashLog => {
                let path = CString::new(r.bytes()).unwrap_or_default();
                (unsafe { (x.set_crash_log)(path.as_ptr()) } as i64, 0, Vec::new())
            }
            Op::LogDrain => text_out!(|p, l| (x.log_drain)(m, p, l)),
            Op::Create => self.create(&mut r),
            Op::Free => {
                let ret = if m.is_null() { status::NULL } else { unsafe { (x.free)(m) } };
                self.m = std::ptr::null_mut();
                self.done = true;
                (ret as i64, 0, Vec::new())
            }
            Op::MachineInfo => text_out!(|p, l| (x.machine_info)(m, p, l)),
            Op::LastError => text_out!(|p, l| (x.last_error)(m, p, l)),
            Op::SettingNotes => match x.setting_notes {
                Some(f) => text_out!(|p, l| f(m, p, l)),
                None => none,
            },
            Op::Advance => {
                let ranges: Vec<(u32, u32, u32)> = (0..r.u32().min(4096)).map(|_| (r.u32(), r.u32(), r.u32())).collect();
                let mut detail = 0u64;
                let ret = unsafe { (x.advance)(m, &mut detail) };
                let mut window = Vec::new();
                if ret >= 0 {
                    for (space, address, len) in ranges {
                        let at = window.len();
                        window.resize(at + len as usize, 0);
                        let got = unsafe { (x.space_read)(m, space, address, window[at..].as_mut_ptr(), len as usize) };
                        if got < 0 {
                            window.truncate(at);
                            break;
                        }
                    }
                }
                (ret as i64, detail as i64, window)
            }
            Op::SetOptions => (unsafe { (x.set_options)(m, r.u32()) } as i64, 0, Vec::new()),
            Op::FrameCount => (unsafe { (x.frame_count)(m) }, 0, Vec::new()),
            Op::Events => {
                let count = (h.out_len as usize / size_of::<Event>()).min(1 << 16);
                let mut ev = vec![Event { size: 0, kind: 0, a: 0, b: 0 }; count];
                let p = if h.out_null { std::ptr::null_mut() } else { ev.as_mut_ptr() };
                let ret = unsafe { (x.events)(m, p, count, size_of::<Event>()) };
                let n = if h.out_null { 0 } else { filled(ret, count, 1) };
                let bytes = unsafe { std::slice::from_raw_parts(ev.as_ptr() as *const u8, n * size_of::<Event>()) }.to_vec();
                (ret, 0, bytes)
            }
            Op::FrameInfo => {
                let mut f: FrameInfo = unsafe { std::mem::zeroed() };
                f.size = size_of::<FrameInfo>() as u32;
                let ret = unsafe { (x.frame_info)(m, &mut f) };
                (ret as i64, 0, unsafe { std::slice::from_raw_parts(&f as *const FrameInfo as *const u8, size_of::<FrameInfo>()) }.to_vec())
            }
            Op::FrameCopy => text_out!(|p, l| (x.frame_copy)(m, p, l)),
            Op::AudioRate => (unsafe { (x.audio_rate)(m) } as i64, 0, Vec::new()),
            Op::AudioBuffered => (unsafe { (x.audio_buffered)(m) }, 0, Vec::new()),
            Op::AudioDrain => {
                let max_frames = r.i64();
                let samples = (h.out_len as usize / 2).min(1 << 24);
                let mut buf = vec![0i16; samples];
                let mut rate = 0i32;
                let p = if h.out_null { std::ptr::null_mut() } else { buf.as_mut_ptr() };
                let ret = unsafe { (x.audio_drain)(m, p, samples, max_frames, &mut rate) };
                let n = if h.out_null { 0 } else { filled(ret, samples, 1) };
                (ret, rate as i64, unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, n * 2) }.to_vec())
            }
            Op::SetAudioLimit => (unsafe { (x.set_audio_limit)(m, r.u64()) } as i64, 0, Vec::new()),
            Op::SetButtons => {
                let (port, mask, changed) = (r.u32(), r.u32(), r.u32());
                (unsafe { (x.set_buttons)(m, port, mask, changed) } as i64, 0, Vec::new())
            }
            Op::StateSize => (unsafe { (x.state_size)(m, r.u32()) }, 0, Vec::new()),
            Op::StateSave => {
                let kind = r.u32();
                text_out!(|p, l| (x.state_save)(m, kind, p, l))
            }
            Op::StateLoad => {
                let data = r.bytes();
                (unsafe { (x.state_load)(m, data.as_ptr(), data.len()) } as i64, 0, Vec::new())
            }
            Op::StateLayout => {
                let kind = r.u32();
                text_out!(|p, l| (x.state_layout)(m, kind, p, l))
            }
            Op::SpaceSize => (unsafe { (x.space_size)(m, r.u32()) }, 0, Vec::new()),
            Op::SpaceRead => {
                let (space, address) = (r.u32(), r.u32());
                text_out!(|p, l| (x.space_read)(m, space, address, p, l))
            }
            Op::SpaceWrite => {
                let (space, address) = (r.u32(), r.u32());
                let data = r.bytes();
                (unsafe { (x.space_write)(m, space, address, data.as_ptr(), data.len()) }, 0, Vec::new())
            }
            Op::Battery => {
                let which = r.u32();
                let mut flags = 0u32;
                let (mut b, null) = match out(h) {
                    Ok(o) => o,
                    Err(e) => return (e, 0, Vec::new()),
                };
                let len = b.len();
                let ret = unsafe { (x.battery)(m, which, ptr(&mut b, null), len, &mut flags) };
                b.truncate(if null { 0 } else { filled(ret, len, 1) });
                (ret, flags as i64, b)
            }
            Op::BatterySaved => (unsafe { (x.battery_saved)(m, r.u32()) } as i64, 0, Vec::new()),
            Op::Reset => x.reset.map_or(none, |f| (unsafe { f(m) } as i64, 0, Vec::new())),
            Op::Phases => match x.phases {
                Some(f) => {
                    let n = (h.out_len as usize / 8).min(1 << 12);
                    let mut v = vec![0i64; n];
                    let p = if h.out_null { std::ptr::null_mut() } else { v.as_mut_ptr() };
                    let ret = unsafe { f(m, p, n) };
                    let k = if h.out_null { 0 } else { filled(ret, n, 1) };
                    (ret, 0, v[..k].iter().flat_map(|x| x.to_le_bytes()).collect())
                }
                None => none,
            },
            Op::SetAxis => {
                let (port, axis, value) = (r.u32(), r.u32(), r.f64());
                x.set_axis.map_or(none, |f| (unsafe { f(m, port, axis, value) } as i64, 0, Vec::new()))
            }
            Op::SetSettings => {
                let text = r.bytes();
                x.set_settings.map_or(none, |f| (unsafe { f(m, text.as_ptr(), text.len()) } as i64, 0, Vec::new()))
            }
            Op::MediaLid => {
                let (drive, open) = (r.u32(), r.u32());
                x.media_lid.map_or(none, |f| (unsafe { f(m, drive, open) } as i64, 0, Vec::new()))
            }
            Op::MediaInsert => self.insert(&mut r),
            Op::RingLend => self.ring(args, fds),
            Op::FrameRing => match x.frame_ring {
                Some(f) => {
                    let mut out = RingFrame { size: size_of::<RingFrame>() as u32, ..RingFrame::default() };
                    let ret = unsafe { f(m, &mut out) };
                    (ret as i64, 0, unsafe { std::slice::from_raw_parts(&out as *const RingFrame as *const u8, size_of::<RingFrame>()) }.to_vec())
                }
                None => none,
            },
            Op::SetPresent => {
                let (port, present) = (r.u32(), r.u32());
                x.set_present.map_or(none, |f| (unsafe { f(m, port, present) } as i64, 0, Vec::new()))
            }
        }
    }

    /// An image's bytes as sent, or mapped from its path where only the path came.
    fn image(&mut self, sent: Option<&[u8]>, path: Option<&[u8]>) -> Result<(*const u8, usize), String> {
        if let Some(b) = sent {
            return Ok((b.as_ptr(), b.len()));
        }
        let Some(path) = path.and_then(|p| std::str::from_utf8(p).ok()) else { return Ok((std::ptr::null(), 0)) };
        let mapped = Mapped::open(path).map_err(|e| format!("{path}: {e}"))?;
        let (p, n) = (mapped.bytes().as_ptr(), mapped.bytes().len());
        self.images.push(mapped);
        Ok((p, n))
    }

    fn create(&mut self, r: &mut Reader<'_>) -> (i64, i64, Vec<u8>) {
        let settings = r.bytes().to_vec();
        let files = files(r);
        let pixel_formats = r.u64();
        let flags = r.u32();
        let system = r.text().map(|s| CString::new(s).unwrap_or_default());
        let sent = r.text().map(<[u8]>::to_vec);
        let path = r.text().map(|s| CString::new(s).unwrap_or_default());
        if r.short {
            return (status::BAD_STRUCT as i64, 0, b"the broker's create request was short".to_vec());
        }
        let (image, image_len) = match self.image(sent.as_deref(), path.as_ref().map(|p| p.as_bytes())) {
            Ok(i) => i,
            Err(why) => return (status::BAD_IMAGE as i64, 0, why.into_bytes()),
        };
        let mut error = vec![0u8; 4096];
        let p = CreateParams {
            size: size_of::<CreateParams>() as u32,
            host_abi_version: emusen_native::core::sys::ABI_VERSION,
            image,
            image_len,
            settings: settings.as_ptr(),
            settings_len: settings.len(),
            files: files.entries.as_ptr(),
            file_count: files.entries.len(),
            file_size: size_of::<FileEntry>(),
            pixel_formats,
            error: error.as_mut_ptr(),
            error_len: error.len(),
            flags,
            device: std::ptr::null(),
            system: system.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
            image_path: path.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
        };
        let mut code = 0i32;
        let m = unsafe { (self.x.create)(&p, &mut code) };
        if m.is_null() {
            let end = error.iter().position(|&b| b == 0).unwrap_or(0);
            error.truncate(end);
            return (code as i64, 0, error);
        }
        self.m = m;
        (0, 0, Vec::new())
    }

    /// The ring lent with this process's descriptors written over the host's handles; they close after the call, since
    /// the core duplicates what it keeps (EmuSen_CoreAPI.md §6.43).
    fn ring(&mut self, args: &[u8], fds: Vec<OwnedFd>) -> (i64, i64, Vec<u8>) {
        let Some(f) = self.x.ring_lend else { return (status::NOT_SUPPORTED as i64, 0, Vec::new()) };
        let ring = match wire::read_ring(args) {
            Ok(r) => r,
            Err(why) => return (status::BAD_STRUCT as i64, 0, why.into_bytes()),
        };
        let Some((mut images, device_uuid, driver_uuid)) = ring else { return (unsafe { f(self.m, std::ptr::null()) } as i64, 0, Vec::new()) };
        if fds.len() != images.len() + 2 {
            return (status::BAD_STRUCT as i64, 0, b"the ring's descriptors did not arrive".to_vec());
        }
        for (i, fd) in images.iter_mut().zip(&fds) {
            i.handle = fd.as_raw_fd() as u64;
        }
        let lend = RingLend {
            size: size_of::<RingLend>() as u32,
            count: images.len() as u32,
            images: images.as_ptr(),
            image_size: size_of::<emusen_native::core::sys::RingImage>(),
            ready: fds[images.len()].as_raw_fd() as u64,
            done: fds[images.len() + 1].as_raw_fd() as u64,
            device_uuid,
            driver_uuid,
        };
        (unsafe { f(self.m, &lend) } as i64, 0, Vec::new())
    }

    fn insert(&mut self, r: &mut Reader<'_>) -> (i64, i64, Vec<u8>) {
        let Some(f) = self.x.media_insert else { return (status::NOT_SUPPORTED as i64, 0, Vec::new()) };
        let drive = r.u32();
        let flags = r.u32();
        let files = files(r);
        let sent = r.text().map(<[u8]>::to_vec);
        let path = r.text().map(|s| CString::new(s).unwrap_or_default());
        let empty = r.u32() == 1;
        let (image, image_len) = if empty {
            (std::ptr::null(), 0)
        } else {
            match self.image(sent.as_deref(), path.as_ref().map(|p| p.as_bytes())) {
                Ok(i) => i,
                Err(why) => return (status::BAD_IMAGE as i64, 0, why.into_bytes()),
            }
        };
        let media = Media {
            size: size_of::<Media>() as u32,
            flags,
            image: if empty { std::ptr::null() } else if image.is_null() { b"".as_ptr() } else { image },
            image_len,
            files: files.entries.as_ptr(),
            file_count: files.entries.len(),
            file_size: size_of::<FileEntry>(),
            image_path: path.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
        };
        (unsafe { f(self.m, drive, &media) } as i64, 0, Vec::new())
    }
}

/// Requests read and answered until the caller frees the machine or goes.
pub fn serve(served: &mut Served, io: &mut UnixStream) -> std::io::Result<()> {
    loop {
        let (h, args) = match wire::read_request(io) {
            Ok(r) => r,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        };
        // A ring's descriptors come in the message after its request; a stream out of step is a broken caller.
        let fds = match (h.op == Op::RingLend as u16).then(|| wire::read_ring(&args)) {
            Some(Ok(Some((images, _, _)))) => wire::recv_fds(io, images.len() + 2)?,
            _ => Vec::new(),
        };
        let (ret, aux, bytes) = served.answer(&h, &args, fds);
        wire::send_reply(io, ret, aux, &bytes)?;
        if served.done {
            return Ok(());
        }
    }
}
