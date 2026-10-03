//! `BinaryReader` as `StateSerializer` drives it, with its readings of bytes no C# writer makes.

use crate::{State, StringError, Truncated};

/// Reads the bytes a `StateWriter` or the C# serializer wrote.
pub struct StateReader<'a> {
    data: &'a [u8],
    pos: usize,
    version: i32,
}

impl<'a> StateReader<'a> {
    /// A reader for the current version until the header names another.
    pub fn new(data: &'a [u8]) -> Self {
        StateReader { data, pos: 0, version: i32::MAX }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    /// The version the header names, so each part of the format reads the fields that version carried.
    pub fn set_version(&mut self, version: i32) {
        self.version = version;
    }

    /// True for a state older than `version`, whose retired fields are read and dropped.
    pub fn before(&self, version: i32) -> bool {
        self.version < version
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], Truncated> {
        let slice = self.data.get(self.pos..self.pos + n).ok_or(Truncated { at: self.pos, wanted: n })?;
        self.pos += n;
        Ok(slice)
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], Truncated> {
        Ok(self.take(N)?.try_into().expect("take returned N bytes"))
    }

    pub fn skip(&mut self, n: usize) -> Result<(), Truncated> {
        self.take(n).map(|_| ())
    }

    pub fn u8(&mut self) -> Result<u8, Truncated> {
        Ok(self.fixed::<1>()?[0])
    }

    pub fn i8(&mut self) -> Result<i8, Truncated> {
        Ok(i8::from_le_bytes(self.fixed()?))
    }

    /// `BinaryReader.ReadBoolean`: any nonzero byte is true.
    pub fn bool(&mut self) -> Result<bool, Truncated> {
        Ok(self.u8()? != 0)
    }

    pub fn i16(&mut self) -> Result<i16, Truncated> {
        Ok(i16::from_le_bytes(self.fixed()?))
    }

    pub fn u16(&mut self) -> Result<u16, Truncated> {
        Ok(u16::from_le_bytes(self.fixed()?))
    }

    pub fn i32(&mut self) -> Result<i32, Truncated> {
        Ok(i32::from_le_bytes(self.fixed()?))
    }

    pub fn u32(&mut self) -> Result<u32, Truncated> {
        Ok(u32::from_le_bytes(self.fixed()?))
    }

    pub fn i64(&mut self) -> Result<i64, Truncated> {
        Ok(i64::from_le_bytes(self.fixed()?))
    }

    pub fn u64(&mut self) -> Result<u64, Truncated> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }

    pub fn f64(&mut self) -> Result<f64, Truncated> {
        Ok(f64::from_le_bytes(self.fixed()?))
    }

    pub fn bytes(&mut self, into: &mut [u8]) -> Result<(), Truncated> {
        into.copy_from_slice(self.take(into.len())?);
        Ok(())
    }

    /// Each byte as `ReadBoolean` reads it.
    pub fn bools(&mut self, into: &mut [bool]) -> Result<(), Truncated> {
        let src = self.take(into.len())?;
        for (dst, &b) in into.iter_mut().zip(src) {
            *dst = b != 0;
        }
        Ok(())
    }

    /// `BinaryReader.ReadString`: at most five length bytes, a negative count refused, invalid UTF-8 replaced as .NET replaces it.
    pub fn string(&mut self) -> Result<String, StringError> {
        let at = self.pos;
        let mut count: u32 = 0;
        let mut shift = 0;
        loop {
            let b = self.u8()?;
            if shift == 28 && b > 0x0F {
                return Err(StringError::BadLength { at });
            }
            count |= ((b & 0x7F) as u32) << shift;
            if b & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        if (count as i32) < 0 {
            return Err(StringError::BadLength { at });
        }
        Ok(String::from_utf8_lossy(self.take(count as usize)?).into_owned())
    }

    pub fn u16s(&mut self, into: &mut [u16]) -> Result<(), Truncated> {
        self.array(into, u16::from_le_bytes)
    }

    pub fn i32s(&mut self, into: &mut [i32]) -> Result<(), Truncated> {
        self.array(into, i32::from_le_bytes)
    }

    pub fn u32s(&mut self, into: &mut [u32]) -> Result<(), Truncated> {
        self.array(into, u32::from_le_bytes)
    }

    pub fn u64s(&mut self, into: &mut [u64]) -> Result<(), Truncated> {
        self.array(into, u64::from_le_bytes)
    }

    fn array<T, const N: usize>(&mut self, into: &mut [T], le: fn([u8; N]) -> T) -> Result<(), Truncated> {
        let src = self.take(into.len() * N)?;
        for (dst, &chunk) in into.iter_mut().zip(src.as_chunks::<N>().0) {
            *dst = le(chunk);
        }
        Ok(())
    }

    /// A class-typed field: C# reads its fields only when the flag is set, and otherwise leaves the object as it was.
    pub fn class<T: State>(&mut self, into: &mut T) -> Result<(), T::Error> {
        if self.present()? {
            into.read_state(self)?;
        }
        Ok(())
    }

    /// The present flag of a class read by hand; false leaves the object as it was.
    pub fn present(&mut self) -> Result<bool, Truncated> {
        self.bool()
    }

    /// An array of structs or classes: each element's fields, with no flags.
    pub fn structures<T: State>(&mut self, into: &mut [T]) -> Result<(), T::Error> {
        for item in into {
            item.read_state(self)?;
        }
        Ok(())
    }
}
