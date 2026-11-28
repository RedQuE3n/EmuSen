//! `emusen.session`: session inspection, `summary`, `coretop` (refusal or `--once`), and `sysinfo`.

use dianash_module::{Answer, Call, Manifest, Module};
use emusen_galaxia::sysinfo::{self, SysTelemetrySnapshot, TelemetryPaths};
use emusen_galaxia::tree;
use emusen_session_service::{abi, Record, Session};
use serde_json::Value;

/// The commands this module has, with their read-only flags and usage lines.
pub const COMMANDS: &[(&str, bool, &str)] = &[
    (
        "coretop",
        false,
        "  coretop [-w] [--once [--json]]\n                                live htop-style dashboard of the loaded core's hardware (Ctrl+C to exit - interactive terminal only;\n                                -w opens it in a separate window; --once prints one snapshot and exits)",
    ),
    ("summary", false, "  summary                       free-text state dump (whatever isn't structured above yet)"),
    ("sysinfo", true, "  sysinfo [--json]              host CPU/RAM/GPU/storage and EmuSen process resource telemetry"),
];

/// The module: one value, no state of its own.
pub struct SessionCommands;

pub static MODULE: SessionCommands = SessionCommands;

fn summary(call: &Call) -> Answer {
    let session = Session::from_service(call.service(abi::SERVICE, abi::MAJOR));
    match session.and_then(|s| s.summary()) {
        Ok(text) => Answer::ok(text),
        Err(refusal) if refusal.is(abi::status::NO_SESSION) => Answer::ok("No target attached."),
        Err(refusal) => Answer::fault(refusal.words),
    }
}

fn fmt_mib(bytes: Option<u64>) -> String {
    bytes.map_or_else(|| "N/A".to_string(), |b| format!("{:.1} MiB", b as f64 / (1024.0 * 1024.0)))
}

fn fmt_gib(bytes: u64) -> String {
    format!("{:.2} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
}

fn fmt_pct(pct: Option<f64>) -> String {
    pct.map_or_else(|| "N/A".to_string(), |v| format!("{v:.1}%"))
}

fn format_sysinfo_text(snap: &SysTelemetrySnapshot) -> String {
    let mut lines = Vec::new();
    let cpu = &snap.host.cpu;
    lines.push(format!(
        "Host CPU: {} ({}C/{}T)  Load: {} (iowait {})  Freq: {} avg / {} max MHz",
        cpu.model.as_deref().unwrap_or("Unknown CPU"),
        cpu.physical_cores,
        cpu.logical_cores,
        fmt_pct(cpu.total_pct),
        fmt_pct(cpu.iowait_pct),
        cpu.avg_mhz.map_or_else(|| "N/A".to_string(), |m| format!("{m:.0}")),
        cpu.max_mhz.map_or_else(|| "N/A".to_string(), |m| format!("{m:.0}")),
    ));
    if let Some(mem) = &snap.host.memory {
        lines.push(format!(
            "Host RAM: {} / {} used ({} avail)  Swap: {} / {}",
            fmt_gib(mem.used_bytes),
            fmt_gib(mem.total_bytes),
            fmt_gib(mem.available_bytes),
            fmt_gib(mem.swap_used_bytes),
            fmt_gib(mem.swap_total_bytes),
        ));
    } else {
        lines.push("Host RAM: N/A".to_string());
    }
    if let Some(gpu) = &snap.host.gpu {
        let vram = match (gpu.vram_used_bytes, gpu.vram_total_bytes) {
            (Some(u), Some(t)) => format!("{} / {}", fmt_mib(Some(u)), fmt_mib(Some(t))),
            _ => "N/A".to_string(),
        };
        lines.push(format!(
            "Host GPU: {} ({})  Busy: {} (Mem {})  VRAM: {}  GTT: {}  Temp: {}",
            gpu.card,
            gpu.driver.as_deref().unwrap_or("unknown"),
            fmt_pct(gpu.busy_pct),
            fmt_pct(gpu.mem_busy_pct),
            vram,
            fmt_mib(gpu.gtt_used_bytes),
            gpu.temp_c.map_or_else(|| "N/A".to_string(), |c| format!("{c:.1} C")),
        ));
    } else {
        lines.push("Host GPU: N/A".to_string());
    }
    for mount in &snap.host.mounts {
        lines.push(format!(
            "Mount {} ({}) [{}]: {} / {} used ({} free)",
            mount.mountpoint,
            mount.fs_type,
            mount.roles.join(","),
            fmt_gib(mount.used_bytes),
            fmt_gib(mount.total_bytes),
            fmt_gib(mount.free_bytes),
        ));
    }
    let proc_info = &snap.process;
    let bound = if proc_info.single_core_bound { " [1-core bound]" } else { "" };
    lines.push(format!(
        "Process {} CPU: {} host / {} of 1 core{}",
        proc_info.pid,
        fmt_pct(proc_info.cpu_total_host_pct),
        fmt_pct(proc_info.cpu_one_core_pct),
        bound,
    ));
    let mut group_parts = Vec::new();
    for (name, g) in &proc_info.thread_groups {
        if g.count > 0 {
            group_parts.push(format!("{name}({}):{}", g.count, fmt_pct(g.cpu_one_core_pct)));
        }
    }
    if !group_parts.is_empty() {
        lines.push(format!("Thread groups: {}", group_parts.join("  ")));
    }
    if let Some(pm) = &proc_info.memory {
        lines.push(format!(
            "Process RAM: RSS {}  PSS {}  Private {}  Swap {}",
            fmt_mib(Some(pm.rss_bytes)),
            fmt_mib(pm.pss_bytes),
            fmt_mib(pm.private_bytes),
            fmt_mib(pm.swap_bytes),
        ));
    }
    let pg = &proc_info.gpu;
    if pg.available {
        lines.push(format!(
            "Process GPU: gfx {}  compute {}  copy {}  VRAM {}  GTT {}",
            fmt_pct(pg.gfx_pct),
            fmt_pct(pg.compute_pct),
            fmt_pct(pg.copy_pct),
            fmt_mib(pg.vram_bytes),
            fmt_mib(pg.gtt_bytes),
        ));
    } else {
        lines.push("Process GPU: N/A (driver lacks fdinfo)".to_string());
    }
    let st = &proc_info.storage;
    let status_tag = if st.pending { " [pending]" } else if st.stale { " [stale]" } else { "" };
    lines.push(format!(
        "EmuSen Storage{status_tag}: writable {} (saves {}, states {}, media {}, shaders {}, cores {})  games ref {}",
        fmt_mib(Some(st.writable_total_bytes)),
        fmt_mib(Some(st.categories.saves)),
        fmt_mib(Some(st.categories.save_states)),
        fmt_mib(Some(st.categories.media)),
        fmt_mib(Some(st.categories.shaders)),
        fmt_mib(Some(st.categories.cores)),
        fmt_mib(st.games_library_bytes),
    ));
    lines.join("\n")
}

fn sysinfo_cmd(args: &[String]) -> Answer {
    let mut json = false;
    for arg in args {
        if arg == "--json" {
            json = true;
        } else {
            return Answer::fail(format!("sysinfo: unknown option '{arg}'"));
        }
    }
    let paths = TelemetryPaths::from_tree(&tree::tree());
    let snap = sysinfo::snapshot(&paths);
    if json {
        match serde_json::to_string(&snap) {
            Ok(text) => Answer::ok(text),
            Err(err) => Answer::fault(format!("sysinfo: failed to serialize telemetry: {err}")),
        }
    } else {
        Answer::ok(format_sysinfo_text(&snap))
    }
}

fn coretop_once(session: &Session, json: bool) -> Answer {
    let info = match session.info() {
        Ok(info) => info,
        Err(refusal) => return Answer::fail(refusal.words),
    };
    let summary_text = session.summary().ok();
    let telemetry = session.telemetry(0).ok();
    let sprites = session.sprites().ok();
    let palettes = session.palettes().ok();
    let audio_channels = session.audio_channels().ok();
    let sys = sysinfo::snapshot(&TelemetryPaths::from_tree(&tree::tree()));

    if json {
        let doc = serde_json::json!({
            "schema": 1,
            "session": info,
            "summary": summary_text,
            "telemetry": telemetry,
            "sprites_count": sprites.as_ref().map(|v| v.list("sprites").len()),
            "palettes_count": palettes.as_ref().map(|v| v.list("palettes").len()),
            "audio_channels": audio_channels,
            "sysinfo": sys,
        });
        return match serde_json::to_string(&doc) {
            Ok(text) => Answer::ok(text),
            Err(err) => Answer::fault(format!("coretop: failed to serialize snapshot: {err}")),
        };
    }

    let engine = info.optional_text("engine").unwrap_or_else(|| "Unknown".to_string());
    let system = info.optional_text("system").unwrap_or_else(|| "unknown".to_string());
    let frame = info.int("frame");
    let state = if info.flag("paused") { "paused" } else { "running" };
    let mut lines = vec![format!("coretop: {engine} ({system})  frame={frame}  state={state}")];
    lines.push(format!(
        "Process CPU: {} host / {} of 1 core  RAM: RSS {}  GPU VRAM: {}",
        fmt_pct(sys.process.cpu_total_host_pct),
        fmt_pct(sys.process.cpu_one_core_pct),
        fmt_mib(sys.process.memory.as_ref().map(|m| m.rss_bytes)),
        fmt_mib(sys.process.gpu.vram_bytes),
    ));
    if let Some(sc) = sprites.as_ref().map(|v| v.list("sprites").len()) {
        lines.push(format!("Sprites: {sc}"));
    }
    if let Some(pc) = palettes.as_ref().map(|v| v.list("palettes").len()) {
        lines.push(format!("Palettes: {pc}"));
    }
    if let Some(ac) = audio_channels.as_ref().map(|v| v.list("channels")) {
        let active = ac.iter().filter(|c| c.flag("active")).count();
        lines.push(format!("Audio channels: {active}/{} active", ac.len()));
    }
    if let Some(Value::Object(map)) = &telemetry {
        let counters: Vec<String> = map.iter().take(8).map(|(k, v)| format!("{k}={v}")).collect();
        if !counters.is_empty() {
            lines.push(format!("Telemetry: {}", counters.join("  ")));
        }
    }
    if let Some(s) = summary_text {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            lines.push(trimmed.to_string());
        }
    }
    Answer::ok(lines.join("\n"))
}

/// `coretop`: handles `--once [--json]` cleanly, and refuses `-w` or bare interactive TTY calls as before.
fn coretop(call: &Call, args: &[String]) -> Answer {
    match Session::from_service(call.service(abi::SERVICE, abi::MAJOR)) {
        Ok(session) => coretop_with_session(Some(&session), None, args),
        Err(refusal) => coretop_with_session(None, Some(refusal), args),
    }
}

fn coretop_with_session(session: Option<&Session>, missing: Option<emusen_session_service::Refusal>, args: &[String]) -> Answer {
    let mut once = false;
    let mut json = false;
    let mut windowed = false;
    for arg in args {
        if arg.eq_ignore_ascii_case("-w") {
            windowed = true;
        } else if arg == "--once" {
            once = true;
        } else if arg == "--json" {
            json = true;
        } else {
            return Answer::fail(format!("coretop: unknown option '{arg}'"));
        }
    }

    let Some(session) = session else {
        return Answer::fail(missing.map_or_else(|| emusen_session_service::words::NO_SESSION.to_string(), |r| r.words));
    };
    if windowed {
        if let Err(refusal) = session.info() {
            return Answer::fail(refusal.words);
        }
        return Answer::fail("coretop: -w is not supported by this frontend.");
    }
    if once {
        return coretop_once(session, json);
    }
    if let Err(refusal) = session.info() {
        return Answer::fail(refusal.words);
    }
    if json {
        return Answer::fail("coretop: --json requires --once");
    }
    Answer::fail("coretop: needs a real interactive terminal (stdin/stdout is redirected, or this console has none)")
}

impl Module for SessionCommands {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(|| {
            Manifest {
                id: "emusen.session",
                version: env!("CARGO_PKG_VERSION"),
                services: &[(abi::SERVICE, abi::MAJOR, 0, false)],
                commands: COMMANDS,
                aliases: &[],
                jobs: &[],
                settings: &[],
                approvals: false,
            }
            .json()
        })
    }

    fn run(&self, call: &Call) -> i32 {
        let args = call.args();
        call.answer(match call.command().as_str() {
            "summary" => summary(call),
            "coretop" => coretop(call, &args),
            "sysinfo" => sysinfo_cmd(&args),
            other => Answer::fault(format!("emusen.session has no command '{other}'")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use emusen_session_service::abi::emusen_session_v1;
    use std::ffi::c_void;

    unsafe extern "C" fn mock_last_error(_ctx: *mut c_void, _out: *mut u8, _len: usize) -> i64 {
        0
    }

    unsafe fn mock_json(text: &[u8], out: *mut u8, len: usize) -> i64 {
        if !out.is_null() && text.len() <= len {
            unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), out, text.len()) };
        }
        text.len() as i64
    }

    unsafe extern "C" fn mock_info(_ctx: *mut c_void, _session: u32, out: *mut u8, len: usize) -> i64 {
        unsafe { mock_json(br#"{"engine":"VenusRT","system":"snes","frame":42,"paused":true}"#, out, len) }
    }

    unsafe extern "C" fn mock_summary(_ctx: *mut c_void, _session: u32, out: *mut u8, len: usize) -> i64 {
        unsafe { mock_json(b"VenusRT snes frame 42", out, len) }
    }

    unsafe extern "C" fn mock_telemetry(_ctx: *mut c_void, _session: u32, _kind: u32, out: *mut u8, len: usize) -> i64 {
        unsafe { mock_json(br#"{"cycles":1000}"#, out, len) }
    }

    fn mock_table() -> emusen_session_v1 {
        let mut table: emusen_session_v1 = unsafe { std::mem::zeroed() };
        table.size = std::mem::size_of::<emusen_session_v1>() as u32;
        table.version = abi::VERSION;
        table.last_error = Some(mock_last_error);
        table.info = Some(mock_info);
        table.summary = Some(mock_summary);
        table.telemetry = Some(mock_telemetry);
        table
    }

    #[test]
    fn the_manifest_names_all_three_commands() {
        let manifest = SessionCommands.manifest();
        assert!(manifest.contains("\"name\":\"summary\""));
        assert!(manifest.contains("\"name\":\"coretop\""));
        assert!(manifest.contains("\"name\":\"sysinfo\""));
        assert!(manifest.starts_with("{\"manifest\":1,\"id\":\"emusen.session\""));
    }

    #[test]
    fn sysinfo_returns_json_and_text_and_rejects_unknown_flags() {
        let json_ans = sysinfo_cmd(&["--json".to_string()]);
        assert_eq!(json_ans.status, 0);
        let v: Value = serde_json::from_str(&json_ans.out).unwrap();
        assert_eq!(v["schema"].as_u64(), Some(1));
        assert!(v["host"]["cpu"]["logical_cores"].as_u64().unwrap_or(0) >= 1);

        let text_ans = sysinfo_cmd(&[]);
        assert_eq!(text_ans.status, 0);
        assert!(text_ans.out.contains("Host CPU:") && text_ans.out.contains("Process "));

        let bad = sysinfo_cmd(&["--bogus".to_string()]);
        assert_eq!(bad.status, 1);
    }

    #[test]
    fn coretop_once_works_while_bare_coretop_refuses_tty() {
        let table = mock_table();
        let session = unsafe { Session::from_table((&table as *const emusen_session_v1).cast()) }.unwrap();

        let no_target = coretop_with_session(None, Some(emusen_session_service::words::no_session()), &[]);
        assert_eq!(no_target.status, 1);
        assert_eq!(no_target.err, emusen_session_service::words::NO_SESSION);

        let bare = coretop_with_session(Some(&session), None, &[]);
        assert_eq!(bare.status, 1);
        assert!(bare.err.contains("needs a real interactive terminal"));

        let win = coretop_with_session(Some(&session), None, &["-w".to_string()]);
        assert_eq!(win.status, 1);
        assert!(win.err.contains("-w is not supported"));

        let json_only = coretop_with_session(Some(&session), None, &["--json".to_string()]);
        assert_eq!(json_only.status, 1);
        assert!(json_only.err.contains("requires --once"));

        let text_ans = coretop_with_session(Some(&session), None, &["--once".to_string()]);
        assert_eq!(text_ans.status, 0);
        assert!(text_ans.out.contains("VenusRT (snes)  frame=42  state=paused"));
        assert!(text_ans.out.contains("cycles=1000"));

        let json_ans = coretop_with_session(Some(&session), None, &["--once".to_string(), "--json".to_string()]);
        assert_eq!(json_ans.status, 0);
        let v: Value = serde_json::from_str(&json_ans.out).unwrap();
        assert_eq!(v["schema"].as_u64(), Some(1));
        assert_eq!(v["session"]["engine"].as_str(), Some("VenusRT"));
        assert_eq!(v["sysinfo"]["schema"].as_u64(), Some(1));
    }
}
