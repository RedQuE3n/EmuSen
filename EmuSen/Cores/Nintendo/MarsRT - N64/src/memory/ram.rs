//! RDRAM and its hidden bits, reached by both threads through one raw pointer; an index borrows only its bytes, `Deref` only when joined. See Mars_Native.md §5.6.1.

use std::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo};
use std::ptr::NonNull;

pub struct Ram {
    ptr: NonNull<u8>,
    len: usize,
}

// SAFETY: `Ram` owns its bytes; sharing them across threads is governed by the page marks, whose invariant `dp_threads` documents.
unsafe impl Send for Ram {}
unsafe impl Sync for Ram {}

impl Ram {
    pub fn zeroed(len: usize) -> Ram {
        let bytes: Box<[u8]> = vec![0u8; len].into_boxed_slice();
        let ptr = NonNull::new(Box::into_raw(bytes) as *mut u8).expect("a boxed slice is never null");
        Ram { ptr, len }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The pointer every access derives from; the drain holds a copy of it for the machine's life.
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    #[inline(always)]
    pub fn be32(&self, at: u32) -> u32 {
        let at = at as usize;
        assert!(at + 4 <= self.len);
        // SAFETY: in bounds; the marks keep the drain off these four bytes.
        u32::from_be(unsafe { self.ptr.as_ptr().add(at).cast::<u32>().read_unaligned() })
    }

    #[inline(always)]
    pub fn put_be32(&mut self, at: u32, value: u32) {
        let at = at as usize;
        assert!(at + 4 <= self.len);
        // SAFETY: in bounds; the marks keep the drain off these four bytes.
        unsafe { self.ptr.as_ptr().add(at).cast::<u32>().write_unaligned(value.to_be()) }
    }

    /// An aligned big-endian access of one, two, four or eight bytes, as the processor's direct load reads it.
    #[inline(always)]
    pub fn read(&self, at: u32, size: u32) -> u64 {
        let at = at as usize;
        assert!(at + size as usize <= self.len);
        // SAFETY: in bounds; the marks keep the drain off these bytes.
        unsafe {
            let p = self.ptr.as_ptr().add(at);
            match size {
                1 => p.read() as u64,
                2 => u16::from_be(p.cast::<u16>().read_unaligned()) as u64,
                4 => u32::from_be(p.cast::<u32>().read_unaligned()) as u64,
                _ => u64::from_be(p.cast::<u64>().read_unaligned()),
            }
        }
    }

    #[inline(always)]
    pub fn write(&mut self, at: u32, value: u64, size: u32) {
        let at = at as usize;
        assert!(at + size as usize <= self.len);
        // SAFETY: in bounds; the marks keep the drain off these bytes.
        unsafe {
            let p = self.ptr.as_ptr().add(at);
            match size {
                1 => p.write(value as u8),
                2 => p.cast::<u16>().write_unaligned((value as u16).to_be()),
                4 => p.cast::<u32>().write_unaligned((value as u32).to_be()),
                _ => p.cast::<u64>().write_unaligned(value.to_be()),
            }
        }
    }

    #[inline(always)]
    fn span(&self, start: usize, end: usize) -> (usize, usize) {
        assert!(start <= end && end <= self.len, "range {start}..{end} out of {}", self.len);
        (start, end - start)
    }

    #[inline(always)]
    fn slice(&self, start: usize, end: usize) -> &[u8] {
        let (at, n) = self.span(start, end);
        // SAFETY: in bounds; the reference covers only these bytes.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr().add(at), n) }
    }

    #[inline(always)]
    fn slice_mut(&mut self, start: usize, end: usize) -> &mut [u8] {
        let (at, n) = self.span(start, end);
        // SAFETY: in bounds; the reference covers only these bytes.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr().add(at), n) }
    }
}

impl Drop for Ram {
    fn drop(&mut self) {
        // SAFETY: the pointer and length came from `Box::into_raw` of a boxed slice of this length.
        drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), self.len)) });
    }
}

impl Clone for Ram {
    fn clone(&self) -> Ram {
        let copy = Ram::zeroed(self.len);
        // SAFETY: both are `len` bytes and distinct allocations.
        unsafe { std::ptr::copy_nonoverlapping(self.ptr.as_ptr(), copy.ptr.as_ptr(), self.len) };
        copy
    }
}

impl PartialEq for Ram {
    fn eq(&self, other: &Ram) -> bool {
        **self == **other
    }
}

impl Eq for Ram {}

impl fmt::Debug for Ram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl Deref for Ram {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.slice(0, self.len)
    }
}

impl DerefMut for Ram {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.slice_mut(0, self.len)
    }
}

impl Index<usize> for Ram {
    type Output = u8;
    #[inline(always)]
    fn index(&self, i: usize) -> &u8 {
        assert!(i < self.len, "index {i} out of {}", self.len);
        // SAFETY: in bounds.
        unsafe { &*self.ptr.as_ptr().add(i) }
    }
}

impl IndexMut<usize> for Ram {
    #[inline(always)]
    fn index_mut(&mut self, i: usize) -> &mut u8 {
        assert!(i < self.len, "index {i} out of {}", self.len);
        // SAFETY: in bounds.
        unsafe { &mut *self.ptr.as_ptr().add(i) }
    }
}

macro_rules! ranges {
    ($($range:ty => |$r:ident, $len:ident| $bounds:expr;)*) => {$(
        impl Index<$range> for Ram {
            type Output = [u8];
            #[inline(always)]
            fn index(&self, $r: $range) -> &[u8] {
                let $len = self.len;
                let (start, end) = $bounds;
                self.slice(start, end)
            }
        }

        impl IndexMut<$range> for Ram {
            #[inline(always)]
            fn index_mut(&mut self, $r: $range) -> &mut [u8] {
                let $len = self.len;
                let (start, end) = $bounds;
                self.slice_mut(start, end)
            }
        }
    )*};
}

ranges! {
    Range<usize> => |r, _len| (r.start, r.end);
    RangeFrom<usize> => |r, len| (r.start, len);
    RangeTo<usize> => |r, _len| (0, r.end);
    RangeInclusive<usize> => |r, _len| (*r.start(), *r.end() + 1);
    RangeFull => |_r, len| (0, len);
}

/// A value on the heap reached through its raw pointer, so a `&mut` of its owner never covers it while a drain holds it. See Mars_Native.md §5.6.1.
pub struct Detached<T>(NonNull<T>);

// SAFETY: `Detached` owns its value as a `Box` would; which thread may touch it when is the drain's rule (`dp_threads`).
unsafe impl<T: Send> Send for Detached<T> {}
unsafe impl<T: Sync> Sync for Detached<T> {}

impl<T> Detached<T> {
    pub fn new(value: T) -> Detached<T> {
        Detached(NonNull::new(Box::into_raw(Box::new(value))).expect("a box is never null"))
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }
}

impl<T> Drop for Detached<T> {
    fn drop(&mut self) {
        // SAFETY: the pointer came from `Box::into_raw`.
        drop(unsafe { Box::from_raw(self.0.as_ptr()) });
    }
}

impl<T> Deref for Detached<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        // SAFETY: owned and live; see the type's rule.
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for Detached<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: owned and live; see the type's rule.
        unsafe { self.0.as_mut() }
    }
}

impl<T: Default> Default for Detached<T> {
    fn default() -> Self {
        Detached::new(T::default())
    }
}

impl<T: Clone> Clone for Detached<T> {
    fn clone(&self) -> Self {
        Detached::new((**self).clone())
    }
}

impl<T: PartialEq> PartialEq for Detached<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Eq> Eq for Detached<T> {}

impl<T: fmt::Debug> fmt::Debug for Detached<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ram_reads_back_what_is_written_at_every_width() {
        let mut ram = Ram::zeroed(64);
        ram.write(8, 0x0102_0304_0506_0708, 8);
        assert_eq!(ram.read(8, 8), 0x0102_0304_0506_0708);
        assert_eq!(ram.read(8, 4), 0x0102_0304);
        assert_eq!(ram.read(10, 2), 0x0304);
        assert_eq!(ram.read(15, 1), 0x08);
        assert_eq!(ram.be32(12), 0x0506_0708);
        ram.put_be32(0, 0xDEAD_BEEF);
        assert_eq!(&ram[0..4], &[0xDE, 0xAD, 0xBE, 0xEF]);
        ram[63] = 7;
        assert_eq!(ram[63], 7);
        let copy = ram.clone();
        assert_eq!(copy, ram);
        assert_eq!(copy.len(), 64);
    }

    #[test]
    #[should_panic]
    fn a_ram_refuses_a_read_past_its_end() {
        Ram::zeroed(16).read(13, 4);
    }
}
