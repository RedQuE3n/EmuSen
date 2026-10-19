//! A state read and written field by field through its `state_layout` listing, with no machine behind it; the C# host's
//! `StateLayout` (EmuSen_NativeCores.md §3.14.1, Mercury_Native.md §14.5).

use std::collections::HashMap;

/// One line of the listing: `<offset> <length> <type> <path>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub offset: usize,
    pub length: usize,
    pub kind: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct StateLayout {
    pub name: String,
    pub text: String,
    pub fields: Vec<Field>,
    by_path: HashMap<String, usize>,
}

fn le(b: &[u8]) -> u64 {
    b.iter().rev().fold(0u64, |v, &x| (v << 8) | x as u64)
}

impl StateLayout {
    /// The listing `text`, or the first line that is not a field.
    pub fn parse(name: &str, text: &str) -> Result<StateLayout, String> {
        let mut fields = Vec::new();
        for line in text.split('\n').filter(|l| !l.is_empty()) {
            let mut p = line.splitn(4, ' ');
            let (Some(o), Some(l), Some(k), Some(path)) = (p.next(), p.next(), p.next(), p.next()) else { return Err(format!("{name}: {line:?} is not a field")) };
            let (Ok(offset), Ok(length)) = (o.parse(), l.parse()) else { return Err(format!("{name}: {line:?} is not a field")) };
            fields.push(Field { offset, length, kind: k.to_owned(), path: path.to_owned() });
        }
        let by_path = fields.iter().enumerate().map(|(i, f)| (f.path.clone(), i)).collect();
        Ok(StateLayout { name: name.to_owned(), text: text.to_owned(), fields, by_path })
    }

    /// The state's length the listing describes: the end of its last field.
    pub fn length(&self) -> usize {
        self.fields.last().map_or(0, |f| f.offset + f.length)
    }

    pub fn has(&self, path: &str) -> bool {
        self.by_path.contains_key(path)
    }

    pub fn field(&self, path: &str) -> Result<&Field, String> {
        self.by_path.get(path).map(|&i| &self.fields[i]).ok_or_else(|| format!("{} has no field {path}", self.name))
    }

    /// A scalar as a signed integer: bools 0 or 1, enums their number, doubles their bits.
    pub fn value(state: &[u8], f: &Field) -> Result<i64, String> {
        let b = state.get(f.offset..f.offset + f.length).ok_or_else(|| format!("{} lies past the state's end", f.path))?;
        Ok(match f.kind.as_str() {
            "u8" | "class" => b[0] as i64,
            "i8" => b[0] as i8 as i64,
            "bool" => (b[0] != 0) as i64,
            "u16" => le(&b[..2]) as i64,
            "i16" => le(&b[..2]) as u16 as i16 as i64,
            "u32" => le(&b[..4]) as i64,
            "i32" => le(&b[..4]) as u32 as i32 as i64,
            "i64" | "u64" | "f64" => le(&b[..8]) as i64,
            other => return Err(format!("{} is a {other}, not a scalar", f.path)),
        })
    }

    pub fn get(&self, state: &[u8], path: &str) -> Result<i64, String> {
        Self::value(state, self.field(path)?)
    }

    /// An array field's element count, 1 for a scalar.
    pub fn count(&self, path: &str) -> Result<usize, String> {
        let kind = &self.field(path)?.kind;
        match kind.find('[') {
            None => Ok(1),
            Some(b) => kind[b + 1..kind.len() - 1].parse().map_err(|_| format!("{path}'s type {kind} has no count")),
        }
    }

    /// A scalar, or element `index` of an array of scalars such as `i32[8]`.
    pub fn int(&self, state: &[u8], path: &str, index: usize) -> Result<i64, String> {
        let f = self.field(path)?;
        let Some(bracket) = f.kind.find('[') else {
            return if index == 0 { Self::value(state, f) } else { Err(format!("{path} is not an array")) };
        };
        let n = self.count(path)?;
        if index >= n {
            return Err(format!("{path} has {n} elements"));
        }
        let size = f.length / n;
        Self::value(state, &Field { offset: f.offset + index * size, length: size, kind: f.kind[..bracket].to_owned(), path: f.path.clone() })
    }

    pub fn double(&self, state: &[u8], path: &str) -> Result<f64, String> {
        self.get(state, path).map(|v| f64::from_bits(v as u64))
    }

    pub fn set(&self, state: &mut [u8], path: &str, value: i64) -> Result<(), String> {
        let f = self.field(path)?;
        let width = match f.kind.as_str() {
            "u8" | "bool" => 1,
            "u16" => 2,
            "i32" | "u32" => 4,
            "i64" | "f64" => 8,
            other => return Err(format!("{path} is a {other}, not a scalar")),
        };
        let b = state.get_mut(f.offset..f.offset + width).ok_or_else(|| format!("{path} lies past the state's end"))?;
        b.copy_from_slice(&value.to_le_bytes()[..width]);
        Ok(())
    }

    /// A byte array field's bytes in place.
    pub fn bytes<'s>(&self, state: &'s mut [u8], path: &str) -> Result<&'s mut [u8], String> {
        let f = self.field(path)?;
        if !f.kind.starts_with("u8[") {
            return Err(format!("{path} is a {}, not a byte array", f.kind));
        }
        state.get_mut(f.offset..f.offset + f.length).ok_or_else(|| format!("{path} lies past the state's end"))
    }

    /// The fields two states of this layout differ in, each with both values, for a failure message.
    pub fn differences(&self, a: &[u8], b: &[u8]) -> Vec<String> {
        let mut found = Vec::new();
        for f in &self.fields {
            let (Some(x), Some(y)) = (a.get(f.offset..f.offset + f.length), b.get(f.offset..f.offset + f.length)) else { continue };
            if x == y {
                continue;
            }
            if f.kind.starts_with("u8[") {
                let at = x.iter().zip(y).position(|(p, q)| p != q).unwrap_or(0);
                found.push(format!("{}[{at}]: {:02X} against {:02X}", f.path, x[at], y[at]));
            } else {
                let (p, q) = (Self::value(a, f), Self::value(b, f));
                found.push(format!("{}: {} against {}", f.path, p.unwrap_or_default(), q.unwrap_or_default()));
            }
        }
        found
    }
}
