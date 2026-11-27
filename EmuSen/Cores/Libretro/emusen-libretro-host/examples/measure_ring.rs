//! Libretro ring vs read-back / software / device-picture measurement (EmuSen_LibretroHost.md §10f): build `device_test_core` then run `cargo run --release -p emusen-libretro-host --example measure_ring`.

use std::sync::OnceLock;

use emusen_core_host::library::Library;
use emusen_core_host::machine::{Create, Machine};
use emusen_native::core::sys::{DeviceFrame, DeviceLend, RingImage, RingLend};
use emusen_serenity::lend::Lend;
use emusen_serenity::ring::Ring;
use emusen_serenity::vulkan::Device;

static LIB: OnceLock<&'static Library> = OnceLock::new();
static DEVICE_CORE: OnceLock<&'static Library> = OnceLock::new();
const STEM: &str = "testcore";
const GAME: &[u8] = b"TCG1 a test game";

fn library() -> &'static Library {
    LIB.get_or_init(|| {
        let suffix = emusen_core_entry::library_suffix();
        let deps = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().join("deps");
        let root = std::env::temp_dir().join(format!("emusen-libretro-measure-{}", std::process::id()));
        let entry = root.join("libretro").join(STEM);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&entry).unwrap();
        let host = entry.join(format!("emusen-libretro-host.{suffix}"));
        std::fs::copy(deps.join(format!("libemusen_libretro_host.{suffix}")), &host).unwrap();
        std::fs::copy(deps.join(format!("libemusen_libretro_testcore.{suffix}")), entry.join(format!("{STEM}_libretro.{suffix}"))).unwrap();
        std::fs::write(entry.join(format!("{STEM}_libretro.info")), "display_name = \"EmuSen - Test Core\"\ncorename = \"TestCore\"\nlicense = \"GPLv3\"\nsystemid = \"\"\ndatabase = \"\"\n").unwrap();
        Library::load(&host).unwrap_or_else(|e| panic!("{e:?}"))
    })
}

fn device_core() -> &'static Library {
    DEVICE_CORE.get_or_init(|| {
        let suffix = emusen_core_entry::library_suffix();
        let path = std::env::current_exe().unwrap().parent().unwrap().join(format!("libdevice_test_core.{suffix}"));
        Library::load(&path).unwrap_or_else(|e| panic!("{e:?}: build the example device_test_core first"))
    })
}

fn make(settings: &str) -> Machine {
    let c = Create { settings, pixel_formats: 1, ..Create::new(GAME) };
    Machine::create(library(), &c).unwrap_or_else(|r| panic!("{r:?}"))
}

fn lend_of(ring: &Ring) -> (RingLend, Vec<RingImage>) {
    let images: Vec<RingImage> = ring.images.iter().map(|i| RingImage { size: size_of::<RingImage>() as u32, memory_type: i.memory_type, handle: i.handle as u64, allocation_size: i.allocation_size, width: i.width, height: i.height, format: i.format, usage: i.usage }).collect();
    let lend = RingLend { size: size_of::<RingLend>() as u32, count: images.len() as u32, images: images.as_ptr(), image_size: size_of::<RingImage>(), ready: ring.ready_fd as u64, done: ring.done_fd as u64, device_uuid: ring.device_uuid, driver_uuid: ring.driver_uuid };
    (lend, images)
}

fn device_lend_of(lend: &Lend) -> DeviceLend {
    let l = lend.lent();
    let gipa = unsafe { std::mem::transmute::<usize, unsafe extern "C" fn()>(l.get_instance_proc_addr) };
    DeviceLend { size: size_of::<DeviceLend>() as u32, api_version: l.api, get_instance_proc_addr: Some(gipa), instance: l.instance, physical_device: l.physical, device: l.device, queue_family: l.family, queue_index: l.queue, host_queue_family: l.host_family, reserved: 0, done: l.done }
}

fn painted_image() -> Vec<u8> {
    let payload = [0xD1u8, 1, 2, 3];
    let mut v = b"V1TC".to_vec();
    v.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    v.extend_from_slice(&payload);
    v
}

fn take(m: &mut Machine, lend: &Lend) -> DeviceFrame {
    let f = m.frame_device().unwrap().expect("the picture is on the lent device");
    assert!(lend.take(f.buffer, f.offset, f.width as u32, f.height as u32, f.stride as u32, f.ready, f.value).unwrap());
    f
}

fn main() {
    let wanted = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let validate = std::env::var("EMUSEN_VALIDATE").is_ok_and(|v| v == "1");
    let gpu = Device::create(wanted.as_deref(), validate).expect("Vulkan hardware device required");
    assert!(gpu.rings(), "Vulkan hardware device with ring support required");
    let frames = 600usize;
    let stats = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        (v.iter().sum::<f64>() / v.len() as f64, v[v.len() / 2], v[v.len() * 99 / 100])
    };
    for (name, settings) in [("software", ""), ("read-back", "tc_hw=vulkan"), ("ring", "tc_hw=vulkan")] {
        let mut m = make(settings);
        let ring = gpu.create_ring(640, 480, 3).unwrap();
        let (lend, _keep) = lend_of(&ring);
        if name == "ring" {
            m.ring_lend(Some(&lend)).unwrap();
        }
        let mut copy = vec![0u8; 640 * 480 * 4];
        let (mut cost, mut delay) = (Vec::new(), Vec::new());
        for n in 0..frames {
            let press = n % 10 == 5;
            let t0 = std::time::Instant::now();
            if press || n % 10 == 6 {
                m.set_buttons(0, if press { 1 } else { 0 }, 1).unwrap();
            }
            let t1 = std::time::Instant::now();
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
        println!("{name:9}: emulation thread mean {:.3} median {:.3} p99 {:.3} ms; set_buttons to picture mean {:.3} median {:.3} p99 {:.3} ms", c.0, c.1, c.2, d.0, d.1, d.2);
        if name == "ring" {
            m.ring_lend(None).unwrap();
        }
    }
    for name in ["ring", "device picture"] {
        let ring = gpu.create_ring(640, 480, 3).unwrap();
        let (rlend, _keep) = lend_of(&ring);
        let lend = Lend::new(&gpu).unwrap();
        let lent = device_lend_of(&lend);
        let image = painted_image();
        let mut m = if name == "ring" {
            let mut m = make("tc_hw=vulkan");
            m.ring_lend(Some(&rlend)).unwrap();
            m
        } else {
            Machine::create(device_core(), &Create { device: Some(&lent), ..Create::new(&image) }).unwrap()
        };
        let (mut cost, mut delay, mut sound) = (Vec::new(), Vec::new(), vec![0i16; 8192]);
        for n in 0..frames {
            let pressed = n % 10 == 5;
            let t0 = std::time::Instant::now();
            if pressed || n % 10 == 6 {
                m.set_buttons(0, if pressed { 1 } else { 0 }, 1).unwrap();
            }
            let t1 = std::time::Instant::now();
            m.advance().unwrap();
            m.audio_drain(&mut sound).unwrap();
            let shown = if name == "ring" { m.frame_ring().unwrap() } else { None };
            if name != "ring" {
                take(&mut m, &lend);
            }
            let spent = t1.elapsed();
            if let Some(f) = shown {
                ring.read(f.slot as usize, f.value, f.width as u32, f.height as u32).unwrap();
            } else {
                let taken = lend.claim(0).unwrap();
                lend.read(&taken).unwrap();
                lend.release();
            }
            if n >= 60 {
                cost.push(spent.as_secs_f64() * 1e3);
                if pressed {
                    delay.push(t0.elapsed().as_secs_f64() * 1e3);
                }
            }
        }
        let (c, d) = (stats(&mut cost), stats(&mut delay));
        println!("{name:14}: emulation thread mean {:.3} median {:.3} p99 {:.3} ms; set_buttons to picture mean {:.3} median {:.3} p99 {:.3} ms", c.0, c.1, c.2, d.0, d.1, d.2);
        if name == "ring" {
            m.ring_lend(None).unwrap();
        }
        lend.settle().unwrap();
    }
}
