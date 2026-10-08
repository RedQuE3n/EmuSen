//! Galaxia's C layer, second half: config files, the models, the cheat lists, the error log and the suggestion text.

use crate::{diagnostics_rewind, diagnostics_waiting, fail, join_nul, put, report, required, status, text};
use emusen_galaxia::config::{self, Loaded};
use emusen_galaxia::dotnet_path::Style;
use emusen_galaxia::models::{self, Model};
use emusen_galaxia::{cheats, error_log, json, model, suggestion, tree};
use std::time::{Duration, SystemTime};

/// `EMUSEN_GALAXIA_MODEL_UPGRADE`: apply the model's own upgrade to what was read or made.
pub const UPGRADE: u32 = 1;

/// What `emusen_galaxia_number_format` formats.
pub mod number {
    pub const DOUBLE: u32 = 0;
    pub const SINGLE: u32 = 1;
}

/// What `emusen_galaxia_suggest` answers with.
pub mod suggest {
    pub const HINT: u32 = 0;
    pub const NEAREST: u32 = 1;
}

fn answer(result: Result<String, i32>, out: *mut u8, len: usize) -> i64 {
    match result {
        Ok(text) => unsafe { put(text.as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

fn model_of(id: u32) -> Result<&'static Model, i32> {
    models::model(id).ok_or_else(|| fail(status::BAD_ARGUMENT, format!("no model {id}")))
}

fn flag(result: Result<bool, i32>) -> i32 {
    match result {
        Ok(answer) => i32::from(answer),
        Err(status) => status,
    }
}

/// Deletes a config file: 1 when it is gone afterwards, 0 when its folder is not there or it would not go.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_config_delete(category: *const u8, category_len: usize, file: *const u8, file_len: usize) -> i32 {
    flag((|| Ok(config::delete(&tree::tree(), unsafe { text(category, category_len) }?, unsafe { required(file, file_len) }?)))())
}

/// A config file's text, decoded as .NET decodes a file, copied in first from where it sat before Galaxia when it is only there. `ABSENT` for no file; `IO` with the system's words for one that will not read.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_config_read(category: *const u8, category_len: usize, file: *const u8, file_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let (category, file) = (unsafe { text(category, category_len) }?, unsafe { required(file, file_len) }?);
        config::read_text(&tree::tree(), category, file).map_err(|words| fail(status::IO, words))?.ok_or(status::ABSENT)
    };
    answer(result(), out, len)
}

/// Writes text as a config file through a temp file and a rename. 0, or `IO`; nothing is reported, as C#'s save reports nothing. Null text fails after the folder is made.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_config_write(category: *const u8, category_len: usize, file: *const u8, file_len: usize, contents: *const u8, len: usize) -> i32 {
    let result = || -> Result<(), i32> {
        let (category, file) = (unsafe { text(category, category_len) }?, unsafe { required(file, file_len) }?);
        let contents = unsafe { text(contents, len) }?;
        if config::write_text(&tree::tree(), category, file, contents) { Ok(()) } else { Err(fail(status::IO, "The file could not be written.")) }
    };
    result().map_or_else(|status| status, |()| 0)
}

/// A model's file as its bound document, in JSON a reader binds again: every settable field, `null` for a file whose document is null. `ABSENT` for no file; `PARSE` or `IO` with the words to report for one that will not load.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_model_load(model: u32, category: *const u8, category_len: usize, file: *const u8, file_len: usize, flags: u32, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let model = model_of(model)?;
        let (category, file) = (unsafe { text(category, category_len) }?, unsafe { required(file, file_len) }?);
        match config::load(&tree::tree(), model, category, file, flags & UPGRADE != 0).map_err(|words| fail(status::PARSE, words))? {
            Loaded::Missing => Err(status::ABSENT),
            Loaded::Null => Ok("null".to_string()),
            Loaded::Document(document) => Ok(model::transport(&document)),
        }
    };
    answer(result(), out, len)
}

/// Saves a model's document, given as JSON, as its file: the folder made, the bytes .NET would write put in place by a rename. A null document fails after the folder is made, as a document C# could not serialize does.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_model_save(model: u32, category: *const u8, category_len: usize, file: *const u8, file_len: usize, document: *const u8, len: usize) -> i32 {
    let result = || -> Result<(), i32> {
        let model = model_of(model)?;
        let (category, file) = (unsafe { text(category, category_len) }?, unsafe { required(file, file_len) }?);
        let document = unsafe { text(document, len) }?;
        config::save(&tree::tree(), model, category, file, document).map_err(|words| fail(status::IO, words))
    };
    result().map_or_else(|status| status, |()| 0)
}

/// A new instance of a model as JSON, upgraded when asked.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_model_new(model: u32, flags: u32, out: *mut u8, len: usize) -> i64 {
    answer(model_of(model).map(|model| model::transport(&config::new_instance(model, flags & UPGRADE != 0))), out, len)
}

/// Text bound to a model with no file involved: the bound document as JSON, `null` for a document that is null, or `PARSE` with the words.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_model_bind(model: u32, document: *const u8, document_len: usize, flags: u32, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let (model, document) = (model_of(model)?, unsafe { required(document, document_len) }?);
        let bound = config::bind_text(model, document, flags & UPGRADE != 0).map_err(|words| fail(status::PARSE, words))?;
        Ok(bound.map_or_else(|| "null".to_string(), |bound| model::transport(&bound)))
    };
    answer(result(), out, len)
}

/// A model's document, given as JSON, as the bytes of its file; `PARSE` with the words for one that cannot be written.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_model_format(model: u32, document: *const u8, document_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let (model, document) = (model_of(model)?, unsafe { required(document, document_len) }?);
        match config::bind_text(model, document, false).map_err(|words| fail(status::PARSE, words))? {
            Some(bound) => model::format(model.schema, &bound).map_err(|words| fail(status::PARSE, words)),
            None => Ok("null".to_string()),
        }
    };
    answer(result(), out, len)
}

/// A model's fields, one to a line, for a host to hold its own classes against.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_model_schema(model: u32, out: *mut u8, len: usize) -> i64 {
    answer(model_of(model).map(|model| model::describe(model.schema)), out, len)
}

/// A model read from any path, as JSON. `ABSENT` for a file that is missing, unreadable or will not bind, of which nothing is said.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_model_load_from(model: u32, file: *const u8, file_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let (model, file) = (model_of(model)?, unsafe { required(file, file_len) }?);
        config::load_from(model, file).map(|document| model::transport(&document)).ok_or(status::ABSENT)
    };
    answer(result(), out, len)
}

/// A model's document written straight to any path, its folder made first; no temp file, as C# writes a file the player picked.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_model_save_to(model: u32, file: *const u8, file_len: usize, document: *const u8, len: usize) -> i32 {
    let result = || -> Result<(), i32> {
        let (model, file) = (model_of(model)?, unsafe { required(file, file_len) }?);
        let document = unsafe { text(document, len) }?;
        if config::save_to(model, file, document) { Ok(()) } else { Err(fail(status::IO, "The file could not be written.")) }
    };
    result().map_or_else(|status| status, |()| 0)
}

/// A file's bytes as .NET's `File.ReadAllText` decodes them, for a test to hold the decoding to .NET's.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_text_decode(bytes: *const u8, bytes_len: usize, out: *mut u8, len: usize) -> i64 {
    let bytes = if bytes.is_null() || bytes_len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(bytes, bytes_len) } };
    answer(Ok(json::decode(bytes)), out, len)
}

/// A number as .NET writes it into JSON, from its bits: a double's 64, or a float's in the low 32. `ABSENT` for a value JSON has no number for.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_number_format(kind: u32, bits: u64, out: *mut u8, len: usize) -> i64 {
    let formatted = match kind {
        number::DOUBLE => json::format_f64(f64::from_bits(bits)),
        number::SINGLE => json::format_f32(f32::from_bits(bits as u32)),
        _ => return fail(status::BAD_ARGUMENT, format!("no number kind {kind}")) as i64,
    };
    answer(formatted.ok_or(status::ABSENT), out, len)
}

/// The saved cheat lists by name, in .NET's caseless order, each followed by a NUL.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_cheat_names(out: *mut u8, len: usize) -> i64 {
    unsafe { put(&join_nul(&cheats::names(&tree::tree())), out, len) }
}

/// Whether a cheat list may have this name: 1 or 0. A null name is not one.
///
/// # Safety
/// `name` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_cheat_name_valid(name: *const u8, len: usize) -> i32 {
    flag(unsafe { text(name, len) }.map(|name| cheats::is_valid_name(Style::HOST, name)))
}

/// The error log's folder: the one given, else the `LogDirectory` setting when it can be made, else the default. A settings file that will not load is reported.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_log_root(directory: *const u8, directory_len: usize, out: *mut u8, len: usize) -> i64 {
    let directory = match unsafe { text(directory, directory_len) } {
        Ok(directory) => directory,
        Err(status) => return status as i64,
    };
    let waiting = diagnostics_waiting();
    let (root, said) = error_log::root(&tree::tree(), directory);
    said.into_iter().for_each(report);
    if out.is_null() || root.len() > len {
        // The caller comes back with room, and the settings are read and reported then.
        diagnostics_rewind(waiting);
    }
    unsafe { put(root.as_bytes(), out, len) }
}

/// Whether a folder is named and can be made: 1 or 0. It is made by the asking.
///
/// # Safety
/// `directory` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_log_usable(directory: *const u8, len: usize) -> i32 {
    flag(unsafe { text(directory, len) }.map(error_log::usable))
}

/// A day's log file in a root: `<root>/emusen_<day>.log`, the day as the caller formatted it.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_log_path(root: *const u8, root_len: usize, day: *const u8, day_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> { Ok(error_log::path_for(unsafe { required(root, root_len) }?, unsafe { required(day, day_len) }?)) };
    answer(result(), out, len)
}

/// One log entry as text: its line, its context on a second when there is any, and a fault's text indented beneath. The time is the caller's own text.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_log_format(
    stamp: *const u8,
    stamp_len: usize,
    level: *const u8,
    level_len: usize,
    area: *const u8,
    area_len: usize,
    message: *const u8,
    message_len: usize,
    context: *const u8,
    context_len: usize,
    fault: *const u8,
    fault_len: usize,
    out: *mut u8,
    len: usize,
) -> i64 {
    let result = || -> Result<String, i32> {
        let (stamp, level) = (unsafe { required(stamp, stamp_len) }?, unsafe { required(level, level_len) }?);
        // A null area is written as nothing between its brackets; a null message is no entry, as in C#.
        let (area, message) = (unsafe { text(area, area_len) }?.unwrap_or(""), unsafe { required(message, message_len) }?);
        let (context, fault) = (unsafe { text(context, context_len) }?, unsafe { text(fault, fault_len) }?);
        Ok(error_log::format_entry(stamp, level, area, message, context, fault))
    };
    answer(result(), out, len)
}

/// Appends an entry to a day's file, pruning the folder first if this process has not: 1 when it was written, 0 when the day is full or it could not be. `now` is milliseconds since 1970, by which a file's age is judged. A null entry is one the caller could not put into UTF-8, and fails at the write.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_log_append(file: *const u8, file_len: usize, stamp: *const u8, stamp_len: usize, now_unix_ms: i64, entry: *const u8, len: usize) -> i32 {
    let result = || -> Result<bool, i32> {
        let (file, stamp, entry) = (unsafe { required(file, file_len) }?, unsafe { required(stamp, stamp_len) }?, unsafe { text(entry, len) }?);
        let now = SystemTime::UNIX_EPOCH + Duration::from_millis(now_unix_ms.max(0) as u64);
        Ok(error_log::append_to(file, stamp, now, entry))
    };
    flag(result())
}

/// Lets the next entry prune again; for a test.
#[unsafe(no_mangle)]
pub extern "C" fn emusen_galaxia_log_reset() -> i32 {
    error_log::reset();
    0
}

/// How far apart two names are: edits, a swap of neighbours counting once, case ignored.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_suggest_distance(a: *const u8, a_len: usize, b: *const u8, b_len: usize) -> i32 {
    let result = || -> Result<i32, i32> { Ok(suggestion::distance(unsafe { required(a, a_len) }?, unsafe { required(b, b_len) }?) as i32) };
    result().unwrap_or_else(|status| status)
}

/// The candidates nearest a name that matched none of them, each candidate given followed by a NUL. As `EMUSEN_GALAXIA_SUGGEST_HINT` the answer is the end of a sentence, or nothing; as `NEAREST` the names, each followed by a NUL.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_suggest(what: u32, typed: *const u8, typed_len: usize, candidates: *const u8, candidates_len: usize, max: i32, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let (typed, candidates) = (unsafe { required(typed, typed_len) }?, unsafe { text(candidates, candidates_len) }?.unwrap_or(""));
        let candidates: Vec<&str> = candidates.split_terminator('\0').collect();
        match what {
            suggest::HINT => Ok(suggestion::hint(typed, &candidates, max)),
            suggest::NEAREST => Ok(suggestion::nearest(typed, &candidates, max).iter().map(|name| format!("{name}\0")).collect()),
            _ => Err(fail(status::BAD_ARGUMENT, format!("no suggestion {what}"))),
        }
    };
    answer(result(), out, len)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(call: impl Fn(*mut u8, usize) -> i64) -> Result<String, i64> {
        let length = call(std::ptr::null_mut(), 0);
        if length < 0 {
            return Err(length);
        }
        let mut out = vec![0u8; length as usize];
        assert_eq!(call(out.as_mut_ptr(), out.len()), length);
        Ok(String::from_utf8(out).unwrap())
    }

    #[test]
    fn a_model_is_bound_and_formatted_through_the_c_layer() {
        let text = r#"{"SampleRate": 48000, /* by hand */ "mastervolume": 0.8,}"#;
        let bound = ask(|out, len| unsafe { emusen_galaxia_model_bind(models::AUDIO_CONFIG_ID, text.as_ptr(), text.len(), 0, out, len) }).unwrap();
        assert!(bound.starts_with(r#"{"SampleRate":48000,"#) && bound.contains(r#""MasterVolume":0.8"#));
        let file = ask(|out, len| unsafe { emusen_galaxia_model_format(models::AUDIO_CONFIG_ID, bound.as_ptr(), bound.len(), out, len) }).unwrap();
        assert!(file.contains("\"MasterVolume\": 0.8,"));
        assert_eq!(ask(|out, len| unsafe { emusen_galaxia_model_bind(models::AUDIO_CONFIG_ID, b"null".as_ptr(), 4, 0, out, len) }), Ok("null".to_string()));
        assert_eq!(ask(|out, len| unsafe { emusen_galaxia_model_bind(models::AUDIO_CONFIG_ID, b"[]".as_ptr(), 2, 0, out, len) }), Err(status::PARSE as i64));
        assert_eq!(ask(|out, len| unsafe { emusen_galaxia_model_bind(99, b"{}".as_ptr(), 2, 0, out, len) }), Err(status::BAD_ARGUMENT as i64));
        let new = ask(|out, len| unsafe { emusen_galaxia_model_new(models::APP_SETTINGS_ID, UPGRADE, out, len) }).unwrap();
        assert!(new.contains(r#""SelectedCoreUpgraded":true"#));
        let schema = ask(|out, len| unsafe { emusen_galaxia_model_schema(models::CHEAT_FILE_ID, out, len) }).unwrap();
        assert!(schema.contains("CheatFileEntry.EffectiveWrites\tlist<CheatFileWrite>\tderived\n"));
    }

    #[test]
    fn numbers_and_suggestions_cross_as_text() {
        let double = |value: f64| ask(|out, len| unsafe { emusen_galaxia_number_format(number::DOUBLE, value.to_bits(), out, len) });
        let single = |value: f32| ask(|out, len| unsafe { emusen_galaxia_number_format(number::SINGLE, value.to_bits() as u64, out, len) });
        assert_eq!(double(1e-7), Ok("1E-07".to_string()));
        assert_eq!(single(0.8), Ok("0.8".to_string()));
        assert_eq!(double(f64::NAN), Err(status::ABSENT as i64));
        let candidates = "DPadUp\0DPadDown\0";
        let suggest = |what, typed: &str| ask(|out, len| unsafe { emusen_galaxia_suggest(what, typed.as_ptr(), typed.len(), candidates.as_ptr(), candidates.len(), 2, out, len) });
        assert_eq!(suggest(suggest::HINT, "DPadUpp"), Ok(" Did you mean 'DPadUp'?".to_string()));
        assert_eq!(suggest(suggest::NEAREST, "DPadUpp"), Ok("DPadUp\0".to_string()));
        assert_eq!(suggest(suggest::HINT, "qwertyuiop"), Ok(String::new()));
        assert_eq!(unsafe { emusen_galaxia_suggest_distance(b"chaet".as_ptr(), 5, b"cheat".as_ptr(), 5) }, 1);
    }

    #[test]
    fn the_header_numbers_the_models_and_kinds_as_the_library_does() {
        let header = include_str!("../../include/emusen_platform.h");
        let defines = |name: &str, value: u32| assert!(header.contains(&format!("#define {name} {value}u")), "{name} {value}");
        defines("EMUSEN_GALAXIA_MODEL_APP_SETTINGS", models::APP_SETTINGS_ID);
        defines("EMUSEN_GALAXIA_MODEL_AUDIO_CONFIG", models::AUDIO_CONFIG_ID);
        defines("EMUSEN_GALAXIA_MODEL_GRAPHICS_CONFIG", models::GRAPHICS_CONFIG_ID);
        defines("EMUSEN_GALAXIA_MODEL_CHEAT_FILE", models::CHEAT_FILE_ID);
        defines("EMUSEN_GALAXIA_MODEL_UPGRADE", UPGRADE);
        defines("EMUSEN_GALAXIA_NUMBER_DOUBLE", number::DOUBLE);
        defines("EMUSEN_GALAXIA_NUMBER_SINGLE", number::SINGLE);
        defines("EMUSEN_GALAXIA_SUGGEST_HINT", suggest::HINT);
        defines("EMUSEN_GALAXIA_SUGGEST_NEAREST", suggest::NEAREST);
        assert_eq!(models::MODELS.len(), 4);
    }
}
