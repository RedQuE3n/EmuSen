//! The C layer over the packs' debugger code: the disassemblers, the NES's board names and the consoles' views, for the
//! C# runtime's facades (EmuSen_Backend.md §13.5). A view reads memory in rounds: the host passes the bytes it has read
//! so far, the view asks for what it lacks, and the host reads that and asks again. Nothing is decided here.

use crate::{bytes, fail, given, put, required, status};
use emusen_dotnet::json::{self, Value};
use emusen_systems::debug::{Memory, Reference, Space, Tape};
use emusen_systems::{gb, n64, nes, snes};

fn object(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(k, v)| (json::Text::new(k), v)).collect())
}

fn compact(value: &Value) -> String {
    let mut out = String::new();
    json::write_compact(&mut out, value);
    out
}

fn reference(r: Option<(Reference, i64)>) -> Value {
    r.map_or(Value::Null, |(kind, target)| object(vec![("kind", Value::string(kind.name())), ("target", Value::number(target))]))
}

/// One instruction of `processor` (`6502`, `sm83`, `vr4300` or, since interface 19, `rsp`) from its bytes (a MIPS
/// processor's one big-endian word) at `pc`, as JSON: length, mnemonic, operands and reference (kind and target, or null), the last classified from
/// `operand` for the two 8-bit processors as their C# classifiers took it. ABSENT for another processor.
///
/// # Safety
/// Each pointer must be valid for its length; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_disassemble(processor: *const u8, processor_len: usize, code: *const u8, code_len: usize, pc: i32, operand: i32, out: *mut u8, len: usize) -> i64 {
    let (processor, code) = match unsafe { (required(processor, processor_len), bytes(code, code_len)) } {
        (Ok(p), Ok(c)) => (p, c),
        (Err(s), _) | (_, Err(s)) => return s as i64,
    };
    let b = |i: usize| code.get(i).copied().unwrap_or(0);
    let (length, mnemonic, operands, r) = match processor {
        "6502" => {
            let op = b(0);
            let r = nes::disasm::classify(op, operand).map(|(k, t)| (k, t as i64));
            (nes::disasm::length_of(op), nes::disasm::mnemonic_of(op).to_string(), nes::disasm::format_operand(op, pc, b(1), b(2)), r)
        }
        "sm83" => {
            let op = b(0);
            let r = gb::disasm::classify(op, operand, pc).map(|(k, t)| (k, t as i64));
            (gb::disasm::length_of(op), gb::disasm::mnemonic_of(op, b(1)), gb::disasm::format_operand(op, pc, b(1), b(2)), r)
        }
        "vr4300" => {
            let (mnemonic, operands, r) = n64::disasm::decode(u32::from_be_bytes([b(0), b(1), b(2), b(3)]), pc as u32);
            (4, mnemonic, operands, r.map(|(k, t)| (k, t as i64)))
        }
        "rsp" => {
            let (mnemonic, operands, r) = n64::disasm::decode_rsp(u32::from_be_bytes([b(0), b(1), b(2), b(3)]), pc as u32);
            (4, mnemonic, operands, r.map(|(k, t)| (k, t as i64)))
        }
        _ => return fail(status::ABSENT, format!("no disassembler is named {processor}")) as i64,
    };
    let answer = object(vec![
        ("length", Value::number(length)),
        ("mnemonic", Value::string(mnemonic)),
        ("operands", Value::string(operands)),
        ("reference", reference(r)),
    ]);
    unsafe { put(compact(&answer).as_bytes(), out, len) }
}

/// The name of the board an iNES mapper number means; ABSENT for a number the NES pack names no board for.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_board_name(mapper: i32, out: *mut u8, len: usize) -> i64 {
    match nes::boards::name(mapper) {
        Some(name) => unsafe { put(name.as_bytes(), out, len) },
        None => fail(status::ABSENT, format!("no board is named for mapper {mapper}")) as i64,
    }
}

fn text(v: Option<&Value>) -> Option<&str> {
    v.and_then(Value::as_str)
}

fn int(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Number(n)) => n.parse().ok(),
        _ => None,
    }
}

/// The answer's envelope: the JSON's length as four little-endian bytes, the JSON, then any bytes (a picture's RGBA, a
/// tile's pixels).
fn envelope(answer: Value, payload: &[u8]) -> Vec<u8> {
    let text = compact(&answer);
    let mut out = (text.len() as u32).to_le_bytes().to_vec();
    out.extend_from_slice(text.as_bytes());
    out.extend_from_slice(payload);
    out
}

fn palettes_json(p: Vec<(i32, Vec<(u8, u8, u8)>)>) -> Value {
    let rows = p
        .into_iter()
        .map(|(index, colours)| {
            let colours = colours.into_iter().map(|(r, g, b)| Value::Array(vec![Value::number(r), Value::number(g), Value::number(b)])).collect();
            object(vec![("index", Value::number(index)), ("colors", Value::Array(colours))])
        })
        .collect();
    object(vec![("palettes", Value::Array(rows))])
}

fn vectors_json(v: &[(&str, i32, i32, &str)]) -> Value {
    let rows = v
        .iter()
        .map(|&(name, address, width, mode)| {
            object(vec![("name", Value::string(name)), ("address", Value::number(address)), ("width", Value::number(width)), ("mode", Value::string(mode))])
        })
        .collect();
    object(vec![("vectors", Value::Array(rows))])
}

/// One view of a system's, for a request as JSON: `system` (nes or snes), `view`, `spaces` (each name, size and
/// side_effects) and what the view takes (`space`, `address`, `bpp`, `register`, `name`, `registers` as name and value).
/// `tape` is the bytes the host has read for this request so far, in the order asked. The answer is an envelope: the
/// JSON's length in four little-endian bytes, the JSON, then any bytes. The JSON is `need` (each space, address and
/// length, read in order and the call made again with them appended) or the view's answer. ABSENT for a system with no
/// views or a view it has not; PARSE for a request that is not one.
///
/// # Safety
/// Each pointer must be valid for its length, or null with no length; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_view(request: *const u8, request_len: usize, tape: *const u8, tape_len: usize, out: *mut u8, len: usize) -> i64 {
    let (request, tape) = match unsafe { (given(request, request_len), bytes(tape, tape_len)) } {
        (Ok(r), Ok(t)) => (r, t),
        (Err(s), _) | (_, Err(s)) => return s as i64,
    };
    let Ok(r) = json::parse(request) else { return fail(status::PARSE, "the request is not JSON") as i64 };
    let (Some(system), Some(view)) = (text(r.get("system")), text(r.get("view"))) else { return fail(status::PARSE, "the request names no system or view") as i64 };
    let spaces: Vec<Space> = match r.get("spaces") {
        Some(Value::Array(items)) => items
            .iter()
            .map(|s| Space {
                name: text(s.get("name")).unwrap_or_default().to_string(),
                size: int(s.get("size")).unwrap_or(0) as i32,
                side_effects: s.get("side_effects").and_then(Value::as_bool).unwrap_or(false),
            })
            .collect(),
        _ => Vec::new(),
    };
    let registers: Vec<(String, u64)> = match r.get("registers") {
        Some(Value::Array(items)) => items
            .iter()
            .map(|x| (text(x.get("name")).unwrap_or_default().to_string(), match x.get("value") { Some(Value::Number(n)) => n.parse().unwrap_or(0), _ => 0 }))
            .collect(),
        _ => Vec::new(),
    };
    let space = text(r.get("space")).unwrap_or_default().to_string();
    let address = int(r.get("address")).unwrap_or(0) as i32;
    let bpp = int(r.get("bpp")).unwrap_or(0) as i32;
    let mut memory = Tape::new(spaces, tape);
    let m: &mut dyn Memory = &mut memory;
    let picture = |(rgba, w, h): (Vec<u8>, i32, i32)| (object(vec![("width", Value::number(w)), ("height", Value::number(h))]), rgba);
    let pixels = |p: Result<Vec<u8>, (String, Option<&'static str>)>| match p {
        Ok(p) => (object(vec![("pixels", Value::Bool(true))]), p),
        Err((words, param)) => (object(vec![("refused", Value::string(words)), ("param", param.map_or(Value::Null, Value::string))]), Vec::new()),
    };
    let textual = |t: Option<String>| (object(vec![("text", t.map_or(Value::Null, Value::string))]), Vec::new());
    let (answer, payload) = match (system, view) {
        ("nes", "palettes") => (palettes_json(nes::views::palettes(m)), Vec::new()),
        ("nes", "palette_swatch") => picture(nes::views::palette_swatch(m)),
        ("nes", "tile_sheet") => picture(nes::views::tile_sheet(m)),
        ("nes", "stride") => (object(vec![("stride", Value::number(nes::views::TILEMAP_ENTRY_STRIDE))]), Vec::new()),
        ("nes", "tilemap_entry") => textual(Some(nes::views::decode_tilemap_entry(m, &space, address))),
        ("nes", "tile_pixels") => pixels(nes::views::decode_tile_pixels(m, &space, address, bpp)),
        ("nes", "vectors") => (vectors_json(&nes::views::VECTORS), Vec::new()),
        ("snes", "palettes") => (palettes_json(snes::views::palettes(m)), Vec::new()),
        ("snes", "palette_swatch") => picture(snes::views::palette_swatch(m)),
        ("snes", "tile_sheet") => picture(snes::views::tile_sheet(m)),
        ("snes", "stride") => (object(vec![("stride", Value::number(snes::views::TILEMAP_ENTRY_STRIDE))]), Vec::new()),
        ("snes", "tilemap_entry") => textual(Some(snes::views::decode_tilemap_entry(m, &space, address))),
        ("snes", "tile_pixels") => pixels(snes::views::decode_tile_pixels(m, &space, address, bpp)),
        ("snes", "dma_channels") => {
            let rows = snes::views::dma_channels(m)
                .into_iter()
                .map(|c| {
                    object(vec![
                        ("index", Value::number(c.index)),
                        ("general", Value::Bool(c.general)),
                        ("hdma", Value::Bool(c.hdma)),
                        ("control", Value::number(c.control)),
                        ("destination", Value::number(c.destination)),
                        ("source", Value::number(c.source)),
                        ("size", Value::number(c.size)),
                        ("hdma_active", Value::Bool(c.hdma_active)),
                        ("table", Value::number(c.table)),
                        ("line_counter", Value::number(c.line_counter)),
                        ("indirect", Value::number(c.indirect)),
                    ])
                })
                .collect();
            (object(vec![("channels", Value::Array(rows))]), Vec::new())
        }
        ("snes", "dma_destination") => textual(Some(snes::views::name_dma_destination(int(r.get("register")).unwrap_or(0) as u8))),
        ("snes", "vectors") => (vectors_json(&snes::views::VECTORS), Vec::new()),
        ("snes", "symbol_names") => (object(vec![("names", Value::Array(snes::views::symbol_names().into_iter().map(Value::string).collect()))]), Vec::new()),
        ("snes", "symbol") => (object(vec![("value", snes::views::symbol(text(r.get("name")).unwrap_or_default(), &registers).map_or(Value::Null, Value::number))]), Vec::new()),
        ("snes", "summary") => textual(snes::views::summary(&registers)),
        ("n64", "disassemble") => {
            let rows = n64::views::disassemble(m, &space, address, int(r.get("count")).unwrap_or(0) as i32)
                .into_iter()
                .map(|l| {
                    object(vec![
                        ("address", Value::number(l.address)),
                        ("bytes", Value::Array(l.bytes.iter().map(|&b| Value::number(b)).collect())),
                        ("mnemonic", Value::string(l.mnemonic)),
                        ("operands", Value::string(l.operands)),
                        ("reference", reference(l.reference.map(|(k, t)| (k, t as i64)))),
                    ])
                })
                .collect();
            (object(vec![("instructions", Value::Array(rows))]), Vec::new())
        }
        ("n64", "instruction") => {
            let (mnemonic, operands, r) = n64::views::decode_in(&space, int(r.get("word")).unwrap_or(0) as u32, address as u32);
            (object(vec![("mnemonic", Value::string(mnemonic)), ("operands", Value::string(operands)), ("reference", reference(r.map(|(k, t)| (k, t as i64))))]), Vec::new())
        }
        ("n64", "physical") => {
            let (name, offset, addressable) = n64::views::physical_in(m, address as u32);
            (object(vec![("space", Value::string(name)), ("offset", Value::number(offset)), ("addressable", Value::Bool(addressable))]), Vec::new())
        }
        ("n64", "summary") => {
            let vi: Option<Vec<(String, u64)>> = match r.get("vi") {
                Some(Value::Array(items)) => Some(
                    items.iter().map(|x| (text(x.get("name")).unwrap_or_default().to_string(), match x.get("value") { Some(Value::Number(n)) => n.parse().unwrap_or(0), _ => 0 })).collect(),
                ),
                _ => None,
            };
            textual(n64::views::summary(&registers, vi.as_deref()))
        }
        _ => return fail(status::ABSENT, format!("{system} has no view {view}")) as i64,
    };
    let out_bytes = if memory.wants.is_empty() {
        envelope(answer, &payload)
    } else {
        let wants = memory
            .wants
            .iter()
            .map(|w| object(vec![("space", Value::string(w.space.as_str())), ("address", Value::number(w.address)), ("length", Value::number(w.length))]))
            .collect();
        envelope(object(vec![("need", Value::Array(wants))]), &[])
    };
    unsafe { put(&out_bytes, out, len) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(request: &str, tape: &[u8]) -> (String, Vec<u8>) {
        let n = unsafe { emusen_systems_view(request.as_ptr(), request.len(), tape.as_ptr(), tape.len(), std::ptr::null_mut(), 0) };
        assert!(n > 0, "{n}");
        let mut out = vec![0u8; n as usize];
        unsafe { emusen_systems_view(request.as_ptr(), request.len(), tape.as_ptr(), tape.len(), out.as_mut_ptr(), out.len()) };
        let j = u32::from_le_bytes([out[0], out[1], out[2], out[3]]) as usize;
        (String::from_utf8(out[4..4 + j].to_vec()).unwrap(), out[4 + j..].to_vec())
    }

    #[test]
    fn a_view_asks_for_its_bytes_then_answers_from_them() {
        let request = r#"{"system":"nes","view":"tilemap_entry","spaces":[{"name":"CIRAM","size":2048,"side_effects":false}],"space":"CIRAM","address":33}"#;
        let (first, _) = call(request, &[]);
        assert_eq!(first, r#"{"need":[{"space":"CIRAM","address":33,"length":1},{"space":"CIRAM","address":960,"length":1}]}"#);
        let (second, _) = call(request, &[0x42, 0b0000_0011]);
        assert_eq!(second, r#"{"text":"$42 p3"}"#);
    }

    #[test]
    fn an_instruction_is_disassembled_with_its_reference() {
        let p = "6502";
        let code = [0x8D, 0x00, 0x20];
        let n = unsafe { emusen_systems_disassemble(p.as_ptr(), p.len(), code.as_ptr(), 3, 0x8000, 0x2000, std::ptr::null_mut(), 0) };
        let mut out = vec![0u8; n as usize];
        unsafe { emusen_systems_disassemble(p.as_ptr(), p.len(), code.as_ptr(), 3, 0x8000, 0x2000, out.as_mut_ptr(), out.len()) };
        assert_eq!(std::str::from_utf8(&out).unwrap(), r#"{"length":3,"mnemonic":"STA","operands":"$2000","reference":{"kind":"Write","target":8192}}"#);
    }
}
