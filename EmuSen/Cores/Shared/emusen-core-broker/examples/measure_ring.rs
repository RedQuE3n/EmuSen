//! Brokered Vulkan ring measurement (EmuSen_LibretroHost.md §10f): run with `cargo run --release -p emusen-core-broker --example measure_ring`.

use std::path::PathBuf;
use std::time::Instant;

use emusen_core_host::library::Library;
use emusen_core_host::machine::{Create, Machine};
use emusen_native::core::sys::{RingImage, RingLend};

const GAME: &[u8] = b"TCG1 a test game";

fn suffix() -> &'static str {
    if cfg!(windows) { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" }
}

fn entry(root: &std::path::Path) -> (PathBuf, String) {
    let exe = std::env::current_exe().unwrap();
    let profile = exe.parent().unwrap().parent().unwrap();
    let deps = profile.join("deps");
    let entry = root.join("libretro").join("testcore");
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(&entry).unwrap();
    let s = suffix();
    std::fs::copy(deps.join(format!("libemusen_libretro_proxy.{s}")), entry.join(format!("emusen-core-broker.{s}"))).unwrap();
    std::fs::copy(deps.join(format!("libemusen_libretro_host.{s}")), entry.join(format!("emusen-libretro-host.{s}"))).unwrap();
    std::fs::copy(deps.join(format!("libemusen_libretro_testcore.{s}")), entry.join(format!("testcore_libretro.{s}"))).unwrap();
    std::fs::write(entry.join("testcore_libretro.info"), "corename = \"TestCore\"\nlicense = \"GPLv3\"\n").unwrap();
    if std::env::var_os("EMUSEN_CORE_RUNNER").is_none() {
        let runner = profile.join("emusen-core-runner");
        unsafe { std::env::set_var("EMUSEN_CORE_RUNNER", runner) };
    }
    let hash = emusen_core_host::sha256::sha256_hex(&std::fs::read(entry.join(format!("testcore_libretro.{s}"))).unwrap());
    (entry.join(format!("emusen-core-broker.{s}")), hash)
}

fn main() {
    let gpu = emusen_serenity::vulkan::Device::create(std::env::var("EMUSEN_GPU_DEVICE").ok().as_deref(), false).expect("Vulkan hardware device required");
    let root = std::env::temp_dir().join(format!("emusen-broker-measure-{}", std::process::id()));
    let (proxy, hash) = entry(&root);
    let lib = Box::leak(Box::new(Library::load_expecting(&proxy, &hash).unwrap_or_else(|e| panic!("{e:?}"))));
    let stats = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        (v.iter().sum::<f64>() / v.len() as f64, v[v.len() / 2], v[v.len() * 99 / 100])
    };
    for (name, settings) in [("software", ""), ("read-back", "tc_hw=vulkan"), ("ring", "tc_hw=vulkan")] {
        let mut m = Machine::create(lib, &Create { settings, pixel_formats: 1, ..Create::new(GAME) }).unwrap();
        let ring = gpu.create_ring(640, 480, 3).unwrap();
        let images: Vec<RingImage> = ring.images.iter().map(|i| RingImage { size: size_of::<RingImage>() as u32, memory_type: i.memory_type, handle: i.handle as u64, allocation_size: i.allocation_size, width: i.width, height: i.height, format: i.format, usage: i.usage }).collect();
        let lend = RingLend { size: size_of::<RingLend>() as u32, count: 3, images: images.as_ptr(), image_size: size_of::<RingImage>(), ready: ring.ready_fd as u64, done: ring.done_fd as u64, device_uuid: ring.device_uuid, driver_uuid: ring.driver_uuid };
        if name == "ring" {
            m.ring_lend(Some(&lend)).unwrap();
        }
        let mut copy = vec![0u8; 640 * 480 * 4];
        let (mut cost, mut delay) = (Vec::new(), Vec::new());
        for n in 0..600 {
            let press = n % 10 == 5;
            let t0 = Instant::now();
            if press || n % 10 == 6 {
                m.set_buttons(0, press as u32, 1).unwrap();
            }
            let t1 = Instant::now();
            m.advance().unwrap();
            let shown = if name == "ring" { m.frame_ring().unwrap() } else { None };
            if name != "ring" {
                m.frame_copy(&mut copy).unwrap();
            }
            let spent = t1.elapsed();
            if let Some(f) = shown {
                ring.read(f.slot as usize, f.value, f.width as u32, f.height as u32).unwrap();
            }
            if n >= 60 {
                cost.push(spent.as_secs_f64() * 1e3);
                if press {
                    delay.push(t0.elapsed().as_secs_f64() * 1e3);
                }
            }
        }
        let (c, d) = (stats(&mut cost), stats(&mut delay));
        println!("brokered {name:9}: emulation thread mean {:.3} median {:.3} p99 {:.3} ms; set_buttons to picture mean {:.3} median {:.3} p99 {:.3} ms", c.0, c.1, c.2, d.0, d.1, d.2);
    }
    let _ = std::fs::remove_dir_all(&root);
}
