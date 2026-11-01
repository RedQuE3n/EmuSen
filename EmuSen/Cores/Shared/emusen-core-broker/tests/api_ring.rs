// The ring through the Rust runtime's frontend API 1.4: an installed libretro entry opened by emusen_crystal_open with
// OPEN_RING and Serenity's device, each frame's picture drawn from the lent ring by a chain as the same frame read back
// and uploaded; the ring given back, lent again at MACHINE_INFO, and released after a crash (EmuSen_LibretroHost.md §10l).
// Runs on the card EMUSEN_GPU_DEVICE names (6800 when unset); EMUSEN_VALIDATE=1 holds the host's device to the layer.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use emusen_core_host::entry::Entry;
use emusen_core_host::trust::MANIFEST;
use emusen_crystal::abi::{self, emusen_crystal_config, emusen_crystal_frame_info, emusen_crystal_open_params, emusen_crystal_runtime};
use emusen_galaxia::tree::{Override, tree_mut};
use emusen_platform::crystal as api;
use emusen_platform::serenity_device as sd;

static TURN: Mutex<()> = Mutex::new(());
const SUFFIX: &str = if cfg!(target_os = "macos") { "dylib" } else { "so" };
const STOCK: &str = "#version 450\nlayout(push_constant) uniform Push { vec4 SourceSize; vec4 OriginalSize; vec4 OutputSize; uint FrameCount; } params;\nlayout(std140, set = 0, binding = 0) uniform UBO { mat4 MVP; } global;\n#pragma stage vertex\nlayout(location = 0) in vec4 Position;\nlayout(location = 1) in vec2 TexCoord;\nlayout(location = 0) out vec2 vTexCoord;\nvoid main() { gl_Position = global.MVP * Position; vTexCoord = TexCoord; }\n#pragma stage fragment\nlayout(location = 0) in vec2 vTexCoord;\nlayout(location = 0) out vec4 FragColor;\nlayout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { FragColor = texture(Source, vTexCoord); }\n";

fn turn() -> MutexGuard<'static, ()> {
    TURN.lock().unwrap_or_else(|e| e.into_inner())
}

fn deps() -> PathBuf {
    std::env::current_exe().unwrap().parent().unwrap().to_path_buf()
}

fn hash(path: &Path) -> String {
    emusen_core_host::sha256::file_hex(path).unwrap()
}

fn platform_words() -> String {
    let mut words = vec![0u8; 4096];
    let n = unsafe { emusen_platform::emusen_platform_last_error(words.as_mut_ptr(), words.len()) }.clamp(0, 4096) as usize;
    String::from_utf8_lossy(&words[..n]).into_owned()
}

/// The host's device and Shaderc lent to the platform library, or `None` (the test then passes without running).
struct Gpu(*mut sd::DeviceHandle);

impl Gpu {
    fn new() -> Option<Gpu> {
        let shaderc = std::env::var("EMUSEN_SHADERC").unwrap_or_else(|_| "/usr/lib64/libshaderc_shared.so.1".to_string());
        let path = std::ffi::CString::new(shaderc.clone()).unwrap();
        let handle = unsafe { libc::dlopen(path.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
        if handle.is_null() || unsafe { emusen_platform::serenity::emusen_serenity_shaderc_lend(handle, shaderc.as_ptr(), shaderc.len()) } != 0 {
            eprintln!("skipped: no Shaderc at {shaderc}");
            return None;
        }
        let name = std::env::var("EMUSEN_GPU_DEVICE").unwrap_or_else(|_| "6800".to_string());
        let validate = std::env::var("EMUSEN_VALIDATE").is_ok_and(|v| v == "1");
        let d = unsafe { sd::emusen_serenity_device_new(name.as_ptr(), name.len(), validate as i32) };
        if d.is_null() {
            eprintln!("skipped: no Vulkan device named {name}: {}", platform_words());
            return None;
        }
        Some(Gpu(d))
    }

    /// What the validation layer said of the host's device, each finding a line.
    fn findings(&self) -> Vec<String> {
        let mut out = vec![0u8; 1 << 20];
        let n = unsafe { sd::emusen_serenity_device_findings(self.0, out.as_mut_ptr(), out.len()) }.max(0) as usize;
        out[..n.min(out.len())].split(|b| *b == 0).filter(|s| !s.is_empty()).map(|s| String::from_utf8_lossy(s).into_owned()).collect()
    }
}

impl Drop for Gpu {
    fn drop(&mut self) {
        unsafe { sd::emusen_serenity_device_free(self.0) };
    }
}

/// A chain of one pass that draws its source unchanged, nearest-sampled at the source's size.
struct Chain(*mut sd::ChainHandle);

impl Chain {
    fn stock(gpu: &Gpu, dir: &Path) -> Chain {
        std::fs::write(dir.join("stock.slang"), STOCK).unwrap();
        let preset = dir.join("stock.slangp");
        std::fs::write(&preset, "shaders = 1\nshader0 = stock.slang\nfilter_linear0 = false\nscale_type0 = source\nscale0 = 1.0\n").unwrap();
        let p = preset.to_string_lossy().into_owned();
        let mut doc = vec![0u8; 1 << 16];
        let n = unsafe { emusen_platform::serenity::emusen_serenity_preset_load(p.as_ptr(), p.len(), doc.as_mut_ptr(), doc.len()) };
        assert!(n > 0 && (n as usize) <= doc.len(), "the preset loads: {n}");
        let h = unsafe { sd::emusen_serenity_chain_new(gpu.0, doc.as_ptr(), n as usize, std::ptr::null(), 0, 1) };
        assert!(!h.is_null(), "the chain is built: {}", platform_words());
        Chain(h)
    }

    fn from_ring(&self, ring: *const (), f: &emusen_crystal_frame_info) {
        let st = unsafe { sd::emusen_serenity_chain_advance_ring(self.0, ring as *const sd::RingHandle, f.ring_slot, f.ring_value, f.width, f.height) };
        assert_eq!(st, 0, "advance_ring: {}", platform_words());
    }

    fn upload(&self, rgba: &[u8], w: i32, h: i32) {
        assert_eq!(unsafe { sd::emusen_serenity_chain_advance(self.0, rgba.as_ptr(), rgba.len(), w, h, 1) }, 0, "advance: {}", platform_words());
    }

    fn render(&self, w: i32, h: i32) -> Vec<u8> {
        let mut out = vec![0u8; (w.max(1) * h.max(1) * 4) as usize];
        let n = unsafe { sd::emusen_serenity_chain_render(self.0, w, h, out.as_mut_ptr(), out.len()) };
        assert_eq!(n, out.len() as i64, "render: {}", platform_words());
        out
    }
}

impl Drop for Chain {
    fn drop(&mut self) {
        unsafe { sd::emusen_serenity_chain_free(self.0) };
    }
}

/// An installation whose manifest vouches for the proxy and the libretro host, the test core's entry under the player's
/// cores, approved, and a runtime made by the platform library over them, which lends rings through Serenity.
struct Rt {
    rt: *mut emusen_crystal_runtime,
    root: PathBuf,
    game: String,
}

impl Rt {
    fn new(name: &str) -> Rt {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("api-ring-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (installation, cores) = (root.join("lib"), root.join("Cores"));
        std::fs::create_dir_all(&installation).unwrap();
        let (proxy, host) = (deps().join(format!("libemusen_libretro_proxy.{SUFFIX}")), deps().join(format!("libemusen_libretro_host.{SUFFIX}")));
        std::fs::write(installation.join(MANIFEST), format!("{}  libemusen_libretro_proxy.{SUFFIX}\n{}  libemusen_libretro_host.{SUFFIX}\n", hash(&proxy), hash(&host))).unwrap();
        let entry = Entry::new(&cores, "testcore").unwrap();
        std::fs::create_dir_all(&entry.folder).unwrap();
        std::fs::copy(&proxy, entry.broker_path()).unwrap();
        std::fs::copy(&host, entry.adapter_path()).unwrap();
        std::fs::copy(deps().join(format!("libemusen_libretro_testcore.{SUFFIX}")), entry.core_path()).unwrap();
        std::fs::write(entry.info_path(), "display_name = \"EmuSen - Test Core\"\ncorename = \"TestCore\"\nlicense = \"GPLv3\"\nsystemid = \"\"\n").unwrap();
        {
            let mut t = tree_mut();
            t.set_override(Override::Data, Some(root.join("Home").to_string_lossy().into_owned()));
            t.set_override(Override::Config, Some(root.join("Config").to_string_lossy().into_owned()));
        }
        unsafe { std::env::set_var("EMUSEN_CORE_RUNNER", env!("CARGO_BIN_EXE_emusen-core-runner")) };
        let (c, p) = (format!("[{:?}]", installation.to_string_lossy()), format!("[{:?}]", cores.to_string_lossy()));
        let config = emusen_crystal_config { size: size_of::<emusen_crystal_config>() as u32, flags: 0, sqlite: std::ptr::null_mut(), cores: c.as_ptr(), cores_len: c.len(), player_cores: p.as_ptr(), player_cores_len: p.len() };
        let rt = unsafe { api::emusen_crystal_new(&config) };
        assert!(!rt.is_null(), "{}", platform_words());
        let core = hash(&entry.core_path());
        assert_eq!(unsafe { api::emusen_crystal_core_approve(rt, core.as_ptr(), core.len()) }, 0);
        let game = root.join("game.tcg");
        std::fs::write(&game, b"TCG1 a test game").unwrap();
        Rt { rt, game: game.to_string_lossy().into_owned(), root }
    }

    fn words(&self) -> String {
        let mut out = vec![0u8; 4096];
        let n = unsafe { api::emusen_crystal_last_error(self.rt, out.as_mut_ptr(), out.len()) }.clamp(0, 4096) as usize;
        String::from_utf8_lossy(&out[..n]).into_owned()
    }

    fn open(&self, device: Option<&Gpu>, ring: bool) -> u32 {
        let flags = abi::OPEN_NO_BATTERY | if ring { abi::OPEN_RING } else { 0 };
        let p = emusen_crystal_open_params {
            size: size_of::<emusen_crystal_open_params>() as u32,
            flags,
            path: self.game.as_ptr(),
            path_len: self.game.len(),
            console: std::ptr::null(),
            console_len: 0,
            engine: std::ptr::null(),
            engine_len: 0,
            audio_buffer_max_samples: 0,
            reserved: 0,
            device: device.map_or(std::ptr::null(), |g| g.0 as *const std::ffi::c_void),
        };
        let id = unsafe { api::emusen_crystal_open(self.rt, &p) };
        assert!(id > 0, "open: {}", self.words());
        id as u32
    }

    /// The console's settings stored through a session of their own, as the settings window stores them.
    fn settings(&self, pairs: &[(&str, &str)]) {
        let s = self.open(None, false);
        for (k, v) in pairs {
            assert_eq!(unsafe { api::emusen_crystal_settings_set(self.rt, s, abi::SCOPE_CONSOLE, std::ptr::null(), 0, k.as_ptr(), k.len(), v.as_ptr(), v.len()) }, 0, "{k}: {}", self.words());
        }
        self.close(s);
    }

    fn frame(&self, s: u32) -> Result<emusen_crystal_frame_info, String> {
        let mut f = emusen_crystal_frame_info { size: size_of::<emusen_crystal_frame_info>() as u32, ..Default::default() };
        match unsafe { api::emusen_crystal_frame(self.rt, s, &mut f) } {
            0 => Ok(f),
            st => Err(format!("{st}: {}", self.words())),
        }
    }

    /// A frame on a thread of its own, failed when it does not return within `seconds`: a core waiting on a slot never
    /// given back would otherwise hang the test.
    fn frame_within(&self, s: u32, seconds: u64) -> Result<emusen_crystal_frame_info, String> {
        let (tx, rx) = mpsc::channel();
        let rt = self.rt as usize;
        std::thread::spawn(move || {
            let mut f = emusen_crystal_frame_info { size: size_of::<emusen_crystal_frame_info>() as u32, ..Default::default() };
            let rt = rt as *mut emusen_crystal_runtime;
            let st = unsafe { api::emusen_crystal_frame(rt, s, &mut f) };
            let mut out = vec![0u8; 4096];
            let n = unsafe { api::emusen_crystal_last_error(rt, out.as_mut_ptr(), out.len()) }.clamp(0, 4096) as usize;
            let _ = tx.send((st, f, String::from_utf8_lossy(&out[..n]).into_owned()));
        });
        match rx.recv_timeout(Duration::from_secs(seconds)) {
            Ok((0, f, _)) => Ok(f),
            Ok((st, _, words)) => Err(format!("{st}: {words}")),
            Err(_) => panic!("a frame did not return within {seconds} s: the core waits on a slot nobody gave back"),
        }
    }

    fn picture(&self, s: u32) -> Vec<u8> {
        let n = unsafe { api::emusen_crystal_picture_copy(self.rt, s, std::ptr::null_mut(), 0) };
        assert!(n >= 0, "{}", self.words());
        let mut out = vec![0u8; n as usize];
        assert_eq!(unsafe { api::emusen_crystal_picture_copy(self.rt, s, out.as_mut_ptr(), out.len()) }, n);
        out
    }

    fn ring(&self, s: u32) -> *const () {
        unsafe { api::emusen_crystal_picture_ring(self.rt, s) }
    }

    fn close(&self, s: u32) {
        assert_eq!(unsafe { api::emusen_crystal_close(self.rt, s) }, 0, "close: {}", self.words());
    }
}

impl Drop for Rt {
    fn drop(&mut self) {
        unsafe { api::emusen_crystal_free(self.rt) };
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn assert_silent(gpu: &Gpu) {
    let heard = gpu.findings();
    assert!(heard.is_empty(), "the validation layer spoke {} times; the first: {}", heard.len(), heard[0]);
}

#[test]
fn six_frames_in_the_apis_ring_draw_as_the_frames_read_back_and_one_frame_advance_holds() {
    let _t = turn();
    let Some(gpu) = Gpu::new() else { return };
    let rt = Rt::new("six");
    rt.settings(&[("tc_hw", "vulkan")]);
    let (from_ring, uploaded) = (Chain::stock(&gpu, &rt.root), Chain::stock(&gpu, &rt.root));
    let s = rt.open(Some(&gpu), true);
    let mut handle = std::ptr::null();
    for n in 0..6u64 {
        let f = rt.frame(s).unwrap();
        assert_eq!((f.ran, f.picture_in_ring, f.picture_on_device, f.width, f.height, f.fields), (1, 1, 0, 320, 240, 0), "frame {n}'s record");
        assert_eq!((f.ring_slot, f.ring_value), ((n % 3) as u32, n + 1), "frame {n}'s slot and value");
        let now = rt.ring(s);
        assert!(!now.is_null() && (n == 0 || now == handle), "frame {n}: the session's one ring");
        handle = now;
        let picture = rt.picture(s);
        assert_eq!(picture.len(), 320 * 240 * 4);
        from_ring.from_ring(handle, &f);
        uploaded.upload(&picture, f.width, f.height);
        let (got, want) = (from_ring.render(f.width, f.height), uploaded.render(f.width, f.height));
        assert!(got == want, "frame {n}: the ring's picture drawn differs from the read-back one in {} bytes", got.iter().zip(&want).filter(|(a, b)| a != b).count());
        assert!(got == picture, "frame {n}: the stock pass changed the picture");
    }
    assert_eq!(unsafe { api::emusen_crystal_frame_advance(rt.rt, s) }, 0);
    let (one, held) = (rt.frame(s).unwrap(), rt.frame(s).unwrap());
    assert_eq!((one.ran, one.picture_in_ring, held.ran, held.paused, one.total_frames), (1, 1, 0, 1, held.total_frames));
    from_ring.from_ring(rt.ring(s), &one);
    from_ring.render(one.width, one.height);
    rt.close(s);
    let plain = rt.open(Some(&gpu), false);
    let f = rt.frame(plain).unwrap();
    assert_eq!((f.ran, f.picture_in_ring, rt.ring(plain)), (1, 0, std::ptr::null()), "a session without OPEN_RING lends none");
    rt.close(plain);
    drop((from_ring, uploaded));
    assert_silent(&gpu);
}

#[test]
fn ring_frames_given_back_by_a_one_pixel_draw_never_stall_the_core() {
    let _t = turn();
    let Some(gpu) = Gpu::new() else { return };
    let rt = Rt::new("release");
    rt.settings(&[("tc_hw", "vulkan")]);
    let release = Chain::stock(&gpu, &rt.root);
    let s = rt.open(Some(&gpu), true);
    for n in 0..12 {
        let f = rt.frame_within(s, 10).unwrap();
        assert_eq!(f.picture_in_ring, 1, "frame {n}");
        release.from_ring(rt.ring(s), &f);
        release.render(1, 1);
    }
    rt.close(s);
    drop(release);
    assert_silent(&gpu);
}

#[test]
fn a_ring_is_lent_again_at_machine_info_when_the_core_draws_past_its_maxima() {
    let _t = turn();
    let Some(gpu) = Gpu::new() else { return };
    let rt = Rt::new("grow");
    rt.settings(&[("tc_hw", "vulkan"), ("tc_geometry", "grow")]);
    let (from_ring, uploaded) = (Chain::stock(&gpu, &rt.root), Chain::stock(&gpu, &rt.root));
    let s = rt.open(Some(&gpu), true);
    let (mut seen, mut smaller, mut later): (Vec<String>, *const (), *const ()) = (Vec::new(), std::ptr::null(), std::ptr::null());
    for n in 0..10 {
        let f = rt.frame(s).unwrap();
        seen.push(format!("{}x{} ring {}", f.width, f.height, f.picture_in_ring));
        if f.picture_in_ring == 0 {
            continue;
        }
        let handle = rt.ring(s);
        if f.width == 320 {
            smaller = if smaller.is_null() { handle } else { smaller };
        } else {
            later = handle;
        }
        let picture = rt.picture(s);
        from_ring.from_ring(handle, &f);
        uploaded.upload(&picture, f.width, f.height);
        assert!(from_ring.render(f.width, f.height) == uploaded.render(f.width, f.height), "frame {n} at {}x{}", f.width, f.height);
    }
    let want: Vec<String> = ["320x240 ring 1", "320x240 ring 1", "640x480 ring 0"].iter().map(|s| s.to_string()).chain(std::iter::repeat_n("640x480 ring 1".to_string(), 7)).collect();
    assert_eq!(seen, want);
    assert!(!smaller.is_null() && !later.is_null() && smaller != later, "a second, larger ring was lent at MACHINE_INFO");
    rt.close(s);
    drop((from_ring, uploaded));
    assert_silent(&gpu);
}

#[test]
fn a_core_that_crashes_with_a_ring_lent_ends_in_words_and_the_device_runs_the_next_ringed_session() {
    let _t = turn();
    let Some(gpu) = Gpu::new() else { return };
    let rt = Rt::new("crash");
    rt.settings(&[("tc_hw", "vulkan"), ("tc_fault", "segv")]);
    let release = Chain::stock(&gpu, &rt.root);
    let s = rt.open(Some(&gpu), true);
    let mut ended = None;
    for _ in 0..6 {
        match rt.frame_within(s, 15) {
            Ok(f) if f.picture_in_ring == 1 => {
                release.from_ring(rt.ring(s), &f);
                release.render(1, 1);
            }
            Ok(_) => {}
            Err(words) => {
                ended = Some(words);
                break;
            }
        }
    }
    let ended = ended.expect("the crash ends the session");
    assert!(ended.contains("signal 11"), "{ended}");
    let _ = unsafe { api::emusen_crystal_close(rt.rt, s) };
    rt.settings(&[("tc_fault", "none")]);
    let next = rt.open(Some(&gpu), true);
    for n in 0..4 {
        let f = rt.frame_within(next, 15).unwrap();
        assert_eq!(f.picture_in_ring, 1, "frame {n} after the crash");
        release.from_ring(rt.ring(next), &f);
        release.render(1, 1);
    }
    rt.close(next);
    drop(release);
    assert_silent(&gpu);
}
