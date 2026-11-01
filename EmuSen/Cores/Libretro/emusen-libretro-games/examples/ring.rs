// `ring <home> <console> <game> <from> <to> [key=value]...`: a game opened through the platform library's frontend API
// with OPEN_RING on Serenity's device, each frame drawn from the ring by a one-pass chain (a frame read back uploaded to it
// instead), frames from..to timed; a pad
// script (EMUSEN_RING_PADS=FILE, lines `<frame> <control> [<frames held>]`), PNGs (EMUSEN_RING_PNG=DIR:F,F,...) and a
// state taken and loaded at the end, the log's lines holding EMUSEN_RING_LOG's text (EmuSen_LibretroHost.md §10m).

use std::path::Path;
use std::time::Instant;

use emusen_crystal::abi::{self, emusen_crystal_config, emusen_crystal_frame_info, emusen_crystal_open_params, emusen_crystal_runtime};
use emusen_libretro_games::{Home, pads, timing};
use emusen_platform::crystal as api;
use emusen_platform::serenity_device as sd;

const STOCK: &str = "#version 450\nlayout(push_constant) uniform Push { vec4 SourceSize; vec4 OriginalSize; vec4 OutputSize; uint FrameCount; } params;\nlayout(std140, set = 0, binding = 0) uniform UBO { mat4 MVP; } global;\n#pragma stage vertex\nlayout(location = 0) in vec4 Position;\nlayout(location = 1) in vec2 TexCoord;\nlayout(location = 0) out vec2 vTexCoord;\nvoid main() { gl_Position = global.MVP * Position; vTexCoord = TexCoord; }\n#pragma stage fragment\nlayout(location = 0) in vec2 vTexCoord;\nlayout(location = 0) out vec4 FragColor;\nlayout(set = 0, binding = 2) uniform sampler2D Source;\nvoid main() { FragColor = texture(Source, vTexCoord); }\n";

fn fail(words: String) -> ! {
    println!("error: {words}");
    std::process::exit(1);
}

fn platform_words() -> String {
    let mut words = vec![0u8; 4096];
    let n = unsafe { emusen_platform::emusen_platform_last_error(words.as_mut_ptr(), words.len()) }.clamp(0, 4096) as usize;
    String::from_utf8_lossy(&words[..n]).into_owned()
}

struct Api(*mut emusen_crystal_runtime);

impl Api {
    fn words(&self) -> String {
        let mut out = vec![0u8; 4096];
        let n = unsafe { api::emusen_crystal_last_error(self.0, out.as_mut_ptr(), out.len()) }.clamp(0, 4096) as usize;
        String::from_utf8_lossy(&out[..n]).into_owned()
    }

    fn check(&self, st: i32, what: &str) {
        if st != 0 {
            fail(format!("{what}: {st}: {}", self.words()));
        }
    }
}

/// The device the chain draws on and Shaderc lent to the platform library.
fn device() -> *mut sd::DeviceHandle {
    let shaderc = std::env::var("EMUSEN_SHADERC").unwrap_or_else(|_| "/usr/lib64/libshaderc_shared.so.1".to_string());
    let path = std::ffi::CString::new(shaderc.clone()).unwrap();
    let handle = unsafe { libc::dlopen(path.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
    if handle.is_null() || unsafe { emusen_platform::serenity::emusen_serenity_shaderc_lend(handle, shaderc.as_ptr(), shaderc.len()) } != 0 {
        fail(format!("no Shaderc at {shaderc}"));
    }
    let name = std::env::var("EMUSEN_GPU_DEVICE").unwrap_or_else(|_| "6800".to_string());
    let d = unsafe { sd::emusen_serenity_device_new(name.as_ptr(), name.len(), 0) };
    if d.is_null() {
        fail(format!("no Vulkan device named {name}: {}", platform_words()));
    }
    d
}

fn chain(d: *mut sd::DeviceHandle, dir: &Path) -> *mut sd::ChainHandle {
    std::fs::write(dir.join("stock.slang"), STOCK).unwrap();
    let preset = dir.join("stock.slangp");
    std::fs::write(&preset, "shaders = 1\nshader0 = stock.slang\nfilter_linear0 = false\nscale_type0 = source\nscale0 = 1.0\n").unwrap();
    let p = preset.to_string_lossy().into_owned();
    let mut doc = vec![0u8; 1 << 16];
    let n = unsafe { emusen_platform::serenity::emusen_serenity_preset_load(p.as_ptr(), p.len(), doc.as_mut_ptr(), doc.len()) };
    if n <= 0 || n as usize > doc.len() {
        fail(format!("the preset did not load: {}", platform_words()));
    }
    let h = unsafe { sd::emusen_serenity_chain_new(d, doc.as_ptr(), n as usize, std::ptr::null(), 0, 1) };
    if h.is_null() {
        fail(format!("the chain was not built: {}", platform_words()));
    }
    h
}

fn render(c: *mut sd::ChainHandle, w: i32, h: i32) -> Vec<u8> {
    let mut out = vec![0u8; (w.max(1) * h.max(1) * 4) as usize];
    let n = unsafe { sd::emusen_serenity_chain_render(c, w.max(1), h.max(1), out.as_mut_ptr(), out.len()) };
    if n != out.len() as i64 {
        fail(format!("render: {}", platform_words()));
    }
    out
}

fn digest(p: &[u8]) -> u64 {
    p.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 5 {
        fail("usage: ring <home> <console> <game> <from> <to> [key=value]...".into());
    }
    let home = Home::at(Path::new(&args[0]));
    let (console, game) = (&args[1], &args[2]);
    let n = |s: &str| s.parse::<u64>().unwrap_or_else(|_| fail(format!("{s} is not a frame")));
    let (from, to) = (n(&args[3]), n(&args[4]));
    home.point_tree();
    unsafe { std::env::set_var("EMUSEN_CORE_RUNNER", home.installation.join("emusen-core-runner")) };
    let script = std::env::var("EMUSEN_RING_PADS").ok().map(|f| pads(&std::fs::read_to_string(&f).unwrap_or_else(|e| fail(format!("{f}: {e}")))).unwrap_or_else(|e| fail(e))).unwrap_or_default();
    let shots: (String, Vec<u64>) = std::env::var("EMUSEN_RING_PNG").ok().and_then(|v| v.split_once(':').map(|(d, f)| (d.to_string(), f.split(',').map(n).collect()))).unwrap_or_default();

    let (c, p) = (format!("[{:?}]", home.installation.to_string_lossy()), format!("[{:?}]", home.cores.to_string_lossy()));
    let config = emusen_crystal_config { size: size_of::<emusen_crystal_config>() as u32, flags: 0, sqlite: std::ptr::null_mut(), cores: c.as_ptr(), cores_len: c.len(), player_cores: p.as_ptr(), player_cores_len: p.len() };
    let rt = Api(unsafe { api::emusen_crystal_new(&config) });
    if rt.0.is_null() {
        fail(platform_words());
    }
    for kv in &args[5..] {
        let (k, v) = kv.split_once('=').unwrap_or_else(|| fail(format!("{kv} is not key=value")));
        rt.check(unsafe { api::emusen_crystal_settings_set(rt.0, 0, abi::SCOPE_CONSOLE, console.as_ptr(), console.len(), k.as_ptr(), k.len(), v.as_ptr(), v.len()) }, k);
    }
    let gpu = device();
    let draw = chain(gpu, &home.root);
    let params = emusen_crystal_open_params {
        size: size_of::<emusen_crystal_open_params>() as u32,
        flags: abi::OPEN_NO_BATTERY | abi::OPEN_RING,
        path: game.as_ptr(),
        path_len: game.len(),
        console: std::ptr::null(),
        console_len: 0,
        engine: std::ptr::null(),
        engine_len: 0,
        audio_buffer_max_samples: 0,
        reserved: 0,
        device: gpu as *const std::ffi::c_void,
    };
    let log = unsafe { api::emusen_crystal_subscribe(rt.0, 0, abi::EVENT_LOG) };
    let opened = Instant::now();
    let id = unsafe { api::emusen_crystal_open(rt.0, &params) };
    if id <= 0 {
        fail(format!("open: {}", rt.words()));
    }
    let s = id as u32;
    println!("opened in {:.2} s", opened.elapsed().as_secs_f64());

    let mut f = emusen_crystal_frame_info::default();
    let (mut times, mut in_ring, mut next, mut drawn, mut size) = (Vec::new(), 0u64, 0usize, 0u64, (0, 0));
    let (mut pixels, mut uploaded) = (Vec::<u8>::new(), 0u64);
    let start = Instant::now();
    for frame in 0..to {
        while next < script.len() && script[next].0 == frame {
            let (_, control, pressed) = script[next];
            rt.check(unsafe { api::emusen_crystal_input_button(rt.0, s, 0, control, pressed as u32) }, "button");
            next += 1;
        }
        let t = Instant::now();
        f = emusen_crystal_frame_info { size: size_of::<emusen_crystal_frame_info>() as u32, ..Default::default() };
        rt.check(unsafe { api::emusen_crystal_frame(rt.0, s, &mut f) }, &format!("frame {frame}"));
        if f.picture_in_ring != 0 {
            let ring = unsafe { api::emusen_crystal_picture_ring(rt.0, s) };
            let st = unsafe { sd::emusen_serenity_chain_advance_ring(draw, ring as *const sd::RingHandle, f.ring_slot, f.ring_value, f.width, f.height) };
            if st != 0 {
                fail(format!("advance_ring at frame {frame}: {}", platform_words()));
            }
            render(draw, 1, 1);
            drawn += 1;
            size = (f.width, f.height);
            if frame >= from {
                in_ring += 1;
            }
        } else if f.rendered != 0 {
            let n = unsafe { api::emusen_crystal_picture_copy(rt.0, s, std::ptr::null_mut(), 0) }.max(0) as usize;
            pixels.resize(n, 0);
            unsafe { api::emusen_crystal_picture_copy(rt.0, s, pixels.as_mut_ptr(), pixels.len()) };
            if n == (f.width * f.height * 4) as usize && n > 0 {
                if unsafe { sd::emusen_serenity_chain_advance(draw, pixels.as_ptr(), pixels.len(), f.width, f.height, 1) } != 0 {
                    fail(format!("advance at frame {frame}: {}", platform_words()));
                }
                render(draw, 1, 1);
                uploaded += 1;
                size = (f.width, f.height);
            }
        }
        if frame >= from {
            times.push(t.elapsed().as_secs_f64());
        }
        if shots.1.contains(&frame) && drawn + uploaded > 0 {
            let png = format!("{}/{frame}.png", shots.0);
            let rgba = render(draw, size.0, size.1);
            match emusen_crystal::png::write(&png, &rgba, size.0 as u32, size.1 as u32) {
                Ok(()) => println!("png: {png}"),
                Err(e) => println!("png: {e}"),
            }
        }
    }
    println!("run: {to} frames in {:.2} s, {drawn} pictures in the ring, {uploaded} read back and uploaded", start.elapsed().as_secs_f64());
    let t = timing(&times).unwrap_or_else(|| fail("no frames timed".into()));
    println!("timed: frames {from} to {to}, {} frames, {in_ring} in the ring, {:.2} fps, median {:.3} ms, worst {:.3} ms", t.frames, t.fps, t.median_ms, t.worst_ms);
    println!("picture: {} x {}, fields {}, aspect {:.4}, rate {:.3} Hz; the last frame rendered {}, in the ring {}, on the device {}", f.width, f.height, f.fields, f.display_aspect, f.frame_rate_hz, f.rendered, f.picture_in_ring, f.picture_on_device);

    let step = |rt: &Api| {
        let mut f = emusen_crystal_frame_info { size: size_of::<emusen_crystal_frame_info>() as u32, ..Default::default() };
        rt.check(unsafe { api::emusen_crystal_frame(rt.0, s, &mut f) }, "frame");
        if f.picture_in_ring != 0 {
            let ring = unsafe { api::emusen_crystal_picture_ring(rt.0, s) };
            if unsafe { sd::emusen_serenity_chain_advance_ring(draw, ring as *const sd::RingHandle, f.ring_slot, f.ring_value, f.width, f.height) } != 0 {
                return 0;
            }
            return digest(&render(draw, f.width, f.height));
        }
        let n = unsafe { api::emusen_crystal_picture_copy(rt.0, s, std::ptr::null_mut(), 0) }.max(0) as usize;
        let mut rgba = vec![0u8; n];
        unsafe { api::emusen_crystal_picture_copy(rt.0, s, rgba.as_mut_ptr(), rgba.len()) };
        digest(&rgba)
    };
    match unsafe { api::emusen_crystal_state_slot_save(rt.0, s, 1) } {
        0 => {
            let saved = step(&rt);
            for _ in 0..60 {
                step(&rt);
            }
            let moved = step(&rt);
            rt.check(unsafe { api::emusen_crystal_state_slot_load(rt.0, s, 1) }, "state load");
            let loaded = step(&rt);
            let later: Vec<String> = (0..4).map(|_| format!("{:016x}", step(&rt))).collect();
            if let Ok(png) = std::env::var("EMUSEN_RING_PNG_LOADED") {
                let rgba = render(draw, size.0, size.1);
                let _ = emusen_crystal::png::write(&png, &rgba, size.0 as u32, size.1 as u32);
            }
            for _ in 0..60 {
                step(&rt);
            }
            rt.check(unsafe { api::emusen_crystal_state_slot_load(rt.0, s, 1) }, "state load");
            let again = step(&rt);
            println!("state: slot 1; after save {saved:016x}, 60 frames on {moved:016x}, after load {loaded:016x} ({}), then {}; after a second load {again:016x} ({})", if loaded == saved { "the same picture" } else { "a different picture" }, later.join(" "), if again == loaded { "the same as the first load" } else { "not the first load's" });
        }
        st => println!("state: save refused: {st}: {}", rt.words()),
    }
    rt.check(unsafe { api::emusen_crystal_close(rt.0, s) }, "close");
    if let Ok(pattern) = std::env::var("EMUSEN_RING_LOG") {
        let n = unsafe { api::emusen_crystal_subscription_drain(log, std::ptr::null_mut(), 0) }.max(0) as usize;
        let mut said = vec![0u8; n];
        unsafe { api::emusen_crystal_subscription_drain(log, said.as_mut_ptr(), said.len()) };
        for l in String::from_utf8_lossy(&said).split("\",\"").filter(|l| l.contains(pattern.as_str())) {
            println!("log: {l}");
        }
    }
    unsafe { api::emusen_crystal_unsubscribe(log) };
    unsafe {
        sd::emusen_serenity_chain_free(draw);
        api::emusen_crystal_free(rt.0);
        sd::emusen_serenity_device_free(gpu);
    }
    println!("closed");
}
