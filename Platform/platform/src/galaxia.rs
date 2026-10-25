//! Galaxia's C layer: the tree, the save library's names, atomic files, the migration and the ROM hash.

use crate::{diagnostics_rewind, diagnostics_waiting, fail, fail_io, join_nul, put, report, required, status, text};
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::migration::{self, Copied};
use emusen_galaxia::tree::{self, Directory, Override};
use emusen_galaxia::mapped::Mapped;
use emusen_galaxia::{atomic, package, rom_hash, saves};

/// What `emusen_galaxia_save_path` names.
pub mod save {
    pub const SRAM: u32 = 0;
    pub const FLAT_SRAM: u32 = 1;
    pub const STATE: u32 = 2;
    pub const RESUME_STATE: u32 = 3;
    pub const PICTURE: u32 = 4;
}

/// What `emusen_galaxia_migrate` copies.
pub mod migrate {
    pub const OWN: u32 = 0;
    pub const OWN_CONFIG: u32 = 1;
    pub const OWN_SEED: u32 = 2;
    pub const BETWEEN: u32 = 3;
    pub const TREE: u32 = 4;
    pub const SEED: u32 = 5;
}

/// The path rules `emusen_galaxia_dotnet_path` answers with text.
pub mod path_op {
    pub const COMBINE: u32 = 0;
    pub const FILE_NAME: u32 = 1;
    pub const FILE_NAME_WITHOUT_EXTENSION: u32 = 2;
    pub const CHANGE_EXTENSION: u32 = 3;
    pub const DIRECTORY_NAME: u32 = 4;
    pub const TRIM_ENDING_SEPARATOR: u32 = 5;
    pub const FULL_PATH: u32 = 6;
}

/// The rules `emusen_galaxia_dotnet_test` answers with 1 or 0.
pub mod test_op {
    pub const FILE_EXISTS: u32 = 0;
    pub const DIRECTORY_EXISTS: u32 = 1;
    pub const IS_BLANK: u32 = 2;
    pub const EQUALS_IGNORE_CASE: u32 = 3;
    pub const ENDS_WITH_IGNORE_CASE: u32 = 4;
}

/// A text answer, or the status that stands in for it.
fn answer(result: Result<String, i32>, out: *mut u8, len: usize) -> i64 {
    match result {
        Ok(text) => unsafe { put(text.as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

fn absent() -> i32 {
    status::ABSENT
}

/// Names the program's directory and .NET's application-data folder before the tree is first asked for; 1 when it came too late to count.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_init(base: *const u8, base_len: usize, application_data: *const u8, application_data_len: usize) -> i32 {
    let arguments = || -> Result<(&str, Option<&str>), i32> { Ok((unsafe { required(base, base_len) }?, unsafe { text(application_data, application_data_len) }?)) };
    match arguments() {
        Ok((base, application_data)) => i32::from(!tree::init(base, application_data)),
        Err(status) => status,
    }
}

/// A directory of the process's tree, by its `EMUSEN_GALAXIA_DIR_*` number; `ABSENT` for a seed directory outside a bundle.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_directory(which: u32, out: *mut u8, len: usize) -> i64 {
    let directory = Directory::from_u32(which).ok_or_else(|| fail(status::BAD_ARGUMENT, format!("no directory {which}")));
    answer(directory.and_then(|which| tree::tree().directory(which).ok_or_else(absent)), out, len)
}

/// Moves one of the three redirects, or with a null path restores the real location.
///
/// # Safety
/// `directory` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_set_override(which: u32, directory: *const u8, len: usize) -> i32 {
    let Some(which) = Override::from_u32(which) else {
        return fail(status::BAD_ARGUMENT, format!("no override {which}"));
    };
    match unsafe { text(directory, len) } {
        Ok(directory) => {
            tree::tree_mut().set_override(which, directory.map(str::to_string));
            0
        }
        Err(status) => status,
    }
}

/// A config file's path, in a category's subdirectory when one is given.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_config_path(category: *const u8, category_len: usize, file: *const u8, file_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let (category, file) = (unsafe { text(category, category_len) }?, unsafe { required(file, file_len) }?);
        Ok(tree::tree().config_path(category, file))
    };
    answer(result(), out, len)
}

/// Where a config file sat before Galaxia, to migrate out of; `ABSENT` when config is redirected and the legacy directory is not.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_legacy_config_path(file: *const u8, file_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> { tree::tree().legacy_config_path(unsafe { required(file, file_len) }?).ok_or_else(absent) };
    answer(result(), out, len)
}

/// The root a program in `base` would have, with the platform and the home directory given so every branch can be asked on any host.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_root_for(base: *const u8, base_len: usize, mac_os: i32, user_home: *const u8, user_home_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let (base, user_home) = (unsafe { required(base, base_len) }?, unsafe { required(user_home, user_home_len) }?);
        Ok(tree::root_for(base, mac_os != 0, user_home))
    };
    answer(result(), out, len)
}

/// The bundle's `Contents` when `base` is its `Contents/MacOS`, else `ABSENT`.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_bundle_contents_for(base: *const u8, base_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> { tree::bundle_contents_for(unsafe { required(base, base_len) }?).ok_or_else(absent) };
    answer(result(), out, len)
}

/// The skeleton a bundle carries, `Contents/Resources/home`; `ABSENT` outside a bundle or off macOS.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_seed_directory_for(base: *const u8, base_len: usize, mac_os: i32, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> { tree::seed_directory_for(unsafe { required(base, base_len) }?, mac_os != 0).ok_or_else(absent) };
    answer(result(), out, len)
}

/// `Library/Application Support/EmuSen` under a home directory.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_mac_data_directory_for(user_home: *const u8, user_home_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> { Ok(tree::mac_data_directory_for(Style::HOST, unsafe { required(user_home, user_home_len) }?)) };
    answer(result(), out, len)
}

/// Where the tree lived under a root before it collapsed to `<root>/home`.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_legacy_root_for(config_root: *const u8, config_root_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> { Ok(tree::legacy_root_for(unsafe { required(config_root, config_root_len) }?)) };
    answer(result(), out, len)
}

/// A save file's path, by its `EMUSEN_GALAXIA_SAVE_*` kind. `rom` is the ROM's path, or the state's for a picture; `extra` is the console for a save, the directory override for a state.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_save_path(kind: u32, rom: *const u8, rom_len: usize, extra: *const u8, extra_len: usize, slot: i32, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let (rom, extra) = (unsafe { required(rom, rom_len) }?, unsafe { text(extra, extra_len) }?);
        let tree = tree::tree();
        Ok(match kind {
            save::SRAM => saves::sram_path(&tree, rom, extra.ok_or_else(|| fail(status::NULL, "a save needs its console"))?),
            save::FLAT_SRAM => saves::flat_sram_path(&tree, rom),
            save::STATE => saves::state_path(&tree, rom, slot, extra),
            save::RESUME_STATE => saves::resume_state_path(&tree, rom, extra),
            save::PICTURE => saves::picture_path(rom),
            _ => return Err(fail(status::BAD_ARGUMENT, format!("no save kind {kind}"))),
        })
    };
    answer(result(), out, len)
}

/// A file's bytes: its whole length, with `out` holding the file when that length is no more than `len`. A null `out` asks the length; `ABSENT` for no file; a file that will not read is `IO` with a diagnostic.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_file_read(file: *const u8, file_len: usize, out: *mut u8, len: usize) -> i64 {
    let file = match unsafe { required(file, file_len) } {
        Ok(file) => file,
        Err(status) => return status as i64,
    };
    if out.is_null() {
        return atomic::length(file).map_or(status::ABSENT as i64, |length| length as i64);
    }
    let buffer = if len == 0 { &mut [][..] } else { unsafe { std::slice::from_raw_parts_mut(out, len) } };
    match atomic::read_into(file, buffer) {
        Ok(Some(length)) => length as i64,
        Ok(None) => status::ABSENT as i64,
        Err(diagnostic) => {
            report(diagnostic.clone());
            fail(status::IO, diagnostic) as i64
        }
    }
}

/// The bytes written beside the file and renamed over it. A failure is `IO` with a diagnostic, and the live file is as it was.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_file_write(file: *const u8, file_len: usize, data: *const u8, len: usize) -> i32 {
    let file = match unsafe { required(file, file_len) } {
        Ok(file) => file,
        Err(status) => return status,
    };
    let contents = if data.is_null() || len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(data, len) } };
    match atomic::write(file, contents) {
        Ok(()) => 0,
        Err(diagnostic) => {
            report(diagnostic.clone());
            fail(status::IO, diagnostic)
        }
    }
}

/// A migration by its `EMUSEN_GALAXIA_MIGRATE_*` kind: how many files it copied, with a diagnostic for each it could not. A directory that will not list is a failing status.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_migrate(what: u32, source: *const u8, source_len: usize, destination: *const u8, destination_len: usize) -> i64 {
    let run = || -> Result<std::io::Result<Copied>, i32> {
        let source = unsafe { text(source, source_len) }?;
        let destination = unsafe { text(destination, destination_len) }?;
        let both = || -> Result<(&str, &str), i32> { Ok((source.ok_or_else(|| fail(status::NULL, "no source"))?, destination.ok_or_else(|| fail(status::NULL, "no destination"))?)) };
        Ok(match what {
            migrate::OWN => migration::run_own(&tree::tree().clone()),
            migrate::OWN_CONFIG => migration::run_config(&tree::tree().clone()),
            migrate::OWN_SEED => {
                let tree = tree::tree().clone();
                migration::seed_from_bundle(tree.seed.as_deref(), &tree.home())
            }
            migrate::BETWEEN => both().map(|(source, destination)| migration::run(source, destination))?,
            migrate::TREE => both().map(|(source, destination)| migration::copy_tree(source, destination))?,
            migrate::SEED => migration::seed_from_bundle(source, destination.ok_or_else(|| fail(status::NULL, "no destination"))?),
            _ => return Err(fail(status::BAD_ARGUMENT, format!("no migration {what}"))),
        })
    };
    match run() {
        Ok(Ok(copied)) => {
            copied.diagnostics.into_iter().for_each(report);
            copied.files as i64
        }
        Ok(Err(error)) => fail_io(&error) as i64,
        Err(status) => status as i64,
    }
}

/// The old ROM folders under a legacy root that still hold files, each path ended by a NUL; a null root asks about the process's own.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_remaining_library(legacy_root: *const u8, legacy_root_len: usize, out: *mut u8, len: usize) -> i64 {
    let legacy_root = match unsafe { text(legacy_root, legacy_root_len) } {
        Ok(Some(root)) => root.to_string(),
        Ok(None) => tree::legacy_root_for(&tree::tree().root),
        Err(status) => return status as i64,
    };
    let waiting = diagnostics_waiting();
    let (found, diagnostics) = migration::remaining_library_directories(&legacy_root);
    diagnostics.into_iter().for_each(report);
    let joined = join_nul(&found);
    if !joined.is_empty() && (out.is_null() || joined.len() > len) {
        // The caller comes back with room, and the walk reports again then.
        diagnostics_rewind(waiting);
    }
    unsafe { put(&joined, out, len) }
}

/// A file's MD5 as 32 lower-case hexadecimal digits.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_rom_md5(file: *const u8, file_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> { rom_hash::md5(unsafe { required(file, file_len) }?).map_err(|error| fail_io(&error)) };
    answer(result(), out, len)
}

/// A dump's hashes as the JSON object `{"crc32","md5","sha1","size"}`.
fn hashes_json(h: &rom_hash::Hashes) -> String {
    format!(r#"{{"crc32":"{}","md5":"{}","sha1":"{}","size":{}}}"#, h.crc32, h.md5, h.sha1.as_deref().unwrap_or(""), h.size)
}

/// A stop flag the host may set from any thread while a hash reads: nonzero stops it before its next 64 KiB.
fn stopped(stop: *const i32) -> impl Fn() -> bool {
    move || !stop.is_null() && unsafe { std::ptr::read_volatile(stop) } != 0
}

/// A hashing failure's status: a stop or a read is `IO`, a missing file `NOT_FOUND`, a sheet that is not one `PARSE`.
fn fail_hash(error: &std::io::Error) -> i32 {
    match error.kind() {
        std::io::ErrorKind::InvalidData => fail(status::PARSE, error.to_string()),
        std::io::ErrorKind::Interrupted => fail(status::IO, "stopped"),
        _ => fail_io(error),
    }
}

/// A file's CRC-32, MD5 and SHA-1 in one read, of `length` bytes from `offset` (`UINT64_MAX` to its end), as JSON.
///
/// # Safety
/// Each pointer must be valid for its length, or null; `stop`, when not null, must stay valid for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_rom_hashes(file: *const u8, file_len: usize, offset: u64, length: u64, stop: *const i32, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let path = unsafe { required(file, file_len) }?;
        let range = (offset != 0 || length != u64::MAX).then_some((offset, length));
        rom_hash::hashes_with(path, range, true, &stopped(stop)).map(|h| hashes_json(&h)).map_err(|e| fail_hash(&e))
    };
    answer(result(), out, len)
}

/// The three hashes of bytes the host holds, as JSON: a system pack's comparable form of a dump.
///
/// # Safety
/// `data` must be valid for `data_len` bytes, or null with no length; `out` for `len`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_rom_hashes_of(data: *const u8, data_len: usize, out: *mut u8, len: usize) -> i64 {
    answer(unsafe { crate::bytes(data, data_len) }.map(|b| hashes_json(&rom_hash::hashes_of(b))), out, len)
}

/// Each track of a cue sheet hashed where `emusen-disc` places it, as a JSON array of
/// `{"track","file","offset","crc32","md5","sha1","size"}`.
///
/// # Safety
/// As `emusen_galaxia_rom_hashes`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_rom_tracks(sheet: *const u8, sheet_len: usize, stop: *const i32, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let path = unsafe { required(sheet, sheet_len) }?;
        let tracks = rom_hash::tracks(path, &stopped(stop)).map_err(|e| fail_hash(&e))?;
        let items: Vec<String> = tracks
            .iter()
            .map(|t| {
                let h = hashes_json(&t.hashes);
                format!(r#"{{"track":{},"file":{},"offset":{},{}"#, t.number, emusen_galaxia::json::quoted(&t.file), t.offset, &h[1..])
            })
            .collect();
        Ok(format!("[{}]", items.join(",")))
    };
    answer(result(), out, len)
}

/// A manifest's refusal as its status: `NEWER_FILE` for a newer major, `PARSE` for the rest, with the words.
fn fail_package(refusal: &package::Refusal) -> i32 {
    match refusal {
        package::Refusal::Newer(words) => fail(status::NEWER_FILE, words.clone()),
        other => fail(status::PARSE, other.words()),
    }
}

/// The manifest at `file`, checked, as compact JSON.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_package_read(file: *const u8, file_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let doc = package::read(unsafe { required(file, file_len) }?).map_err(|e| fail_io(&e))?.map_err(|r| fail_package(&r))?;
        Ok(emusen_galaxia::model::transport(&doc))
    };
    answer(result(), out, len)
}

/// A manifest checked and written atomically; a refused one is not written, and a failed write queues its diagnostic.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_package_write(file: *const u8, file_len: usize, manifest: *const u8, manifest_len: usize) -> i32 {
    let result = || -> Result<(), i32> {
        let path = unsafe { required(file, file_len) }?;
        let doc = package::parse(unsafe { required(manifest, manifest_len) }?).map_err(|r| fail_package(&r))?;
        match package::write(path, &doc).map_err(|r| fail_package(&r))? {
            Ok(()) => Ok(()),
            Err(diagnostic) => {
                report(diagnostic.clone());
                Err(fail(status::IO, diagnostic))
            }
        }
    };
    result().map_or_else(|status| status, |()| 0)
}

/// A manifest checked and laid out as it is written.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_package_format(manifest: *const u8, manifest_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let doc = package::parse(unsafe { required(manifest, manifest_len) }?).map_err(|r| fail_package(&r))?;
        Ok(package::format(&doc))
    };
    answer(result(), out, len)
}

/// Maps the file whole and read-only, its handle in `mapped`; the bytes stay in place until `emusen_galaxia_unmap`.
///
/// # Safety
/// `file` must be valid for `file_len` bytes, or null; `mapped` must be valid to write, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_map(file: *const u8, file_len: usize, mapped: *mut *mut Mapped) -> i32 {
    if mapped.is_null() {
        return fail(status::NULL, "no place for the mapping's handle");
    }
    let path = match unsafe { required(file, file_len) } {
        Ok(path) => path,
        Err(status) => return status,
    };
    match Mapped::open(path) {
        Ok(map) => {
            unsafe { *mapped = Box::into_raw(Box::new(map)) };
            0
        }
        Err(error) => fail_io(&error),
    }
}

/// The mapped bytes' length, and their address in `bytes`; null and zero for an empty file.
///
/// # Safety
/// `mapped` must come from `emusen_galaxia_map` and not have been unmapped, or be null; `bytes` must be valid to write, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_mapped_bytes(mapped: *const Mapped, bytes: *mut *const u8) -> i64 {
    let Some(map) = (unsafe { mapped.as_ref() }) else { return fail(status::NULL, "no mapping") as i64 };
    let held = map.bytes();
    if !bytes.is_null() {
        unsafe { *bytes = if held.is_empty() { std::ptr::null() } else { held.as_ptr() } };
    }
    held.len() as i64
}

/// Advises the system that `len` bytes of the mapping from `offset` will be read soon (`Mapped::advise`): 1 where it
/// was advised, 0 where the range holds nothing of the file or the system refused. No byte the mapping gives changes.
///
/// # Safety
/// As `emusen_galaxia_mapped_bytes`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_mapped_advise(mapped: *const Mapped, offset: u64, len: u64) -> i32 {
    let Some(map) = (unsafe { mapped.as_ref() }) else { return fail(status::NULL, "no mapping") };
    map.advise(offset, len) as i32
}

/// Releases a mapping; null is nothing to release.
///
/// # Safety
/// As `emusen_galaxia_mapped_bytes`, and no address the mapping gave may be read afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_unmap(mapped: *mut Mapped) -> i32 {
    if !mapped.is_null() {
        drop(unsafe { Box::from_raw(mapped) });
    }
    0
}

/// One of .NET's path rules as this library applies it, by its `EMUSEN_GALAXIA_PATH_*` number, so the parity tests can hold each rule to .NET's own answer.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_dotnet_path(op: u32, a: *const u8, a_len: usize, b: *const u8, b_len: usize, out: *mut u8, len: usize) -> i64 {
    let result = || -> Result<String, i32> {
        let (a, b) = (unsafe { required(a, a_len) }?, unsafe { text(b, b_len) }?.unwrap_or(""));
        let style = Style::HOST;
        Ok(match op {
            path_op::COMBINE => path::combine(style, a, b),
            path_op::FILE_NAME => path::file_name(style, a).to_string(),
            path_op::FILE_NAME_WITHOUT_EXTENSION => path::file_name_without_extension(style, a).to_string(),
            path_op::CHANGE_EXTENSION => path::change_extension(style, a, b),
            path_op::DIRECTORY_NAME => path::directory_name(style, a).ok_or_else(absent)?,
            path_op::TRIM_ENDING_SEPARATOR => path::trim_ending_separator(style, a).to_string(),
            path_op::FULL_PATH => path::full_path(a),
            _ => return Err(fail(status::BAD_ARGUMENT, format!("no path rule {op}"))),
        })
    };
    answer(result(), out, len)
}

/// One of .NET's yes-or-no rules as this library applies it, by its `EMUSEN_GALAXIA_TEST_*` number: 1, 0 or a status. A null `a` is a null string.
///
/// # Safety
/// Each pointer must be valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_dotnet_test(op: u32, a: *const u8, a_len: usize, b: *const u8, b_len: usize) -> i32 {
    let result = || -> Result<bool, i32> {
        let (a, b) = (unsafe { text(a, a_len) }?, unsafe { text(b, b_len) }?.unwrap_or(""));
        Ok(match op {
            test_op::IS_BLANK => path::is_blank(a),
            test_op::FILE_EXISTS => tree::file_exists(a.unwrap_or("")),
            test_op::DIRECTORY_EXISTS => tree::directory_exists(a.unwrap_or("")),
            test_op::EQUALS_IGNORE_CASE => path::equals_ignore_case(a.unwrap_or(""), b),
            test_op::ENDS_WITH_IGNORE_CASE => path::ends_with_ignore_case(a.unwrap_or(""), b),
            _ => return Err(fail(status::BAD_ARGUMENT, format!("no rule {op}"))),
        })
    };
    match result() {
        Ok(answer) => i32::from(answer),
        Err(status) => status,
    }
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
    fn a_rule_asked_through_the_c_layer_is_the_crates_answer() {
        let rule = |op, a: &str, b: &str| ask(|out, len| unsafe { emusen_galaxia_dotnet_path(op, a.as_ptr(), a.len(), b.as_ptr(), b.len(), out, len) });
        let separator = Style::HOST.separator();
        assert_eq!(rule(path_op::COMBINE, "a", "b"), Ok(format!("a{separator}b")));
        assert_eq!(rule(path_op::FILE_NAME_WITHOUT_EXTENSION, "Game (U).sfc", ""), Ok("Game (U)".to_string()));
        assert_eq!(rule(path_op::CHANGE_EXTENSION, "a.state", ".png"), Ok("a.png".to_string()));
        assert_eq!(rule(path_op::DIRECTORY_NAME, "", ""), Err(status::ABSENT as i64));
        assert_eq!(rule(99, "a", ""), Err(status::BAD_ARGUMENT as i64));
    }

    #[test]
    fn a_file_is_mapped_read_and_released_through_the_c_layer() {
        let dir = std::env::temp_dir().join(format!("emusen-platform-map-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (file, empty, absent) = (dir.join("track.bin"), dir.join("empty.bin"), dir.join("absent.bin"));
        let bytes: Vec<u8> = (0..90_000u32).map(|i| (i * 31) as u8).collect();
        std::fs::write(&file, &bytes).unwrap();
        std::fs::write(&empty, b"").unwrap();
        let map = |path: &std::path::Path| {
            let (text, mut handle) = (path.to_str().unwrap(), std::ptr::null_mut());
            (unsafe { emusen_galaxia_map(text.as_ptr(), text.len(), &mut handle) }, handle)
        };
        let (status, handle) = map(&file);
        assert_eq!(status, 0);
        let mut at = std::ptr::null();
        assert_eq!(unsafe { emusen_galaxia_mapped_bytes(handle, &mut at) }, bytes.len() as i64);
        assert_eq!(unsafe { std::slice::from_raw_parts(at, bytes.len()) }, &bytes[..]);
        assert_eq!(unsafe { emusen_galaxia_mapped_bytes(handle, std::ptr::null_mut()) }, bytes.len() as i64);
        assert_eq!(unsafe { emusen_galaxia_mapped_advise(handle, 4096, 8192) }, cfg!(any(unix, windows)) as i32);
        assert_eq!(unsafe { emusen_galaxia_mapped_advise(handle, bytes.len() as u64, 8192) }, 0, "past the file's end");
        assert_eq!(unsafe { emusen_galaxia_mapped_advise(std::ptr::null(), 0, 8192) }, status::NULL);
        assert_eq!(unsafe { std::slice::from_raw_parts(at, bytes.len()) }, &bytes[..], "advice changes no byte");
        assert_eq!(unsafe { emusen_galaxia_unmap(handle) }, 0);
        let (status, handle) = map(&empty);
        assert_eq!(status, 0);
        assert_eq!((unsafe { emusen_galaxia_mapped_bytes(handle, &mut at) }, at), (0, std::ptr::null()));
        assert_eq!(unsafe { emusen_galaxia_unmap(handle) }, 0);
        assert_eq!(map(&absent), (status::NOT_FOUND, std::ptr::null_mut()));
        assert_eq!(map(&dir), (status::ACCESS, std::ptr::null_mut()), "a directory, refused alike everywhere");
        let mut words = [0u8; 256];
        let length = unsafe { crate::emusen_platform_last_error(words.as_mut_ptr(), words.len()) } as usize;
        assert_eq!(std::str::from_utf8(&words[..length]), Ok("is a directory"), "words a frontend can show");
        assert_eq!(unsafe { emusen_galaxia_map(std::ptr::null(), 0, &mut std::ptr::null_mut()) }, status::NULL);
        assert_eq!(unsafe { emusen_galaxia_map(b"x".as_ptr(), 1, std::ptr::null_mut()) }, status::NULL);
        assert_eq!(unsafe { emusen_galaxia_mapped_bytes(std::ptr::null(), &mut at) }, status::NULL as i64);
        assert_eq!(unsafe { emusen_galaxia_unmap(std::ptr::null_mut()) }, 0);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_dumps_hashes_and_tracks_cross_as_json_and_a_set_flag_stops_them() {
        let dir = std::env::temp_dir().join(format!("emusen-platform-hashes-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let rom = dir.join("rom.bin").to_string_lossy().into_owned();
        std::fs::write(&rom, b"123456789").unwrap();
        let hashes = |offset: u64, length: u64, stop: *const i32| ask(|out, len| unsafe { emusen_galaxia_rom_hashes(rom.as_ptr(), rom.len(), offset, length, stop, out, len) });
        let whole = hashes(0, u64::MAX, std::ptr::null()).unwrap();
        assert!(whole.starts_with(r#"{"crc32":"cbf43926","md5":"25f9e794323b453885f5181f1b624d0b","sha1":"f7c3bc1d808e04732adf679965ccc34ca7ae3441","size":9}"#), "{whole}");
        assert!(hashes(6, 100, std::ptr::null()).unwrap().contains(r#""md5":"68053af2923e00204c3ca7c6a3150cf7""#));
        let flag = 1i32;
        assert_eq!(hashes(0, u64::MAX, &flag), Err(status::IO as i64));
        let mut words = [0u8; 64];
        let length = unsafe { crate::emusen_platform_last_error(words.as_mut_ptr(), words.len()) } as usize;
        assert_eq!(&words[..length], b"stopped");
        assert_eq!(ask(|out, len| unsafe { emusen_galaxia_rom_hashes_of(b"123456789".as_ptr(), 9, out, len) }).unwrap(), whole);
        let sheet = dir.join("g.cue").to_string_lossy().into_owned();
        std::fs::write(&sheet, "FILE \"rom.bin\" BINARY\nTRACK 01 AUDIO\nINDEX 01 00:00:00\n").unwrap();
        let tracks = ask(|out, len| unsafe { emusen_galaxia_rom_tracks(sheet.as_ptr(), sheet.len(), std::ptr::null(), out, len) }).unwrap();
        assert_eq!(tracks, format!(r#"[{{"track":1,"file":"rom.bin","offset":0,{}]"#, &whole[1..]));
        std::fs::write(&sheet, "TRACK 01 AUDIO\n").unwrap();
        assert_eq!(ask(|out, len| unsafe { emusen_galaxia_rom_tracks(sheet.as_ptr(), sheet.len(), std::ptr::null(), out, len) }), Err(status::PARSE as i64));
        let absent = dir.join("absent.bin").to_string_lossy().into_owned();
        assert_eq!(ask(|out, len| unsafe { emusen_galaxia_rom_hashes(absent.as_ptr(), absent.len(), 0, u64::MAX, std::ptr::null(), out, len) }), Err(status::NOT_FOUND as i64));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_manifest_is_read_written_and_laid_out_through_the_c_layer() {
        let dir = std::env::temp_dir().join(format!("emusen-platform-package-{}", std::process::id()));
        let file = dir.join("g.nes").join("package.json").to_string_lossy().into_owned();
        let text = r#"{"format":"emusen-package","version":"1.3","system":"nes","mode":"linked","game":{"role":"image","where":"library","path":"NES/g.nes"},"x-kept":[1.0]}"#;
        assert_eq!(unsafe { emusen_galaxia_package_write(file.as_ptr(), file.len(), text.as_ptr(), text.len()) }, 0);
        assert_eq!(ask(|out, len| unsafe { emusen_galaxia_package_read(file.as_ptr(), file.len(), out, len) }), Ok(text.to_string()));
        let laid = ask(|out, len| unsafe { emusen_galaxia_package_format(text.as_ptr(), text.len(), out, len) }).unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), laid);
        let newer = r#"{"format":"emusen-package","version":"2.0"}"#;
        assert_eq!(ask(|out, len| unsafe { emusen_galaxia_package_format(newer.as_ptr(), newer.len(), out, len) }), Err(status::NEWER_FILE as i64));
        assert_eq!(unsafe { emusen_galaxia_package_write(file.as_ptr(), file.len(), newer.as_ptr(), newer.len()) }, status::NEWER_FILE);
        assert_eq!(ask(|out, len| unsafe { emusen_galaxia_package_format(b"{".as_ptr(), 1, out, len) }), Err(status::PARSE as i64));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), laid, "a refused manifest is not written");
        let absent = dir.join("absent.json").to_string_lossy().into_owned();
        assert_eq!(ask(|out, len| unsafe { emusen_galaxia_package_read(absent.as_ptr(), absent.len(), out, len) }), Err(status::NOT_FOUND as i64));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_null_text_is_told_from_an_empty_one() {
        let blank = |a: Option<&str>| unsafe { emusen_galaxia_dotnet_test(test_op::IS_BLANK, a.map_or(std::ptr::null(), str::as_ptr), a.map_or(0, str::len), std::ptr::null(), 0) };
        assert_eq!(blank(None), 1);
        assert_eq!(blank(Some("")), 1);
        assert_eq!(blank(Some(" \t")), 1);
        assert_eq!(blank(Some("x")), 0);
        assert_eq!(unsafe { emusen_galaxia_dotnet_path(path_op::FILE_NAME, std::ptr::null(), 0, std::ptr::null(), 0, std::ptr::null_mut(), 0) }, status::NULL as i64);
    }

    #[test]
    fn a_file_goes_out_and_comes_back_through_the_callers_buffer() {
        let directory = std::env::temp_dir().join(format!("emusen-platform-file-{}", std::process::id()));
        let file = directory.join("Saves").join("a.srm").to_string_lossy().into_owned();
        let contents = b"battery";
        assert_eq!(unsafe { emusen_galaxia_file_read(file.as_ptr(), file.len(), std::ptr::null_mut(), 0) }, status::ABSENT as i64);
        assert_eq!(unsafe { emusen_galaxia_file_write(file.as_ptr(), file.len(), contents.as_ptr(), contents.len()) }, 0);
        assert_eq!(unsafe { emusen_galaxia_file_read(file.as_ptr(), file.len(), std::ptr::null_mut(), 0) }, 7);
        let mut small = [0u8; 3];
        assert_eq!(unsafe { emusen_galaxia_file_read(file.as_ptr(), file.len(), small.as_mut_ptr(), small.len()) }, 7);
        let mut out = [0u8; 7];
        assert_eq!(unsafe { emusen_galaxia_file_read(file.as_ptr(), file.len(), out.as_mut_ptr(), out.len()) }, 7);
        assert_eq!(&out, contents);
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn the_header_numbers_every_directory_override_and_kind_as_the_library_does() {
        let header = include_str!("../../include/emusen_platform.h");
        let defines = |name: &str, value: u32| assert!(header.contains(&format!("#define {name} {value}u")), "{name} {value}");
        for (name, which) in [
            ("ROOT", Directory::Root),
            ("SEED", Directory::Seed),
            ("CONFIG", Directory::Config),
            ("CONFIG_PREVIOUS", Directory::ConfigPrevious),
            ("CONFIG_LEGACY", Directory::ConfigLegacy),
            ("HOME", Directory::Home),
            ("LOGS", Directory::Logs),
            ("SAVES", Directory::Saves),
            ("SAVE_STATES", Directory::SaveStates),
            ("LIBRARY", Directory::Library),
            ("SCREENSHOTS", Directory::Screenshots),
            ("ARTWORK", Directory::Artwork),
            ("MEDIA", Directory::Media),
            ("FIRMWARE", Directory::Firmware),
            ("SHADERS", Directory::Shaders),
            ("THEMES", Directory::Themes),
            ("CHEATS", Directory::Cheats),
            ("GAMES", Directory::Games),
            ("LEGACY_ROOT", Directory::LegacyRoot),
            ("LOG_DEFAULT", Directory::LogDefault),
            ("UNPACKED", Directory::Unpacked),
            ("CORES", Directory::Cores),
            ("PACKAGES", Directory::Packages),
            ("COMPRESSED", Directory::Compressed),
        ] {
            defines(&format!("EMUSEN_GALAXIA_DIR_{name}"), which as u32);
        }
        assert_eq!(Directory::ALL.iter().map(|d| *d as u32).collect::<Vec<_>>(), (0..24).collect::<Vec<_>>());
        defines("EMUSEN_GALAXIA_OVERRIDE_CONFIG", Override::Config as u32);
        defines("EMUSEN_GALAXIA_OVERRIDE_DATA", Override::Data as u32);
        defines("EMUSEN_GALAXIA_OVERRIDE_LEGACY", Override::Legacy as u32);
        for (name, value) in [("SRAM", save::SRAM), ("FLAT_SRAM", save::FLAT_SRAM), ("STATE", save::STATE), ("RESUME_STATE", save::RESUME_STATE), ("PICTURE", save::PICTURE)] {
            defines(&format!("EMUSEN_GALAXIA_SAVE_{name}"), value);
        }
        for (name, value) in [("OWN", migrate::OWN), ("OWN_CONFIG", migrate::OWN_CONFIG), ("OWN_SEED", migrate::OWN_SEED), ("BETWEEN", migrate::BETWEEN), ("TREE", migrate::TREE), ("SEED", migrate::SEED)] {
            defines(&format!("EMUSEN_GALAXIA_MIGRATE_{name}"), value);
        }
        for (name, value) in [
            ("COMBINE", path_op::COMBINE),
            ("FILE_NAME", path_op::FILE_NAME),
            ("FILE_NAME_WITHOUT_EXTENSION", path_op::FILE_NAME_WITHOUT_EXTENSION),
            ("CHANGE_EXTENSION", path_op::CHANGE_EXTENSION),
            ("DIRECTORY_NAME", path_op::DIRECTORY_NAME),
            ("TRIM_ENDING_SEPARATOR", path_op::TRIM_ENDING_SEPARATOR),
            ("FULL_PATH", path_op::FULL_PATH),
        ] {
            defines(&format!("EMUSEN_GALAXIA_PATH_{name}"), value);
        }
        for (name, value) in [
            ("FILE_EXISTS", test_op::FILE_EXISTS),
            ("DIRECTORY_EXISTS", test_op::DIRECTORY_EXISTS),
            ("IS_BLANK", test_op::IS_BLANK),
            ("EQUALS_IGNORE_CASE", test_op::EQUALS_IGNORE_CASE),
            ("ENDS_WITH_IGNORE_CASE", test_op::ENDS_WITH_IGNORE_CASE),
        ] {
            defines(&format!("EMUSEN_GALAXIA_TEST_{name}"), value);
        }
    }
}
