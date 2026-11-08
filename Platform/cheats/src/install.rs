//! Installing a downloaded cheat archive, as the C# `CheatDatabaseInstaller.Install` did: every `.cht` entry of a zip
//! written under the target folder, as `ZipArchiveEntry.ExtractToFile` writes it (its Unix mode on a file it makes, its
//! time as local time), anything else or anything that would land outside the folder counted as skipped. The archive is
//! read by Galaxia's `dotnet_zip`, every form .NET's reader reads, and one it refuses is refused in its words. The
//! download is not here (EmuSen_Backend.md SR9). See EmuSen_Backend.md §15.3 and §40.1.

use emusen_galaxia::dotnet_path::{self as path, Style};
use std::io::Write;

/// What an install did: entries written, and entries passed over or that failed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Installed {
    pub installed: usize,
    pub skipped: usize,
}

/// What .NET threw where it stopped an install: the exception's name and its words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thrown {
    pub exception: &'static str,
    pub words: String,
}

/// Why nothing was installed: the archive refused whole, or the target not made.
#[derive(Debug)]
pub enum Refusal {
    Thrown(Thrown),
    Io(std::io::Error),
}

use emusen_galaxia::dotnet_zip::{self as zip, Archive, Entry};

/// `Path.GetFullPath`'s refusal of a path holding a NUL, which the C# met before its catch.
const NUL_IN_PATH: &str = "Null character in path. (Parameter 'path')";

/// `ZipArchiveEntry.Name`: after the last `/`, and for an entry made on Windows also after `\` or `:`.
fn name_of(e: &Entry) -> &str {
    let cut = if e.platform == 0 { e.full_name.rfind(['\\', '/', ':']) } else { e.full_name.rfind('/') };
    cut.map_or(&e.full_name, |at| &e.full_name[at + 1..])
}

/// `ZipHelper.DosTimeToDateTime`: the fields, or 1 January 1980 for a time that is none.
fn dos_fields(t: u32) -> (i32, i32, i32, i32, i32, i32) {
    let fields = (1980 + (t >> 25) as i32, ((t >> 21) & 0xF) as i32, ((t >> 16) & 0x1F) as i32, ((t >> 11) & 0x1F) as i32, ((t >> 5) & 0x3F) as i32, ((t & 0x1F) * 2) as i32);
    let (y, mo, d, h, mi, s) = fields;
    let days = match mo {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        2 => 28,
        _ => 0,
    };
    if t == 0 || d < 1 || d > days || h > 23 || mi > 59 || s > 59 { (1980, 1, 1, 0, 0, 0) } else { fields }
}

/// A local clock time as Unix seconds, as `File.SetLastWriteTime` reads an unspecified time: by the zone's offset at
/// that time, and by its standard offset where the time falls twice or never.
#[cfg(unix)]
fn local_seconds((y, mo, d, h, mi, s): (i32, i32, i32, i32, i32, i32)) -> i64 {
    let made = |dst: i32| -> (i64, bool) {
        // SAFETY: tm is plain data, and mktime and localtime_r only read and write the structures given.
        unsafe {
            let mut tm: libc::tm = std::mem::zeroed();
            (tm.tm_year, tm.tm_mon, tm.tm_mday, tm.tm_hour, tm.tm_min, tm.tm_sec, tm.tm_isdst) = (y - 1900, mo - 1, d, h, mi, s, dst);
            let t = libc::mktime(&mut tm);
            let mut back: libc::tm = std::mem::zeroed();
            libc::localtime_r(&t, &mut back);
            let same = (back.tm_year, back.tm_mon, back.tm_mday, back.tm_hour, back.tm_min, back.tm_sec) == (y - 1900, mo - 1, d, h, mi, s) && back.tm_isdst == dst;
            (t as i64, same)
        }
    };
    let (standard, standard_holds) = made(0);
    let (summer, summer_holds) = made(1);
    if !standard_holds && summer_holds { summer } else { standard }
}

/// Elsewhere the zone is read when the other systems' port resumes (EmuSen_RustPlatform.md §31); until then UTC.
#[cfg(not(unix))]
fn local_seconds((y, mo, d, h, mi, s): (i32, i32, i32, i32, i32, i32)) -> i64 {
    let days_before = |y: i64| y * 365 + y / 4 - y / 100 + y / 400;
    let month_days = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let day = days_before(y as i64 - 1) - days_before(1969) + month_days[(mo - 1) as usize] + if leap && mo > 2 { 1 } else { 0 } + d as i64 - 1;
    day * 86400 + (h * 3600 + mi * 60 + s) as i64
}

/// `ExtractToFile(destination, overwrite: true)`: made with the entry's mode or truncated once locked, written, its time set.
fn extract(archive: &Archive, e: &Entry, destination: &str) -> Result<(), ()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mode = (e.external >> 16) & 0o777;
        if mode != 0 {
            options.mode(mode);
        }
    }
    let mut file = options.open(destination).map_err(|_| ())?;
    file.try_lock().map_err(|_| ())?;
    file.set_len(0).map_err(|_| ())?;
    let bytes = archive.read(e).map_err(|_| ())?;
    file.write_all(&bytes).map_err(|_| ())?;
    let when = std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(local_seconds(dos_fields(e.dos_time)).max(0) as u64)).ok_or(())?;
    let _ = file.set_modified(when);
    Ok(())
}

/// `CheatDatabaseInstaller.Install`: the target made first, then the archive opened, as .NET's reader opens it; what
/// .NET threw part way, after the entries before it were written, comes back beside the counts.
pub fn install(bytes: &[u8], target: &str) -> Result<(Installed, Option<Thrown>), Refusal> {
    std::fs::create_dir_all(target).map_err(Refusal::Io)?;
    let archive = zip::open(bytes).map_err(|words| Refusal::Thrown(Thrown { exception: "InvalidDataException", words: words.to_string() }))?;
    let full = path::full_path(target);
    let root = path::trim_ending_separator(Style::HOST, &full).to_string();
    let inside = format!("{root}{}", Style::HOST.separator());
    let mut done = Installed::default();
    for e in &archive.entries {
        let name = name_of(e);
        if e.full_name.ends_with('/') || (e.length == 0 && name.is_empty()) {
            continue;
        }
        if !name.to_uppercase().ends_with(".CHT") {
            done.skipped += 1;
            continue;
        }
        if e.full_name.contains('\0') {
            return Ok((done, Some(Thrown { exception: "ArgumentException", words: NUL_IN_PATH.to_string() })));
        }
        let destination = path::full_path(&path::combine(Style::HOST, &root, &e.full_name));
        if !destination.starts_with(&inside) {
            done.skipped += 1;
            continue;
        }
        let folder = path::directory_name(Style::HOST, &destination).unwrap_or_default();
        if std::fs::create_dir_all(&folder).is_ok() && extract(&archive, e, &destination).is_ok() {
            done.installed += 1;
        } else {
            done.skipped += 1;
        }
    }
    Ok((done, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dos_times_that_are_none_are_the_first_of_1980() {
        assert_eq!(dos_fields(0), (1980, 1, 1, 0, 0, 0));
        assert_eq!(dos_fields((46 << 25) | (2 << 21) | (29 << 16)), (1980, 1, 1, 0, 0, 0));
        assert_eq!(dos_fields((44 << 25) | (2 << 21) | (29 << 16) | (23 << 11) | (59 << 5) | 29), (2024, 2, 29, 23, 59, 58));
    }

    #[test]
    fn what_is_not_a_zip_is_refused_in_dotnets_words() {
        let target = std::env::temp_dir().join(format!("emusen-install-{}", std::process::id()));
        let refused = install(&[0u8; 100], target.to_str().unwrap());
        let _ = std::fs::remove_dir_all(&target);
        assert!(matches!(refused, Err(Refusal::Thrown(Thrown { exception: "InvalidDataException", ref words })) if words == zip::words::END_NOT_FOUND));
    }
}
