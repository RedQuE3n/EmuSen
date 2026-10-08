//! Galaxia's C layer: the tree, the save library's names, atomic files, the migration and the ROM hash.

use crate::{diagnostics_rewind, diagnostics_waiting, fail, fail_io, join_nul, put, report, required, status, text};
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::migration::{self, Copied};
use emusen_galaxia::tree::{self, Directory, Override};
use emusen_galaxia::{atomic, rom_hash, saves};

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
        ] {
            defines(&format!("EMUSEN_GALAXIA_DIR_{name}"), which as u32);
        }
        assert_eq!(Directory::ALL.iter().map(|d| *d as u32).collect::<Vec<_>>(), (0..20).collect::<Vec<_>>());
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
