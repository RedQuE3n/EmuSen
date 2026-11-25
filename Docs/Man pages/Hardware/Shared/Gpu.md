# `emusen-gpu` — Shared Vulkan Compute Device

Live crate: `EmuSen/Cores/Shared/emusen-gpu` (`emusen_gpu`, v1.6.0, dependency: `ash` 0.38 with runtime `loaded` feature).

`emusen-gpu` provides a single, console-agnostic Vulkan compute device layer shared by GPU-accelerated cores (`MarsRT`, `Diamond`, `Sapphire`). It handles runtime Vulkan loading, physical device ranking and selection (`EMUSEN_GPU_DEVICE`), host/device storage buffers, compute programs with optional specialization constants, synchronous/pending/double-buffered submissions, resource-leak tallying, and a host-lent Vulkan 1.2 device seam for zero-copy device-resident frame handoff. It owns no shaders, no console memory layouts, and no dependency on `emusen-native`.

---

## 1. Device Selection & Environment

- **Runtime loading:** Uses `ash::Entry::load()` (`libloading`) so systems without a Vulkan loader or ICD fail cleanly with a diagnostic string rather than a link error.
- **Environment variable:** `DEVICE_VARIABLE = "EMUSEN_GPU_DEVICE"`.
  - `GpuDevice::try_create(Some(substr))` selects the first compute-capable physical device whose name contains `substr` (case-insensitive).
  - `GpuDevice::try_create(None)` checks `EMUSEN_GPU_DEVICE`; if unset or empty, picks the highest-ranked compute-capable device.
  - `choose_device(&ash::Instance)` applies the same selection policy against an externally created Vulkan instance.
- **Device ranking order:** Discrete GPU > Integrated GPU > Virtual GPU > CPU/Other (e.g., `llvmpipe` / `lavapipe`, flagged via `GpuDevice::software() -> bool`).
- **Portability:** Enables `VK_KHR_portability_enumeration` and `VK_KHR_portability_subset` when offered by the loader/driver (e.g., MoltenVK).
- **Optional shader capabilities enabled on owned devices:**
  - `shaderInt64` (`GpuDevice::int64() -> bool`): Enabled when supported; required by wide integer rasterizers (e.g., Sapphire GS).
  - `VK_KHR_shader_float_controls` (`GpuDevice::float_controls() -> bool`): Enabled when the device supports 32-bit round-towards-zero and flush-to-zero controls.
  - On a lent device, both `int64()` and `float_controls()` return `false` because device feature activation belongs to the lender.

---

## 2. Public API Surface (v1.6.0)

| Item | Description |
|---|---|
| `GpuDevice::device_names() -> Vec<String>` | Compute-capable devices in selection rank order (empty if no Vulkan loader/driver). |
| `GpuDevice::try_create(Option<&str>) -> Result<GpuDevice, String>` | Opens an owned Vulkan instance, physical device, logical device, and single compute queue. |
| `GpuDevice::lent(&Lent) -> Result<GpuDevice, String>` | (`unsafe`) Borrows a host-owned Vulkan 1.2+ device and dedicated compute queue. |
| `GpuDevice::{name, software, max_buffer_bytes, report, is_lent, int64, float_controls}` | Read-only device metadata (`report` formats `"{name}, Vulkan {major}.{minor}"`). |
| `Where::{Host, Device}` | Buffer placement: host-visible/coherent mapped memory (prefers cached memory types for fast readback) vs device-local memory. |
| `GpuDevice::{create_buffer, create_buffer_shared, destroy_buffer}` | Allocates/destroys storage + transfer-src/dst buffers (`create_buffer_shared` uses concurrent sharing across borrower and lender queue families). |
| `GpuBuffer::{bytes, raw, slice::<T>, slice_mut::<T>}` | Buffer size, raw `VkBuffer` handle as `u64`, and typed slice access for host buffers (panics on device-only buffers). |
| `GpuDevice::{create_program, create_program_with_sets, create_program_specialised, destroy_program}` | Creates compute pipelines (entry `main`) with `0..buffers` storage buffer bindings in set 0, one push-constant range, `1..=AHEAD` descriptor sets, and optional `(constant_id, u32)` specialization constants. |
| `GpuDevice::submit(record)` | Records, submits, and waits synchronously (`Submission::Now`). |
| `GpuDevice::{submit_pending, pending, wait_for_pending}`, `Pending::wait` | Submits one asynchronous batch (`Submission::Pending`) that another thread (e.g., presenter) may await via `Arc<Pending>`. |
| `AHEAD` (`2`), `GpuDevice::{submit_ahead, wait_ahead}` | Two pipelined submission slots (`Submission::Ahead(0 | 1)`); each slot waits only when reused. |
| `GpuDevice::finished(Submission) -> bool` | Non-blocking fence poll for any submission slot. |
| `Timeline`, `GpuDevice::{create_timeline, timeline_value, destroy_timeline}`, `Timeline::raw` | Vulkan 1.2 timeline semaphores for cross-queue synchronization on lent devices. |
| `PictureOut::{new, on_device, slot, tell, ready, hand, destroy}`, `Handed` | Double-buffered frame handoff: copies into host slots on owned devices, or shared device buffers synchronized by `ready`/`done` timeline semaphores on lent devices. |
| `Tally`, `GpuDevice::tally()`, `Tally::count()` | Atomic count of live buffers and programs created and not yet destroyed (timelines excluded), inspectable after `GpuDevice` drops to verify leak-free teardown. |

### Command Recording (`Commands<'a>`)
- `submission() -> Submission`: Identifies the active submission slot.
- `fill(buffer, word)`: Whole-buffer `vkCmdFillBuffer`.
- `copy(from, to, bytes, from_offset, to_offset)`: Buffer-to-buffer copy.
- `dispatch(program, buffers, push, x, y, z)`: Writes descriptor set 0 and dispatches.
- `dispatch_in(program, set, buffers, push, x, y, z)`: Writes descriptor set `set` and dispatches; records the set as written for this command buffer (up to `WRITTEN_SETS = 16`).
- `dispatch_again(program, set, push, x, y, z)`: Re-dispatches a program using a descriptor set already written by `dispatch_in` in the same recording without rewriting the set (avoiding Vulkan `VUID-vkCmdBindPipeline-commandBuffer-recording` invalidation when a shader runs multiple passes per submission). Panics if `set` was not previously written in the current recording.
- `wait_for(timeline, value)` / `signal(timeline, value)`: Attaches at most one timeline semaphore wait before commands and one timeline signal after commands.

---

## 3. Synchronization & Usage Rules

1. **Per-command barriers:** Every `fill`, `copy`, and `dispatch*` emits a memory barrier ordering compute and transfer writes before all subsequent compute and transfer reads/writes in submission order.
2. **Host visibility barrier:** Every submission ends with a barrier to host reads so mapped host memory reflects completed GPU writes once the submission fence signals.
3. **Descriptor set lifetime:** A descriptor set written during recording must not be rewritten until the submission using it has completed (`create_program_with_sets` allocates one set per in-flight slot, up to `AHEAD`).
4. **Host buffer ownership:** Host buffers read or written by a submission remain owned by the GPU until that submission finishes.
5. **Single-submitter threading:** One thread submits to the compute queue. `Pending::wait` is the only cross-thread call permitted while no new submission is in flight.

---

## 4. Host-Lent Device Seam (`Lent` & `PictureOut`)

When a host renderer (such as the platform Vulkan presentation layer) already owns a Vulkan 1.2+ device, it lends a dedicated compute queue to a core through `Lent` / `Lent::from_raw`:
- **ABI-neutral handles:** `Lent` passes `vkGetInstanceProcAddr` as a plain function pointer plus raw `u64` Vulkan handles (`instance`, `physical`, `device`), API version (`>= 1.2`), borrower compute `(family, queue)`, and `host_family`. No `ash` types cross the FFI boundary.
- **Ownership & drop:** `GpuDevice::lent` creates its own command pool, command buffers, and fences. On `Drop`, it waits only on its own pending/ahead fences (never `vkDeviceWaitIdle`) and destroys only the resources it created, leaving the lender's instance and device intact.
- **Zero-copy `PictureOut` handoff:**
  - When constructed with `done: Some(host_done_timeline)` on a lent device (`on_device() == true`), `PictureOut` allocates two shared device-local buffers (`create_buffer_shared`) and an owned `ready` timeline semaphore.
  - `hand` copies the rendered frame into slot `i`, waits on `done` for the last timeline value told to the lender for that slot, and signals `ready` with the next monotonically increasing value.
  - `tell(value)` marks a value as actually handed to the lender; if the host takes frames via CPU copy instead of the device path and never calls `tell`, `PictureOut` does not stall waiting on an unsignalled `done` value.
  - On owned devices (or lent devices with `done: None`), `PictureOut` allocates two host-visible buffers and hands frames with `value = 0`.
