// The ring (core ABI 1.18) end to end in-process: Serenity's exported images lent to the libretro host, the test core's
// Vulkan picture copied into them on its own device, read on the host's and held against its software picture
// (EmuSen_LibretroHost.md §10f). Runs on the card EMUSEN_GPU_DEVICE names; skipped where no card can make a ring.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use emusen_core_host::library::Library;
use emusen_core_host::machine::{Create, Machine};
use emusen_native::core::sys::{DeviceLend, RingImage, RingLend, event, status};
use emusen_serenity::lend::Lend;
use emusen_serenity::ring::Ring;
use emusen_serenity::vulkan::Device;

static TURN: Mutex<()> = Mutex::new(());
static LIB: OnceLock<&'static Library> = OnceLock::new();
static DEVICE_CORE: OnceLock<&'static Library> = OnceLock::new();
const STEM: &str = "testcore";
const GAME: &[u8] = b"TCG1 a test game";

fn library() -> &'static Library {
    LIB.get_or_init(|| {
        let suffix = emusen_core_entry::library_suffix();
        let deps = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("libretro-ring-{}", std::process::id()));
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

fn turn() -> MutexGuard<'static, ()> {
    TURN.lock().unwrap_or_else(|e| e.into_inner())
}

fn make(settings: &str) -> Machine {
    let c = Create { settings, pixel_formats: 1, ..Create::new(GAME) };
    Machine::create(library(), &c).unwrap_or_else(|r| panic!("{r:?}"))
}

/// The host's device, or `None` (the test then passes without running) where no card can make a ring.
fn host() -> Option<Device> {
    let wanted = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let validate = std::env::var("EMUSEN_VALIDATE").is_ok_and(|v| v == "1");
    match Device::create(wanted.as_deref(), validate) {
        Ok(d) if d.rings() => Some(d),
        Ok(_) | Err(_) => {
            eprintln!("skipped: no Vulkan device here can make a ring");
            None
        }
    }
}

/// The ABI's lend of a ring, its images kept alive beside it.
fn lend_of(ring: &Ring) -> (RingLend, Vec<RingImage>) {
    let images: Vec<RingImage> = ring.images.iter().map(|i| RingImage { size: size_of::<RingImage>() as u32, memory_type: i.memory_type, handle: i.handle as u64, allocation_size: i.allocation_size, width: i.width, height: i.height, format: i.format, usage: i.usage }).collect();
    let lend = RingLend { size: size_of::<RingLend>() as u32, count: images.len() as u32, images: images.as_ptr(), image_size: size_of::<RingImage>(), ready: ring.ready_fd as u64, done: ring.done_fd as u64, device_uuid: ring.device_uuid, driver_uuid: ring.driver_uuid };
    (lend, images)
}

/// The software pictures of the first `frames` frames, RGBA rows of their own width.
fn software(frames: usize) -> Vec<(i32, i32, Vec<u8>)> {
    let mut m = make("");
    (0..frames)
        .map(|_| {
            m.advance().unwrap();
            let f = m.frame_info().unwrap();
            let mut all = vec![0u8; m.frame_bytes().unwrap()];
            m.frame_copy(&mut all).unwrap();
            let rows = (0..f.height as usize).flat_map(|y| all[y * f.stride as usize..y * f.stride as usize + f.width as usize * 4].to_vec()).collect();
            (f.width, f.height, rows)
        })
        .collect()
}

fn ring_run(hw: &str, frames: usize, want: &[(i32, i32, Vec<u8>)], gpu: &Device) {
    let ring = gpu.create_ring(640, 480, 3).unwrap();
    let mut m = make(&format!("tc_hw={hw}"));
    let (lend, _keep) = lend_of(&ring);
    m.ring_lend(Some(&lend)).unwrap();
    for (n, (w, h, rows)) in want.iter().enumerate().take(frames) {
        m.advance().unwrap();
        let f = m.frame_ring().unwrap().unwrap_or_else(|| panic!("{hw}: frame {n} is not in the ring"));
        assert_eq!((f.width, f.height, f.value, f.slot), (*w, *h, n as u64 + 1, (n % 3) as u32), "{hw}: frame {n}'s record");
        assert_eq!((f.fields, f.aspect_num, f.aspect_den), (0, 4, 3));
        let got = ring.read(f.slot as usize, f.value, f.width as u32, f.height as u32).unwrap();
        assert!(got == *rows, "{hw}: frame {n}'s ring picture differs from its software picture");
    }
    m.ring_lend(None).unwrap();
}

#[test]
fn a_vulkan_picture_reaches_the_ring_as_the_software_picture_rgba_by_copy_and_bgra_by_blit() {
    let _t = turn();
    let Some(gpu) = host() else { return };
    let want = software(6);
    ring_run("vulkan", 6, &want, &gpu);
    ring_run("vulkan_bgra", 6, &want, &gpu);
}

// §10o: a core that negotiates v1 makes its device itself; the host's vkCreateDevice, handed through the
// vkGetInstanceProcAddr it is given, adds what the ring needs.
#[test]
fn a_core_negotiating_v1_reaches_the_ring_through_the_hosts_vk_create_device() {
    let _t = turn();
    let Some(gpu) = host() else { return };
    let want = software(6);
    ring_run("vulkan_v1", 6, &want, &gpu);
}

// §10o: a v1 core that takes vkCreateDevice from the loader instead gets no ring, says why, and is read back.
#[test]
fn a_v1_core_that_bypasses_the_hosts_vk_create_device_is_read_back_and_says_why() {
    let _t = turn();
    let Some(gpu) = host() else { return };
    let want = software(3);
    let mut m = make("tc_hw=vulkan_v1_loader");
    let ring = gpu.create_ring(640, 480, 3).unwrap();
    let (lend, _keep) = lend_of(&ring);
    assert_eq!(m.ring_lend(Some(&lend)), Err(status::NOT_SUPPORTED));
    for (n, (w, h, rows)) in want.iter().enumerate() {
        m.advance().unwrap();
        assert_eq!(m.frame_ring().unwrap(), None);
        let f = m.frame_info().unwrap();
        assert_eq!((f.width, f.height), (*w, *h));
        let mut all = vec![0u8; m.frame_bytes().unwrap()];
        m.frame_copy(&mut all).unwrap();
        let got: Vec<u8> = (0..f.height as usize).flat_map(|y| all[y * f.stride as usize..y * f.stride as usize + f.width as usize * 4].to_vec()).collect();
        assert!(got == *rows, "frame {n} read back differs");
    }
    let mut log = String::new();
    m.log_drain(&mut log).unwrap();
    assert!(log.contains("without the host's vkCreateDevice"), "{log}");
}

// §10o: a Vulkan core that declares each libretro format, the host offering each mask; every hardware frame, read
// back or taken from the ring by frame_copy, lands in the handed format with no panic (Flycast's 16-bit picture did).
#[test]
fn a_vulkan_cores_frames_land_in_whatever_format_it_declared_and_the_host_accepts() {
    let _t = turn();
    let Some(gpu) = host() else { return };
    let want = software(2);
    for format in ["xrgb8888", "rgb565", "0rgb1555"] {
        for mask in [0b001u64, 0b011, 0b101, 0b111] {
            for ringed in [false, true] {
                let c = format!("tc_hw=vulkan\ntc_format={format}");
                let mut m = Machine::create(library(), &Create { settings: &c, pixel_formats: mask, ..Create::new(GAME) }).unwrap_or_else(|r| panic!("{r:?}"));
                let ring = gpu.create_ring(640, 480, 3).unwrap();
                let (lend, _keep) = lend_of(&ring);
                if ringed {
                    m.ring_lend(Some(&lend)).unwrap();
                }
                for (n, (w, h, rows)) in want.iter().enumerate() {
                    m.advance().unwrap();
                    let f = m.frame_info().unwrap();
                    let at = format!("{format}, mask {mask:03b}, ringed {ringed}, frame {n}");
                    assert_eq!((f.width, f.height), (*w, *h), "{at}");
                    let mut all = vec![0u8; m.frame_bytes().unwrap()];
                    m.frame_copy(&mut all).unwrap();
                    let bpp = if f.format == 2 { 2 } else { 4 };
                    let got: Vec<u8> = (0..f.height as usize).flat_map(|y| all[y * f.stride as usize..y * f.stride as usize + f.width as usize * bpp].to_vec()).collect();
                    let expected: Vec<u8> = rows
                        .chunks_exact(4)
                        .flat_map(|p| match f.format {
                            2 => ((((p[0] as u16) >> 3) << 11) | (((p[1] as u16) >> 2) << 5) | ((p[2] as u16) >> 3)).to_le_bytes().to_vec(),
                            1 => vec![p[2], p[1], p[0], 255],
                            _ => p.to_vec(),
                        })
                        .collect();
                    assert!(got == expected, "{at}: the picture in format {} differs", f.format);
                }
                if ringed {
                    m.ring_lend(None).unwrap();
                }
            }
        }
    }
}

// §12 Q5 (decided 2026-11-01): a ringed frame's picture is read back only when frame_copy asks, as a screenshot does.
#[test]
fn a_ringed_frames_picture_is_read_back_when_frame_copy_asks_as_a_screenshot_does() {
    let _t = turn();
    let Some(gpu) = host() else { return };
    let want = software(6);
    for hw in ["vulkan", "vulkan_bgra"] {
        let ring = gpu.create_ring(640, 480, 3).unwrap();
        let mut m = make(&format!("tc_hw={hw}"));
        let (lend, _keep) = lend_of(&ring);
        m.ring_lend(Some(&lend)).unwrap();
        let mut serial = 0;
        for (n, (w, h, rows)) in want.iter().enumerate() {
            m.advance().unwrap();
            let r = m.frame_ring().unwrap().unwrap_or_else(|| panic!("{hw}: frame {n} is not in the ring"));
            let f = m.frame_info().unwrap();
            assert_eq!((f.width, f.height), (*w, *h), "{hw}: frame {n}'s shape before any read-back");
            assert!(f.serial > serial, "{hw}: frame {n} keeps the last frame's number");
            serial = f.serial;
            if n % 2 == 1 {
                let mut all = vec![0u8; m.frame_bytes().unwrap()];
                m.frame_copy(&mut all).unwrap();
                let got: Vec<u8> = (0..f.height as usize).flat_map(|y| all[y * f.stride as usize..y * f.stride as usize + f.width as usize * 4].to_vec()).collect();
                assert!(got == *rows, "{hw}: frame {n}'s screenshot differs from its software picture");
                assert_eq!(m.frame_info().unwrap().serial, serial, "{hw}: a read-back is not a new frame");
            }
            assert!(ring.read(r.slot as usize, r.value, r.width as u32, r.height as u32).unwrap() == *rows, "{hw}: frame {n} in the ring");
        }
        m.ring_lend(None).unwrap();
    }
}

#[test]
fn with_no_ring_the_picture_is_read_back_and_a_ring_on_another_card_is_refused() {
    let _t = turn();
    let Some(gpu) = host() else { return };
    let want = software(3);
    let mut m = make("tc_hw=vulkan");
    let ring = gpu.create_ring(640, 480, 3).unwrap();
    let (mut lend, _keep) = lend_of(&ring);
    lend.driver_uuid[0] ^= 0xFF;
    assert_eq!(m.ring_lend(Some(&lend)), Err(status::NOT_SUPPORTED));
    for (n, (w, h, rows)) in want.iter().enumerate() {
        m.advance().unwrap();
        assert_eq!(m.frame_ring().unwrap(), None);
        let f = m.frame_info().unwrap();
        assert_eq!((f.width, f.height), (*w, *h));
        let mut all = vec![0u8; m.frame_bytes().unwrap()];
        m.frame_copy(&mut all).unwrap();
        let got: Vec<u8> = (0..f.height as usize).flat_map(|y| all[y * f.stride as usize..y * f.stride as usize + f.width as usize * 4].to_vec()).collect();
        assert!(got == *rows, "frame {n} read back differs");
    }
}

#[test]
fn a_picture_larger_than_the_ring_is_held_and_copied_into_the_next_ring_that_holds_it() {
    let _t = turn();
    let Some(gpu) = host() else { return };
    let want = software(1);
    let mut m = make("tc_hw=vulkan");
    let small = gpu.create_ring(256, 200, 3).unwrap();
    let (lend, _keep) = lend_of(&small);
    m.ring_lend(Some(&lend)).unwrap();
    m.advance().unwrap();
    assert_eq!(m.frame_ring().unwrap(), None, "a picture larger than the ring is not cut into it");
    assert!(m.events().unwrap().iter().any(|e| e.kind == event::MACHINE_INFO), "the larger maxima are raised");
    let big = gpu.create_ring(640, 480, 3).unwrap();
    let (lend, _keep2) = lend_of(&big);
    m.ring_lend(Some(&lend)).unwrap();
    let f = m.frame_ring().unwrap().expect("the held frame lands in the new ring");
    assert_eq!((f.width, f.height, f.value), (want[0].0, want[0].1, 1));
    assert!(big.read(f.slot as usize, f.value, f.width as u32, f.height as u32).unwrap() == want[0].2);
    m.ring_lend(None).unwrap();
}

/// The measurement of §10f: per frame, the cost on the host's emulation thread (advance, then frame_ring or
/// frame_copy) and the delay from set_buttons to the picture on the host's side (the ring read on the host's device,
/// the copy in host memory otherwise), for software, the read-back and the ring. Run with --ignored --nocapture.
#[test]
#[ignore]
fn measure_the_ring_against_the_read_back_and_software() {
    let _t = turn();
    let Some(gpu) = host() else { return };
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
}

/// emusen-native's test core painting the libretro test core's picture on a lent device, this crate's example
/// `device_test_core`, built first (`cargo build --example device_test_core`).
fn device_core() -> &'static Library {
    DEVICE_CORE.get_or_init(|| {
        let suffix = emusen_core_entry::library_suffix();
        let deps = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
        let path = deps.parent().unwrap().join("examples").join(format!("libdevice_test_core.{suffix}"));
        Library::load(&path).unwrap_or_else(|e| panic!("{e:?}: build the example device_test_core first"))
    })
}

/// What the core is lent, as the ABI's `emusen_device_lend` holds Serenity's lend.
fn device_lend_of(lend: &Lend) -> DeviceLend {
    let l = lend.lent();
    let gipa = unsafe { std::mem::transmute::<usize, unsafe extern "C" fn()>(l.get_instance_proc_addr) };
    DeviceLend { size: size_of::<DeviceLend>() as u32, api_version: l.api, get_instance_proc_addr: Some(gipa), instance: l.instance, physical_device: l.physical, device: l.device, queue_family: l.family, queue_index: l.queue, host_queue_family: l.host_family, reserved: 0, done: l.done }
}

/// The test core's image whose payload begins with `PAINTED`.
fn painted_image() -> Vec<u8> {
    let payload = [0xD1u8, 1, 2, 3];
    let mut v = b"V1TC".to_vec();
    v.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    v.extend_from_slice(&payload);
    v
}

/// After `advance`, the device core's picture taken into the lend's ring on the emulation thread; its record.
fn take(m: &mut Machine, lend: &Lend) -> emusen_native::core::sys::DeviceFrame {
    let f = m.frame_device().unwrap().expect("the picture is on the lent device");
    assert!(lend.take(f.buffer, f.offset, f.width as u32, f.height as u32, f.stride as u32, f.ready, f.value).unwrap());
    f
}

fn press(n: usize) -> Option<u32> {
    match n % 7 {
        3 => Some(1),
        5 => Some(0),
        _ => None,
    }
}

#[test]
fn the_v1_test_cores_device_picture_is_the_libretro_test_cores_software_picture() {
    let _t = turn();
    let Some(gpu) = host() else { return };
    let lend = Lend::new(&gpu).unwrap();
    let lent = device_lend_of(&lend);
    let image = painted_image();
    let mut v1 = Machine::create(device_core(), &Create { device: Some(&lent), ..Create::new(&image) }).unwrap_or_else(|r| panic!("{r:?}"));
    let mut lr = make("");
    for n in 0..130usize {
        if let Some(mask) = press(n) {
            v1.set_buttons(0, mask, 1).unwrap();
            lr.set_buttons(0, mask, 1).unwrap();
        }
        v1.advance().unwrap();
        lr.advance().unwrap();
        let f = lr.frame_info().unwrap();
        let mut all = vec![0u8; lr.frame_bytes().unwrap()];
        lr.frame_copy(&mut all).unwrap();
        let want: Vec<u8> = (0..f.height as usize).flat_map(|y| all[y * f.stride as usize..y * f.stride as usize + f.width as usize * 4].to_vec()).collect();
        let d = take(&mut v1, &lend);
        assert_eq!((d.width, d.height, d.value), (f.width, f.height, n as u64 + 1), "frame {n}'s record");
        let taken = lend.claim(0).expect("the copy is claimed");
        let got = lend.read(&taken).unwrap();
        lend.release();
        assert!(got == want, "frame {n}: the device picture differs from the libretro test core's");
        let mut copy = vec![0u8; v1.frame_bytes().unwrap()];
        v1.frame_copy(&mut copy).unwrap();
        assert!(copy == want, "frame {n}: frame_copy differs from the device picture");
    }
    lend.settle().unwrap();
}

/// The acceptance measurement of stage 3 (§10f): the ring against DEVICE_PICTURE on one picture, the libretro test
/// core's 320 by 240, for the emulation thread's cost (advance and the sound drained, then frame_ring, or frame_device
/// and the lend's take)
/// and the delay from set_buttons to the picture in host memory (Ring::read, or the lend's read). Run with --ignored
/// --nocapture.
#[test]
#[ignore]
fn measure_the_ring_against_device_picture() {
    let _t = turn();
    let Some(gpu) = host() else { return };
    let frames = 600usize;
    let stats = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        (v.iter().sum::<f64>() / v.len() as f64, v[v.len() / 2], v[v.len() * 99 / 100])
    };
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
