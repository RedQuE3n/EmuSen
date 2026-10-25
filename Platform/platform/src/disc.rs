//! The C layer over emusen-disc's set module, so that the runtime reads a cue sheet's companions and an `.m3u` with the
//! one parser the cores use (EmuSen_CoreAPI_Extensions.md §2.3). Nothing here reads a file: the host reads them.

use crate::{fail, join_nul, put, required, status};
use emusen_disc::set;

/// The files a sheet names, each followed by a NUL; `PARSE` with words for a sheet that is not one.
///
/// # Safety
/// `sheet` must be valid for `sheet_len` bytes and `out` for `len`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_disc_companions(sheet: *const u8, sheet_len: usize, out: *mut u8, len: usize) -> i64 {
    let sheet = match unsafe { required(sheet, sheet_len) } {
        Ok(text) => text,
        Err(status) => return status as i64,
    };
    match set::companions(sheet) {
        Ok(files) => unsafe { put(&join_nul(&files), out, len) },
        Err(e) => fail(status::PARSE, e.to_string()) as i64,
    }
}

/// The discs an `.m3u` lists, each followed by a NUL.
///
/// # Safety
/// As `emusen_disc_companions`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_disc_playlist(list: *const u8, list_len: usize, out: *mut u8, len: usize) -> i64 {
    match unsafe { required(list, list_len) } {
        Ok(text) => unsafe { put(&join_nul(&set::playlist(text)), out, len) },
        Err(status) => status as i64,
    }
}

/// The `.sbi` looked for beside an image.
///
/// # Safety
/// As `emusen_disc_companions`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_disc_sbi_beside(image: *const u8, image_len: usize, out: *mut u8, len: usize) -> i64 {
    match unsafe { required(image, image_len) } {
        Ok(path) => unsafe { put(set::sbi_beside(path).as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

/// Each track of a sheet as a byte range of its file, as a JSON array of `{"track","file","offset","length"}`, `file`
/// the FILE line's position and `length` null to the file's end; `PARSE` with words for a sheet that is not one, or a
/// WAVE file naming several tracks.
///
/// # Safety
/// As `emusen_disc_companions`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_disc_track_spans(sheet: *const u8, sheet_len: usize, out: *mut u8, len: usize) -> i64 {
    let sheet = match unsafe { required(sheet, sheet_len) } {
        Ok(text) => text,
        Err(status) => return status as i64,
    };
    match set::track_spans(sheet) {
        Ok(spans) => {
            let items: Vec<String> = spans
                .iter()
                .map(|s| format!(r#"{{"track":{},"file":{},"offset":{},"length":{}}}"#, s.number, s.file, s.offset, s.length.map_or("null".to_string(), |l| l.to_string())))
                .collect();
            unsafe { put(format!("[{}]", items.join(",")).as_bytes(), out, len) }
        }
        Err(e) => fail(status::PARSE, e) as i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(f: unsafe extern "C" fn(*const u8, usize, *mut u8, usize) -> i64, input: &str) -> Result<String, i64> {
        let n = unsafe { f(input.as_ptr(), input.len(), std::ptr::null_mut(), 0) };
        if n < 0 {
            return Err(n);
        }
        let mut out = vec![0u8; n as usize];
        assert_eq!(unsafe { f(input.as_ptr(), input.len(), out.as_mut_ptr(), out.len()) }, n);
        Ok(String::from_utf8(out).unwrap())
    }

    #[test]
    fn a_sheets_files_a_lists_discs_and_the_sbi_beside_come_through_the_c_layer() {
        let sheet = "FILE \"Game (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\nFILE \"Game (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 01 00:00:00\n";
        assert_eq!(call(emusen_disc_companions, sheet).unwrap(), "Game (Track 1).bin\0Game (Track 2).bin\0");
        assert_eq!(call(emusen_disc_playlist, "#EXTM3U\nA (Disc 1).cue\n\nA (Disc 2).cue\n").unwrap(), "A (Disc 1).cue\0A (Disc 2).cue\0");
        assert_eq!(call(emusen_disc_sbi_beside, "/r/Game (Europe).cue").unwrap(), "/r/Game (Europe).sbi");
        assert_eq!(call(emusen_disc_companions, "TRACK 01 AUDIO\n"), Err(status::PARSE as i64));
        let one = "FILE a.bin BINARY\nTRACK 01 MODE1/2048\nINDEX 01 00:00:00\nTRACK 02 AUDIO\nINDEX 01 00:00:10\n";
        assert_eq!(call(emusen_disc_track_spans, one).unwrap(), r#"[{"track":1,"file":0,"offset":0,"length":20480},{"track":2,"file":0,"offset":20480,"length":null}]"#);
        assert_eq!(call(emusen_disc_track_spans, "TRACK 01 AUDIO\n"), Err(status::PARSE as i64));
        assert_eq!(unsafe { emusen_disc_playlist(std::ptr::null(), 0, std::ptr::null_mut(), 0) }, status::NULL as i64);
    }
}
