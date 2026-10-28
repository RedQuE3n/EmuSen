//! A device's memory types priced for unified memory (Sapphire_Redesign.md, R0 results, R0.5): for each type the
//! processor can map, 4 MB written by the device (a transfer's fill), waited on by a fence, then read by the processor,
//! the read timed from the fence's end; and 4 MB written in place by the processor (an upload with no staging copy),
//! timed, then copied by the device into a second buffer and checked. Types the processor cannot map are priced on the
//! staging path: the device's fill and its copy into the fastest mapped type, then the read. The median of the rounds.
//!
//! `cargo run --release --example memtypes -- [device name] [rounds]`

use std::hint::black_box;
use std::time::Instant;

use ash::vk;

const BYTES: u64 = 4 << 20;

struct Gpu {
    _entry: ash::Entry,
    instance: ash::Instance,
    device: ash::Device,
    physical: vk::PhysicalDevice,
    queue: vk::Queue,
    pool: vk::CommandPool,
    memory: vk::PhysicalDeviceMemoryProperties,
}

struct Buffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    map: Option<*mut u64>,
    coherent: bool,
}

fn flags(f: vk::MemoryPropertyFlags) -> String {
    let mut v = Vec::new();
    for (bit, name) in [
        (vk::MemoryPropertyFlags::DEVICE_LOCAL, "device-local"),
        (vk::MemoryPropertyFlags::HOST_VISIBLE, "host-visible"),
        (vk::MemoryPropertyFlags::HOST_COHERENT, "coherent"),
        (vk::MemoryPropertyFlags::HOST_CACHED, "cached"),
        (vk::MemoryPropertyFlags::LAZILY_ALLOCATED, "lazy"),
        (vk::MemoryPropertyFlags::PROTECTED, "protected"),
        (vk::MemoryPropertyFlags::DEVICE_COHERENT_AMD, "device-coherent"),
        (vk::MemoryPropertyFlags::DEVICE_UNCACHED_AMD, "device-uncached"),
    ] {
        if f.contains(bit) {
            v.push(name);
        }
    }
    if v.is_empty() { "none".into() } else { v.join(",") }
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

impl Gpu {
    fn new(name: Option<&str>) -> Gpu {
        // SAFETY: the loader and every handle below are used as the specification allows and destroyed at the end.
        unsafe {
            let entry = ash::Entry::load().expect("a Vulkan loader");
            let app = vk::ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 1, 0));
            let instance = entry.create_instance(&vk::InstanceCreateInfo::default().application_info(&app), None).expect("an instance");
            let physical = instance
                .enumerate_physical_devices()
                .expect("devices")
                .into_iter()
                .find(|&p| {
                    let n = instance.get_physical_device_properties(p).device_name_as_c_str().map(|c| c.to_string_lossy().into_owned()).unwrap_or_default();
                    name.is_none_or(|w| n.contains(w))
                })
                .expect("a device of that name");
            let props = instance.get_physical_device_properties(physical);
            println!("device\t{}\tdriver {:x}", props.device_name_as_c_str().map(|c| c.to_string_lossy().into_owned()).unwrap_or_default(), props.driver_version);
            let family = instance
                .get_physical_device_queue_family_properties(physical)
                .iter()
                .position(|q| q.queue_flags.intersects(vk::QueueFlags::COMPUTE | vk::QueueFlags::GRAPHICS))
                .expect("a compute queue") as u32;
            let prio = [1.0];
            let qi = [vk::DeviceQueueCreateInfo::default().queue_family_index(family).queue_priorities(&prio)];
            let device = instance.create_device(physical, &vk::DeviceCreateInfo::default().queue_create_infos(&qi), None).expect("a device");
            let queue = device.get_device_queue(family, 0);
            let pool = device.create_command_pool(&vk::CommandPoolCreateInfo::default().queue_family_index(family).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER), None).expect("a pool");
            let memory = instance.get_physical_device_memory_properties(physical);
            Gpu { _entry: entry, instance, device, physical, queue, pool, memory }
        }
    }

    /// A buffer of `BYTES` in memory type `t`, mapped if the processor can map it; none if the type refuses it.
    fn buffer(&self, t: u32) -> Option<Buffer> {
        // SAFETY: as in `new`.
        unsafe {
            let usage = vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::STORAGE_BUFFER;
            let buffer = self.device.create_buffer(&vk::BufferCreateInfo::default().size(BYTES).usage(usage), None).ok()?;
            let req = self.device.get_buffer_memory_requirements(buffer);
            if req.memory_type_bits >> t & 1 == 0 {
                self.device.destroy_buffer(buffer, None);
                return None;
            }
            let Ok(memory) = self.device.allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(req.size).memory_type_index(t), None) else {
                self.device.destroy_buffer(buffer, None);
                return None;
            };
            self.device.bind_buffer_memory(buffer, memory, 0).ok()?;
            let f = self.memory.memory_types[t as usize].property_flags;
            let map = f.contains(vk::MemoryPropertyFlags::HOST_VISIBLE).then(|| self.device.map_memory(memory, 0, BYTES, vk::MemoryMapFlags::empty()).expect("mapped").cast::<u64>());
            Some(Buffer { buffer, memory, map, coherent: f.contains(vk::MemoryPropertyFlags::HOST_COHERENT) })
        }
    }

    fn drop_buffer(&self, b: Buffer) {
        // SAFETY: the device is idle after every submission here.
        unsafe {
            self.device.destroy_buffer(b.buffer, None);
            self.device.free_memory(b.memory, None);
        }
    }

    /// Records `f` into a command buffer, submits it with a fence, waits; the host's nanoseconds from submit to the
    /// fence's end.
    fn run(&self, f: impl FnOnce(vk::CommandBuffer)) -> f64 {
        // SAFETY: as in `new`.
        unsafe {
            let cb = self.device.allocate_command_buffers(&vk::CommandBufferAllocateInfo::default().command_pool(self.pool).command_buffer_count(1)).expect("a command buffer")[0];
            self.device.begin_command_buffer(cb, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)).expect("begun");
            f(cb);
            let barrier = [vk::MemoryBarrier::default().src_access_mask(vk::AccessFlags::TRANSFER_WRITE).dst_access_mask(vk::AccessFlags::HOST_READ)];
            self.device.cmd_pipeline_barrier(cb, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::HOST, vk::DependencyFlags::empty(), &barrier, &[], &[]);
            self.device.end_command_buffer(cb).expect("ended");
            let fence = self.device.create_fence(&vk::FenceCreateInfo::default(), None).expect("a fence");
            let cbs = [cb];
            let t = Instant::now();
            self.device.queue_submit(self.queue, &[vk::SubmitInfo::default().command_buffers(&cbs)], fence).expect("submitted");
            self.device.wait_for_fences(&[fence], true, u64::MAX).expect("waited");
            let ns = t.elapsed().as_secs_f64() * 1e9;
            self.device.destroy_fence(fence, None);
            self.device.free_command_buffers(self.pool, &cbs);
            ns
        }
    }

    fn invalidate(&self, b: &Buffer) {
        if !b.coherent {
            // SAFETY: the range is the mapped allocation.
            unsafe { self.device.invalidate_mapped_memory_ranges(&[vk::MappedMemoryRange::default().memory(b.memory).size(vk::WHOLE_SIZE)]).expect("invalidated") };
        }
    }

    fn flush(&self, b: &Buffer) {
        if !b.coherent {
            // SAFETY: as above.
            unsafe { self.device.flush_mapped_memory_ranges(&[vk::MappedMemoryRange::default().memory(b.memory).size(vk::WHOLE_SIZE)]).expect("flushed") };
        }
    }
}

/// The processor's read of a mapped buffer: its sum, and whether every word was `want`.
fn read(p: *mut u64, want: u64) -> (u64, bool) {
    // SAFETY: a mapped allocation of `BYTES`, which no submission is writing.
    let words = unsafe { std::slice::from_raw_parts(p.cast_const(), (BYTES / 8) as usize) };
    let (mut sum, mut all) = (0u64, true);
    for w in words.chunks_exact(8) {
        let mut x = 0;
        for &v in w {
            x ^= v;
            all &= v == want;
        }
        sum = sum.wrapping_add(x);
    }
    (black_box(sum), all)
}

fn write(p: *mut u64, v: u64) {
    // SAFETY: as above.
    let words = unsafe { std::slice::from_raw_parts_mut(p, (BYTES / 8) as usize) };
    for w in words.iter_mut() {
        *w = v;
    }
    black_box(&words[0]);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rounds: usize = args.get(1).map_or(9, |a| a.parse().expect("rounds"));
    let g = Gpu::new(args.first().map(String::as_str).filter(|s| !s.is_empty()));
    let types = g.memory.memory_type_count;
    for t in 0..types {
        let m = g.memory.memory_types[t as usize];
        println!("type {t}\theap {} ({} MB)\t{}", m.heap_index, g.memory.memory_heaps[m.heap_index as usize].size >> 20, flags(m.property_flags));
    }
    // The staging path's target: the mapped type that reads fastest, chosen below.
    let mut best: Option<(u32, f64)> = None;
    for t in 0..types {
        let Some(b) = g.buffer(t) else { continue };
        let Some(p) = b.map else {
            g.drop_buffer(b);
            continue;
        };
        let (mut reads, mut writes, mut ok) = (Vec::new(), Vec::new(), true);
        let check = g.buffer(t).expect("a second buffer of the same type");
        for r in 0..rounds {
            let pattern = 0x0101_0101u32.wrapping_mul(r as u32 + 3);
            g.run(|cb| unsafe { g.device.cmd_fill_buffer(cb, b.buffer, 0, BYTES, pattern) });
            let t0 = Instant::now();
            g.invalidate(&b);
            let (_, good) = read(p, u64::from(pattern) * 0x1_0000_0001);
            reads.push(t0.elapsed().as_secs_f64() * 1e3);
            ok &= good;
            let v = 0x5EED_0000_0000_0000 | r as u64;
            let t1 = Instant::now();
            write(p, v);
            g.flush(&b);
            writes.push(t1.elapsed().as_secs_f64() * 1e3);
            g.run(|cb| unsafe { g.device.cmd_copy_buffer(cb, b.buffer, check.buffer, &[vk::BufferCopy::default().size(BYTES)]) });
            g.invalidate(&check);
            ok &= read(check.map.expect("mapped"), v).1;
        }
        let (rd, wr) = (median(reads), median(writes));
        println!("type {t}\t{}\tread after the fence {rd:.3} ms for 4 MB\twritten in place {wr:.3} ms\t{}", flags(g.memory.memory_types[t as usize].property_flags), if ok { "every word checked" } else { "WRONG WORDS" });
        if best.is_none_or(|(_, x)| rd < x) {
            best = Some((t, rd));
        }
        g.drop_buffer(check);
        g.drop_buffer(b);
    }
    let Some((host, _)) = best else { return };
    let staging = g.buffer(host).expect("the staging buffer");
    for t in 0..types {
        if g.memory.memory_types[t as usize].property_flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE) {
            continue;
        }
        let Some(b) = g.buffer(t) else { continue };
        let mut total = Vec::new();
        let mut ok = true;
        for r in 0..rounds {
            let pattern = 0x0303_0303u32.wrapping_mul(r as u32 + 5);
            g.run(|cb| unsafe { g.device.cmd_fill_buffer(cb, b.buffer, 0, BYTES, pattern) });
            let t0 = Instant::now();
            g.run(|cb| unsafe { g.device.cmd_copy_buffer(cb, b.buffer, staging.buffer, &[vk::BufferCopy::default().size(BYTES)]) });
            g.invalidate(&staging);
            ok &= read(staging.map.expect("mapped"), u64::from(pattern) * 0x1_0000_0001).1;
            total.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        println!("type {t}\t{}\tstaged through type {host}: copy and read {:.3} ms for 4 MB\t{}", flags(g.memory.memory_types[t as usize].property_flags), median(total), if ok { "every word checked" } else { "WRONG WORDS" });
        g.drop_buffer(b);
    }
    g.drop_buffer(staging);
    // SAFETY: the device is idle and nothing of it is used after.
    unsafe {
        g.device.destroy_command_pool(g.pool, None);
        g.device.destroy_device(None);
        g.instance.destroy_instance(None);
    }
    let _ = g.physical;
}
