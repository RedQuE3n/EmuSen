//! The settings service through the shell's guard (module ABI 1.1): a module reaches only the keys its manifest names,
//! one that names none is handed no service, and an approval passes only from a command typed at the prompt
//! (EmuSen_DianaSh.md §9.9).

use dianash_module::abi::{dianash_bytes, dianash_service, dianash_settings_v1, status, SETTINGS_CONSOLE, SETTINGS_MAJOR, SETTINGS_SERVICE};
use dianash_module::{Answer, Call, Manifest, Module};
use emusen_dianash::{Config, Shell};
use std::ffi::c_void;
use std::sync::Mutex;

/// What the host's table was asked, in order.
static ASKED: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// The two tests share ASKED, so they take turns.
static TURN: Mutex<()> = Mutex::new(());

fn words(b: dianash_bytes) -> String {
    unsafe { b.slice() }.map(|s| String::from_utf8_lossy(s).into_owned()).unwrap_or_else(|| "-".into())
}

unsafe extern "C" fn host_get(_: *mut c_void, scope: u32, target: dianash_bytes, key: dianash_bytes, out: *mut u8, cap: usize, needed: *mut i64) -> i32 {
    ASKED.lock().unwrap().push(format!("get {scope} {} {}", words(target), words(key)));
    let value = b"Sapphire";
    unsafe { *needed = value.len() as i64 };
    if cap < value.len() {
        return status::BUFFER_TOO_SMALL;
    }
    unsafe { std::ptr::copy_nonoverlapping(value.as_ptr(), out, value.len()) };
    status::OK
}

unsafe extern "C" fn host_set(_: *mut c_void, scope: u32, target: dianash_bytes, key: dianash_bytes, value: dianash_bytes) -> i32 {
    ASKED.lock().unwrap().push(format!("set {scope} {} {} {}", words(target), words(key), words(value)));
    status::OK
}

unsafe extern "C" fn host_approve(_: *mut c_void, sha: dianash_bytes) -> i32 {
    ASKED.lock().unwrap().push(format!("approve {}", words(sha)));
    status::OK
}

unsafe extern "C" fn host_revoke(_: *mut c_void, sha: dianash_bytes) -> i32 {
    ASKED.lock().unwrap().push(format!("revoke {}", words(sha)));
    status::OK
}

unsafe extern "C" fn host_error(_: *mut c_void, _: *mut u8, _: usize, needed: *mut i64) -> i32 {
    unsafe { *needed = 0 };
    status::OK
}

struct Host(dianash_settings_v1);

// SAFETY: the table holds no state; its functions take a null ctx.
unsafe impl Sync for Host {}

static HOST: Host = Host(dianash_settings_v1 { size: std::mem::size_of::<dianash_settings_v1>() as u32, ctx: std::ptr::null_mut(), get: host_get, set: host_set, approve: host_approve, revoke: host_revoke, last_error: host_error });

/// A module that asks the service whatever its arguments say: `ask get <key>`, `ask set <key> <value>`, `ask approve <sha>`, `ask revoke <sha>`.
struct Asker {
    manifest: fn() -> &'static str,
}

fn run(call: &Call) -> Answer {
    let args = call.args();
    let Some(s) = call.settings() else { return Answer::fail("no settings service") };
    let said = match (args.get(1).map(String::as_str), args.get(2), args.get(3)) {
        (Some("get"), Some(key), _) => s.get(SETTINGS_CONSOLE, "PlayStation 2", key).map(|v| v.unwrap_or_default()),
        (Some("set"), Some(key), Some(value)) => s.set(SETTINGS_CONSOLE, "PlayStation 2", key, Some(value)).map(|_| "set".into()),
        (Some("approve"), Some(sha), _) => s.approve(sha).map(|_| "approved".into()),
        (Some("revoke"), Some(sha), _) => s.revoke(sha).map(|_| "revoked".into()),
        _ => Err("usage".into()),
    };
    match said {
        Ok(out) => Answer::ok(format!("{out} at_prompt={}\n", call.at_prompt())),
        Err(words) => Answer::fail(words),
    }
}

impl Module for Asker {
    fn manifest(&self) -> &'static str {
        (self.manifest)()
    }

    fn run(&self, call: &Call) -> i32 {
        call.answer(run(call))
    }
}

fn engine_manifest() -> &'static str {
    static M: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    M.get_or_init(|| Manifest { id: "test.engine", version: "1", services: &[(SETTINGS_SERVICE, SETTINGS_MAJOR, 0, false)], commands: &[("ask", false, "ask")], aliases: &[], jobs: &[], settings: &["Engine"], approvals: true }.json())
}

fn plain_manifest() -> &'static str {
    static M: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    M.get_or_init(|| Manifest { id: "test.plain", version: "1", services: &[(SETTINGS_SERVICE, SETTINGS_MAJOR, 0, false)], commands: &[("plain", false, "plain")], aliases: &[], jobs: &[], settings: &[], approvals: false }.json())
}

static ENGINE: Asker = Asker { manifest: engine_manifest };
static PLAIN: Asker = Asker { manifest: plain_manifest };

fn shell(interactive: bool) -> Shell {
    let root = std::env::temp_dir().join(format!("dianash-settings-{:?}", std::thread::current().id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut shell = Shell::new(Config { sessions: false, interactive, root: Some(root.to_string_lossy().into_owned()), ..Config::default() });
    shell.add_module(&ENGINE).unwrap();
    shell.add_module(&PLAIN).unwrap();
    shell.add_module(&dianash_text::MODULE).unwrap();
    let name = SETTINGS_SERVICE.as_bytes();
    shell
        .set_services(&[dianash_service { size: std::mem::size_of::<dianash_service>() as u32, name: dianash_bytes::of(name), major: SETTINGS_MAJOR, minor: 0, table: &HOST.0 as *const dianash_settings_v1 as *const c_void }])
        .unwrap();
    shell
}

fn line(shell: &mut Shell, text: &str) -> String {
    shell.submit(0, text).unwrap();
    shell.take_output()
}

#[test]
fn a_module_reaches_only_the_keys_its_manifest_names_and_one_naming_none_has_no_service() {
    let _turn = TURN.lock().unwrap_or_else(|p| p.into_inner());
    let mut sh = shell(true);
    ASKED.lock().unwrap().clear();
    assert_eq!(line(&mut sh, "ask get Engine"), "Sapphire at_prompt=true\n");
    assert_eq!(line(&mut sh, "ask set Engine ExtraPS2"), "set at_prompt=true\n");
    let refused = line(&mut sh, "ask set Volume 3");
    assert!(refused.contains("names no setting \"Volume\""), "{refused}");
    assert!(line(&mut sh, "ask get Volume").contains("names no setting"));
    assert!(line(&mut sh, "plain get Engine").contains("no settings service"));
    assert_eq!(*ASKED.lock().unwrap(), ["get 0 PlayStation 2 Engine", "set 0 PlayStation 2 Engine ExtraPS2"]);
}

#[test]
fn an_approval_passes_only_from_a_command_typed_at_the_prompt() {
    let _turn = TURN.lock().unwrap_or_else(|p| p.into_inner());
    let mut sh = shell(true);
    let sha = "a".repeat(64);
    ASKED.lock().unwrap().clear();
    assert_eq!(line(&mut sh, &format!("ask approve {sha}")), "approved at_prompt=true\n");
    for script in [format!("echo $(ask approve {sha})"), format!("echo x | ask approve {sha}")] {
        let out = line(&mut sh, &script);
        assert!(out.contains("approved at the prompt, not by a script"), "{script}: {out}");
    }
    assert_eq!(line(&mut sh, &format!("echo x | ask revoke {sha}")), "revoked at_prompt=false\n");
    let root = std::env::temp_dir().join(format!("dianash-settings-{:?}", std::thread::current().id()));
    std::fs::write(root.join("approve.sh"), format!("ask approve {sha}\n")).unwrap();
    assert!(line(&mut sh, "source /approve.sh").contains("approved at the prompt"), "a sourced script approved");
    assert_eq!(*ASKED.lock().unwrap(), [format!("approve {sha}"), format!("revoke {sha}")]);
}
