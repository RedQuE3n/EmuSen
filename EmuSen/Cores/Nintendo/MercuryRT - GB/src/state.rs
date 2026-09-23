//! The C# `StateSerializer`'s encodings: little-endian, a bool as one byte, arrays with no length, `BinaryWriter` strings. MarsRT's `state.rs` plus strings; see Mercury_Native.md §3.1.

use std::fmt::Write as _;

/// Why a state could not be read or written; each has a status code for the C ABI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    NotAMercuryState(u32),
    Version(i32),
    BadStringLength { at: usize },
    BufferTooSmall { needed: usize },
}

impl StateError {
    pub fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => -2,
            StateError::NotAMercuryState(_) => -3,
            StateError::Version(_) => -4,
            StateError::BadStringLength { .. } => -5,
            StateError::BufferTooSmall { .. } => -7,
        }
    }
}

pub type StateResult<T = ()> = Result<T, StateError>;

/// One C# class or struct: its serialized fields, written and read in the C# ordinal order.
pub trait State {
    fn write_state(&self, w: &mut StateWriter);
    fn read_state(&mut self, r: &mut StateReader) -> StateResult;
}

/// Writes into a caller's buffer, or only counts; with a layout, also records each field's C# path, type and offset.
pub struct StateWriter<'a> {
    out: Option<&'a mut [u8]>,
    len: usize,
    overflow: bool,
    layout: Option<Layout>,
}

struct Layout {
    path: String,
    marks: Vec<usize>,
    text: String,
}

impl StateWriter<'static> {
    /// Counts the bytes a state would take, writing nothing.
    pub fn counter() -> Self {
        StateWriter { out: None, len: 0, overflow: false, layout: None }
    }

    /// Counts, and records the layout: one line per field, `offset length type path`.
    pub fn layout() -> Self {
        StateWriter {
            out: None,
            len: 0,
            overflow: false,
            layout: Some(Layout { path: String::new(), marks: Vec::new(), text: String::new() }),
        }
    }
}

impl<'a> StateWriter<'a> {
    pub fn new(out: &'a mut [u8]) -> Self {
        StateWriter { out: Some(out), len: 0, overflow: false, layout: None }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// True when the buffer was shorter than what was written; `len` is then what it needed.
    pub fn overflowed(&self) -> bool {
        self.overflow
    }

    pub fn into_layout(self) -> String {
        self.layout.map(|l| l.text).unwrap_or_default()
    }

    fn record(&mut self, name: &str, ty: &str, count: Option<usize>, bytes: usize) {
        if let Some(layout) = &mut self.layout {
            let _ = write!(layout.text, "{} {} {}", self.len, bytes, ty);
            if let Some(n) = count {
                let _ = write!(layout.text, "[{n}]");
            }
            let _ = writeln!(layout.text, " {}{}", layout.path, name);
        }
    }

    fn put(&mut self, bytes: &[u8]) {
        if let Some(out) = &mut self.out {
            match out.get_mut(self.len..self.len + bytes.len()) {
                Some(dst) => dst.copy_from_slice(bytes),
                None => self.overflow = true,
            }
        }
        self.len += bytes.len();
    }

    /// Children of a class, a struct or an array element are named under `name.`.
    fn enter(&mut self, name: &str) {
        if let Some(layout) = &mut self.layout {
            layout.marks.push(layout.path.len());
            layout.path.push_str(name);
            layout.path.push('.');
        }
    }

    fn leave(&mut self) {
        if let Some(layout) = &mut self.layout
            && let Some(mark) = layout.marks.pop()
        {
            layout.path.truncate(mark);
        }
    }

    pub fn u8(&mut self, name: &str, v: u8) {
        self.record(name, "u8", None, 1);
        self.put(&[v]);
    }

    pub fn i8(&mut self, name: &str, v: i8) {
        self.record(name, "i8", None, 1);
        self.put(&v.to_le_bytes());
    }

    pub fn bool(&mut self, name: &str, v: bool) {
        self.record(name, "bool", None, 1);
        self.put(&[v as u8]);
    }

    pub fn i16(&mut self, name: &str, v: i16) {
        self.record(name, "i16", None, 2);
        self.put(&v.to_le_bytes());
    }

    pub fn u16(&mut self, name: &str, v: u16) {
        self.record(name, "u16", None, 2);
        self.put(&v.to_le_bytes());
    }

    pub fn i32(&mut self, name: &str, v: i32) {
        self.record(name, "i32", None, 4);
        self.put(&v.to_le_bytes());
    }

    pub fn u32(&mut self, name: &str, v: u32) {
        self.record(name, "u32", None, 4);
        self.put(&v.to_le_bytes());
    }

    pub fn i64(&mut self, name: &str, v: i64) {
        self.record(name, "i64", None, 8);
        self.put(&v.to_le_bytes());
    }

    pub fn u64(&mut self, name: &str, v: u64) {
        self.record(name, "u64", None, 8);
        self.put(&v.to_le_bytes());
    }

    pub fn bytes(&mut self, name: &str, v: &[u8]) {
        self.record(name, "u8", Some(v.len()), v.len());
        self.put(v);
    }

    pub fn bools(&mut self, name: &str, v: &[bool]) {
        self.record(name, "bool", Some(v.len()), v.len());
        if self.out.is_none() {
            self.len += v.len();
            return;
        }
        for &b in v {
            self.put(&[b as u8]);
        }
    }

    /// `BinaryWriter.Write(string)`: a 7-bit-encoded byte count, then UTF-8; C#'s null is written as "".
    pub fn string(&mut self, name: &str, v: Option<&str>) {
        let text = v.unwrap_or("").as_bytes();
        let mut prefix = [0u8; 5];
        let mut n = 0;
        let mut count = text.len() as u32;
        while count > 0x7F {
            prefix[n] = (count as u8) | 0x80;
            count >>= 7;
            n += 1;
        }
        prefix[n] = count as u8;
        n += 1;
        self.record(name, "string", None, n + text.len());
        self.put(&prefix[..n]);
        self.put(text);
    }

    pub fn u16s(&mut self, name: &str, v: &[u16]) {
        self.array(name, "u16", v, u16::to_le_bytes);
    }

    pub fn i32s(&mut self, name: &str, v: &[i32]) {
        self.array(name, "i32", v, i32::to_le_bytes);
    }

    pub fn u32s(&mut self, name: &str, v: &[u32]) {
        self.array(name, "u32", v, u32::to_le_bytes);
    }

    pub fn u64s(&mut self, name: &str, v: &[u64]) {
        self.array(name, "u64", v, u64::to_le_bytes);
    }

    fn array<T: Copy, const N: usize>(&mut self, name: &str, ty: &str, v: &[T], le: fn(T) -> [u8; N]) {
        self.record(name, ty, Some(v.len()), v.len() * N);
        if self.out.is_none() {
            self.len += v.len() * N;
            return;
        }
        for &x in v {
            self.put(&le(x));
        }
    }

    /// A class-typed field: the present flag, then its fields.
    pub fn class<T: State>(&mut self, name: &str, v: &T) {
        self.record(name, "class", None, 1);
        self.put(&[1]);
        self.enter(name);
        v.write_state(self);
        self.leave();
    }

    /// A struct-typed field: its fields, with no flag.
    pub fn structure<T: State>(&mut self, name: &str, v: &T) {
        self.enter(name);
        v.write_state(self);
        self.leave();
    }

    /// An array of structs or classes: each element's fields, with no flags.
    pub fn structures<T: State>(&mut self, name: &str, v: &[T]) {
        for (i, item) in v.iter().enumerate() {
            if self.layout.is_some() {
                self.enter(&format!("{name}[{i}]"));
            }
            item.write_state(self);
            if self.layout.is_some() {
                self.leave();
            }
        }
    }

    /// A class-typed field written by hand: the present flag, then what `f` writes under `name.`.
    pub fn group_class(&mut self, name: &str, f: impl FnOnce(&mut Self)) {
        self.record(name, "class", None, 1);
        self.put(&[1]);
        self.enter(name);
        f(self);
        self.leave();
    }

    /// Fields a hand-written part of the format names under `name.`, as the C# side walks them.
    pub fn group(&mut self, name: &str, f: impl FnOnce(&mut Self)) {
        self.enter(name);
        f(self);
        self.leave();
    }
}

/// Reads the bytes a `StateWriter` or the C# serializer wrote.
pub struct StateReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> StateReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        StateReader { data, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    fn take(&mut self, n: usize) -> StateResult<&'a [u8]> {
        let slice = self.data.get(self.pos..self.pos + n).ok_or(StateError::Truncated { at: self.pos, wanted: n })?;
        self.pos += n;
        Ok(slice)
    }

    fn fixed<const N: usize>(&mut self) -> StateResult<[u8; N]> {
        Ok(self.take(N)?.try_into().expect("take returned N bytes"))
    }

    pub fn skip(&mut self, n: usize) -> StateResult {
        self.take(n).map(|_| ())
    }

    pub fn u8(&mut self) -> StateResult<u8> {
        Ok(self.fixed::<1>()?[0])
    }

    pub fn i8(&mut self) -> StateResult<i8> {
        Ok(i8::from_le_bytes(self.fixed()?))
    }

    /// `BinaryReader.ReadBoolean`: any nonzero byte is true.
    pub fn bool(&mut self) -> StateResult<bool> {
        Ok(self.u8()? != 0)
    }

    pub fn i16(&mut self) -> StateResult<i16> {
        Ok(i16::from_le_bytes(self.fixed()?))
    }

    pub fn u16(&mut self) -> StateResult<u16> {
        Ok(u16::from_le_bytes(self.fixed()?))
    }

    pub fn i32(&mut self) -> StateResult<i32> {
        Ok(i32::from_le_bytes(self.fixed()?))
    }

    pub fn u32(&mut self) -> StateResult<u32> {
        Ok(u32::from_le_bytes(self.fixed()?))
    }

    pub fn i64(&mut self) -> StateResult<i64> {
        Ok(i64::from_le_bytes(self.fixed()?))
    }

    pub fn u64(&mut self) -> StateResult<u64> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }

    pub fn bytes(&mut self, into: &mut [u8]) -> StateResult {
        into.copy_from_slice(self.take(into.len())?);
        Ok(())
    }

    pub fn bools(&mut self, into: &mut [bool]) -> StateResult {
        let src = self.take(into.len())?;
        for (dst, &b) in into.iter_mut().zip(src) {
            *dst = b != 0;
        }
        Ok(())
    }

    /// `BinaryReader.ReadString`: at most five length bytes, a negative count refused, invalid UTF-8 replaced as .NET replaces it.
    pub fn string(&mut self) -> StateResult<String> {
        let at = self.pos;
        let mut count: u32 = 0;
        let mut shift = 0;
        loop {
            let b = self.u8()?;
            if shift == 28 && b > 0x0F {
                return Err(StateError::BadStringLength { at });
            }
            count |= ((b & 0x7F) as u32) << shift;
            if b & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        if (count as i32) < 0 {
            return Err(StateError::BadStringLength { at });
        }
        Ok(String::from_utf8_lossy(self.take(count as usize)?).into_owned())
    }

    pub fn u16s(&mut self, into: &mut [u16]) -> StateResult {
        self.array(into, u16::from_le_bytes)
    }

    pub fn i32s(&mut self, into: &mut [i32]) -> StateResult {
        self.array(into, i32::from_le_bytes)
    }

    pub fn u32s(&mut self, into: &mut [u32]) -> StateResult {
        self.array(into, u32::from_le_bytes)
    }

    pub fn u64s(&mut self, into: &mut [u64]) -> StateResult {
        self.array(into, u64::from_le_bytes)
    }

    fn array<T, const N: usize>(&mut self, into: &mut [T], le: fn([u8; N]) -> T) -> StateResult {
        let src = self.take(into.len() * N)?;
        for (dst, &chunk) in into.iter_mut().zip(src.as_chunks::<N>().0) {
            *dst = le(chunk);
        }
        Ok(())
    }

    /// A class-typed field: C# reads its fields only when the flag is set, and otherwise leaves the object as it was.
    pub fn class<T: State>(&mut self, into: &mut T) -> StateResult {
        if self.bool()? {
            into.read_state(self)?;
        }
        Ok(())
    }

    pub fn structures<T: State>(&mut self, into: &mut [T]) -> StateResult {
        for item in into {
            item.read_state(self)?;
        }
        Ok(())
    }
}

/// A zeroed array on the heap, too large to build on the stack.
pub fn boxed<T: Copy, const N: usize>(value: T) -> Box<[T; N]> {
    vec![value; N].into_boxed_slice().try_into().unwrap_or_else(|_| unreachable!())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives_are_little_endian_and_bools_one_byte() {
        let mut buf = [0u8; 16];
        let mut w = StateWriter::new(&mut buf);
        w.u32("a", 0x1122_3344);
        w.bool("b", true);
        w.i16("c", -2);
        w.u64("d", 0x0102_0304_0506_0708);
        assert_eq!(w.len(), 15);
        assert!(!w.overflowed());
        assert_eq!(&buf[..15], &[0x44, 0x33, 0x22, 0x11, 1, 0xFE, 0xFF, 8, 7, 6, 5, 4, 3, 2, 1]);

        let mut r = StateReader::new(&buf);
        assert_eq!(r.u32(), Ok(0x1122_3344));
        assert_eq!(r.bool(), Ok(true));
        assert_eq!(r.i16(), Ok(-2));
        assert_eq!(r.u64(), Ok(0x0102_0304_0506_0708));
    }

    #[test]
    fn strings_carry_a_seven_bit_length_as_binarywriter_writes_it() {
        for (text, prefix) in [("", vec![0u8]), ("a", vec![1]), (&"x".repeat(127)[..], vec![0x7F]), (&"x".repeat(128)[..], vec![0x80, 0x01]), (&"y".repeat(16384)[..], vec![0x80, 0x80, 0x01])] {
            let mut buf = vec![0u8; text.len() + 5];
            let mut w = StateWriter::new(&mut buf);
            w.string("s", Some(text));
            let n = w.len();
            assert_eq!(n, prefix.len() + text.len());
            assert_eq!(&buf[..prefix.len()], &prefix[..]);
            assert_eq!(StateReader::new(&buf[..n]).string().as_deref(), Ok(text));
        }
        let mut buf = [0u8; 8];
        let mut w = StateWriter::new(&mut buf);
        w.string("s", None);
        assert_eq!(w.len(), 1);
        assert_eq!(buf[0], 0);
    }

    #[test]
    fn a_string_length_binaryreader_refuses_is_refused() {
        assert_eq!(StateReader::new(&[0x80, 0x80, 0x80, 0x80, 0x10]).string(), Err(StateError::BadStringLength { at: 0 }));
        assert_eq!(StateReader::new(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]).string(), Err(StateError::BadStringLength { at: 0 }));
        assert_eq!(StateReader::new(&[0x05, b'a']).string(), Err(StateError::Truncated { at: 1, wanted: 5 }));
        assert_eq!(StateReader::new(&[0x02, 0xC3, 0x28]).string().as_deref(), Ok("\u{FFFD}("));
    }

    #[test]
    fn a_short_buffer_overflows_and_a_short_input_is_truncated() {
        let mut buf = [0u8; 3];
        let mut w = StateWriter::new(&mut buf);
        w.u32("a", 1);
        assert!(w.overflowed());
        assert_eq!(w.len(), 4);
        assert_eq!(StateReader::new(&buf).u32(), Err(StateError::Truncated { at: 0, wanted: 4 }));
    }
}
