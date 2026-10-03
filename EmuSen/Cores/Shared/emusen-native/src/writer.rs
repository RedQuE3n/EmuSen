//! `BinaryWriter` as `StateSerializer` drives it: little-endian, a bool as one byte, arrays with no length.

use std::fmt::Write as _;

use crate::State;

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

    /// `BinaryWriter.Write(double)`: the IEEE 754 bits, little-endian.
    pub fn f64(&mut self, name: &str, v: f64) {
        self.record(name, "f64", None, 8);
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
        self.group_class(name, |w| v.write_state(w));
    }

    /// A class-typed field written by hand: the present flag, then what `f` writes under `name.`.
    pub fn group_class(&mut self, name: &str, f: impl FnOnce(&mut Self)) {
        self.record(name, "class", None, 1);
        self.put(&[1]);
        self.group(name, f);
    }

    /// A struct-typed field: its fields, with no flag.
    pub fn structure<T: State>(&mut self, name: &str, v: &T) {
        self.group(name, |w| v.write_state(w));
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

    /// Fields a hand-written part of the format names under `name.`, as the C# side walks them.
    pub fn group(&mut self, name: &str, f: impl FnOnce(&mut Self)) {
        self.enter(name);
        f(self);
        self.leave();
    }
}
