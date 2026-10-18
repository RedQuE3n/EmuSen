//! The C layer over emusen-systems: the table as JSON, and the save media's format modules, for the C# runtime's
//! `SystemTable` and `SystemCode` facades (EmuSen_Backend.md §5.4, §12, §13). Nothing is decided here.

use crate::{bytes, fail, given, put, required, status};
use emusen_dotnet::json::{self, Value};
use emusen_systems::cheats;
use emusen_systems::probe::{self, Candidate, Head, Recognition};
use emusen_systems::media::{self, Fault};

/// The table as `emusen_systems::to_json` writes it, by the length-query idiom.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_table(out: *mut u8, len: usize) -> i64 {
    match emusen_systems::table_json() {
        Ok(text) => unsafe { put(text.as_bytes(), out, len) },
        Err(e) => fail(status::PARSE, e.to_string()) as i64,
    }
}

/// Every save medium's module facts as JSON, by the length-query idiom.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_media(out: *mut u8, len: usize) -> i64 {
    unsafe { put(media::modules_json().as_bytes(), out, len) }
}

fn medium(id: &str) -> Result<&'static str, i32> {
    media::module(id).map(|m| m.id).ok_or_else(|| fail(status::ABSENT, format!("no module reads the medium {id}")))
}

fn edited(result: Option<Result<Vec<u8>, Fault>>, out: *mut u8, len: usize) -> i64 {
    match result {
        Some(Ok(image)) => unsafe { put(&image, out, len) },
        Some(Err(Fault::Refused(words))) => fail(status::REFUSED, words) as i64,
        Some(Err(Fault::Outside)) => fail(status::BAD_ARGUMENT, "the directory reaches outside the image") as i64,
        None => fail(status::ABSENT, "no module reads the medium") as i64,
    }
}

/// A medium's listing as JSON, by the length-query idiom.
///
/// # Safety
/// Each pointer must be valid for its length, or null; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_media_read(id: *const u8, id_len: usize, image: *const u8, image_len: usize, out: *mut u8, len: usize) -> i64 {
    let (id, image) = match unsafe { (required(id, id_len), bytes(image, image_len)) } {
        (Ok(id), Ok(image)) => (id, image),
        (Err(s), _) | (_, Err(s)) => return s as i64,
    };
    let id = match medium(id) {
        Ok(id) => id,
        Err(s) => return s as i64,
    };
    match media::read(id, image) {
        Some(Ok(listing)) => unsafe { put(media::listing_json(&listing).as_bytes(), out, len) },
        Some(Err(_)) => fail(status::BAD_ARGUMENT, "the directory reaches outside the image") as i64,
        None => fail(status::ABSENT, "no module reads the medium") as i64,
    }
}

/// `to` with the save named `save` on `from` copied onto it, by the length-query idiom; REFUSED with the words.
///
/// # Safety
/// Each pointer must be valid for its length, or null; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_media_copy(
    id: *const u8,
    id_len: usize,
    from: *const u8,
    from_len: usize,
    save: *const u8,
    save_len: usize,
    to: *const u8,
    to_len: usize,
    out: *mut u8,
    len: usize,
) -> i64 {
    let (id, from, save, to) = match unsafe { (required(id, id_len), bytes(from, from_len), given(save, save_len), bytes(to, to_len)) } {
        (Ok(id), Ok(from), Ok(save), Ok(to)) => (id, from, save, to),
        (Err(s), ..) | (_, Err(s), ..) | (_, _, Err(s), _) | (.., Err(s)) => return s as i64,
    };
    match medium(id) {
        Ok(id) => edited(media::copy(id, from, save, to), out, len),
        Err(s) => s as i64,
    }
}

/// `image` with the save named `save` deleted, by the length-query idiom; REFUSED with the words.
///
/// # Safety
/// Each pointer must be valid for its length, or null; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_media_delete(id: *const u8, id_len: usize, image: *const u8, image_len: usize, save: *const u8, save_len: usize, out: *mut u8, len: usize) -> i64 {
    let (id, image, save) = match unsafe { (required(id, id_len), bytes(image, image_len), given(save, save_len)) } {
        (Ok(id), Ok(image), Ok(save)) => (id, image, save),
        (Err(s), ..) | (_, Err(s), _) | (.., Err(s)) => return s as i64,
    };
    match medium(id) {
        Ok(id) => edited(media::delete(id, image, save), out, len),
        Err(s) => s as i64,
    }
}

/// A formatted blank of the medium, by the length-query idiom.
///
/// # Safety
/// `id` must be valid for `id_len` bytes; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_media_format(id: *const u8, id_len: usize, out: *mut u8, len: usize) -> i64 {
    let id = match unsafe { required(id, id_len) }.and_then(medium) {
        Ok(id) => id,
        Err(s) => return s as i64,
    };
    edited(media::format(id).map(Ok), out, len)
}

/// Every cheat codec and each system's pair as JSON, by the length-query idiom.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_cheat_codecs(out: *mut u8, len: usize) -> i64 {
    unsafe { put(cheats::codecs_json().as_bytes(), out, len) }
}

/// A code's every answer from the codec `codec` as JSON, by the length-query idiom; ABSENT for a codec there is not.
///
/// # Safety
/// Each pointer must be valid for its length; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_cheat_decode(codec: *const u8, codec_len: usize, code: *const u8, code_len: usize, out: *mut u8, len: usize) -> i64 {
    let (codec, code) = match unsafe { (required(codec, codec_len), given(code, code_len)) } {
        (Ok(codec), Ok(code)) => (codec, code),
        (Err(s), _) | (_, Err(s)) => return s as i64,
    };
    match cheats::decode(codec, code) {
        Some(answer) => unsafe { put(cheats::answer_json(&answer).as_bytes(), out, len) },
        None => fail(status::ABSENT, format!("no codec is named {codec}")) as i64,
    }
}

/// The Game Genie code of the SNES's or the Genesis's that patches `value` at `address`, by the length-query idiom;
/// ABSENT for a codec with no encoder.
///
/// # Safety
/// `codec` must be valid for `codec_len` bytes; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_cheat_encode(codec: *const u8, codec_len: usize, address: i32, value: u32, out: *mut u8, len: usize) -> i64 {
    let codec = match unsafe { required(codec, codec_len) } {
        Ok(codec) => codec,
        Err(s) => return s as i64,
    };
    let code = match codec {
        "snes.gg" => emusen_systems::snes::cheats::encode_game_genie(address, value as u8),
        "md.gg" => emusen_systems::genesis::cheats::encode_game_genie(address, value as u16),
        _ => return fail(status::ABSENT, format!("the codec {codec} has no encoder")) as i64,
    };
    unsafe { put(code.as_bytes(), out, len) }
}

/// Each system's code as JSON: whether it has a recogniser, an OpenVGDB rule and a header reader, by id in the table's order.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_code(out: *mut u8, len: usize) -> i64 {
    let rows = emusen_systems::table().unwrap_or(&[]).iter().map(|s| {
        object(vec![
            ("id", Value::string(s.id.as_str())),
            ("recognises", Value::Bool(probe::recognises(&s.id))),
            ("openvgdb", Value::Bool(emusen_systems::openvgdb::transforms(&s.id))),
            ("header", Value::Bool(matches!(s.id.as_str(), "nes" | "gb" | "gbc"))),
        ])
    });
    unsafe { put(compact(&Value::Array(rows.collect())).as_bytes(), out, len) }
}

fn object(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(k, v)| (json::Text::new(k), v)).collect())
}

fn compact(value: &Value) -> String {
    let mut out = String::new();
    json::write_compact(&mut out, value);
    out
}

fn recognition_json(r: Option<&Recognition>) -> Value {
    r.map_or(Value::Null, |r| {
        object(vec![
            ("system", Value::string(r.system.as_str())),
            ("serial", r.serial.as_deref().map_or(Value::Null, Value::string)),
            ("region", r.region.as_deref().map_or(Value::Null, Value::string)),
        ])
    })
}

/// A system's recogniser over the first bytes of a file, the host having asked for `asked` of them: its claim as JSON,
/// or `null`; or 0 with `need` set to the bytes it must read first. ABSENT for a system not in the table.
///
/// # Safety
/// Each pointer must be valid for its length; `need` must be valid; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_recognise(system: *const u8, system_len: usize, head: *const u8, head_len: usize, asked: u64, need: *mut u64, out: *mut u8, len: usize) -> i64 {
    let (system, head) = match unsafe { (required(system, system_len), bytes(head, head_len)) } {
        (Ok(s), Ok(h)) => (s, h),
        (Err(s), _) | (_, Err(s)) => return s as i64,
    };
    if need.is_null() {
        return fail(status::NULL, "need is null") as i64;
    }
    if emusen_systems::find(system).is_none() {
        return fail(status::ABSENT, format!("no system is named {system}")) as i64;
    }
    match probe::recognise(system, &Head { bytes: head, asked: asked as usize }) {
        Ok(claim) => {
            unsafe { *need = 0 };
            unsafe { put(compact(&recognition_json(claim.as_ref())).as_bytes(), out, len) }
        }
        Err(probe::Need(n)) => {
            unsafe { *need = n as u64 };
            0
        }
    }
}

fn text_of(v: Option<&Value>) -> Option<String> {
    v.and_then(Value::as_str).map(str::to_string)
}

/// The decision among candidates: the request as JSON, `file`, `folder` (or null), `candidates` (each `id`, `name`,
/// `esde`) and `claims` (each `at`, the candidate's place, with `system`, `serial` and `region`); the answer as JSON,
/// `system` (an id or null), `recognition` and `reason`. PARSE for a request that is not one.
///
/// # Safety
/// `request` must be valid for `request_len` bytes; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_route(request: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    let text = match unsafe { required(request, request_len) } {
        Ok(t) => t,
        Err(s) => return s as i64,
    };
    let Ok(r) = json::parse(text) else { return fail(status::PARSE, "the request is not JSON") as i64 };
    let (Some(file), Some(Value::Array(cs)), Some(Value::Array(ks))) = (r.get("file").and_then(Value::as_str), r.get("candidates"), r.get("claims")) else {
        return fail(status::PARSE, "the request lacks file, candidates or claims") as i64;
    };
    let folder = text_of(r.get("folder"));
    let fields: Vec<(String, String, String)> = cs.iter().map(|c| (text_of(c.get("id")).unwrap_or_default(), text_of(c.get("name")).unwrap_or_default(), text_of(c.get("esde")).unwrap_or_default())).collect();
    let candidates: Vec<Candidate> = fields.iter().map(|(id, name, esde)| Candidate { id, name, esde_system: esde }).collect();
    let mut claims = Vec::new();
    for k in ks {
        let at = match k.get("at") {
            Some(Value::Number(n)) => n.parse::<usize>().ok().filter(|&a| a < candidates.len()),
            _ => None,
        };
        let Some(at) = at else { return fail(status::PARSE, "a claim names no candidate") as i64 };
        claims.push((at, Recognition { system: text_of(k.get("system")).unwrap_or_default(), serial: text_of(k.get("serial")), region: text_of(k.get("region")) }));
    }
    let routed = probe::route(file, folder.as_deref(), &candidates, &claims);
    let answer = object(vec![
        ("system", routed.system.as_deref().map_or(Value::Null, Value::string)),
        ("recognition", recognition_json(routed.recognition.as_ref())),
        ("reason", routed.reason.as_deref().map_or(Value::Null, Value::string)),
    ]);
    unsafe { put(compact(&answer).as_bytes(), out, len) }
}

/// The bytes of a file OpenVGDB hashed for the system, by the length-query idiom; ABSENT where they are the file's own.
///
/// # Safety
/// Each pointer must be valid for its length; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_openvgdb(system: *const u8, system_len: usize, file: *const u8, file_len: usize, out: *mut u8, len: usize) -> i64 {
    let (system, file) = match unsafe { (required(system, system_len), bytes(file, file_len)) } {
        (Ok(s), Ok(f)) => (s, f),
        (Err(s), _) | (_, Err(s)) => return s as i64,
    };
    match emusen_systems::openvgdb::bytes(system, file) {
        Some(b) => unsafe { put(&b, out, len) },
        None => fail(status::ABSENT, "the file's own bytes") as i64,
    }
}

/// A cartridge's header as JSON from the first bytes of an image of `length` bytes (a reader reads within its first
/// 4 KiB): the NES's or the Game Boy's fields, or `refused` (`invalid` or `unsupported`) with `words`. ABSENT for a
/// system with no header reader.
///
/// # Safety
/// Each pointer must be valid for its length; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_header(system: *const u8, system_len: usize, head: *const u8, head_len: usize, length: u64, out: *mut u8, len: usize) -> i64 {
    let (system, head) = match unsafe { (required(system, system_len), bytes(head, head_len)) } {
        (Ok(s), Ok(h)) => (s, h),
        (Err(s), _) | (_, Err(s)) => return s as i64,
    };
    let refused = |kind: &str, words: String| object(vec![("refused", Value::string(kind)), ("words", Value::string(words))]);
    let answer = match system {
        "nes" => match emusen_systems::nes::header::read(head, length as usize) {
            Ok(h) => object(vec![
                ("mapper", Value::number(h.mapper)),
                ("prg_banks", Value::number(h.prg_banks)),
                ("chr_banks", Value::number(h.chr_banks)),
                ("battery", Value::Bool(h.battery)),
                ("trainer", Value::Bool(h.trainer)),
                ("nes20", Value::Bool(h.nes20)),
                ("trust", Value::number(h.trust)),
                ("mirroring", Value::number(h.mirroring)),
            ]),
            Err(words) => refused("invalid", words),
        },
        "gb" | "gbc" => match emusen_systems::gb::cartridge::read(head, length as usize) {
            Ok(c) => object(vec![
                ("type", Value::number(c.kind)),
                ("board", Value::string(c.board)),
                ("rom_banks", Value::number(c.rom_banks)),
                ("ram_length", Value::number(c.ram_length)),
                ("battery", Value::Bool(c.battery)),
                ("timer", Value::Bool(c.timer)),
                ("rumble", Value::Bool(c.rumble)),
                ("cgb_flag", Value::number(c.cgb_flag)),
            ]),
            Err(emusen_systems::gb::cartridge::Refusal::Invalid(w)) => refused("invalid", w),
        },
        _ => return fail(status::ABSENT, format!("{system} has no header reader")) as i64,
    };
    unsafe { put(compact(&answer).as_bytes(), out, len) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_comes_through_the_c_layer_whole() {
        let n = unsafe { emusen_systems_table(std::ptr::null_mut(), 0) };
        assert!(n > 0);
        let mut small = vec![0u8; n as usize - 1];
        assert_eq!(unsafe { emusen_systems_table(small.as_mut_ptr(), small.len()) }, n);
        assert!(small.iter().all(|&b| b == 0), "a buffer too small is not written");
        let mut out = vec![0u8; n as usize];
        assert_eq!(unsafe { emusen_systems_table(out.as_mut_ptr(), out.len()) }, n);
        assert_eq!(std::str::from_utf8(&out).unwrap(), emusen_systems::table_json().unwrap());
    }

    fn call(f: impl Fn(*mut u8, usize) -> i64) -> Result<Vec<u8>, (i64, String)> {
        let n = f(std::ptr::null_mut(), 0);
        if n < 0 {
            let mut words = vec![0u8; 512];
            let w = unsafe { crate::emusen_platform_last_error(words.as_mut_ptr(), words.len()) };
            return Err((n, String::from_utf8_lossy(&words[..w as usize]).into_owned()));
        }
        let mut out = vec![0u8; n as usize];
        assert_eq!(f(out.as_mut_ptr(), out.len()), n);
        Ok(out)
    }

    #[test]
    fn a_card_is_formatted_read_and_refused_through_the_c_layer() {
        let id = "psx.card";
        let blank = call(|o, l| unsafe { emusen_systems_media_format(id.as_ptr(), id.len(), o, l) }).unwrap();
        assert_eq!(blank.len(), 128 * 1024);
        let listing = call(|o, l| unsafe { emusen_systems_media_read(id.as_ptr(), id.len(), blank.as_ptr(), blank.len(), o, l) }).unwrap();
        assert_eq!(std::str::from_utf8(&listing).unwrap(), r#"{"formatted":true,"saves":[],"free":15,"problem":null}"#);
        let save = "BASLUS-00001";
        let refused = call(|o, l| unsafe { emusen_systems_media_delete(id.as_ptr(), id.len(), blank.as_ptr(), blank.len(), save.as_ptr(), save.len(), o, l) });
        assert_eq!(refused, Err((status::REFUSED as i64, "no save named BASLUS-00001 is on the card".into())));
        let nameless = call(|o, l| unsafe { emusen_systems_media_delete(id.as_ptr(), id.len(), blank.as_ptr(), blank.len(), std::ptr::null(), 0, o, l) });
        assert_eq!(nameless, Err((status::REFUSED as i64, "no save named  is on the card".into())), "an empty name may come as a null");
        let empty = call(|o, l| unsafe { emusen_systems_media_read(id.as_ptr(), id.len(), std::ptr::null(), 0, o, l) }).unwrap();
        assert!(std::str::from_utf8(&empty).unwrap().contains("this file is 0"));
        let unknown = "snes.sram";
        assert_eq!(call(|o, l| unsafe { emusen_systems_media_format(unknown.as_ptr(), unknown.len(), o, l) }).unwrap_err().0, status::ABSENT as i64);
        assert!(call(|o, l| unsafe { emusen_systems_media(o, l) }).unwrap().starts_with(br#"[{"id":"psx.card","length":131072,"capacity":15,"unit":"block"}"#));
    }
}
