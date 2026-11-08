//! Endymion's C layer for the binding files: each read as a program starts with it, saved, and a map's defaults.

use crate::{bytes, diagnostics_rewind, diagnostics_waiting, fail, put, report, required, status, text};
use emusen_endymion::bindings::{self, File, Unsaved};
use emusen_galaxia::tree;

fn file_of(which: u32) -> Result<File, i32> {
    File::from_u32(which).ok_or_else(|| fail(status::BAD_ARGUMENT, format!("no binding file {which}")))
}

/// The console names a caller passed, each followed by a NUL.
fn consoles(joined: &[u8]) -> Result<Vec<&str>, i32> {
    let text = std::str::from_utf8(joined).map_err(|_| fail(status::BAD_STRING, "the console names are not UTF-8"))?;
    Ok(text.split_terminator('\0').collect())
}

/// A binding file's model as a program starts with it; a file that would not read is its defaults with a diagnostic.
///
/// # Safety
/// `names` must be valid for `names_len` bytes, or null with no length; `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_bindings_load(which: u32, names: *const u8, names_len: usize, out: *mut u8, len: usize) -> i64 {
    let file = match file_of(which) {
        Ok(file) => file,
        Err(status) => return status as i64,
    };
    let names = match unsafe { bytes(names, names_len) }.and_then(consoles) {
        Ok(names) => names,
        Err(status) => return status as i64,
    };
    let waiting = diagnostics_waiting();
    let (model, said) = bindings::load(&tree::tree(), file, &names);
    said.into_iter().for_each(report);
    if out.is_null() || model.len() > len {
        // The caller comes back with room, and the file is read and reported then.
        diagnostics_rewind(waiting);
    }
    unsafe { put(model.as_bytes(), out, len) }
}

/// A model saved as its binding file: 0, PARSE for a document the model does not bind, or IO.
///
/// # Safety
/// `document` must be valid for `document_len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_bindings_save(which: u32, document: *const u8, document_len: usize) -> i32 {
    let (file, document) = match (file_of(which), unsafe { required(document, document_len) }) {
        (Ok(file), Ok(document)) => (file, document),
        (Err(status), _) | (_, Err(status)) => return status,
    };
    match bindings::save(&tree::tree(), file, document) {
        Ok(()) => 0,
        Err(Unsaved::Document(words)) => fail(status::PARSE, words),
        Err(Unsaved::Write(words)) => fail(status::IO, words),
    }
}

/// One map at its defaults: a console's keys, a console's gamepad buttons, or the hotkeys; BAD_ARGUMENT for the profiles.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_bindings_defaults(which: u32, out: *mut u8, len: usize) -> i64 {
    match file_of(which).and_then(|file| file.default_map().ok_or_else(|| fail(status::BAD_ARGUMENT, "the profiles have no map of defaults"))) {
        Ok(map) => unsafe { put(map.as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

/// The SDL button each PadButton is bound to for a player of a console, -1 for none, by the profiles' rules, from the
/// files as a program starts with them; `game` is the game's path, for a profile it names. The count of PadButtons.
///
/// # Safety
/// `console` must be valid for `console_len` bytes; `game` for `game_len` bytes, or null; `out` for `cap` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_bindings_bound(
    console: *const u8,
    console_len: usize,
    player: i32,
    game: *const u8,
    game_len: usize,
    out: *mut i32,
    cap: usize,
) -> i64 {
    if !(1..=emusen_endymion::slots::MAX_PLAYERS as i32).contains(&player) {
        return fail(status::BAD_ARGUMENT, format!("no player {player}")) as i64;
    }
    let console = match unsafe { required(console, console_len) } {
        Ok(console) => console,
        Err(status) => return status as i64,
    };
    let game = match unsafe { text(game, game_len) } {
        Ok(game) => game.filter(|g| !g.is_empty()),
        Err(status) => return status as i64,
    };
    let (bound, said) = bindings::bound_in(&tree::tree(), console, player, game);
    said.into_iter().for_each(report);
    if !out.is_null() {
        // SAFETY: the caller gave room for `cap` values.
        unsafe { std::ptr::copy_nonoverlapping(bound.as_ptr(), out, cap.min(bound.len())) };
    }
    bound.len() as i64
}
