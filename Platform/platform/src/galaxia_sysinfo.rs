//! Galaxia's C layer for host hardware and EmuSen process resource telemetry.

use crate::serenity::guard;
use crate::{fail, status};
use emusen_galaxia::sysinfo::{self, TelemetryPaths};

thread_local! {
    static WAITING: crate::reply::Waiting = const { std::cell::RefCell::new(None) };
}

/// # Safety: `paths_json` must be valid for `paths_json_len` bytes or null with length 0; `out` must be valid for `len` bytes or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_sys_telemetry_snapshot(paths_json: *const u8, paths_json_len: usize, out: *mut u8, len: usize) -> i64 {
    guard(|s| s as i64, || {
        let raw = match unsafe { crate::reply::text(paths_json, paths_json_len, |code, msg| fail(code, msg)) } {
            Ok(raw) => raw,
            Err(status) => return status as i64,
        };
        let paths = match Some(raw.trim()).filter(|s| !s.is_empty()) {
            Some(json) => match serde_json::from_str::<TelemetryPaths>(json) {
                Ok(parsed) => parsed,
                Err(err) => return fail(status::PARSE, format!("paths_json is not valid telemetry paths JSON: {err}")) as i64,
            },
            None => TelemetryPaths::default(),
        };
        if out.is_null() {
            WAITING.with_borrow_mut(|w| *w = None);
        }
        let make = || -> Result<String, i32> {
            let snap = sysinfo::snapshot(&paths);
            serde_json::to_string(&snap).map_err(|err| fail(status::IO, format!("failed to serialize telemetry snapshot: {err}")))
        };
        unsafe { crate::reply::answer(&WAITING, 0, raw, out, len, make, |s| s) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(paths: Option<&str>) -> Result<String, i64> {
        let (ptr, plen) = paths.map_or((std::ptr::null(), 0), |s| (s.as_ptr(), s.len()));
        let len = unsafe { emusen_sys_telemetry_snapshot(ptr, plen, std::ptr::null_mut(), 0) };
        if len < 0 {
            return Err(len);
        }
        let mut out = vec![0u8; len as usize];
        let written = unsafe { emusen_sys_telemetry_snapshot(ptr, plen, out.as_mut_ptr(), out.len()) };
        assert_eq!(written, len);
        Ok(String::from_utf8(out).unwrap())
    }

    #[test]
    fn telemetry_snapshot_validates_inputs_and_returns_structured_json() {
        let bad_utf8 = [0xFFu8];
        assert_eq!(unsafe { emusen_sys_telemetry_snapshot(bad_utf8.as_ptr(), 1, std::ptr::null_mut(), 0) }, status::BAD_STRING as i64);
        assert_eq!(unsafe { emusen_sys_telemetry_snapshot(std::ptr::null(), 5, std::ptr::null_mut(), 0) }, status::NULL as i64);
        assert_eq!(ask(Some("{not-json")), Err(status::PARSE as i64));

        let dir = std::env::temp_dir().join(format!("platform-sysinfo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let ds = dir.join("home");
        let games = dir.join("games");
        std::fs::create_dir_all(ds.join("Saves/Save States")).unwrap();
        std::fs::create_dir_all(&games).unwrap();
        std::fs::write(ds.join("Saves/a.srm"), vec![0u8; 64]).unwrap();
        std::fs::write(ds.join("Saves/Save States/a.state"), vec![0u8; 128]).unwrap();
        std::fs::write(games.join("a.nes"), vec![0u8; 512]).unwrap();

        let paths = serde_json::json!({
            "data_store": ds.to_string_lossy(),
            "games": games.to_string_lossy(),
            "refresh_storage": true
        })
        .to_string();
        let len1 = unsafe { emusen_sys_telemetry_snapshot(paths.as_ptr(), paths.len(), std::ptr::null_mut(), 0) };
        assert!(len1 > 0);
        let mut tiny = [0u8; 4];
        assert_eq!(unsafe { emusen_sys_telemetry_snapshot(paths.as_ptr(), paths.len(), tiny.as_mut_ptr(), tiny.len()) }, len1);
        let mut full = vec![0u8; len1 as usize];
        assert_eq!(unsafe { emusen_sys_telemetry_snapshot(paths.as_ptr(), paths.len(), full.as_mut_ptr(), full.len()) }, len1);
        let v: serde_json::Value = serde_json::from_slice(&full).unwrap();
        assert_eq!(v["schema"].as_u64(), Some(1));
        assert!(v["host"]["cpu"]["logical_cores"].as_u64().unwrap_or(0) >= 1);
        assert_eq!(v["process"]["storage"]["categories"]["saves"].as_u64(), Some(64));
        assert_eq!(v["process"]["storage"]["categories"]["save_states"].as_u64(), Some(128));
        assert_eq!(v["process"]["storage"]["writable_total_bytes"].as_u64(), Some(192));
        assert_eq!(v["process"]["storage"]["games_library_bytes"].as_u64(), Some(512));

        let null_text = ask(None).unwrap();
        let nv: serde_json::Value = serde_json::from_str(&null_text).unwrap();
        assert_eq!(nv["schema"].as_u64(), Some(1));
        assert!(nv["process"]["storage"]["games_library_bytes"].is_null());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
