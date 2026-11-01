//! The broker's messages: a request is an operation, how much the caller's buffer holds, and its arguments; a reply is
//! the export's return, one word more, and the bytes the export wrote. Lengths first, little-endian, no padding
//! (EmuSen_LibretroHost.md §3.2).

use std::io::{self, Read, Write};

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
}

impl Op {
    pub fn from_u16(v: u16) -> Option<Op> {
        (Op::Info as u16..=Op::Ping as u16).contains(&v).then(|| unsafe { std::mem::transmute::<u16, Op>(v) })
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
    }
}
