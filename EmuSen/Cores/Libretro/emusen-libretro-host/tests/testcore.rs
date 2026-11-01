// The libretro host driven through the Rust host with the project's test core, in-process (EmuSen_LibretroHost.md §9).

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use emusen_core_host::library::Library;
use emusen_core_host::machine::{Create, Machine};
use emusen_libretro_testcore::layout::*;
use emusen_libretro_testcore::probe::PROBES;
use emusen_native::core::sys::{event, flags, kind, status};

/// One machine per process, and one library: the tests take turns.
static TURN: Mutex<()> = Mutex::new(());
static LIB: OnceLock<(&'static Library, PathBuf)> = OnceLock::new();

const STEM: &str = "testcore";

fn suffix() -> &'static str {
    emusen_core_entry::library_suffix()
}

/// The entry folder with copies of the libretro host and the test core, as Cores lays it out, and the library loaded.
fn library() -> (&'static Library, PathBuf) {
    let (lib, root) = LIB.get_or_init(|| {
        let deps = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("libretro-{}", std::process::id()));
        let entry = root.join("libretro").join(STEM);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&entry).unwrap();
        let host = entry.join(format!("emusen-libretro-host.{}", suffix()));
        std::fs::copy(deps.join(format!("libemusen_libretro_host.{}", suffix())), &host).unwrap();
        std::fs::copy(deps.join(format!("libemusen_libretro_testcore.{}", suffix())), entry.join(format!("{STEM}_libretro.{}", suffix()))).unwrap();
        std::fs::write(entry.join(format!("{STEM}_libretro.info")), "display_name = \"EmuSen - Test Core\"\ncorename = \"TestCore\"\nlicense = \"GPLv3\"\nauthors = \"EmuSen\"\nsystemid = \"\"\ndatabase = \"\"\n").unwrap();
        let lib = Library::load(&host).unwrap_or_else(|e| panic!("{e:?}"));
        (lib, root.join("libretro"))
    });
    (*lib, root.clone())
}

fn turn() -> MutexGuard<'static, ()> {
    TURN.lock().unwrap_or_else(|e| e.into_inner())
}

const GAME: &[u8] = b"TCG1 a test game";

fn make(settings: &str, formats: u64) -> Machine {
    make_with(settings, formats, &[], &[])
}

fn make_with(settings: &str, formats: u64, files: &[(u32, &[u8])], names: &[(u32, &std::ffi::CStr)]) -> Machine {
    let (lib, _) = library();
    let c = Create { settings, pixel_formats: formats, files, names, ..Create::new(GAME) };
    Machine::create(lib, &c).unwrap_or_else(|r| panic!("{r:?}"))
}

fn ram(m: &mut Machine, at: usize, len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    m.space_read(0, at as u32, &mut out).unwrap();
    out
}

fn u16_at(m: &mut Machine, at: usize) -> u16 {
    let b = ram(m, at, 2);
    u16::from_le_bytes([b[0], b[1]])
}

fn frame(m: &Machine) -> Vec<u8> {
    let mut out = vec![0u8; m.frame_bytes().unwrap()];
    m.frame_copy(&mut out).unwrap();
    out
}

fn drain_all(m: &mut Machine) -> usize {
    let mut total = 0;
    let mut buf = vec![0i16; 1 << 16];
    loop {
        let (n, _) = m.audio_drain(&mut buf).unwrap();
        if n == 0 {
            return total;
        }
        total += n;
    }
}

#[test]
fn info_and_schema_describe_the_core_and_its_options() {
    let _t = turn();
    let (lib, _) = library();
    let info = lib.info_text().unwrap();
    for want in [r#""id":"libretro-testcore""#, r#""name":"TestCore""#, r#""license":"GPLv3""#, r#""host_requires":["named_files"]"#, r#""family":"files""#, r#""deterministic":false"#] {
        assert!(info.contains(want), "{want} in {info}");
    }
    let schema = lib.settings_text().unwrap();
    let unrated = schema.matches(r#""effect":"unrated""#).count();
    assert!(unrated == emusen_libretro_testcore::OPTION_COUNT || unrated == emusen_libretro_testcore::OPTION_COUNT + 1, "every option, and the one declared at load once a game has loaded: {schema}");
    assert!(schema.contains(r#""key":"console_picture""#) && schema.contains(r#""kind":"switch""#), "the host's reserved setting: {schema}");
    assert!(schema.contains(r#""category":"Picture""#) && schema.contains(r#""label":"Pixel format (in its category)""#), "{schema}");
    assert!(schema.contains(r#"{"value":"rgb565","label":"rgb565 (label)"}"#), "{schema}");
}

#[test]
fn each_pixel_format_reaches_the_host_in_a_format_it_accepts() {
    let _t = turn();
    for (setting, formats, want_format, bpp) in [("xrgb8888", 0b011u64, 1u32, 4usize), ("xrgb8888", 0b001, 0, 4), ("rgb565", 0b101, 2, 2), ("0rgb1555", 0b001, 0, 4), ("0rgb1555", 0b101, 2, 2)] {
        let mut m = make(&format!("tc_format={setting}"), formats);
        m.advance().unwrap();
        let f = m.frame_info().unwrap();
        assert_eq!((f.format, f.width, f.height, f.stride as usize), (want_format, BASE_WIDTH as i32, BASE_HEIGHT as i32, MAX_WIDTH as usize * bpp), "{setting} {formats:b}");
        let px = frame(&m);
        if bpp == 4 {
            assert!(px.chunks_exact(4).take(BASE_WIDTH as usize).all(|p| p[3] == 0xFF), "{setting}: alpha opaque");
            let (r, b) = if want_format == 1 { (px[2], px[0]) } else { (px[0], px[2]) };
            assert_eq!(r, if setting == "xrgb8888" { 1 } else { 0 }, "{setting}: red is the frame counter at x 0, five bits of it in 16-bit formats");
            assert_eq!(b, 0, "{setting}: blue is the pad and x");
        }
    }
}

#[test]
fn a_core_drawing_into_the_hosts_buffer_gives_the_same_picture() {
    let _t = turn();
    let picture = |setting: &str| {
        let mut m = make(setting, 0b011);
        for _ in 0..3 {
            m.advance().unwrap();
        }
        (ram(&mut m, DREW_IN_FRAMEBUFFER, 1)[0], frame(&m))
    };
    let (drew, drawn) = picture("tc_draw=framebuffer");
    let (copied, copy) = picture("tc_draw=copy");
    assert_eq!((drew, copied), (1, 0));
    assert_eq!(copy, drawn);
}

#[test]
fn a_second_machine_while_one_lives_is_refused() {
    let _t = turn();
    let (lib, _) = library();
    let first = make("", 0b011);
    let refused = Machine::create(lib, &Create::new(GAME)).err().expect("a second machine while one lives");
    assert_eq!(refused.status, -9);
    drop(first);
    make("", 0b011).advance().unwrap();
}

#[test]
fn sound_keeps_its_exact_rate_with_no_drift() {
    let _t = turn();
    for (setting, num, den) in [("48000", 48_000u64, 1u64), ("32040.5", 64_081, 2)] {
        let mut m = make(&format!("tc_rate={setting}"), 0b011);
        let info = m.machine_info().unwrap();
        assert!(info.contains(&format!(r#""rate_exact":{{"num":{num},"den":{den}}}"#)), "{info}");
        let mut samples = 0;
        for n in 1..=3600u64 {
            m.advance().unwrap();
            samples += drain_all(&mut m) / 2;
            assert_eq!(samples as u64, n * num / (den * 60), "frame {n} of {setting}");
        }
        assert_eq!(m.audio_rate(), ((num + den / 2) / den) as i32);
    }
}

#[test]
fn the_single_sample_path_gives_the_same_sound() {
    let _t = turn();
    let collect = |s: &str| {
        let mut m = make(s, 0b011);
        let mut all = Vec::new();
        let mut buf = vec![0i16; 1 << 15];
        for _ in 0..10 {
            m.advance().unwrap();
            let (n, _) = m.audio_drain(&mut buf).unwrap();
            all.extend_from_slice(&buf[..n]);
        }
        all
    };
    let batch = collect("tc_sound=batch");
    assert_eq!(batch.len(), 8000 * 2);
    assert_eq!(collect("tc_sound=single"), batch);
}

#[test]
fn buttons_sticks_and_triggers_reach_the_core_in_its_units() {
    let _t = turn();
    let mut m = make("", 0b011);
    assert_eq!(ram(&mut m, PORT_DEVICE, 2), [TEST_PAD_DEVICE as u8, TEST_PAD_DEVICE as u8], "the analog subclass the core offers");
    m.set_buttons(0, (1 << 8) | (1 << 3), 0xFFFF).unwrap();
    m.set_axis(0, 0, -1.0).unwrap();
    m.set_axis(0, 1, 0.5).unwrap();
    m.set_axis(0, 4, 1.0).unwrap();
    m.advance().unwrap();
    assert_eq!(u16_at(&mut m, PAD), (1 << 8) | (1 << 3));
    assert_eq!((u16_at(&mut m, AXES) as i16, u16_at(&mut m, AXES + 2) as i16), (-32767, 16384));
    assert_eq!((u16_at(&mut m, TRIGGERS) as i16, u16_at(&mut m, TRIGGERS + 2) as i16), (32767, 0));
    assert_eq!(m.set_buttons(9, 1, 1), Err(status::NO_SUCH_PORT));
    m.set_present(0, false).unwrap();
    m.advance().unwrap();
    assert_eq!(ram(&mut m, PORT_DEVICE, 1), [0], "an absent pad is RETRO_DEVICE_NONE");
    assert_eq!(u16_at(&mut m, PAD), 0, "and reads nothing");
}

#[test]
fn a_state_loads_whole_and_each_refusal_is_its_own() {
    let _t = turn();
    let mut m = make("", 0b011);
    for _ in 0..30 {
        m.advance().unwrap();
    }
    let mut saved = vec![0u8; m.state_size(kind::FULL).unwrap()];
    m.state_save(kind::FULL, &mut saved).unwrap();
    let picture = { m.advance().unwrap(); frame(&m) };
    for _ in 0..50 {
        m.advance().unwrap();
    }
    m.state_load(&saved).unwrap();
    m.advance().unwrap();
    assert_eq!(frame(&m), picture, "the frame after the load is the frame after the save");
    assert_eq!(m.state_load(b"not a state at all, not even close to one, and longer than the header is long"), Err(status::FOREIGN));
    let mut damaged = saved.clone();
    *damaged.last_mut().unwrap() ^= 1;
    assert_eq!(m.state_load(&damaged), Err(status::TRUNCATED));
    // §10h: the core build's SHA-256 is in every state, and another build's state is refused in plain words.
    assert!(saved[80..112].iter().any(|&b| b != 0), "the build recorded");
    let mut other = saved.clone();
    other[80..112].fill(0xAB);
    assert_eq!(m.state_load(&other), Err(status::FOREIGN));
    let said = m.last_error();
    assert!(said.contains("another build of") && said.contains(&"ab".repeat(32)) && said.contains("the build installed is"), "{said}");
    let layout = m.layout(kind::FULL).unwrap();
    assert!(layout.starts_with("0 4 bytes header.magic\n") && layout.contains("64 16 bytes header.disc\n80 32 bytes header.build\n112 "), "{layout}");
}

#[test]
fn a_load_the_core_refuses_half_way_leaves_the_machine_as_it_was() {
    let _t = turn();
    let mut m = make("tc_unserialize=fail", 0b011);
    for _ in 0..10 {
        m.advance().unwrap();
    }
    let mut early = vec![0u8; m.state_size(kind::FULL).unwrap()];
    m.state_save(kind::FULL, &mut early).unwrap();
    for _ in 0..20 {
        m.advance().unwrap();
    }
    let before = ram(&mut m, 0, RAM_SIZE);
    assert_eq!(m.state_load(&early), Err(-16));
    assert_eq!(ram(&mut m, 0, RAM_SIZE), before, "restored");
    assert!(m.last_error().contains("restored"), "{}", m.last_error());
}

#[test]
fn quirks_withdraw_the_snapshot_and_record_the_platform() {
    let _t = turn();
    let m = make("tc_quirk=incomplete", 0b011);
    assert!(m.machine_info().unwrap().contains(r#""kinds":[0]"#));
    assert_eq!(m.state_size(kind::SNAPSHOT), Err(status::NOT_SUPPORTED));
    drop(m);
    let mut m = make("", 0b011);
    assert!(m.machine_info().unwrap().contains(r#""kinds":[0,1]"#));
    let n = m.state_size(kind::SNAPSHOT).unwrap();
    let mut snap = vec![0u8; n];
    m.state_save(kind::SNAPSHOT, &mut snap).unwrap();
    m.state_load(&snap).unwrap();
    drop(m);
    let mut m = make("tc_quirk=platform", 0b011);
    let mut s = vec![0u8; m.state_size(kind::FULL).unwrap()];
    m.state_save(kind::FULL, &mut s).unwrap();
    assert_eq!(u16::from_le_bytes([s[6], s[7]]) & 1, 1, "the platform recorded");
    s[20] ^= 0x7F;
    assert_eq!(m.state_load(&s), Err(-15));
}

#[test]
fn save_ram_is_battery_file_0_and_tracks_its_changes() {
    let _t = turn();
    let mut sram = vec![0u8; SRAM_SIZE];
    sram[100] = 0x5A;
    let mut m = make_with("", 0b011, &[(0, &sram)], &[]);
    let mut got = Vec::new();
    assert_eq!(m.battery(0, &mut got).unwrap() & flags::BATTERY_CHANGED, 0);
    assert_eq!(got[100], 0x5A, "passed in after load");
    m.advance().unwrap();
    let f = m.battery(0, &mut got).unwrap();
    assert_eq!(f & (flags::BATTERY_CHANGED | flags::BATTERY_TRACKED), flags::BATTERY_CHANGED | flags::BATTERY_TRACKED);
    m.battery_saved(0).unwrap();
    assert_eq!(m.battery(0, &mut got).unwrap() & flags::BATTERY_CHANGED, 0);
    assert!(m.machine_info().unwrap().contains(r#""suffix":".srm""#));
}

#[test]
fn files_in_the_save_folder_are_named_saves_and_the_system_folder_is_not() {
    let _t = turn();
    let (_, root) = library();
    let notes = c"testcore/notes.txt";
    let mut m = make_with("tc_files=on", 0b011, &[(flags::FILE_NAMED, b"old notes\n")], &[(flags::FILE_NAMED, notes)]);
    let save = root.join(".data").join(STEM).join("save");
    assert_eq!(std::fs::read(save.join("testcore/notes.txt")).unwrap(), b"first notes\n", "the core rewrote the file passed in");
    for _ in 0..15 {
        m.advance().unwrap();
    }
    let events: Vec<_> = m.events().unwrap().to_vec();
    assert!(events.iter().any(|e| e.kind == event::BATTERY && e.a == flags::FILE_NAMED as i64), "{events:?}");
    let info = m.machine_info().unwrap();
    assert!(info.contains(r#""name":"testcore/notes.txt""#), "{info}");
    assert!(!info.contains("card.bin"), "the card in the system folder is the core's own data: {info}");
    let mut got = Vec::new();
    m.battery(flags::FILE_NAMED, &mut got).unwrap();
    assert_eq!(got, b"first notes\n");
    for _ in 0..15 {
        m.advance().unwrap();
    }
    let info = m.machine_info().unwrap();
    assert!(info.contains(r#""name":"testcore/notes.txt","length":0,"kind":"console","removed":true"#), "{info}");
    assert!(root.join(".data").join(STEM).join("system/testcore/card.bin").exists());
}

#[test]
fn options_reach_the_core_and_its_own_changes_are_noted() {
    let _t = turn();
    let mut m = make("tc_set_variable=on", 0b011);
    for _ in 0..6 {
        m.advance().unwrap();
    }
    assert!(m.setting_notes().unwrap().contains("The core set this to changed itself."), "{}", m.setting_notes().unwrap());
    m.set_settings("tc_note=plain").unwrap();
    m.advance().unwrap();
    assert_eq!(ram(&mut m, OPTION_UPDATES, 4), 1u32.to_le_bytes());
    drop(m);
    let m = make("", 0b011);
    assert!(m.setting_notes().unwrap().contains(r#""tc_note":"Not used with the current settings.""#), "{}", m.setting_notes().unwrap());
}

/// The picture's size and whether its rows come in blended pairs, as the test core's scale and deinterlacing draw it.
fn shape(m: &mut Machine) -> (i32, i32, bool) {
    m.advance().unwrap();
    let f = m.frame_info().unwrap();
    let px = frame(m);
    let row = |y: usize| &px[y * f.stride as usize..y * f.stride as usize + f.width as usize * 4];
    let scale = f.width as usize / BASE_WIDTH as usize;
    (f.width, f.height, row(0) == row(scale))
}

#[test]
fn the_console_picture_holds_the_named_exceptions_and_gives_the_players_values_back() {
    let _t = turn();
    let mut m = make("tc_scale=2\ntc_deinterlace=blend", 0b011);
    assert_eq!(shape(&mut m), (MAX_WIDTH as i32, MAX_HEIGHT as i32, true), "the player's upscaled, blended picture");
    let notes = m.setting_notes().unwrap();
    assert!(notes.contains(r#""!locked":"console_picture""#), "{notes}");
    m.set_settings("console_picture=true").unwrap();
    assert_eq!(shape(&mut m), (BASE_WIDTH as i32, BASE_HEIGHT as i32, false), "native and unblended with the CRT filter on");
    let notes = m.setting_notes().unwrap();
    assert!(notes.contains(r#""!locked":"tc_scale tc_deinterlace console_picture""#), "{notes}");
    assert!(notes.contains(r#""tc_scale":"Held at 1 (label) while the CRT filter is on: the test core's native size"#) && notes.contains("decided 2026-11-01"), "{notes}");
    m.set_settings("tc_scale=2").unwrap();
    assert_eq!(shape(&mut m).0, BASE_WIDTH as i32, "a player's change while held waits for the filter to be turned off");
    m.set_settings("console_picture=false").unwrap();
    assert_eq!(shape(&mut m), (MAX_WIDTH as i32, MAX_HEIGHT as i32, true), "the player's values back");
    assert!(m.setting_notes().unwrap().contains(r#""!locked":"console_picture""#));
    drop(m);
    let mut m = make("tc_scale=2\ntc_deinterlace=blend\nconsole_picture=true", 0b011);
    assert_eq!(shape(&mut m), (BASE_WIDTH as i32, BASE_HEIGHT as i32, false), "held from the first frame when made with it on");
}

/// The answer the design gives each environment command at this stage: answered, or refused (EmuSen_LibretroHost.md
/// §4.10; the hardware-rendering commands are refused until stage 3; rumble answered since 1.17).
const ANSWERED: &[&str] = &[
    "GET_CAN_DUPE", "SET_MESSAGE", "SET_PERFORMANCE_LEVEL", "GET_SYSTEM_DIRECTORY", "SET_PIXEL_FORMAT", "SET_INPUT_DESCRIPTORS", "SET_DISK_CONTROL_INTERFACE",
    "GET_VARIABLE", "GET_VARIABLE_UPDATE", "SET_SUPPORT_NO_GAME", "GET_LIBRETRO_PATH", "SET_FRAME_TIME_CALLBACK", "GET_INPUT_DEVICE_CAPABILITIES", "GET_LOG_INTERFACE",
    "GET_PERF_INTERFACE", "GET_CORE_ASSETS_DIRECTORY", "GET_SAVE_DIRECTORY", "SET_SYSTEM_AV_INFO", "SET_PROC_ADDRESS_CALLBACK", "SET_CONTROLLER_INFO", "SET_GEOMETRY",
    "GET_LANGUAGE", "GET_CURRENT_SOFTWARE_FRAMEBUFFER", "SET_SUPPORT_ACHIEVEMENTS", "SET_SERIALIZATION_QUIRKS", "GET_AUDIO_VIDEO_ENABLE", "GET_FASTFORWARDING",
    "GET_TARGET_REFRESH_RATE", "GET_INPUT_BITMASKS", "GET_CORE_OPTIONS_VERSION", "SET_CORE_OPTIONS_DISPLAY", "GET_DISK_CONTROL_INTERFACE_VERSION",
    "SET_DISK_CONTROL_EXT_INTERFACE", "GET_MESSAGE_INTERFACE_VERSION", "SET_MESSAGE_EXT", "GET_INPUT_MAX_USERS", "SET_MINIMUM_AUDIO_LATENCY", "GET_GAME_INFO_EXT",
    "SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK", "SET_VARIABLE", "GET_THROTTLE_STATE", "GET_SAVESTATE_CONTEXT", "GET_JIT_CAPABLE",
    "GET_RUMBLE_INTERFACE",
    // Since stage 3: the Vulkan negotiation, the probe's zeroed interface reading as type 0, Vulkan.
    "SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE", "GET_PREFERRED_HW_RENDER", "GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT",
];

#[test]
fn rumble_reaches_the_host_as_each_motors_level() {
    let _t = turn();
    let mut m = make("tc_rumble=on", 0b011);
    let info = m.machine_info().unwrap();
    let motors = r#""outputs":[{"output":0,"kind":"motor","motor":"strong","label":"Strong motor"},{"output":1,"kind":"motor","motor":"weak","label":"Weak motor"}]"#;
    assert_eq!(info.matches(motors).count(), 2, "{info}");
    let mut levels = std::collections::BTreeMap::new();
    let mut raised = 0;
    for n in 0..30 {
        m.advance().unwrap();
        for e in m.events().unwrap().iter().filter(|e| e.kind == event::OUTPUT) {
            levels.insert(((e.a >> 32) as u32, e.a as u32), e.b);
            raised += 1;
        }
        let frame = u32::from_le_bytes(ram(&mut m, FRAME_COUNTER, 4).try_into().unwrap()) as u64;
        for (port, strong, weak) in emusen_libretro_testcore::rumble_pattern(frame) {
            assert_eq!((levels.get(&(port, 0)).copied(), levels.get(&(port, 1)).copied()), (Some(strong as i64), Some(weak as i64)), "frame {frame}, port {port}");
        }
        if n == 0 {
            assert_eq!(raised, 4, "every declared output raised once after machine info is read");
        }
    }
    // Port 1's motors hold still but for one change, so only changes are raised.
    assert_eq!(raised, 4 + 29 * 2 + 1, "{levels:?}");
}

#[test]
fn every_environment_command_is_answered_or_refused_as_designed() {
    let _t = turn();
    let mut m = make("tc_probe=on", 0b011);
    let got = ram(&mut m, PROBE, PROBES.len());
    let wrong: Vec<String> = PROBES
        .iter()
        .zip(&got)
        .filter(|((name, _, _), g)| **g != ANSWERED.contains(name) as u8)
        .map(|((name, _, _), &g)| format!("{name}: {}", if g == 1 { "answered" } else { "refused" }))
        .collect();
    assert!(wrong.is_empty(), "{wrong:?}");
    assert!(ANSWERED.iter().all(|a| PROBES.iter().any(|p| p.0 == *a)), "every answered name is probed");
    let mut log = String::new();
    m.log_drain(&mut log).unwrap();
    assert!(log.contains("refused environment command 1 (SET_ROTATION)"), "a refusal is logged: {log}");
    assert_eq!(log.matches("(SET_ROTATION)").count(), 1, "once");
}

#[test]
fn the_cores_memory_is_a_space_the_hosts_cheats_write() {
    let _t = turn();
    for setting in ["system_ram", "maps"] {
        let mut m = make(&format!("tc_memory={setting}"), 0b011);
        let info = m.machine_info().unwrap();
        let name = if setting == "maps" { "BUS" } else { "RAM" };
        assert!(info.contains(&format!(r#""name":"{name}""#)), "{info}");
        for _ in 0..59 {
            m.advance().unwrap();
            m.space_write(0, LIVES as u32, &[9]).unwrap();
        }
        m.advance().unwrap();
        assert_eq!(ram(&mut m, LIVES, 1), [8], "{setting}: the game's own write lands after the poke");
        m.space_write(0, LIVES as u32, &[9]).unwrap();
        assert_eq!(ram(&mut m, LIVES, 1), [9]);
    }
    let m = make("tc_memory=none", 0b011);
    let info = m.machine_info().unwrap();
    assert!(!info.contains(r#""name":"RAM""#) && info.contains(r#""name":"SRAM""#), "{info}");
}

#[test]
fn geometry_timing_duplicates_and_shutdown_reach_the_host() {
    let _t = turn();
    let mut m = make("tc_dupes=on\ntc_shutdown=30", 0b011);
    m.advance().unwrap();
    m.advance().unwrap();
    let s2 = m.frame_info().unwrap().serial;
    m.advance().unwrap();
    assert_eq!(m.frame_info().unwrap().serial, s2, "frame 3 is a duplicate and keeps the picture");
    m.advance().unwrap();
    assert_eq!(m.frame_info().unwrap().serial, s2 + 1);
    for _ in 4..30 {
        m.advance().unwrap();
    }
    assert_eq!(m.advance().map_err(|e| e.0), Err(-12), "the core asked to stop at frame 30");
    drop(m);
    let mut m = make("tc_geometry=on", 0b011);
    let mut kinds = Vec::new();
    for _ in 0..121 {
        m.advance().unwrap();
        kinds.extend(m.events().unwrap().iter().map(|e| (e.kind, e.a, e.b)));
    }
    assert!(kinds.contains(&(event::GEOMETRY, 256, 224)), "{kinds:?}");
    assert!(kinds.contains(&(event::AUDIO_RATE, 44_100, 0)), "{kinds:?}");
    assert!(m.machine_info().unwrap().contains(r#""frame_rate":{"num":50,"den":1}"#));
}

#[test]
fn a_log_line_from_a_thread_of_the_core_arrives_formatted() {
    let _t = turn();
    let mut m = make("tc_log_thread=on", 0b011);
    let mut log = String::new();
    m.log_drain(&mut log).unwrap();
    assert!(log.contains("from a thread formatted 42"), "{log}");
}

#[test]
fn the_frame_time_callback_is_given_the_nominal_time() {
    let _t = turn();
    let mut m = make("", 0b011);
    for _ in 0..10 {
        m.advance().unwrap();
    }
    let t = i64::from_le_bytes(ram(&mut m, FRAME_TIME, 8).try_into().unwrap());
    assert_eq!(t, 10 * (1_000_000 / 60));
}

#[test]
fn a_disc_changes_through_the_cores_disk_control() {
    let _t = turn();
    let mut m = make("", 0b011);
    let info = m.machine_info().unwrap();
    assert!(info.contains(r#""drives":[{"drive":0,"label":"Disc drive","kind":"disc""#), "{info}");
    assert_eq!(m.media_insert(0, Some(GAME), &[], 0), Err(status::LID_CLOSED));
    m.media_lid(0, true).unwrap();
    m.advance().unwrap();
    assert_eq!(ram(&mut m, DISC, 3), [0, 1, 1], "the tray open, one image");
    m.media_insert(0, Some(b"TCG1 the second disc"), &[], 0).unwrap();
    m.media_lid(0, false).unwrap();
    m.advance().unwrap();
    assert_eq!(ram(&mut m, DISC, 3), [1, 0, 2], "the second image in, the tray closed");
    assert_eq!(m.media_lid(1, true), Err(status::NO_SUCH_DRIVE));
}


#[test]
fn options_declared_while_loading_are_kept_for_the_next_load() {
    let _t = turn();
    let (_, root) = library();
    let m = make("tc_late_option=on", 0b011);
    assert!(m.setting_notes().unwrap().contains("declared options while loading the game"), "{}", m.setting_notes().unwrap());
    let kept = std::fs::read_to_string(root.join(".data").join(STEM).join("declared-options.txt")).unwrap();
    assert!(kept.contains("option\ttc_late\tprobe\tfirst\tDeclared at load"), "{kept}");
    assert!(kept.contains("value\ttc_late\tsecond\tsecond (label)"), "{kept}");
}

#[test]
fn a_core_that_cannot_start_the_game_is_refused_in_its_own_words() {
    let _t = turn();
    let (lib, _) = library();
    let refused = Machine::create(lib, &Create { settings: "tc_fail=load", pixel_formats: 0b011, ..Create::new(GAME) }).err().expect("load refused");
    assert_eq!(refused.status, -13);
    assert!(refused.words.contains(emusen_libretro_testcore::FAIL_AT_LOAD), "{}", refused.words);
    let mut m = make("tc_fail=first_frame", 0b011);
    for _ in 0..2 {
        assert_eq!(m.advance().map_err(|e| e.0), Err(-18), "every frame after the core said it could not start");
        assert!(m.last_error().contains(&format!("it could not start: {}", emusen_libretro_testcore::FAIL_AT_FIRST_FRAME)), "{}", m.last_error());
    }
    assert_eq!(m.library().words(-18), "the core could not start the game; its log says why");
    drop(m);
    make("", 0b011).advance().unwrap();
}

#[test]
fn a_picture_shown_a_frame_late_is_stale_once_after_a_load_and_exact_without_the_lag() {
    let _t = turn();
    for (lag, exact) in [("off", true), ("on", false)] {
        let mut m = make(&format!("tc_lag={lag}"), 0b011);
        for _ in 0..10 {
            m.advance().unwrap();
        }
        let mut saved = vec![0u8; m.state_size(kind::FULL).unwrap()];
        let len = m.state_save(kind::FULL, &mut saved).unwrap();
        let after = |m: &mut Machine| -> Vec<Vec<u8>> {
            (0..2).map(|_| {
                m.advance().unwrap();
                frame(m)
            }).collect()
        };
        let first = after(&mut m);
        for _ in 0..20 {
            m.advance().unwrap();
        }
        m.state_load(&saved[..len]).unwrap();
        let again = after(&mut m);
        assert_eq!(again[1], first[1], "tc_lag={lag}: the second picture after the load is the saved timeline's");
        assert_eq!(again[0] == first[0], exact, "tc_lag={lag}: the first picture after the load");
    }
}
