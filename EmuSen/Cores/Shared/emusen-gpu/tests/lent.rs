//! A device lent by another: the test is its own lender, as a frontend's presenter is, with a queue for itself and one
//! for the borrower, and reads the pictures handed on the device as the host does. See Hardware/Shared/EmuSen_Gpu.md §5
//! and EmuSen_DevicePicture.md §5 and §6.

use std::time::{Duration, Instant};

use ash::vk::{self, Handle};
use emusen_gpu::{GpuDevice, Lent, PictureOut, Where};

const TRIPLE: &[u8] = include_bytes!("shaders/triple.spv");

/// The lender: an instance and a device of Vulkan 1.2 with timeline semaphores, its own queue of the graphics family and
/// one of a compute family (another family's, or a second of its own) for the borrower.
struct Lender {
    _entry: ash::Entry,
    instance: ash::Instance,
    device: ash::Device,
    lent: Lent,
    host: Lent,
}

impl Drop for Lender {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

fn lender() -> Option<Lender> {
    let entry = unsafe { ash::Entry::load() }.ok()?;
    let application = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_2);
    let instance = unsafe { entry.create_instance(&vk::InstanceCreateInfo::default().application_info(&application), None) }.ok()?;
    let wanted = std::env::var(emusen_gpu::DEVICE_VARIABLE).unwrap_or_default().to_lowercase();
    let mut chosen = None;
    for physical in unsafe { instance.enumerate_physical_devices() }.unwrap_or_default() {
        let properties = unsafe { instance.get_physical_device_properties(physical) };
        let name = properties.device_name_as_c_str().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
        if vk::api_version_minor(properties.api_version) < 2 || (!wanted.is_empty() && !name.contains(&wanted)) {
            continue;
        }
        let queues = unsafe { instance.get_physical_device_queue_family_properties(physical) };
        let Some(graphics) = queues.iter().position(|q| q.queue_flags.contains(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)) else { continue };
        let compute = queues.iter().enumerate().position(|(i, q)| i != graphics && q.queue_flags.contains(vk::QueueFlags::COMPUTE));
        let borrower = match compute {
            Some(family) => (family as u32, 0),
            None if queues[graphics].queue_count > 1 => (graphics as u32, 1),
            None => continue,
        };
        chosen = Some((physical, properties.api_version, graphics as u32, borrower));
        break;
    }
    let Some((physical, api, graphics, (family, index))) = chosen else {
        eprintln!("no device can lend a queue: not run");
        unsafe { instance.destroy_instance(None) };
        return None;
    };
    let priorities = [1.0f32, 1.0];
    let mut infos = vec![vk::DeviceQueueCreateInfo::default().queue_family_index(graphics).queue_priorities(&priorities[..if family == graphics { 2 } else { 1 }])];
    if family != graphics {
        infos.push(vk::DeviceQueueCreateInfo::default().queue_family_index(family).queue_priorities(&priorities[..1]));
    }
    let mut features = vk::PhysicalDeviceVulkan12Features::default().timeline_semaphore(true);
    let device = unsafe { instance.create_device(physical, &vk::DeviceCreateInfo::default().queue_create_infos(&infos).push_next(&mut features), None) }.ok()?;
    let gipa = entry.static_fn().get_instance_proc_addr;
    let lend = |family, queue| Lent {
        get_instance_proc_addr: gipa,
        instance: instance.handle().as_raw(),
        physical: physical.as_raw(),
        device: device.handle().as_raw(),
        api,
        family,
        queue,
        host_family: graphics,
    };
    let (lent, host) = (lend(family, index), lend(graphics, 0));
    Some(Lender { _entry: entry, instance, device, lent, host })
}

#[test]
fn a_lent_device_runs_a_program_and_leaves_the_lender_s_device_alive() {
    let Some(lender) = lender() else { return };
    {
        let gpu = unsafe { GpuDevice::lent(&lender.lent) }.unwrap_or_else(|e| panic!("{e}"));
        assert!(gpu.is_lent());
        let data = gpu.create_buffer(1000 * 4, Where::Host).unwrap();
        data.slice_mut::<u32>().iter_mut().enumerate().for_each(|(i, w)| *w = i as u32);
        let program = gpu.create_program(TRIPLE, 1, 4).unwrap();
        gpu.submit(|c| c.dispatch(&program, &[&data], &7u32, 1000u32.div_ceil(64), 1, 1));
        assert!(data.slice::<u32>().iter().enumerate().all(|(i, &w)| w == (i as u32) * 3 + 7));
        gpu.destroy_program(program);
        gpu.destroy_buffer(data);
    }
    // The borrower's drop destroyed only its own: the lender's device still makes a fence.
    let fence = unsafe { lender.device.create_fence(&vk::FenceCreateInfo::default(), None) }.expect("the lender's device lives");
    unsafe { lender.device.destroy_fence(fence, None) };
}

#[test]
fn a_lent_device_is_refused_before_vulkan_1_2_and_on_a_queue_it_lacks() {
    let Some(lender) = lender() else { return };
    let old = Lent { api: vk::make_api_version(0, 1, 1, 0), ..lender.lent };
    assert!(unsafe { GpuDevice::lent(&old) }.is_err_and(|e| e.contains("1.2")));
    let missing = Lent { queue: 99, ..lender.lent };
    assert!(unsafe { GpuDevice::lent(&missing) }.is_err_and(|e| e.contains("queue")));
}

/// The host's side: wait on the device for the picture, copy its slot out, and let go of it by `done`.
fn host_reads(host: &GpuDevice, out: &PictureOut, done: u64, slot: usize, value: u64, into: &emusen_gpu::GpuBuffer) {
    let ready = out.ready().expect("on the device").raw();
    host.submit(|c| {
        c.wait_for(ready, value);
        c.copy(out.slot(slot), into, 0, 0, 0);
        c.signal(done, value);
    });
}

#[test]
fn a_picture_is_handed_on_the_device_and_its_slot_is_not_written_again_until_the_host_lets_go() {
    let Some(lender) = lender() else { return };
    let core = unsafe { GpuDevice::lent(&lender.lent) }.unwrap();
    let host = unsafe { GpuDevice::lent(&lender.host) }.unwrap();
    let done = host.create_timeline().unwrap();
    const WORDS: u64 = 4096;
    let out = PictureOut::new(&core, WORDS * 4, Some(done.raw())).unwrap();
    assert!(out.on_device());
    let picture = core.create_buffer(WORDS * 4, Where::Device).unwrap();
    let readback = host.create_buffer(WORDS * 4, Where::Host).unwrap();

    // Each picture is the frame's number in every word, handed into the slots in turn and read by the host.
    for frame in 1..=6u32 {
        let mut handed = Default::default();
        core.submit_pending(|c| {
            c.fill(&picture, frame);
            handed = out.hand(c, &picture, 0, (frame % 2) as usize);
        });
        assert_eq!(handed.value, frame as u64, "a picture's value counts the pictures");
        out.tell(handed.value);
        host_reads(&host, &out, done.raw(), handed.slot, handed.value, &readback);
        assert!(readback.slice::<u32>().iter().all(|&w| w == frame), "frame {frame} read on the device as it was drawn");
    }
    core.wait_for_pending();

    // Held back: two pictures handed and not let go of, the third waits on the device for the first's slot.
    let mut values = Vec::new();
    for frame in 7..=8u32 {
        core.submit_pending(|c| {
            c.fill(&picture, frame);
            values.push(out.hand(c, &picture, 0, (frame % 2) as usize));
        });
        out.tell(values.last().unwrap().value);
    }
    core.wait_for_pending();
    assert_eq!(core.timeline_value(out.ready().unwrap()), 8);
    core.submit_pending(|c| {
        c.fill(&picture, 9);
        values.push(out.hand(c, &picture, 0, 1));
    });
    out.tell(9);
    let started = Instant::now();
    while started.elapsed() < Duration::from_millis(200) {
        assert_eq!(core.timeline_value(out.ready().unwrap()), 8, "the third picture waits for the host to let go of the slot it writes");
        std::thread::sleep(Duration::from_millis(10));
    }
    // The slot still holds picture 7 for the host, unwritten by picture 9.
    host_reads(&host, &out, done.raw(), values[0].slot, values[0].value, &readback);
    assert!(readback.slice::<u32>().iter().all(|&w| w == 7));
    host_reads(&host, &out, done.raw(), values[1].slot, values[1].value, &readback);
    core.wait_for_pending();
    assert_eq!(core.timeline_value(out.ready().unwrap()), 9, "let go of, the slot takes the third picture");
    host_reads(&host, &out, done.raw(), values[2].slot, values[2].value, &readback);
    assert!(readback.slice::<u32>().iter().all(|&w| w == 9));

    out.destroy(&core);
    core.destroy_buffer(picture);
    host.destroy_buffer(readback);
    host.destroy_timeline(done);
}

#[test]
fn a_picture_the_lender_was_never_told_of_is_not_waited_for() {
    let Some(lender) = lender() else { return };
    let core = unsafe { GpuDevice::lent(&lender.lent) }.unwrap();
    let host = unsafe { GpuDevice::lent(&lender.host) }.unwrap();
    let done = host.create_timeline().unwrap();
    let out = PictureOut::new(&core, 64, Some(done.raw())).unwrap();
    let picture = core.create_buffer(64, Where::Device).unwrap();
    // Six pictures handed and none told of, nor let go of: a host reading frame_copy alone never stalls the core.
    for frame in 1..=6u64 {
        core.submit(|c| {
            c.fill(&picture, frame as u32);
            out.hand(c, &picture, 0, (frame % 2) as usize);
        });
        assert_eq!(core.timeline_value(out.ready().unwrap()), frame);
    }
    out.destroy(&core);
    core.destroy_buffer(picture);
    host.destroy_timeline(done);
}

#[test]
fn on_a_device_of_its_own_the_picture_is_handed_in_host_memory_as_before() {
    let Some(name) = GpuDevice::device_names().into_iter().next() else { return };
    let gpu = GpuDevice::try_create(Some(&name)).unwrap();
    let out = PictureOut::new(&gpu, 64, Some(1)).unwrap();
    assert!(!out.on_device(), "a device made here hands nothing on the device, whatever it is given");
    let picture = gpu.create_buffer(64, Where::Device).unwrap();
    let mut handed = Default::default();
    gpu.submit(|c| {
        c.fill(&picture, 0xAB);
        handed = out.hand(c, &picture, 0, 1);
    });
    assert_eq!(handed.value, 0);
    assert!(out.slot(1).slice::<u32>().iter().all(|&w| w == 0xAB));
    out.destroy(&gpu);
    gpu.destroy_buffer(picture);
}
