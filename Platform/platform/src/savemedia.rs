//! The C layer over emusen-savemedia, the card manager, for the C# `SaveMediaManager` behind `EMUSEN_SERVICES_NATIVE`,
//! and over Galaxia's medium lock, for the C# `MediumLock` behind Galaxia's own switch (EmuSen_Backend.md §15.2).

use crate::{fail, fail_io, given, put, report, required, status, text};
use emusen_dotnet::json::{self, Value};
use emusen_galaxia::medium_lock;
use emusen_savemedia::{Card, Manager, Media, Medium, Outcome, Thrown};

/// The host's answer for each file a request names: the medium its extension makes it, or none.
struct Given(Vec<(String, Option<String>)>);

impl Media for Given {
    fn medium_of(&self, file: &str) -> Option<Medium> {
        let id = self.0.iter().rev().find(|(f, _)| f == file)?.1.as_deref()?;
        Medium::by_id(id)
    }
}

fn media_from(value: Option<&Value>) -> Result<Given, i32> {
    match value {
        Some(Value::Object(members)) => Ok(Given(members.iter().map(|(k, v)| (k.text.clone(), v.as_str().map(str::to_string))).collect())),
        _ => Err(fail(status::PARSE, "the request has no media")),
    }
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str, i32> {
    value.get(name).and_then(Value::as_str).ok_or_else(|| fail(status::PARSE, format!("the request has no {name}")))
}

fn object(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(k, v)| (json::Text::new(k), v)).collect())
}

fn outcome_json(o: &Outcome) -> Value {
    object(vec![("done", Value::Bool(o.done)), ("words", Value::string(o.words.as_str()))])
}

fn card_json(card: Option<&Card>) -> Result<Value, i32> {
    let Some(card) = card else { return Ok(object(vec![("card", Value::Null)])) };
    let listing = match &card.listing {
        Some(l) => json::parse(&emusen_systems::media::listing_json(l)).map_err(|e| fail(status::PARSE, e.to_string()))?,
        None => Value::Null,
    };
    Ok(object(vec![("card", object(vec![("exists", Value::Bool(card.exists)), ("length", Value::number(card.length)), ("listing", listing)]))]))
}

fn thrown(t: Thrown) -> i32 {
    match t {
        Thrown::Range => fail(status::BAD_ARGUMENT, "the directory reaches outside the image"),
        Thrown::Io(e) => fail_io(&e),
    }
}

fn run(r: &Value) -> Result<String, i32> {
    let media = media_from(r.get("media"))?;
    let m = Manager::new(&media);
    let op = field(r, "op")?;
    let answer = (|| -> Result<Value, i32> {
        match op {
            "open" => card_json(m.open(field(r, "file")?).map_err(thrown)?.as_ref()),
            "copy" => Ok(outcome_json(&m.copy_save(field(r, "from")?, field(r, "save")?, field(r, "to")?).map_err(thrown)?)),
            "move" => Ok(outcome_json(&m.move_save(field(r, "from")?, field(r, "save")?, field(r, "to")?).map_err(thrown)?)),
            "delete" => Ok(outcome_json(&m.delete_save(field(r, "card")?, field(r, "save")?).map_err(thrown)?)),
            "import" => {
                let from = match r.get("from") {
                    Some(Value::Array(items)) => items.iter().map(|v| v.as_str().map(str::to_string)).collect::<Option<Vec<_>>>(),
                    _ => None,
                }
                .ok_or_else(|| fail(status::PARSE, "the request has no from"))?;
                Ok(outcome_json(&m.import(field(r, "to")?, &from).map_err(thrown)?))
            }
            "new" => Ok(outcome_json(&m.new_card(field(r, "file")?))),
            "duplicate" => Ok(outcome_json(&m.duplicate(field(r, "from")?, field(r, "to")?))),
            "rename" => Ok(outcome_json(&m.rename(field(r, "from")?, field(r, "to")?).map_err(thrown)?)),
            _ => Err(fail(status::BAD_ARGUMENT, format!("no operation {op}"))),
        }
    })();
    for line in m.said() {
        report(line);
    }
    let mut out = String::new();
    json::write_compact(&mut out, &answer?);
    Ok(out)
}

/// One operation of the card manager. The request: op (open, copy, move, delete, import, new, duplicate, rename), its
/// files and save, and media, each file the request names with the medium id its extension makes it or null. Out:
/// open's card (exists, length, listing) or null, or an outcome (done, words). BAD_ARGUMENT for a damaged directory
/// reaching outside its image; NOT_FOUND, ACCESS or IO for a rename the system refused.
///
/// # Safety
/// `request` must be valid for `request_len` bytes; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_savemedia_op(request_data: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    let answer = || -> Result<String, i32> {
        let r = json::parse(unsafe { given(request_data, request_len) }?).map_err(|e| fail(status::PARSE, e.to_string()))?;
        run(&r)
    };
    match answer() {
        Ok(text) => unsafe { put(text.as_bytes(), out, len) },
        Err(s) => s as i64,
    }
}

/// `MediumLock.TryTake`: a handle above zero, or zero where another program holds the file or this one holds it unshared.
///
/// # Safety
/// Each pointer must be valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_medium_lock_take(file: *const u8, file_len: usize, holder: *const u8, holder_len: usize, shared: i32) -> u64 {
    let (Ok(file), Ok(holder)) = (unsafe { required(file, file_len) }, unsafe { text(holder, holder_len) }) else { return 0 };
    match medium_lock::try_take(file, holder.unwrap_or(""), shared != 0) {
        Some(lock) => {
            let id = lock.id;
            std::mem::forget(lock);
            id
        }
        None => 0,
    }
}

/// Lets go of a handle; 1 when it was held, 0 when it was let go already.
#[unsafe(no_mangle)]
pub extern "C" fn emusen_galaxia_medium_lock_release(handle: u64) -> i32 {
    i32::from(medium_lock::release(handle))
}

/// `MediumLock.HolderOf`: the holder's words, by the length-query idiom; ABSENT when no one holds the file.
///
/// # Safety
/// `file` must be valid for `file_len` bytes; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_medium_lock_holder(file: *const u8, file_len: usize, out: *mut u8, len: usize) -> i64 {
    let file = match unsafe { required(file, file_len) } {
        Ok(f) => f,
        Err(s) => return s as i64,
    };
    match medium_lock::holder_of(file) {
        Some(holder) => unsafe { put(holder.as_bytes(), out, len) },
        None => fail(status::ABSENT, "no one holds the file") as i64,
    }
}
