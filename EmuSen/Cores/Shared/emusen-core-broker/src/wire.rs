//! The broker's messages: a request is an operation, how much the caller's buffer holds, and its arguments; a reply is
//! the export's return, one word more, and the bytes the export wrote. Lengths first, little-endian, no padding
//! (EmuSen_LibretroHost.md §3.2).

use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::net::UnixStream;

use emusen_native::core::sys::RingImage;

/// The operations, one per forwarded export.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Info = 1,
    SettingsSchema,
    FirmwareFor,
    StatusText,
    SetCrashLog,
    LogDrain,
    Create,
    Free,
    MachineInfo,
    LastError,
    Advance,
    SetOptions,
    FrameCount,
    Events,
    FrameInfo,
    FrameCopy,
    AudioRate,
    AudioBuffered,
    AudioDrain,
    SetAudioLimit,
    SetButtons,
    StateSize,
    StateSave,
    StateLoad,
    StateLayout,
    SpaceSize,
    SpaceRead,
    SpaceWrite,
    Battery,
    BatterySaved,
    Reset,
    Phases,
    SetAxis,
    SetSettings,
    SettingNotes,
    MediaLid,
    MediaInsert,
    SetPresent,
    Capabilities,
    Ping,
    /// Since 1.18: the ring's descriptors follow the request in a message of their own (`SCM_RIGHTS`).
    RingLend,
    FrameRing,
}

impl Op {
    pub fn from_u16(v: u16) -> Option<Op> {
        (Op::Info as u16..=Op::FrameRing as u16).contains(&v).then(|| unsafe { std::mem::transmute::<u16, Op>(v) })
    }
}

/// The most one message may carry: a state or an image of a hosted core's, with room.
pub const MAX_MESSAGE: usize = 1 << 30;

/// Arguments written in order.
#[derive(Default)]
pub struct Writer(pub Vec<u8>);

impl Writer {
    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn i64(&mut self, v: i64) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn f64(&mut self, v: f64) -> &mut Self {
        self.0.extend_from_slice(&v.to_bits().to_le_bytes());
        self
    }
    pub fn bytes(&mut self, b: &[u8]) -> &mut Self {
        self.u64(b.len() as u64);
        self.0.extend_from_slice(b);
        self
    }
    /// Text that may be absent: a flag, then the bytes.
    pub fn text(&mut self, t: Option<&[u8]>) -> &mut Self {
        match t {
            Some(t) => self.u32(1).bytes(t),
            None => self.u32(0),
        }
    }
}

/// Arguments read in order; a short message reads as zeros and is refused by the caller's checks.
pub struct Reader<'a> {
    b: &'a [u8],
    at: usize,
    pub short: bool,
}

impl<'a> Reader<'a> {
    pub fn new(b: &'a [u8]) -> Reader<'a> {
        Reader { b, at: 0, short: false }
    }
    fn take(&mut self, n: usize) -> &'a [u8] {
        match self.b.get(self.at..self.at.saturating_add(n)) {
            Some(s) => {
                self.at += n;
                s
            }
            None => {
                self.short = true;
                &[]
            }
        }
    }
    fn word<const N: usize>(&mut self) -> [u8; N] {
        self.take(N).try_into().unwrap_or([0; N])
    }
    pub fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.word())
    }
    pub fn u64(&mut self) -> u64 {
        u64::from_le_bytes(self.word())
    }
    pub fn i64(&mut self) -> i64 {
        i64::from_le_bytes(self.word())
    }
    pub fn f64(&mut self) -> f64 {
        f64::from_bits(self.u64())
    }
    pub fn bytes(&mut self) -> &'a [u8] {
        let n = self.u64() as usize;
        self.take(n)
    }
    pub fn text(&mut self) -> Option<&'a [u8]> {
        (self.u32() == 1).then(|| self.bytes())
    }
}

/// A request's head: the operation, whether the caller's buffer was null, and its length.
pub struct Head {
    pub op: u16,
    pub out_null: bool,
    pub out_len: u64,
}

pub fn send_request(w: &mut impl Write, op: Op, out_null: bool, out_len: u64, args: &[u8]) -> io::Result<()> {
    let mut m = Vec::with_capacity(16 + args.len());
    m.extend_from_slice(&((12 + args.len()) as u32).to_le_bytes());
    m.extend_from_slice(&(op as u16).to_le_bytes());
    m.extend_from_slice(&(out_null as u16).to_le_bytes());
    m.extend_from_slice(&out_len.to_le_bytes());
    m.extend_from_slice(args);
    w.write_all(&m)
}

fn read_message(r: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len)?;
    let n = u32::from_le_bytes(len) as usize;
    if n > MAX_MESSAGE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "a message longer than the broker carries"));
    }
    let mut m = vec![0u8; n];
    r.read_exact(&mut m)?;
    Ok(m)
}

pub fn read_request(r: &mut impl Read) -> io::Result<(Head, Vec<u8>)> {
    let m = read_message(r)?;
    if m.len() < 12 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "a request shorter than its head"));
    }
    let head = Head { op: u16::from_le_bytes([m[0], m[1]]), out_null: m[2] != 0, out_len: u64::from_le_bytes(m[4..12].try_into().unwrap()) };
    Ok((head, m[12..].to_vec()))
}

pub fn send_reply(w: &mut impl Write, ret: i64, aux: i64, bytes: &[u8]) -> io::Result<()> {
    let mut m = Vec::with_capacity(20 + bytes.len());
    m.extend_from_slice(&((16 + bytes.len()) as u32).to_le_bytes());
    m.extend_from_slice(&ret.to_le_bytes());
    m.extend_from_slice(&aux.to_le_bytes());
    m.extend_from_slice(bytes);
    w.write_all(&m)
}

/// A reply: the export's return, the word beside it, the bytes it wrote.
pub struct Reply {
    pub ret: i64,
    pub aux: i64,
    pub bytes: Vec<u8>,
}

pub fn read_reply(r: &mut impl Read) -> io::Result<Reply> {
    let m = read_message(r)?;
    if m.len() < 16 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "a reply shorter than its head"));
    }
    Ok(Reply { ret: i64::from_le_bytes(m[..8].try_into().unwrap()), aux: i64::from_le_bytes(m[8..16].try_into().unwrap()), bytes: m[16..].to_vec() })
}

/// The most descriptors one ring carries: its images and its two semaphores.
pub const MAX_RING_FDS: usize = 16;

/// A one-byte message over `control`, the frame `send_fds` and `recv_fds` share.
fn with_fd_message<T>(control: &mut [u8], f: impl FnOnce(&mut libc::msghdr) -> T) -> T {
    let mut byte = [0u8; 1];
    let mut iov = libc::iovec { iov_base: byte.as_mut_ptr() as *mut libc::c_void, iov_len: 1 };
    let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = control.as_mut_ptr() as *mut libc::c_void;
    msg.msg_controllen = control.len() as _;
    f(&mut msg)
}

/// Descriptors sent by `SCM_RIGHTS` in a one-byte message of their own, after the request they belong to.
pub fn send_fds(sock: &UnixStream, fds: &[RawFd]) -> io::Result<()> {
    if fds.is_empty() || fds.len() > MAX_RING_FDS {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "a descriptor message of none or too many"));
    }
    let bytes = std::mem::size_of_val(fds) as u32;
    let mut control = vec![0u8; unsafe { libc::CMSG_SPACE(bytes) } as usize];
    let sent = with_fd_message(&mut control, |msg| unsafe {
        let c = libc::CMSG_FIRSTHDR(msg);
        (*c).cmsg_level = libc::SOL_SOCKET;
        (*c).cmsg_type = libc::SCM_RIGHTS;
        (*c).cmsg_len = libc::CMSG_LEN(bytes) as _;
        std::ptr::copy_nonoverlapping(fds.as_ptr() as *const u8, libc::CMSG_DATA(c), bytes as usize);
        libc::sendmsg(sock.as_raw_fd(), msg, libc::MSG_NOSIGNAL)
    });
    if sent != 1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// The descriptors of the message `send_fds` sent, exactly `count`, owned by the receiver; any other count is refused
/// and whatever arrived is closed.
pub fn recv_fds(sock: &UnixStream, count: usize) -> io::Result<Vec<OwnedFd>> {
    let mut control = vec![0u8; unsafe { libc::CMSG_SPACE((MAX_RING_FDS * 4) as u32) } as usize];
    let (n, got, flags) = with_fd_message(&mut control, |msg| unsafe {
        let n = libc::recvmsg(sock.as_raw_fd(), msg, libc::MSG_CMSG_CLOEXEC);
        let mut got = Vec::new();
        if n == 1 {
            let mut c = libc::CMSG_FIRSTHDR(msg);
            while !c.is_null() {
                if (*c).cmsg_level == libc::SOL_SOCKET && (*c).cmsg_type == libc::SCM_RIGHTS {
                    let k = ((*c).cmsg_len as usize - libc::CMSG_LEN(0) as usize) / 4;
                    let data = libc::CMSG_DATA(c) as *const RawFd;
                    got.extend((0..k).map(|i| OwnedFd::from_raw_fd(std::ptr::read_unaligned(data.add(i)))));
                }
                c = libc::CMSG_NXTHDR(msg, c);
            }
        }
        (n, got, msg.msg_flags)
    });
    if n != 1 {
        return Err(if n < 0 { io::Error::last_os_error() } else { io::Error::new(io::ErrorKind::UnexpectedEof, "no descriptor message") });
    }
    if flags & libc::MSG_CTRUNC != 0 || got.len() != count {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("{} descriptors where {count} were due", got.len())));
    }
    Ok(got)
}

/// A ring's request: present, its images but their handles, and the two UUIDs; the handles travel as descriptors in
/// the order images, `ready`, `done` (EmuSen_LibretroHost.md §5.4a).
pub fn ring_args(images: &[RingImage], device_uuid: &[u8; 16], driver_uuid: &[u8; 16]) -> Vec<u8> {
    let mut w = Writer::default();
    w.u32(1).u32(images.len() as u32);
    for i in images {
        w.u32(i.memory_type).u64(i.allocation_size).u32(i.width).u32(i.height).u32(i.format).u32(i.usage);
    }
    w.bytes(device_uuid).bytes(driver_uuid);
    w.0
}

/// A ring's request read back: its images with `handle` zero, and the UUIDs; `None` for a withdrawal.
pub fn read_ring(args: &[u8]) -> Result<Option<(Vec<RingImage>, [u8; 16], [u8; 16])>, String> {
    let mut r = Reader::new(args);
    if r.u32() != 1 {
        return Ok(None);
    }
    let count = r.u32() as usize;
    if count + 2 > MAX_RING_FDS {
        return Err(format!("a ring of {count} images, more than the broker carries"));
    }
    let images = (0..count)
        .map(|_| RingImage { size: size_of::<RingImage>() as u32, memory_type: r.u32(), handle: 0, allocation_size: r.u64(), width: r.u32(), height: r.u32(), format: r.u32(), usage: r.u32() })
        .collect();
    let (dev, drv) = (r.bytes().try_into().unwrap_or([0; 16]), r.bytes().try_into().unwrap_or([0; 16]));
    if r.short {
        return Err("the ring's request was short".into());
    }
    Ok(Some((images, dev, drv)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_and_a_reply_cross_whole() {
        let mut w = Writer::default();
        w.u32(7).f64(-0.5).bytes(b"abc").text(None).text(Some(b"x"));
        let mut pipe = Vec::new();
        send_request(&mut pipe, Op::SetAxis, true, 99, &w.0).unwrap();
        let (h, args) = read_request(&mut &pipe[..]).unwrap();
        assert_eq!((h.op, h.out_null, h.out_len), (Op::SetAxis as u16, true, 99));
        let mut r = Reader::new(&args);
        assert_eq!((r.u32(), r.f64(), r.bytes(), r.text(), r.text()), (7, -0.5, &b"abc"[..], None, Some(&b"x"[..])));
        assert!(!r.short);
        let mut pipe = Vec::new();
        send_reply(&mut pipe, -3, 4, b"out").unwrap();
        let rep = read_reply(&mut &pipe[..]).unwrap();
        assert_eq!((rep.ret, rep.aux, rep.bytes.as_slice()), (-3, 4, &b"out"[..]));
        assert_eq!(Op::from_u16(Op::Ping as u16), Some(Op::Ping));
        assert_eq!(Op::from_u16(0), None);
        assert_eq!(Op::from_u16(Op::FrameRing as u16), Some(Op::FrameRing));
    }

    fn memfd(text: &[u8]) -> OwnedFd {
        let fd = unsafe { OwnedFd::from_raw_fd(libc::memfd_create(c"ring".as_ptr(), libc::MFD_CLOEXEC)) };
        assert_eq!(unsafe { libc::write(fd.as_raw_fd(), text.as_ptr() as *const libc::c_void, text.len()) }, text.len() as isize);
        fd
    }

    fn inode(fd: RawFd) -> u64 {
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        assert_eq!(unsafe { libc::fstat(fd, &mut st) }, 0);
        st.st_ino
    }

    #[test]
    fn a_rings_descriptors_follow_its_request_and_name_the_same_objects() {
        let (a, mut b) = UnixStream::pair().unwrap();
        let images = [RingImage { size: 40, memory_type: 3, handle: 0, allocation_size: 1 << 20, width: 640, height: 512, format: 37, usage: 6 }; 3];
        let fds: Vec<OwnedFd> = (0..5).map(|i| memfd(&[i as u8; 3])).collect();
        let raw: Vec<RawFd> = fds.iter().map(|f| f.as_raw_fd()).collect();
        send_request(&mut &a, Op::RingLend, false, 0, &ring_args(&images, &[1; 16], &[2; 16])).unwrap();
        send_fds(&a, &raw).unwrap();
        send_request(&mut &a, Op::Ping, false, 0, &[]).unwrap();
        let (h, args) = read_request(&mut b).unwrap();
        assert_eq!(h.op, Op::RingLend as u16);
        let (back, dev, drv) = read_ring(&args).unwrap().unwrap();
        assert_eq!((back.as_slice(), dev, drv), (&images[..], [1; 16], [2; 16]));
        let got = recv_fds(&b, back.len() + 2).unwrap();
        for (mine, theirs) in raw.iter().zip(&got) {
            assert_ne!(*mine, theirs.as_raw_fd());
            assert_eq!(inode(*mine), inode(theirs.as_raw_fd()));
        }
        assert_eq!(read_request(&mut b).unwrap().0.op, Op::Ping as u16, "the stream stays in step after the descriptors");
        assert_eq!(read_ring(&[0, 0, 0, 0]).unwrap(), None);
    }

    #[test]
    fn a_descriptor_count_other_than_the_requests_is_refused() {
        let (a, b) = UnixStream::pair().unwrap();
        let fds: Vec<OwnedFd> = (0..2).map(|_| memfd(b"x")).collect();
        send_fds(&a, &fds.iter().map(|f| f.as_raw_fd()).collect::<Vec<_>>()).unwrap();
        assert!(recv_fds(&b, 5).is_err());
        assert!(send_fds(&a, &[]).is_err());
    }
}
