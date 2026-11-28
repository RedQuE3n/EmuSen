//! Per-shell stateful debug commands: `search` (value/delta/pointer scans) and `snapshot`/`diff`.

use crate::memory::{find, read, side_effects, Space};
use crate::{session, Run};
use dianash_module::{Answer, Call};
use emusen_session_service::space::Endian;
use emusen_session_service::words::{hex_digits, parse_dec, parse_hex, split_common_flags, CommonFlags};
use emusen_session_service::Session;
use serde_json::{json, Value};

const SPARSE_THRESHOLD: usize = 262_144;
const UNDO_CAP: usize = 8;
const FLOAT_EPS: f32 = 1e-4;
const NO_SEARCH: &str = "No active search - run 'search <space> <value>' first.";
const NO_POINTER_SEARCH: &str = "No active pointer search - run 'search pointer <space> <target>' first.";

/// A string-keyed table preserving insertion order and reusing freed slots.
#[derive(Default)]
pub(crate) struct Ordered<T> {
    slots: Vec<Option<(String, T)>>,
    free: Vec<usize>,
}

impl<T> Ordered<T> {
    pub fn get(&self, key: &str) -> Option<&T> {
        self.slots.iter().flatten().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn set(&mut self, key: &str, value: T) {
        if let Some(slot) = self.slots.iter_mut().flatten().find(|(k, _)| k == key) {
            slot.1 = value;
            return;
        }
        match self.free.pop() {
            Some(at) => self.slots[at] = Some((key.to_string(), value)),
            None => self.slots.push(Some((key.to_string(), value))),
        }
    }

    pub fn remove(&mut self, key: &str) -> bool {
        match self.slots.iter().position(|s| s.as_ref().is_some_and(|(k, _)| k == key)) {
            Some(at) => {
                self.slots[at] = None;
                self.free.push(at);
                true
            }
            None => false,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &(String, T)> {
        self.slots.iter().flatten()
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum ValueType {
    #[default]
    Unsigned,
    Signed,
    Float32,
}

impl ValueType {
    fn as_str(self) -> &'static str {
        match self {
            ValueType::Unsigned => "unsigned",
            ValueType::Signed => "signed",
            ValueType::Float32 => "float",
        }
    }
}

#[derive(Clone, Copy)]
enum NumericVal {
    Unsigned(u32),
    Signed(i32),
    Float(f32),
}

#[derive(Clone, Copy)]
enum SearchTarget {
    Unsigned(u64),
    Signed(i64),
    Float(f32),
}

#[derive(Clone, Copy)]
enum DeltaVal {
    Int(i64),
    Float(f32),
}

#[derive(Clone, Copy)]
enum CmpOp {
    Eq,
    Ne,
    Gt,
    Lt,
    Between,
}

#[derive(Clone, Copy)]
enum RelOp {
    Changed,
    Unchanged,
    Increased,
    Decreased,
    IncBy,
    DecBy,
}

#[derive(Clone)]
enum Candidates {
    DenseBits { bits: Vec<u64>, len: usize },
    Sparse(Vec<u32>),
}

impl Candidates {
    fn from_dense(bits: Vec<u64>, len: usize) -> Candidates {
        let total: usize = bits.iter().map(|w| w.count_ones() as usize).sum();
        if total <= SPARSE_THRESHOLD {
            let mut sparse = Vec::with_capacity(total);
            for_each_bit(&bits, len, |a| sparse.push(a));
            Candidates::Sparse(sparse)
        } else {
            Candidates::DenseBits { bits, len }
        }
    }

    fn count(&self) -> usize {
        match self {
            Candidates::DenseBits { bits, .. } => bits.iter().map(|w| w.count_ones() as usize).sum(),
            Candidates::Sparse(v) => v.len(),
        }
    }

    fn filter(&self, mut pred: impl FnMut(u32) -> bool) -> Candidates {
        match self {
            Candidates::Sparse(v) => Candidates::Sparse(v.iter().copied().filter(|&a| pred(a)).collect()),
            Candidates::DenseBits { bits, len } => {
                let mut next = vec![0u64; bits.len()];
                for_each_bit(bits, *len, |a| {
                    if pred(a) {
                        let idx = a as usize;
                        next[idx >> 6] |= 1u64 << (idx & 63);
                    }
                });
                Candidates::from_dense(next, *len)
            }
        }
    }

    fn take(&self, limit: usize) -> Vec<u32> {
        let mut out = Vec::with_capacity(limit.min(self.count()));
        match self {
            Candidates::Sparse(v) => out.extend(v.iter().take(limit).copied()),
            Candidates::DenseBits { bits, len } => {
                for (wi, &word) in bits.iter().enumerate() {
                    let mut w = word;
                    let base = wi << 6;
                    while w != 0 {
                        let idx = base + w.trailing_zeros() as usize;
                        if idx >= *len || out.len() >= limit {
                            return out;
                        }
                        out.push(idx as u32);
                        w &= w - 1;
                    }
                }
            }
        }
        out
    }
}

fn for_each_bit(bits: &[u64], len: usize, mut f: impl FnMut(u32)) {
    for (wi, &word) in bits.iter().enumerate() {
        let mut w = word;
        let base = wi << 6;
        while w != 0 {
            let idx = base + w.trailing_zeros() as usize;
            if idx >= len {
                return;
            }
            f(idx as u32);
            w &= w - 1;
        }
    }
}

#[derive(Clone)]
struct SearchSnapshot {
    candidates: Candidates,
    bytes: Vec<u8>,
    val_type: ValueType,
    endian: Endian,
    decimal: bool,
}

#[derive(Clone)]
struct PointerCandidate {
    base_addr: u32,
    offset: i32,
    target: u32,
    pointer: u32,
}

#[derive(Clone)]
struct PointerState {
    space: String,
    endian: Endian,
    max_offset: u32,
    signed_offset: bool,
    candidates: Vec<PointerCandidate>,
}

struct Search {
    space: Option<String>,
    width: i32,
    val_type: ValueType,
    endian: Endian,
    aligned: bool,
    decimal: bool,
    active: Option<SearchSnapshot>,
    undo_stack: Vec<SearchSnapshot>,
    pointer_mode: Option<PointerState>,
}

impl Default for Search {
    fn default() -> Search {
        Search {
            space: None,
            width: 1,
            val_type: ValueType::Unsigned,
            endian: Endian::Little,
            aligned: false,
            decimal: false,
            active: None,
            undo_stack: Vec::new(),
            pointer_mode: None,
        }
    }
}

/// What this module keeps for one shell.
#[derive(Default)]
pub(crate) struct State {
    search: Search,
    snapshots: Ordered<(String, Vec<u8>)>,
}

fn state<'a>(call: &'a Call) -> std::sync::MutexGuard<'a, State> {
    call.slot(State::default)
}

fn decode_raw(bytes: &[u8], at: usize, width: i32, endian: Endian) -> u32 {
    match width {
        1 => u32::from(bytes.get(at).copied().unwrap_or(0)),
        2 => {
            let b = [bytes.get(at).copied().unwrap_or(0), bytes.get(at + 1).copied().unwrap_or(0)];
            u32::from(if endian == Endian::Big { u16::from_be_bytes(b) } else { u16::from_le_bytes(b) })
        }
        _ => {
            let b = [
                bytes.get(at).copied().unwrap_or(0),
                bytes.get(at + 1).copied().unwrap_or(0),
                bytes.get(at + 2).copied().unwrap_or(0),
                bytes.get(at + 3).copied().unwrap_or(0),
            ];
            if endian == Endian::Big { u32::from_be_bytes(b) } else { u32::from_le_bytes(b) }
        }
    }
}

fn decode(bytes: &[u8], at: usize, width: i32, endian: Endian, val_type: ValueType) -> NumericVal {
    let raw = decode_raw(bytes, at, width, endian);
    match val_type {
        ValueType::Unsigned => NumericVal::Unsigned(raw),
        ValueType::Signed => match width {
            1 => NumericVal::Signed(i32::from(raw as u8 as i8)),
            2 => NumericVal::Signed(i32::from(raw as u16 as i16)),
            _ => NumericVal::Signed(raw as i32),
        },
        ValueType::Float32 => NumericVal::Float(f32::from_bits(raw)),
    }
}

fn raw_bits(val: NumericVal) -> u64 {
    match val {
        NumericVal::Unsigned(v) => u64::from(v),
        NumericVal::Signed(v) => u64::from(v as u32),
        NumericVal::Float(v) => u64::from(v.to_bits()),
    }
}

fn format_num(val: NumericVal, width: i32, decimal: bool) -> String {
    match val {
        NumericVal::Unsigned(v) => {
            if decimal {
                v.to_string()
            } else {
                let d = (width as usize) * 2;
                format!("0x{v:0d$X}")
            }
        }
        NumericVal::Signed(v) => {
            if decimal {
                v.to_string()
            } else {
                let d = (width as usize) * 2;
                let mask = match width {
                    1 => 0xFFu32,
                    2 => 0xFFFFu32,
                    _ => 0xFFFF_FFFFu32,
                };
                format!("0x{:0d$X}", (v as u32) & mask)
            }
        }
        NumericVal::Float(v) => format!("{v:.4}").trim_end_matches('0').trim_end_matches('.').to_string(),
    }
}

fn format_delta(now: NumericVal, prev: NumericVal, decimal: bool) -> String {
    match (now, prev) {
        (NumericVal::Unsigned(n), NumericVal::Unsigned(p)) => {
            let d = i64::from(n) - i64::from(p);
            if decimal || d == 0 {
                format!("{d:+}")
            } else if d > 0 {
                format!("+0x{d:X}")
            } else {
                format!("-0x{:X}", -d)
            }
        }
        (NumericVal::Signed(n), NumericVal::Signed(p)) => {
            let d = i64::from(n) - i64::from(p);
            if decimal || d == 0 {
                format!("{d:+}")
            } else if d > 0 {
                format!("+0x{d:X}")
            } else {
                format!("-0x{:X}", -d)
            }
        }
        (NumericVal::Float(n), NumericVal::Float(p)) => {
            let d = n - p;
            if d.is_nan() { "NaN".to_string() } else { format!("{d:+.4}").trim_end_matches('0').trim_end_matches('.').to_string() }
        }
        _ => "0".to_string(),
    }
}

fn parse_target(text: &str, width: i32, decimal: bool, val_type: ValueType) -> Result<(SearchTarget, bool), String> {
    let trimmed = text.trim();
    match val_type {
        ValueType::Float32 => {
            if !decimal && (trimmed.starts_with("0x") || trimmed.starts_with("0X") || trimmed.starts_with('$')) {
                let bits = parse_hex(trimmed)? as u32;
                let v = f32::from_bits(bits);
                if v.is_nan() {
                    return Err(format!("'{text}' is NaN."));
                }
                return Ok((SearchTarget::Float(v), false));
            }
            let v = trimmed.parse::<f32>().map_err(|e| format!("'{text}' is not a float: {e}."))?;
            if v.is_nan() {
                return Err(format!("'{text}' is NaN."));
            }
            let has_frac = trimmed.contains('.') || trimmed.contains('e') || trimmed.contains('E');
            Ok((SearchTarget::Float(v), has_frac))
        }
        ValueType::Signed => {
            if decimal {
                let v = parse_dec(trimmed)?;
                Ok((SearchTarget::Signed(v), false))
            } else {
                let raw = parse_hex(trimmed)? as u32;
                let v = match width {
                    1 => i64::from(raw as u8 as i8),
                    2 => i64::from(raw as u16 as i16),
                    _ => i64::from(raw as i32),
                };
                Ok((SearchTarget::Signed(v), false))
            }
        }
        ValueType::Unsigned => {
            if decimal {
                let v = trimmed.parse::<u64>().map_err(|e| format!("'{text}' is not an unsigned decimal number: {e}."))?;
                Ok((SearchTarget::Unsigned(v), false))
            } else {
                let raw = parse_hex(trimmed)? as u32;
                Ok((SearchTarget::Unsigned(u64::from(raw)), false))
            }
        }
    }
}

fn parse_delta(text: &str, decimal: bool, val_type: ValueType) -> Result<(DeltaVal, bool), String> {
    let trimmed = text.trim();
    if val_type == ValueType::Float32 {
        let v = trimmed.parse::<f32>().map_err(|e| format!("'{text}' is not a float: {e}."))?;
        if v.is_nan() {
            return Err(format!("'{text}' is NaN."));
        }
        let has_frac = trimmed.contains('.') || trimmed.contains('e') || trimmed.contains('E');
        return Ok((DeltaVal::Float(v), has_frac));
    }
    let v = if decimal { parse_dec(trimmed)? } else { i64::from(parse_hex(trimmed)? as u32) };
    Ok((DeltaVal::Int(v), false))
}

fn matches_cmp(now: NumericVal, op: CmpOp, a: SearchTarget, b: Option<SearchTarget>, has_frac: bool) -> bool {
    match (now, a, b) {
        (NumericVal::Unsigned(n), SearchTarget::Unsigned(lo), hi) => {
            let n = u64::from(n);
            match op {
                CmpOp::Eq => n == lo,
                CmpOp::Ne => n != lo,
                CmpOp::Gt => n > lo,
                CmpOp::Lt => n < lo,
                CmpOp::Between => matches!(hi, Some(SearchTarget::Unsigned(h)) if n >= lo.min(h) && n <= lo.max(h)),
            }
        }
        (NumericVal::Signed(n), SearchTarget::Signed(lo), hi) => {
            let n = i64::from(n);
            match op {
                CmpOp::Eq => n == lo,
                CmpOp::Ne => n != lo,
                CmpOp::Gt => n > lo,
                CmpOp::Lt => n < lo,
                CmpOp::Between => matches!(hi, Some(SearchTarget::Signed(h)) if n >= lo.min(h) && n <= lo.max(h)),
            }
        }
        (NumericVal::Float(n), SearchTarget::Float(lo), hi) => {
            if n.is_nan() {
                return false;
            }
            match op {
                CmpOp::Eq => {
                    if has_frac { (n - lo).abs() <= FLOAT_EPS } else { n == lo }
                }
                CmpOp::Ne => {
                    if has_frac { (n - lo).abs() > FLOAT_EPS } else { n != lo }
                }
                CmpOp::Gt => n > lo,
                CmpOp::Lt => n < lo,
                CmpOp::Between => match hi {
                    Some(SearchTarget::Float(h)) => {
                        let (min_v, max_v) = if lo <= h { (lo, h) } else { (h, lo) };
                        if has_frac { n >= min_v - FLOAT_EPS && n <= max_v + FLOAT_EPS } else { n >= min_v && n <= max_v }
                    }
                    _ => false,
                },
            }
        }
        _ => false,
    }
}

fn matches_rel_int(n: i64, p: i64, op: RelOp, delta: Option<(DeltaVal, bool)>) -> bool {
    match op {
        RelOp::Changed => n != p,
        RelOp::Unchanged => n == p,
        RelOp::Increased => n > p,
        RelOp::Decreased => n < p,
        RelOp::IncBy => matches!(delta, Some((DeltaVal::Int(d), _)) if n - p == d),
        RelOp::DecBy => matches!(delta, Some((DeltaVal::Int(d), _)) if p - n == d),
    }
}

fn matches_rel(now: NumericVal, prev: NumericVal, op: RelOp, delta: Option<(DeltaVal, bool)>) -> bool {
    match (now, prev) {
        (NumericVal::Unsigned(n), NumericVal::Unsigned(p)) => matches_rel_int(i64::from(n), i64::from(p), op, delta),
        (NumericVal::Signed(n), NumericVal::Signed(p)) => matches_rel_int(i64::from(n), i64::from(p), op, delta),
        (NumericVal::Float(n), NumericVal::Float(p)) => {
            if n.is_nan() || p.is_nan() {
                return false;
            }
            match op {
                RelOp::Changed => n != p,
                RelOp::Unchanged => n == p,
                RelOp::Increased => n > p,
                RelOp::Decreased => n < p,
                RelOp::IncBy => matches!(delta, Some((DeltaVal::Float(d), _)) if (n - p - d).abs() <= FLOAT_EPS),
                RelOp::DecBy => matches!(delta, Some((DeltaVal::Float(d), _)) if (p - n - d).abs() <= FLOAT_EPS),
            }
        }
        _ => false,
    }
}

/// The whole space, as the candidates' values are read from it.
fn whole(session: &Session, space: &Space) -> Result<Vec<u8>, String> {
    read(session, space, 0, space.size)
}

fn push_undo(st: &mut Search, snap: SearchSnapshot) {
    if st.undo_stack.len() == UNDO_CAP {
        st.undo_stack.remove(0);
    }
    st.undo_stack.push(snap);
}

fn search_summary_value(st: &Search) -> Value {
    let total = st.active.as_ref().map_or(0, |s| s.candidates.count());
    json!({
        "space": st.space,
        "width": st.width,
        "val_type": st.val_type.as_str(),
        "endian": st.endian.as_str(),
        "aligned": st.aligned,
        "total": total,
        "can_undo": !st.undo_stack.is_empty(),
    })
}

fn resolve_type_and_width(flags: &CommonFlags, pos_width: Option<i32>) -> Result<(i32, ValueType), String> {
    if let (Some(fw), Some(pw)) = (flags.width, pos_width) {
        if fw != pw {
            return Err(format!("conflicting widths {fw} and {pw}."));
        }
    }
    let explicit = flags.width.or(pos_width);
    let val_type = if flags.float {
        ValueType::Float32
    } else if flags.signed == Some(true) {
        ValueType::Signed
    } else {
        ValueType::Unsigned
    };
    let width = if val_type == ValueType::Float32 {
        if explicit.is_some_and(|w| w != 4) {
            return Err("--float requires width 4 (32-bit IEEE-754 float).".to_string());
        }
        4
    } else {
        explicit.unwrap_or(1)
    };
    if !matches!(width, 1 | 2 | 4) {
        return Err("width must be 1, 2, or 4".to_string());
    }
    Ok((width, val_type))
}

fn next_refine_params(st: &Search, flags: &CommonFlags) -> Result<(ValueType, Endian, bool), String> {
    if let Some(w) = flags.width {
        if w != st.width {
            return Err(format!("cannot change width from {} to {w} during refinement.", st.width));
        }
    }
    let val_type = if flags.float {
        if st.width != 4 {
            return Err("--float requires an active 4-byte search.".to_string());
        }
        ValueType::Float32
    } else if let Some(s) = flags.signed {
        if s { ValueType::Signed } else { ValueType::Unsigned }
    } else {
        st.val_type
    };
    let endian = flags.endian.unwrap_or(st.endian);
    let decimal = flags.decimal.unwrap_or(st.decimal);
    Ok((val_type, endian, decimal))
}

fn init_search_params(space: &Space, flags: &CommonFlags, pos_width: Option<i32>, auto_align_large: bool) -> Result<(i32, ValueType, Endian, bool, bool), String> {
    let (width, val_type) = resolve_type_and_width(flags, pos_width)?;
    let endian = flags.endian.unwrap_or(space.byte_order);
    let aligned = flags.aligned.unwrap_or(auto_align_large && width > 1 && space.size > 65_536);
    let decimal = flags.decimal.unwrap_or(false);
    Ok((width, val_type, endian, aligned, decimal))
}

fn finish_refine(
    call: &Call,
    st: &mut Search,
    val_type: ValueType,
    endian: Endian,
    decimal: bool,
    json_out: bool,
    keep: impl Fn(&[u8], &[u8], usize, i32, Endian, ValueType) -> bool,
) -> Run {
    let session = session(call)?;
    let space = find(&session, st.space.as_deref().unwrap_or(""))?;
    let bytes = whole(&session, &space)?;
    let prev_snap = st.active.take().unwrap();
    let width = st.width;
    let kept = prev_snap.candidates.filter(|a| keep(&bytes, &prev_snap.bytes, a as usize, width, endian, val_type));
    let count = kept.count();
    push_undo(st, prev_snap);
    st.val_type = val_type;
    st.endian = endian;
    st.decimal = decimal;
    st.active = Some(SearchSnapshot { candidates: kept, bytes, val_type, endian, decimal });
    let out = if json_out { search_summary_value(st).to_string() } else { format!("{count} candidate(s) remain.") };
    Ok(Answer::ok(out))
}

#[allow(clippy::too_many_arguments)]
fn start_initial_search(st: &mut Search, space_name: String, width: i32, val_type: ValueType, endian: Endian, aligned: bool, decimal: bool, candidates: Candidates, bytes: Vec<u8>) -> usize {
    let count = candidates.count();
    st.space = Some(space_name);
    st.width = width;
    st.val_type = val_type;
    st.endian = endian;
    st.aligned = aligned;
    st.decimal = decimal;
    st.undo_stack.clear();
    st.pointer_mode = None;
    st.active = Some(SearchSnapshot { candidates, bytes, val_type, endian, decimal });
    count
}

fn reject_unknown_flags(tokens: &[String]) -> Result<(), String> {
    for tok in tokens {
        if tok.starts_with("--") || (tok.starts_with('-') && tok.len() > 1 && !tok[1..].starts_with(|c: char| c.is_ascii_digit() || c == '.')) {
            return Err(format!("Unknown flag '{tok}'."));
        }
    }
    Ok(())
}

fn build_initial_candidates(space_size: i32, width: i32, aligned: bool, mut keep: impl FnMut(usize) -> bool) -> Candidates {
    if space_size < width {
        return Candidates::Sparse(Vec::new());
    }
    let last = (space_size - width) as usize;
    let len = last + 1;
    let step = if aligned { width as usize } else { 1 };
    let words = len.div_ceil(64);
    let mut bits = vec![0u64; words];
    let mut addr = 0usize;
    while addr <= last {
        if keep(addr) {
            bits[addr >> 6] |= 1u64 << (addr & 63);
        }
        addr += step;
    }
    Candidates::from_dense(bits, len)
}

fn parse_u32_arg(text: &str, decimal: bool) -> Result<u32, String> {
    if decimal {
        text.trim().parse::<u32>().map_err(|e| format!("'{text}' is not an unsigned decimal number: {e}."))
    } else {
        Ok(parse_hex(text)? as u32)
    }
}

struct PointerFlags {
    max_offset: u32,
    base_start: u32,
    base_end: Option<u32>,
    signed_offset: bool,
}

fn split_pointer_flags(tokens: &[String], decimal: bool) -> Result<(Vec<String>, PointerFlags), String> {
    let mut pf = PointerFlags { max_offset: 0x1000, base_start: 0, base_end: None, signed_offset: false };
    let mut pos = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let arg = &tokens[i];
        match arg.as_str() {
            "--max-offset" => {
                let next = tokens.get(i + 1).ok_or_else(|| "--max-offset requires a value.".to_string())?;
                pf.max_offset = parse_u32_arg(next, decimal)?;
                i += 2;
            }
            "--base-start" => {
                let next = tokens.get(i + 1).ok_or_else(|| "--base-start requires an address.".to_string())?;
                pf.base_start = parse_u32_arg(next, decimal)?;
                i += 2;
            }
            "--base-end" => {
                let next = tokens.get(i + 1).ok_or_else(|| "--base-end requires an address.".to_string())?;
                pf.base_end = Some(parse_u32_arg(next, decimal)?);
                i += 2;
            }
            "--signed-offset" => {
                pf.signed_offset = true;
                i += 1;
            }
            _ => {
                pos.push(arg.clone());
                i += 1;
            }
        }
    }
    reject_unknown_flags(&pos)?;
    Ok((pos, pf))
}

fn pointer_matches(ptr: u32, target: u32, max_offset: u32, signed_offset: bool) -> Option<i32> {
    if signed_offset {
        let diff = target.wrapping_sub(ptr) as i32;
        (diff.unsigned_abs() <= max_offset).then_some(diff)
    } else if target >= ptr && target - ptr <= max_offset {
        i32::try_from(target - ptr).ok()
    } else {
        None
    }
}

fn pointer_json(ps: &PointerState, target: u32, space_size: i32, limit: usize) -> Value {
    let digits = hex_digits(space_size);
    let hits: Vec<Value> = ps
        .candidates
        .iter()
        .take(limit)
        .map(|c| {
            let off_hex = if c.offset >= 0 { format!("+0x{:X}", c.offset) } else { format!("-0x{:X}", -i64::from(c.offset)) };
            json!({
                "base_addr": c.base_addr,
                "base_hex": format!("{:0digits$X}", c.base_addr),
                "offset": c.offset,
                "offset_hex": off_hex,
                "target": c.target,
                "pointer": c.pointer,
                "pointer_hex": format!("{:08X}", c.pointer),
            })
        })
        .collect();
    json!({
        "space": ps.space,
        "target": target,
        "target_hex": format!("{target:08X}"),
        "endian": ps.endian.as_str(),
        "max_offset": ps.max_offset,
        "total": ps.candidates.len(),
        "showing": hits.len(),
        "hits": hits,
    })
}

fn run_pointer(call: &Call, st: &mut Search, tokens: &[String], flags: &CommonFlags) -> Run {
    let decimal = flags.decimal.unwrap_or(false);
    let (pos, pf) = split_pointer_flags(tokens, decimal)?;
    if pos.len() < 2 {
        return Ok(Answer::ok("Usage: search pointer <space> <target> [--max-offset N] [--base-start A] [--base-end B] [--le|--be] [--json] | search pointer rescan <target> [--max-offset N] [--json]"));
    }
    let session = session(call)?;
    if pos[1].eq_ignore_ascii_case("rescan") {
        if pos.len() < 3 {
            return Ok(Answer::ok("Usage: search pointer rescan <target> [--max-offset N] [--json]"));
        }
        let Some(prev) = st.pointer_mode.as_ref() else {
            return Ok(Answer::ok(NO_POINTER_SEARCH));
        };
        let target = parse_u32_arg(&pos[2], decimal)?;
        let space = find(&session, &prev.space)?;
        let bytes = whole(&session, &space)?;
        let max_offset = if tokens.iter().any(|t| t == "--max-offset") { pf.max_offset } else { prev.max_offset };
        let endian = flags.endian.unwrap_or(prev.endian);
        let signed_offset = prev.signed_offset || pf.signed_offset;
        let mut kept = Vec::new();
        for c in &prev.candidates {
            if c.base_addr as usize + 4 <= bytes.len() {
                let ptr = decode_raw(&bytes, c.base_addr as usize, 4, endian);
                if let Some(offset) = pointer_matches(ptr, target, max_offset, signed_offset) {
                    kept.push(PointerCandidate { base_addr: c.base_addr, offset, target, pointer: ptr });
                }
            }
        }
        let next = PointerState { space: space.name.clone(), endian, max_offset, signed_offset, candidates: kept };
        let out = if flags.json {
            pointer_json(&next, target, space.size, 200).to_string()
        } else {
            format!("{} pointer(s) remain pointing near 0x{target:08X}.", next.candidates.len())
        };
        st.pointer_mode = Some(next);
        return Ok(Answer::ok(out));
    }
    if pos.len() < 3 {
        return Ok(Answer::ok("Usage: search pointer <space> <target> [--max-offset N] [--base-start A] [--base-end B] [--le|--be] [--json]"));
    }
    let space = find(&session, &pos[1])?;
    if space.side_effects {
        return Ok(Answer::ok(side_effects(&space, "search")));
    }
    let target = parse_u32_arg(&pos[2], decimal)?;
    let endian = flags.endian.unwrap_or(space.byte_order);
    let bytes = whole(&session, &space)?;
    let mut found = Vec::new();
    if bytes.len() >= 4 {
        let max_addr = (bytes.len() - 4) as u32;
        let end = pf.base_end.map_or(max_addr, |b| b.min(max_addr));
        let mut addr = pf.base_start.checked_next_multiple_of(4).filter(|&a| a <= end);
        while let Some(a) = addr {
            let ptr = decode_raw(&bytes, a as usize, 4, endian);
            if let Some(offset) = pointer_matches(ptr, target, pf.max_offset, pf.signed_offset) {
                found.push(PointerCandidate { base_addr: a, offset, target, pointer: ptr });
            }
            addr = a.checked_add(4).filter(|&next| next <= end);
        }
    }
    let state_ptr = PointerState { space: space.name.clone(), endian, max_offset: pf.max_offset, signed_offset: pf.signed_offset, candidates: found };
    let out = if flags.json {
        pointer_json(&state_ptr, target, space.size, 200).to_string()
    } else {
        format!("{} pointer(s) found in {} near 0x{target:08X} (max offset 0x{:X}).", state_ptr.candidates.len(), space.name, pf.max_offset)
    };
    st.pointer_mode = Some(state_ptr);
    Ok(Answer::ok(out))
}

fn search_usage() -> String {
    concat!(
        "Usage: search <space> <value> [<width>] [flags] | search unknown <space> [flags] | ",
        "search eq|ne|gt|lt <space> <value> [flags] | search between <space> <lo> <hi> [flags] | ",
        "search refine|eq|ne|gt|lt <value> | search between <lo> <hi> | ",
        "search changed|unchanged|increased|decreased | search incby|decby <n> | ",
        "search undo | search list [<count>] | search reset | ",
        "search pointer <space> <target> [--max-offset N] | search pointer rescan <target>"
    )
    .to_string()
}

pub fn search(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 2 {
        return Ok(Answer::ok(search_usage()));
    }
    let (pos, flags) = split_common_flags(&parts[1..])?;
    if pos.is_empty() {
        return Ok(Answer::ok(search_usage()));
    }
    let mut held = state(call);
    let st = &mut held.search;
    let first = pos[0].to_lowercase();
    if first == "pointer" {
        return run_pointer(call, st, &pos, &flags);
    }
    reject_unknown_flags(&pos)?;
    match first.as_str() {
        "reset" => {
            *st = Search::default();
            let out = if flags.json { json!({"cleared": true}).to_string() } else { "Search cleared.".to_string() };
            return Ok(Answer::ok(out));
        }
        "undo" => {
            let Some(prev) = st.undo_stack.pop() else {
                if flags.json {
                    return Ok(Answer::ok(json!({"undone": false, "total": st.active.as_ref().map_or(0, |s| s.candidates.count()), "can_undo": false}).to_string()));
                }
                return Ok(Answer::ok("Nothing to undo."));
            };
            let count = prev.candidates.count();
            st.val_type = prev.val_type;
            st.endian = prev.endian;
            st.decimal = prev.decimal;
            st.active = Some(prev);
            let out = if flags.json {
                let mut v = search_summary_value(st);
                v["undone"] = json!(true);
                v.to_string()
            } else {
                format!("Undone. {count} candidate(s) remain.")
            };
            return Ok(Answer::ok(out));
        }
        "list" => {
            let Some(active) = st.active.as_ref() else {
                if flags.json {
                    let mut v = search_summary_value(st);
                    v["showing"] = json!(0);
                    v["hits"] = json!([]);
                    return Ok(Answer::ok(v.to_string()));
                }
                return Ok(Answer::ok(NO_SEARCH));
            };
            let dec = flags.decimal.unwrap_or(st.decimal);
            let count = if pos.len() >= 2 {
                if dec || flags.json {
                    parse_dec(&pos[1])? as i32
                } else {
                    parse_hex(&pos[1])?
                }
            } else {
                20
            };
            let session = session(call)?;
            let space = find(&session, st.space.as_deref().unwrap_or(""))?;
            let bytes = whole(&session, &space)?;
            let addrs = active.candidates.take(count.max(0) as usize);
            let total = active.candidates.count();
            if flags.json {
                let digits = hex_digits(space.size);
                let hits: Vec<Value> = addrs
                    .iter()
                    .map(|&a| {
                        let now = decode(&bytes, a as usize, st.width, st.endian, st.val_type);
                        let last = decode(&active.bytes, a as usize, st.width, st.endian, st.val_type);
                        let prev = if raw_bits(now) != raw_bits(last) {
                            last
                        } else if let Some(u) = st.undo_stack.last() {
                            decode(&u.bytes, a as usize, st.width, st.endian, st.val_type)
                        } else {
                            last
                        };
                        json!({
                            "address": a,
                            "addr_hex": format!("{a:0digits$X}"),
                            "value": format_num(now, st.width, dec),
                            "prev": format_num(prev, st.width, dec),
                            "delta": format_delta(now, prev, dec),
                            "raw": raw_bits(now),
                        })
                    })
                    .collect();
                let mut v = search_summary_value(st);
                v["showing"] = json!(hits.len());
                v["hits"] = json!(hits);
                return Ok(Answer::ok(v.to_string()));
            }
            let shown: Vec<String> = addrs
                .iter()
                .map(|&a| {
                    let now = decode(&bytes, a as usize, st.width, st.endian, st.val_type);
                    format!("  0x{a:X} = {}", format_num(now, st.width, dec))
                })
                .collect();
            return Ok(Answer::ok(format!("{total} candidate(s) in {} (width {}), showing up to {count}:\n{}", space.name, st.width, shown.join("\n"))));
        }
        "unknown" => {
            if pos.len() < 2 {
                return Ok(Answer::ok("Usage: search unknown <space> [flags]"));
            }
            let session = session(call)?;
            let space = find(&session, &pos[1])?;
            if space.side_effects {
                return Ok(Answer::ok(side_effects(&space, "search")));
            }
            let (width, val_type, endian, aligned, decimal) = init_search_params(&space, &flags, None, true)?;
            let bytes = whole(&session, &space)?;
            let candidates = build_initial_candidates(space.size, width, aligned, |_| true);
            let count = start_initial_search(st, space.name.clone(), width, val_type, endian, aligned, decimal, candidates, bytes);
            let out = if flags.json {
                search_summary_value(st).to_string()
            } else {
                format!("{count} candidate(s) in {} (unknown initial, width {width}).", space.name)
            };
            return Ok(Answer::ok(out));
        }
        "refine" | "eq" | "ne" | "gt" | "lt" | "between" => {
            let cmp = match first.as_str() {
                "refine" | "eq" => CmpOp::Eq,
                "ne" => CmpOp::Ne,
                "gt" => CmpOp::Gt,
                "lt" => CmpOp::Lt,
                _ => CmpOp::Between,
            };
            let initial = match cmp {
                CmpOp::Between => pos.len() >= 4,
                _ if first == "refine" => false,
                _ => pos.len() >= 3,
            };
            if initial {
                let session = session(call)?;
                let space = find(&session, &pos[1])?;
                if space.side_effects {
                    return Ok(Answer::ok(side_effects(&space, "search")));
                }
                let (width, val_type, endian, aligned, decimal) = init_search_params(&space, &flags, None, true)?;
                let (t1, f1) = parse_target(&pos[2], width, decimal, val_type)?;
                let (t2, f2) = if matches!(cmp, CmpOp::Between) {
                    let (v, f) = parse_target(&pos[3], width, decimal, val_type)?;
                    (Some(v), f)
                } else {
                    (None, false)
                };
                let bytes = whole(&session, &space)?;
                let candidates = build_initial_candidates(space.size, width, aligned, |a| {
                    matches_cmp(decode(&bytes, a, width, endian, val_type), cmp, t1, t2, f1 || f2)
                });
                let count = start_initial_search(st, space.name.clone(), width, val_type, endian, aligned, decimal, candidates, bytes);
                let out = if flags.json {
                    search_summary_value(st).to_string()
                } else {
                    format!("{count} candidate(s) found in {} (width {width}).", space.name)
                };
                return Ok(Answer::ok(out));
            }
            if st.active.is_none() {
                return Ok(Answer::ok(NO_SEARCH));
            }
            let needed = if matches!(cmp, CmpOp::Between) { 3 } else { 2 };
            if pos.len() < needed {
                return Ok(Answer::ok(if matches!(cmp, CmpOp::Between) {
                    "Usage: search between <lo> <hi>"
                } else if first == "refine" {
                    "Usage: search refine <value>"
                } else {
                    "Usage: search eq|ne|gt|lt <value>"
                }));
            }
            let (val_type, endian, decimal) = next_refine_params(st, &flags)?;
            let (t1, f1) = parse_target(&pos[1], st.width, decimal, val_type)?;
            let (t2, f2) = if matches!(cmp, CmpOp::Between) {
                let (v, f) = parse_target(&pos[2], st.width, decimal, val_type)?;
                (Some(v), f)
            } else {
                (None, false)
            };
            return finish_refine(call, st, val_type, endian, decimal, flags.json, |bytes, _prev, idx, width, endian, val_type| {
                matches_cmp(decode(bytes, idx, width, endian, val_type), cmp, t1, t2, f1 || f2)
            });
        }
        "changed" | "unchanged" | "increased" | "decreased" | "incby" | "decby" => {
            if st.active.is_none() {
                return Ok(Answer::ok(NO_SEARCH));
            }
            let rel = match first.as_str() {
                "changed" => RelOp::Changed,
                "unchanged" => RelOp::Unchanged,
                "increased" => RelOp::Increased,
                "decreased" => RelOp::Decreased,
                "incby" => RelOp::IncBy,
                _ => RelOp::DecBy,
            };
            if matches!(rel, RelOp::IncBy | RelOp::DecBy) && pos.len() < 2 {
                return Ok(Answer::ok(format!("Usage: search {first} <delta>")));
            }
            let (val_type, endian, decimal) = next_refine_params(st, &flags)?;
            let delta = if matches!(rel, RelOp::IncBy | RelOp::DecBy) {
                Some(parse_delta(&pos[1], decimal, val_type)?)
            } else {
                None
            };
            return finish_refine(call, st, val_type, endian, decimal, flags.json, |bytes, prev, idx, width, endian, val_type| {
                matches_rel(decode(bytes, idx, width, endian, val_type), decode(prev, idx, width, endian, val_type), rel, delta)
            });
        }
        _ => {}
    }
    if pos.len() < 2 {
        return Ok(Answer::ok("Usage: search <space> <value> [<width>]"));
    }
    let session = session(call)?;
    let space = find(&session, &pos[0])?;
    if space.side_effects {
        return Ok(Answer::ok(side_effects(&space, "search")));
    }
    let pos_width = if pos.len() >= 3 { Some(parse_hex(&pos[2])?) } else { None };
    let (width, val_type, endian, aligned, decimal) = init_search_params(&space, &flags, pos_width, false)?;
    let (target, has_frac) = parse_target(&pos[1], width, decimal, val_type)?;
    let bytes = whole(&session, &space)?;
    let candidates = build_initial_candidates(space.size, width, aligned, |a| {
        matches_cmp(decode(&bytes, a, width, endian, val_type), CmpOp::Eq, target, None, has_frac)
    });
    let count = start_initial_search(st, space.name.clone(), width, val_type, endian, aligned, decimal, candidates, bytes);
    let out = if flags.json {
        search_summary_value(st).to_string()
    } else {
        let shown = match target {
            SearchTarget::Unsigned(v) => format!("0x{v:X}"),
            SearchTarget::Signed(v) => format!("{v}"),
            SearchTarget::Float(v) => format!("{v}"),
        };
        format!("{count} candidate(s) found in {} matching {shown} (width {width}).", space.name)
    };
    Ok(Answer::ok(out))
}

pub fn snapshot(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: snapshot <space> <name> | snapshot list | snapshot remove <name>"));
    }
    let first = parts[1].to_lowercase();
    let mut held = state(call);
    let snapshots = &mut held.snapshots;
    if first == "list" {
        if snapshots.len() == 0 {
            return Ok(Answer::ok("No saved snapshots."));
        }
        let rows: Vec<String> = snapshots.iter().map(|(name, (space, data))| format!("  {name}: {space} ({} bytes)", data.len())).collect();
        return Ok(Answer::ok(rows.join("\n")));
    }
    if first == "remove" {
        if parts.len() < 3 {
            return Ok(Answer::ok("Usage: snapshot remove <name>"));
        }
        let name = &parts[2];
        return Ok(Answer::ok(if snapshots.remove(name) { format!("Snapshot '{name}' removed.") } else { format!("No snapshot named '{name}'.") }));
    }
    if parts.len() < 3 {
        return Ok(Answer::ok("Usage: snapshot <space> <name>"));
    }
    let session = session(call)?;
    let space = find(&session, &parts[1])?;
    if space.side_effects {
        return Ok(Answer::ok(side_effects(&space, "snapshot")));
    }
    let data = whole(&session, &space)?;
    let name = &parts[2];
    let length = data.len();
    snapshots.set(name, (space.name.clone(), data));
    Ok(Answer::ok(format!("Snapshot '{name}' saved: {}, {length} bytes.", space.name)))
}

pub fn diff(call: &Call, parts: &[String]) -> Run {
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: diff <name> [<count>]"));
    }
    let name = &parts[1];
    let Some((space_name, saved)) = state(call).snapshots.get(name).map(|(s, d)| (s.clone(), d.clone())) else {
        return Ok(Answer::ok(format!("No snapshot named '{name}'. Try 'snapshot list'.")));
    };
    let session = session(call)?;
    let space = find(&session, &space_name)?;
    let count = if parts.len() >= 3 { parse_hex(&parts[2])? } else { 20 };
    let now = read(&session, &space, 0, saved.len() as i32)?;
    let changes: Vec<(usize, u8, u8)> = saved.iter().enumerate().filter_map(|(a, &old)| (now[a] != old).then_some((a, old, now[a]))).collect();
    if changes.is_empty() {
        return Ok(Answer::ok(format!("No differences from snapshot '{name}'.")));
    }
    let shown: Vec<String> = changes.iter().take(count.max(0) as usize).map(|(a, old, new)| format!("  0x{a:X}: 0x{old:02X} -> 0x{new:02X}")).collect();
    Ok(Answer::ok(format!("{} byte(s) differ from snapshot '{name}' ({space_name}), showing up to {count}:\n{}", changes.len(), shown.join("\n"))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use emusen_dianash::shell::{Config, Shell, Submitted};
    use emusen_session_service::abi::{self, emusen_session_v1};
    use std::cell::RefCell;
    use std::ffi::c_void;

    struct FakeCore {
        ram: RefCell<Vec<u8>>,
        big_endian: bool,
    }

    unsafe extern "C" fn fake_info(ctx: *mut c_void, _s: u32, o: *mut u8, l: usize) -> i64 {
        let f = unsafe { &*(ctx as *const FakeCore) };
        let order = if f.big_endian { "big" } else { "little" };
        let doc = format!(r#"{{"id":1,"engine":"Fake","system":"test","byte_order":"{order}","capabilities":[]}}"#);
        unsafe { abi::copy_answer(doc.as_bytes(), o, l) }
    }

    unsafe extern "C" fn fake_spaces(ctx: *mut c_void, _s: u32, o: *mut u8, l: usize) -> i64 {
        let f = unsafe { &*(ctx as *const FakeCore) };
        let order = if f.big_endian { "big" } else { "little" };
        let size = f.ram.borrow().len();
        let doc = format!(r#"[{{"name":"RAM","size":{size},"writable":true,"side_effects":false,"byte_order":"{order}"}}]"#);
        unsafe { abi::copy_answer(doc.as_bytes(), o, l) }
    }

    unsafe extern "C" fn fake_space_read(ctx: *mut c_void, _s: u32, _sp: *const u8, _sl: usize, addr: u32, o: *mut u8, l: usize) -> i64 {
        let f = unsafe { &*(ctx as *const FakeCore) };
        let ram = f.ram.borrow();
        let start = (addr as usize).min(ram.len());
        let n = l.min(ram.len() - start);
        if !o.is_null() && n > 0 {
            unsafe { std::ptr::copy_nonoverlapping(ram[start..].as_ptr(), o, n) };
        }
        l as i64
    }

    unsafe extern "C" fn fake_space_write(ctx: *mut c_void, _s: u32, _sp: *const u8, _sl: usize, addr: u32, d: *const u8, l: usize) -> i32 {
        let f = unsafe { &*(ctx as *const FakeCore) };
        let mut ram = f.ram.borrow_mut();
        let slice = unsafe { std::slice::from_raw_parts(d, l) };
        for (i, &b) in slice.iter().enumerate() {
            if let Some(cell) = ram.get_mut(addr as usize + i) {
                *cell = b;
            }
        }
        0
    }

    unsafe extern "C" fn fake_processors(_c: *mut c_void, _s: u32, o: *mut u8, l: usize) -> i64 {
        unsafe { abi::copy_answer(br#"[{"index":0,"name":"CPU","description":"CPU","can_halt":true,"code_space":"RAM","coverage":false,"call_stack":false,"registers":true,"writable_registers":false,"pc":0}]"#, o, l) }
    }

    unsafe extern "C" fn fake_disassemble(_c: *mut c_void, _s: u32, _p: u32, _sp: *const u8, _sl: usize, _a: u32, _cnt: u32, _h: *const u8, _hl: usize, o: *mut u8, l: usize) -> i64 {
        unsafe { abi::copy_answer(b"[]", o, l) }
    }

    fn make_table(core: &mut FakeCore) -> emusen_session_v1 {
        emusen_session_v1 {
            ctx: (core as *mut FakeCore).cast(),
            info: Some(fake_info),
            spaces: Some(fake_spaces),
            space_read: Some(fake_space_read),
            space_write: Some(fake_space_write),
            processors: Some(fake_processors),
            disassemble: Some(fake_disassemble),
            ..abi::blank_table()
        }
    }

    fn run_cmd(sh: &mut Shell, line: &str) -> String {
        assert_eq!(sh.submit(0, line).unwrap(), Submitted::Done);
        sh.take_output()
    }

    fn run_shell_with_core(core: &mut FakeCore, f: impl FnOnce(&mut Shell, &FakeCore)) {
        let table = make_table(core);
        let mut sh = Shell::new(Config { sessions: false, ..Config::default() });
        sh.add_module(&crate::MODULE).unwrap();
        sh.set_service_table(abi::SERVICE, abi::MAJOR, abi::MINOR, (&table as *const emusen_session_v1).cast()).unwrap();
        f(&mut sh, core);
    }

    #[test]
    fn search_unknown_signed_be_float_undo_and_bitset_sparse_transition() {
        let mut ram = vec![0u8; 300_000];
        ram[0x10..0x12].copy_from_slice(&(-10i16).to_be_bytes());
        ram[0x20..0x22].copy_from_slice(&50i16.to_be_bytes());
        ram[0x100..0x104].copy_from_slice(&12.5f32.to_le_bytes());
        let mut core = FakeCore { ram: RefCell::new(ram), big_endian: true };
        run_shell_with_core(&mut core, |sh, core| {
            let bad_width = run_cmd(sh, "search RAM 1.0 2 --float");
            assert!(bad_width.contains("width 4"), "{bad_width}");

            let res: Value = serde_json::from_str(&run_cmd(sh, "search unknown RAM -w 2 --signed --be --aligned --json")).unwrap();
            assert_eq!(res["total"], 150_000);
            assert_eq!(res["endian"], "big");

            let bad_ref = run_cmd(sh, "search incby not_a_num --unsigned --le --dec");
            assert!(bad_ref.contains("not a decimal"), "{bad_ref}");

            core.ram.borrow_mut()[0x10..0x12].copy_from_slice(&(-5i16).to_be_bytes());
            core.ram.borrow_mut()[0x20..0x22].copy_from_slice(&40i16.to_be_bytes());
            let inc: Value = serde_json::from_str(&run_cmd(sh, "search incby 5 --dec --json")).unwrap();
            assert_eq!(inc["total"], 1);
            assert_eq!(inc["can_undo"], true);
            let list: Value = serde_json::from_str(&run_cmd(sh, "search list 10 --dec --json")).unwrap();
            assert_eq!(list["val_type"], "signed");
            assert_eq!(list["endian"], "big");
            assert_eq!(list["hits"][0]["address"], 0x10);
            assert_eq!(list["hits"][0]["value"], "-5");
            assert_eq!(list["hits"][0]["prev"], "-10");
            assert_eq!(list["hits"][0]["delta"], "+5");

            let flipped: Value = serde_json::from_str(&run_cmd(sh, "search changed --unsigned --le --json")).unwrap();
            assert_eq!(flipped["val_type"], "unsigned");
            assert_eq!(flipped["endian"], "little");

            let undone: Value = serde_json::from_str(&run_cmd(sh, "search undo --json")).unwrap();
            assert_eq!(undone["val_type"], "signed");
            assert_eq!(undone["endian"], "big");
            let undone2: Value = serde_json::from_str(&run_cmd(sh, "search undo --json")).unwrap();
            assert_eq!(undone2["total"], 150_000);
            let dec: Value = serde_json::from_str(&run_cmd(sh, "search decby 10 --dec --json")).unwrap();
            assert_eq!(dec["total"], 1);
            let list2: Value = serde_json::from_str(&run_cmd(sh, "search list 10 --dec --json")).unwrap();
            assert_eq!(list2["hits"][0]["address"], 0x20);

            let gt: Value = serde_json::from_str(&run_cmd(sh, "search gt RAM 10.0 --float --le --json")).unwrap();
            assert_eq!(gt["total"], 1);
            let bet: Value = serde_json::from_str(&run_cmd(sh, "search between 12.4 12.6 --json")).unwrap();
            assert_eq!(bet["total"], 1);
        });
    }

    #[test]
    fn search_dense_bitset_stays_dense_above_threshold_and_undoes_cleanly() {
        let ram = vec![7u8; 300_000];
        let mut core = FakeCore { ram: RefCell::new(ram), big_endian: false };
        run_shell_with_core(&mut core, |sh, core| {
            let init: Value = serde_json::from_str(&run_cmd(sh, "search unknown RAM --json")).unwrap();
            assert_eq!(init["total"], 300_000);
            core.ram.borrow_mut()[42] = 9;
            let chg: Value = serde_json::from_str(&run_cmd(sh, "search changed --json")).unwrap();
            assert_eq!(chg["total"], 1);
            let undone: Value = serde_json::from_str(&run_cmd(sh, "search undo --json")).unwrap();
            assert_eq!(undone["total"], 300_000);
        });
    }

    #[test]
    fn search_pointer_and_rescan_finds_base_plus_offset() {
        let mut ram = vec![0u8; 0x200];
        ram[0x40..0x44].copy_from_slice(&0x8010_0000u32.to_be_bytes());
        ram[0x80..0x84].copy_from_slice(&0x8010_5000u32.to_be_bytes());
        let mut core = FakeCore { ram: RefCell::new(ram), big_endian: true };
        run_shell_with_core(&mut core, |sh, core| {
            let p1: Value = serde_json::from_str(&run_cmd(sh, "search pointer RAM 80100024 --json")).unwrap();
            assert_eq!(p1["total"], 1);
            assert_eq!(p1["hits"][0]["base_addr"], 0x40);
            assert_eq!(p1["hits"][0]["offset"], 0x24);

            core.ram.borrow_mut()[0x40..0x44].copy_from_slice(&0x8012_0000u32.to_be_bytes());
            let p2: Value = serde_json::from_str(&run_cmd(sh, "search pointer rescan 80120024 --json")).unwrap();
            assert_eq!(p2["total"], 1);
            assert_eq!(p2["hits"][0]["base_addr"], 0x40);
            assert_eq!(p2["hits"][0]["offset"], 0x24);

            let oob: Value = serde_json::from_str(&run_cmd(sh, "search pointer RAM 80120024 --base-start 500 --json")).unwrap();
            assert_eq!(oob["total"], 0);
        });
    }

    #[test]
    fn write_mem_memfind_and_writers_json() {
        let mut ram = vec![0u8; 64];
        ram[0x10..0x13].copy_from_slice(&[0xAA, 0xBB, 0xCC]);
        let mut core = FakeCore { ram: RefCell::new(ram), big_endian: true };
        run_shell_with_core(&mut core, |sh, core| {
            let w: Value = serde_json::from_str(&run_cmd(sh, "write RAM 20 999 -w 2 --be --dec --json")).unwrap();
            assert_eq!(w["width"], 2);
            assert_eq!(w["big_endian"], true);
            assert_eq!(&core.ram.borrow()[0x20..0x22], &999u16.to_be_bytes());

            let m: Value = serde_json::from_str(&run_cmd(sh, "mem RAM 20 4 --json")).unwrap();
            assert_eq!(m["rows"][0]["bytes"][0], 0x03);
            assert_eq!(m["rows"][0]["bytes"][1], 0xE7);

            let f: Value = serde_json::from_str(&run_cmd(sh, "memfind RAM 'AA??CC' --json")).unwrap();
            assert_eq!(f["count"], 1);
            assert_eq!(f["hits"][0]["address"], 0x10);

            let wr: Value = serde_json::from_str(&run_cmd(sh, "writers RAM 20 --json")).unwrap();
            assert_eq!(wr["supported"], false);
            assert!(wr["advice"].as_str().unwrap().contains("watch add RAM"));
        });
    }
}
