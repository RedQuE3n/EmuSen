//! Host hardware and process telemetry sampled from Linux procfs/sysfs and Galaxia's tree.

use crate::tree::{self, Directory, Tree};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

const STORAGE_CACHE_TTL: Duration = Duration::from_secs(60);
const COLD_CPU_SAMPLE_DELAY: Duration = Duration::from_millis(40);

/// Caller-supplied path overrides and options deserialized from `paths_json`.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct TelemetryPaths {
    pub data_store: Option<String>,
    pub games: Option<String>,
    pub saves: Option<String>,
    pub save_states: Option<String>,
    pub packages: Option<String>,
    pub media: Option<String>,
    pub artwork: Option<String>,
    pub screenshots: Option<String>,
    pub shaders: Option<String>,
    pub themes: Option<String>,
    pub cores: Option<String>,
    pub firmware: Option<String>,
    pub logs: Option<String>,
    pub unpacked: Option<String>,
    pub compressed: Option<String>,
    pub gpu_pci_slot: Option<String>,
    pub gpu_vendor: Option<String>,
    pub gpu_device: Option<String>,
    pub refresh_storage: bool,
    pub async_storage: bool,
}

impl TelemetryPaths {
    /// Builds path configuration from a Galaxia `Tree` (writable directories only; games left unset).
    pub fn from_tree(tree: &Tree) -> Self {
        let dir = |d| tree.directory(d).map(|p| p.to_string_lossy().into_owned());
        Self {
            data_store: Some(tree.home().to_string_lossy().into_owned()),
            saves: dir(Directory::Saves),
            save_states: dir(Directory::SaveStates),
            packages: dir(Directory::Packages),
            media: dir(Directory::Media),
            artwork: dir(Directory::Artwork),
            screenshots: dir(Directory::Screenshots),
            shaders: dir(Directory::Shaders),
            themes: dir(Directory::Themes),
            cores: dir(Directory::Cores),
            firmware: dir(Directory::Firmware),
            logs: dir(Directory::Logs),
            unpacked: dir(Directory::Unpacked),
            compressed: dir(Directory::Compressed),
            ..Self::default()
        }
    }

    fn has_any_explicit_category(&self) -> bool {
        [
            &self.saves,
            &self.save_states,
            &self.packages,
            &self.media,
            &self.artwork,
            &self.screenshots,
            &self.shaders,
            &self.themes,
            &self.cores,
            &self.firmware,
            &self.logs,
            &self.unpacked,
            &self.compressed,
            &self.games,
        ]
        .into_iter()
        .any(Option::is_some)
    }
}

/// Complete host and EmuSen process telemetry snapshot serialized as JSON.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SysTelemetrySnapshot {
    pub schema: u32,
    pub host: HostTelemetry,
    pub process: ProcessTelemetry,
}

/// Host CPU, memory, GPU, and storage mount metrics.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostTelemetry {
    pub cpu: HostCpu,
    pub memory: Option<HostMemory>,
    pub gpu: Option<HostGpu>,
    pub gpus: Vec<HostGpu>,
    pub mounts: Vec<HostMount>,
}

/// Host CPU topology, frequencies, and utilization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostCpu {
    pub model: Option<String>,
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub avg_mhz: Option<f64>,
    pub max_mhz: Option<f64>,
    pub total_pct: Option<f64>,
    pub iowait_pct: Option<f64>,
    pub per_core: Vec<CpuCoreInfo>,
}

/// Frequency and utilization for a single logical CPU core.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CpuCoreInfo {
    pub id: u32,
    pub mhz: Option<f64>,
    pub max_mhz: Option<f64>,
    pub busy_pct: Option<f64>,
    pub iowait_pct: Option<f64>,
}

/// Host physical memory and swap usage in bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostMemory {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub buffers_bytes: u64,
    pub cached_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_free_bytes: u64,
}

/// Host GPU telemetry read from DRM sysfs; unavailable attributes remain `None`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostGpu {
    pub card: String,
    pub driver: Option<String>,
    pub vendor: Option<String>,
    pub device: Option<String>,
    pub pci_slot: Option<String>,
    pub name: Option<String>,
    pub boot_vga: bool,
    pub busy_pct: Option<f64>,
    pub mem_busy_pct: Option<f64>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
    pub gtt_used_bytes: Option<u64>,
    pub temp_c: Option<f64>,
    pub sclk_mhz: Option<f64>,
}

/// Deduplicated host filesystem mount entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostMount {
    pub mountpoint: String,
    pub fs_type: String,
    pub roles: Vec<String>,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
}

/// Process-level CPU, thread groups, memory, DRM GPU, and Galaxia storage footprint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProcessTelemetry {
    pub pid: u32,
    pub cpu_total_host_pct: Option<f64>,
    pub cpu_one_core_pct: Option<f64>,
    pub single_core_bound: bool,
    pub threads: Vec<ThreadInfo>,
    pub thread_groups: BTreeMap<String, ThreadGroupUsage>,
    pub memory: Option<ProcessMemory>,
    pub gpu: ProcessGpuUsage,
    pub storage: StorageFootprint,
}

/// Single thread CPU usage and assigned functional group.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThreadInfo {
    pub tid: u32,
    pub comm: String,
    pub group: String,
    pub cpu_one_core_pct: Option<f64>,
    pub cpu_total_host_pct: Option<f64>,
}

/// Aggregate thread count and CPU usage for one thread group.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ThreadGroupUsage {
    pub count: u32,
    pub cpu_one_core_pct: Option<f64>,
    pub cpu_total_host_pct: Option<f64>,
}

/// Process memory footprint from `/proc/self/smaps_rollup` or `/proc/self/status`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessMemory {
    pub rss_bytes: u64,
    pub pss_bytes: Option<u64>,
    pub private_bytes: Option<u64>,
    pub swap_bytes: Option<u64>,
}

/// Per-process DRM GPU engine percentages and VRAM/GTT allocations from fdinfo.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProcessGpuUsage {
    pub available: bool,
    pub gfx_pct: Option<f64>,
    pub compute_pct: Option<f64>,
    pub copy_pct: Option<f64>,
    pub vram_bytes: Option<u64>,
    pub gtt_bytes: Option<u64>,
    pub clients: u32,
}

/// Cached directory size breakdown separating writable Galaxia data from read-only games.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageFootprint {
    pub pending: bool,
    pub stale: bool,
    pub categories: StorageCategories,
    pub writable_total_bytes: u64,
    pub games_library_bytes: Option<u64>,
}

/// Byte counts per writable Galaxia directory category.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageCategories {
    pub saves: u64,
    pub save_states: u64,
    pub packages: u64,
    pub media: u64,
    pub artwork: u64,
    pub screenshots: u64,
    pub shaders: u64,
    pub themes: u64,
    pub cores: u64,
    pub firmware: u64,
    pub logs: u64,
    pub unpacked: u64,
    pub compressed: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ResolvedStoragePaths {
    data_store: Option<PathBuf>,
    games: Option<PathBuf>,
    saves: Option<PathBuf>,
    save_states: Option<PathBuf>,
    packages: Option<PathBuf>,
    media: Option<PathBuf>,
    artwork: Option<PathBuf>,
    screenshots: Option<PathBuf>,
    shaders: Option<PathBuf>,
    themes: Option<PathBuf>,
    cores: Option<PathBuf>,
    firmware: Option<PathBuf>,
    logs: Option<PathBuf>,
    unpacked: Option<PathBuf>,
    compressed: Option<PathBuf>,
}

impl ResolvedStoragePaths {
    fn from_input(input: &TelemetryPaths) -> Self {
        let opt = |s: &Option<String>| s.as_deref().filter(|v| !v.trim().is_empty()).map(PathBuf::from);
        if let Some(ds) = opt(&input.data_store) {
            let synthetic = Tree { data_override: Some(ds.clone()), ..Tree::default() };
            let pick = |s: &Option<String>, d: Directory| opt(s).or_else(|| synthetic.directory(d));
            return Self {
                data_store: Some(ds),
                games: opt(&input.games),
                saves: pick(&input.saves, Directory::Saves),
                save_states: pick(&input.save_states, Directory::SaveStates),
                packages: pick(&input.packages, Directory::Packages),
                media: pick(&input.media, Directory::Media),
                artwork: pick(&input.artwork, Directory::Artwork),
                screenshots: pick(&input.screenshots, Directory::Screenshots),
                shaders: pick(&input.shaders, Directory::Shaders),
                themes: pick(&input.themes, Directory::Themes),
                cores: pick(&input.cores, Directory::Cores),
                firmware: pick(&input.firmware, Directory::Firmware),
                logs: pick(&input.logs, Directory::Logs),
                unpacked: pick(&input.unpacked, Directory::Unpacked),
                compressed: pick(&input.compressed, Directory::Compressed),
            };
        }
        if input.has_any_explicit_category() {
            return Self {
                data_store: None,
                games: opt(&input.games),
                saves: opt(&input.saves),
                save_states: opt(&input.save_states),
                packages: opt(&input.packages),
                media: opt(&input.media),
                artwork: opt(&input.artwork),
                screenshots: opt(&input.screenshots),
                shaders: opt(&input.shaders),
                themes: opt(&input.themes),
                cores: opt(&input.cores),
                firmware: opt(&input.firmware),
                logs: opt(&input.logs),
                unpacked: opt(&input.unpacked),
                compressed: opt(&input.compressed),
            };
        }
        let tree = tree::tree();
        Self {
            data_store: Some(tree.home()),
            games: None,
            saves: tree.directory(Directory::Saves),
            save_states: tree.directory(Directory::SaveStates),
            packages: tree.directory(Directory::Packages),
            media: tree.directory(Directory::Media),
            artwork: tree.directory(Directory::Artwork),
            screenshots: tree.directory(Directory::Screenshots),
            shaders: tree.directory(Directory::Shaders),
            themes: tree.directory(Directory::Themes),
            cores: tree.directory(Directory::Cores),
            firmware: tree.directory(Directory::Firmware),
            logs: tree.directory(Directory::Logs),
            unpacked: tree.directory(Directory::Unpacked),
            compressed: tree.directory(Directory::Compressed),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CpuTickRow {
    total: u64,
    busy: u64,
    iowait: u64,
}

#[derive(Clone, Debug, Default)]
struct ProcStatSample {
    aggregate: CpuTickRow,
    cores: Vec<(u32, CpuTickRow)>,
}

#[derive(Clone, Debug, Default)]
struct ProcessSample {
    proc_ticks: u64,
    threads: HashMap<u32, u64>,
}

#[derive(Clone, Debug, Default)]
struct DrmSample {
    clients: HashMap<String, DrmClientTicks>,
}

#[derive(Clone, Debug, Default)]
struct DrmClientTicks {
    gfx_ns: Option<u64>,
    compute_ns: Option<u64>,
    copy_ns: Option<u64>,
}

#[derive(Default)]
struct SamplerState {
    at: Option<Instant>,
    host_stat: Option<ProcStatSample>,
    proc_stat: Option<ProcessSample>,
    drm: Option<DrmSample>,
}

#[derive(Default)]
struct StorageCacheState {
    key: Option<ResolvedStoragePaths>,
    updated_at: Option<Instant>,
    footprint: Option<StorageFootprint>,
    in_flight: bool,
}

static SAMPLER: OnceLock<Mutex<SamplerState>> = OnceLock::new();
static STORAGE_CACHE: OnceLock<Mutex<StorageCacheState>> = OnceLock::new();

fn sampler_lock() -> &'static Mutex<SamplerState> {
    SAMPLER.get_or_init(|| Mutex::new(SamplerState::default()))
}

fn storage_lock() -> &'static Mutex<StorageCacheState> {
    STORAGE_CACHE.get_or_init(|| Mutex::new(StorageCacheState::default()))
}

/// Captures a live telemetry snapshot using the shared incremental sampler and cached storage walk.
pub fn snapshot(paths: &TelemetryPaths) -> SysTelemetrySnapshot {
    snapshot_with_roots(paths, Path::new("/proc"), Path::new("/sys"), COLD_CPU_SAMPLE_DELAY)
}

fn snapshot_with_roots(paths: &TelemetryPaths, proc_root: &Path, sys_root: &Path, cold_delay: Duration) -> SysTelemetrySnapshot {
    let mut state = sampler_lock().lock().unwrap_or_else(|p| p.into_inner());
    if state.host_stat.is_none() && !cold_delay.is_zero() {
        let first_host = read_proc_stat(&proc_root.join("stat"));
        let first_proc = read_process_sample(&proc_root.join("self"));
        let first_drm = read_drm_fdinfo_sample(&proc_root.join("self"));
        state.at = Some(Instant::now());
        state.host_stat = first_host;
        state.proc_stat = Some(first_proc);
        state.drm = Some(first_drm);
        drop(state);
        std::thread::sleep(cold_delay);
        state = sampler_lock().lock().unwrap_or_else(|p| p.into_inner());
    }
    sample_into_state(&mut state, paths, proc_root, sys_root)
}

fn sample_into_state(state: &mut SamplerState, paths: &TelemetryPaths, proc_root: &Path, sys_root: &Path) -> SysTelemetrySnapshot {
    let now = Instant::now();
    let wall_ns = state.at.map(|prev| now.saturating_duration_since(prev).as_nanos() as u64).unwrap_or(0);
    let curr_host = read_proc_stat(&proc_root.join("stat"));
    let curr_proc = read_process_sample(&proc_root.join("self"));
    let curr_drm = read_drm_fdinfo_sample(&proc_root.join("self"));
    let host_delta = match (&state.host_stat, &curr_host) {
        (Some(prev), Some(curr)) => Some(curr.aggregate.total.saturating_sub(prev.aggregate.total)),
        _ => None,
    };

    let cpu = build_host_cpu(proc_root, sys_root, state.host_stat.as_ref(), curr_host.as_ref());
    let memory = std::fs::read_to_string(proc_root.join("meminfo")).ok().and_then(|s| parse_meminfo(&s));
    let (gpu, gpus) = read_drm_gpus(
        &sys_root.join("class/drm"),
        paths.gpu_pci_slot.as_deref(),
        paths.gpu_vendor.as_deref(),
        paths.gpu_device.as_deref(),
    );
    let resolved_paths = ResolvedStoragePaths::from_input(paths);
    let mounts = read_host_mounts(&proc_root.join("self/mountinfo"), &resolved_paths);
    let process = build_process_telemetry(
        proc_root,
        cpu.logical_cores,
        host_delta,
        wall_ns,
        state.proc_stat.as_ref(),
        &curr_proc,
        state.drm.as_ref(),
        &curr_drm,
        resolve_storage_footprint(&resolved_paths, paths.refresh_storage, paths.async_storage),
    );

    state.at = Some(now);
    if curr_host.is_some() {
        state.host_stat = curr_host;
    }
    state.proc_stat = Some(curr_proc);
    state.drm = Some(curr_drm);

    SysTelemetrySnapshot { schema: 1, host: HostTelemetry { cpu, memory, gpu, gpus, mounts }, process }
}

#[derive(Default)]
struct CpuInfoParsed {
    model: Option<String>,
    physical_cores: u32,
    logical_cores: u32,
    mhz: Vec<Option<f64>>,
}

fn parse_cpuinfo(text: &str) -> CpuInfoParsed {
    let mut model = None;
    let mut logical = 0u32;
    let mut core_pairs = BTreeSet::new();
    let mut cpu_cores_hint = None;
    let mut mhz = Vec::new();
    let mut cur_phys = None::<String>;
    let mut cur_core = None::<String>;
    let mut cur_mhz = None::<f64>;
    let mut in_block = false;

    let flush = |in_block: &mut bool, phys: &mut Option<String>, core: &mut Option<String>, m: &mut Option<f64>, pairs: &mut BTreeSet<(String, String)>, mhz_vec: &mut Vec<Option<f64>>| {
        if !*in_block {
            return;
        }
        if let (Some(p), Some(c)) = (phys.take(), core.take()) {
            pairs.insert((p, c));
        } else {
            *phys = None;
            *core = None;
        }
        mhz_vec.push(m.take());
        *in_block = false;
    };

    for line in text.lines() {
        if line.trim().is_empty() {
            flush(&mut in_block, &mut cur_phys, &mut cur_core, &mut cur_mhz, &mut core_pairs, &mut mhz);
            continue;
        }
        let Some((k, v)) = line.split_once(':') else { continue };
        let (key, val) = (k.trim(), v.trim());
        match key {
            "processor" => {
                flush(&mut in_block, &mut cur_phys, &mut cur_core, &mut cur_mhz, &mut core_pairs, &mut mhz);
                in_block = true;
                logical += 1;
            }
            "model name" | "Hardware" | "Processor" if model.is_none() => model = Some(val.to_string()),
            "physical id" => cur_phys = Some(val.to_string()),
            "core id" => cur_core = Some(val.to_string()),
            "cpu cores" if cpu_cores_hint.is_none() => cpu_cores_hint = val.parse::<u32>().ok(),
            "cpu MHz" => cur_mhz = val.parse::<f64>().ok(),
            _ => {}
        }
    }
    flush(&mut in_block, &mut cur_phys, &mut cur_core, &mut cur_mhz, &mut core_pairs, &mut mhz);

    let physical = if !core_pairs.is_empty() {
        core_pairs.len() as u32
    } else {
        cpu_cores_hint.unwrap_or(logical)
    };
    CpuInfoParsed { model, physical_cores: physical, logical_cores: logical, mhz }
}

fn parse_proc_stat_text(text: &str) -> Option<ProcStatSample> {
    let mut aggregate = None;
    let mut cores = Vec::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(label) = parts.next() else { continue };
        if !label.starts_with("cpu") {
            continue;
        }
        let nums: Vec<u64> = parts.filter_map(|p| p.parse::<u64>().ok()).collect();
        if nums.len() < 4 {
            continue;
        }
        let total: u64 = nums.iter().take(8).sum();
        let idle = nums[3];
        let iowait = nums.get(4).copied().unwrap_or(0);
        let row = CpuTickRow { total, busy: total.saturating_sub(idle + iowait), iowait };
        if label == "cpu" {
            aggregate = Some(row);
        } else if let Ok(id) = label[3..].parse::<u32>() {
            cores.push((id, row));
        }
    }
    aggregate.map(|agg| ProcStatSample { aggregate: agg, cores })
}

fn read_proc_stat(path: &Path) -> Option<ProcStatSample> {
    std::fs::read_to_string(path).ok().and_then(|s| parse_proc_stat_text(&s))
}

fn cpu_pct_delta(prev: &CpuTickRow, curr: &CpuTickRow) -> Option<(f64, f64)> {
    let total = curr.total.saturating_sub(prev.total);
    if total == 0 {
        return None;
    }
    let busy = curr.busy.saturating_sub(prev.busy);
    let iowait = curr.iowait.saturating_sub(prev.iowait);
    Some((((busy as f64 / total as f64) * 100.0).clamp(0.0, 100.0), ((iowait as f64 / total as f64) * 100.0).clamp(0.0, 100.0)))
}

fn read_u64_file(path: &Path) -> Option<u64> {
    std::fs::read_to_string(path).ok()?.trim().parse::<u64>().ok()
}

fn build_host_cpu(proc_root: &Path, sys_root: &Path, prev: Option<&ProcStatSample>, curr: Option<&ProcStatSample>) -> HostCpu {
    let info = std::fs::read_to_string(proc_root.join("cpuinfo")).ok().map(|s| parse_cpuinfo(&s)).unwrap_or_default();
    let logical = info.logical_cores.max(curr.map_or(0, |s| s.cores.len() as u32));
    let physical = info.physical_cores.max(if logical > 0 { 1 } else { 0 }).min(logical.max(1));
    let (total_pct, iowait_pct) = match (prev, curr) {
        (Some(p), Some(c)) => cpu_pct_delta(&p.aggregate, &c.aggregate).map_or((None, None), |(b, io)| (Some(b), Some(io))),
        _ => (None, None),
    };

    let prev_map: HashMap<u32, &CpuTickRow> = prev.map(|p| p.cores.iter().map(|(id, r)| (*id, r)).collect()).unwrap_or_default();
    let curr_map: HashMap<u32, &CpuTickRow> = curr.map(|c| c.cores.iter().map(|(id, r)| (*id, r)).collect()).unwrap_or_default();

    let mut per_core = Vec::with_capacity(logical as usize);
    let mut sum_mhz = 0.0f64;
    let mut count_mhz = 0u32;
    let mut max_mhz = None::<f64>;
    for id in 0..logical {
        let cpufreq = sys_root.join(format!("devices/system/cpu/cpu{id}/cpufreq"));
        let cur = read_u64_file(&cpufreq.join("scaling_cur_freq"))
            .map(|khz| khz as f64 / 1000.0)
            .or_else(|| info.mhz.get(id as usize).copied().flatten());
        let core_max = read_u64_file(&cpufreq.join("cpuinfo_max_freq")).map(|khz| khz as f64 / 1000.0);
        if let Some(m) = cur {
            sum_mhz += m;
            count_mhz += 1;
        }
        if let Some(m) = core_max.or(cur) {
            max_mhz = Some(max_mhz.map_or(m, |prev| prev.max(m)));
        }
        let (busy_pct, core_iowait) = match (prev_map.get(&id), curr_map.get(&id)) {
            (Some(p), Some(c)) => cpu_pct_delta(p, c).map_or((None, None), |(b, io)| (Some(b), Some(io))),
            _ => (None, None),
        };
        per_core.push(CpuCoreInfo { id, mhz: cur, max_mhz: core_max, busy_pct, iowait_pct: core_iowait });
    }

    HostCpu {
        model: info.model,
        physical_cores: physical,
        logical_cores: logical,
        avg_mhz: (count_mhz > 0).then(|| sum_mhz / f64::from(count_mhz)),
        max_mhz,
        total_pct,
        iowait_pct,
        per_core,
    }
}

fn parse_meminfo(text: &str) -> Option<HostMemory> {
    let mut map = HashMap::<&str, u64>::new();
    for line in text.lines() {
        let Some((key, rest)) = line.split_once(':') else { continue };
        let Some(num) = rest.split_whitespace().next().and_then(|n| n.parse::<u64>().ok()) else { continue };
        map.insert(key.trim(), num.saturating_mul(1024));
    }
    let total_bytes = *map.get("MemTotal")?;
    let free_bytes = map.get("MemFree").copied().unwrap_or(0);
    let buffers_bytes = map.get("Buffers").copied().unwrap_or(0);
    let cached_bytes = map.get("Cached").copied().unwrap_or(0);
    let available_bytes = map.get("MemAvailable").copied().unwrap_or_else(|| free_bytes + buffers_bytes + cached_bytes);
    let used_bytes = total_bytes.saturating_sub(available_bytes);
    let swap_total_bytes = map.get("SwapTotal").copied().unwrap_or(0);
    let swap_free_bytes = map.get("SwapFree").copied().unwrap_or(0);
    let swap_used_bytes = swap_total_bytes.saturating_sub(swap_free_bytes);
    Some(HostMemory { total_bytes, available_bytes, used_bytes, free_bytes, buffers_bytes, cached_bytes, swap_total_bytes, swap_used_bytes, swap_free_bytes })
}

fn read_trimmed(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn read_hwmon_metrics(device_dir: &Path) -> (Option<f64>, Option<f64>) {
    let hwmon_dir = device_dir.join("hwmon");
    let Ok(entries) = std::fs::read_dir(&hwmon_dir) else { return (None, None) };
    let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    for dir in dirs {
        let temp = read_u64_file(&dir.join("temp1_input")).map(|mdeg| mdeg as f64 / 1000.0);
        let freq = read_u64_file(&dir.join("freq1_input")).map(|hz| hz as f64 / 1_000_000.0);
        if temp.is_some() || freq.is_some() {
            return (temp, freq);
        }
    }
    (None, None)
}

fn parse_active_sclk_mhz(device_dir: &Path) -> Option<f64> {
    let text = std::fs::read_to_string(device_dir.join("pp_dpm_sclk")).ok()?;
    for line in text.lines() {
        if !line.contains('*') {
            continue;
        }
        for tok in line.split_whitespace() {
            if let Some(num) = tok.strip_suffix("Mhz").or_else(|| tok.strip_suffix("MHz")) {
                if let Ok(mhz) = num.parse::<f64>() {
                    return Some(mhz);
                }
            }
        }
    }
    None
}

fn read_drm_gpus(drm_root: &Path, preferred_pci_slot: Option<&str>, preferred_vendor: Option<&str>, preferred_device: Option<&str>) -> (Option<HostGpu>, Vec<HostGpu>) {
    let Ok(entries) = std::fs::read_dir(drm_root) else { return (None, Vec::new()) };
    let mut cards: Vec<(u32, String, PathBuf)> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let num = name.strip_prefix("card").filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))?.parse::<u32>().ok()?;
            Some((num, name, e.path()))
        })
        .collect();
    cards.sort_by_key(|(num, _, _)| *num);

    let mut gpus = Vec::new();
    for (_, card, card_path) in cards {
        let dev = card_path.join("device");
        if !dev.exists() {
            continue;
        }
        let uevent = std::fs::read_to_string(dev.join("uevent")).unwrap_or_default();
        let mut driver = None;
        let mut pci_slot = None;
        for line in uevent.lines() {
            if let Some(v) = line.strip_prefix("DRIVER=") {
                driver = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("PCI_SLOT_NAME=") {
                pci_slot = Some(v.trim().to_string());
            }
        }
        let vendor = read_trimmed(&dev.join("vendor"));
        let device = read_trimmed(&dev.join("device"));
        let name = read_trimmed(&dev.join("product_name")).or_else(|| read_trimmed(&dev.join("marketing_name")));
        let boot_vga = read_trimmed(&dev.join("boot_vga")).as_deref() == Some("1");
        let busy_pct = read_u64_file(&dev.join("gpu_busy_percent")).map(|v| v as f64);
        let mem_busy_pct = read_u64_file(&dev.join("mem_busy_percent")).map(|v| v as f64);
        let vram_used_bytes = read_u64_file(&dev.join("mem_info_vram_used"));
        let vram_total_bytes = read_u64_file(&dev.join("mem_info_vram_total"));
        let gtt_used_bytes = read_u64_file(&dev.join("mem_info_gtt_used"));
        let (temp_c, hwmon_mhz) = read_hwmon_metrics(&dev);
        let sclk_mhz = parse_active_sclk_mhz(&dev).or(hwmon_mhz);
        gpus.push(HostGpu { card, driver, vendor, device, pci_slot, name, boot_vga, busy_pct, mem_busy_pct, vram_used_bytes, vram_total_bytes, gtt_used_bytes, temp_c, sclk_mhz });
    }

    let primary = pick_primary_gpu(&gpus, preferred_pci_slot, preferred_vendor, preferred_device).cloned();
    (primary, gpus)
}

fn pick_primary_gpu<'a>(gpus: &'a [HostGpu], preferred_pci_slot: Option<&str>, preferred_vendor: Option<&str>, preferred_device: Option<&str>) -> Option<&'a HostGpu> {
    if let Some(slot) = preferred_pci_slot.filter(|s| !s.is_empty()) {
        if let Some(found) = gpus.iter().find(|g| g.pci_slot.as_deref().is_some_and(|s| s.eq_ignore_ascii_case(slot))) {
            return Some(found);
        }
    }
    if let (Some(v), Some(d)) = (preferred_vendor.filter(|s| !s.is_empty()), preferred_device.filter(|s| !s.is_empty())) {
        if let Some(found) = gpus.iter().find(|g| g.vendor.as_deref().is_some_and(|x| x.eq_ignore_ascii_case(v)) && g.device.as_deref().is_some_and(|x| x.eq_ignore_ascii_case(d))) {
            return Some(found);
        }
    }
    gpus.iter().max_by_key(|g| {
        let supported = matches!(g.driver.as_deref(), Some("amdgpu" | "i915" | "xe"));
        let vram_total = g.vram_total_bytes.unwrap_or(0);
        let busy = g.busy_pct.map_or(0u64, |v| (v * 10.0) as u64);
        (supported, g.vram_total_bytes.is_some(), g.boot_vga, vram_total, busy)
    })
}

#[derive(Clone, Debug)]
struct MountEntry {
    mount_id: String,
    mountpoint: String,
    fs_type: String,
}

fn unescape_mount_path(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 3 < bytes.len() {
            let oct = &raw[i + 1..i + 4];
            if let Ok(code) = u8::from_str_radix(oct, 8) {
                out.push(code as char);
                i += 4;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn parse_mountinfo(text: &str) -> Vec<MountEntry> {
    let mut entries = Vec::new();
    for line in text.lines() {
        let Some((left, right)) = line.split_once(" - ") else { continue };
        let left_fields: Vec<&str> = left.split(' ').collect();
        let right_fields: Vec<&str> = right.split(' ').collect();
        if left_fields.len() < 5 || right_fields.is_empty() {
            continue;
        }
        entries.push(MountEntry {
            mount_id: left_fields[0].to_string(),
            mountpoint: unescape_mount_path(left_fields[4]),
            fs_type: right_fields[0].to_string(),
        });
    }
    entries
}

fn find_mount<'a>(entries: &'a [MountEntry], target: &Path) -> Option<&'a MountEntry> {
    let norm = std::fs::canonicalize(target).unwrap_or_else(|_| target.to_path_buf());
    let target_str = norm.to_string_lossy();
    let mut best: Option<&MountEntry> = None;
    let mut best_len = 0usize;
    for entry in entries {
        let mp = entry.mountpoint.as_str();
        let matches = if mp == "/" {
            target_str.starts_with('/')
        } else {
            target_str.as_ref() == mp || target_str.starts_with(&format!("{mp}/"))
        };
        if matches && mp.len() >= best_len {
            best_len = mp.len();
            best = Some(entry);
        }
    }
    best
}

fn existing_ancestor(path: &Path) -> Option<PathBuf> {
    let mut cur = Some(path);
    while let Some(p) = cur {
        if p.exists() {
            return Some(p.to_path_buf());
        }
        cur = p.parent();
    }
    None
}

#[cfg(target_os = "linux")]
fn statvfs_bytes(path: &Path) -> Option<(u64, u64, u64)> {
    let c_path = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).ok()?;
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: `c_path` is a NUL-terminated path and `st` is valid writable `statvfs` storage.
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut st) } != 0 {
        return None;
    }
    let frsize = if st.f_frsize > 0 { st.f_frsize as u64 } else { st.f_bsize as u64 };
    let total = (st.f_blocks as u64).saturating_mul(frsize);
    let free = (st.f_bavail as u64).saturating_mul(frsize);
    let used = total.saturating_sub((st.f_bfree as u64).saturating_mul(frsize));
    Some((total, used, free))
}

#[cfg(not(target_os = "linux"))]
fn statvfs_bytes(_path: &Path) -> Option<(u64, u64, u64)> {
    None
}

fn read_host_mounts(mountinfo_path: &Path, paths: &ResolvedStoragePaths) -> Vec<HostMount> {
    let Ok(mountinfo_text) = std::fs::read_to_string(mountinfo_path) else { return Vec::new() };
    let mut targets: Vec<(&str, PathBuf)> = vec![("root", PathBuf::from("/"))];
    if let Some(ds) = &paths.data_store {
        if let Some(existing) = existing_ancestor(ds) {
            targets.push(("data_store", existing));
        }
    }
    if let Some(games) = &paths.games {
        if games.exists() {
            targets.push(("games", games.clone()));
        }
    }
    collect_mounts(&mountinfo_text, &targets, &statvfs_bytes)
}

fn collect_mounts(mountinfo_text: &str, targets: &[(&str, PathBuf)], stat_fn: &dyn Fn(&Path) -> Option<(u64, u64, u64)>) -> Vec<HostMount> {
    let entries = parse_mountinfo(mountinfo_text);
    let mut out: Vec<HostMount> = Vec::new();
    let mut by_key: HashMap<(String, String), usize> = HashMap::new();
    for (role, path) in targets {
        let Some(entry) = find_mount(&entries, path) else { continue };
        let key = (entry.mount_id.clone(), entry.mountpoint.clone());
        if let Some(&idx) = by_key.get(&key) {
            if !out[idx].roles.iter().any(|r| r == role) {
                out[idx].roles.push((*role).to_string());
            }
            continue;
        }
        let Some((total_bytes, used_bytes, free_bytes)) = stat_fn(Path::new(&entry.mountpoint)).or_else(|| stat_fn(path)) else { continue };
        let idx = out.len();
        by_key.insert(key, idx);
        out.push(HostMount {
            mountpoint: entry.mountpoint.clone(),
            fs_type: entry.fs_type.clone(),
            roles: vec![(*role).to_string()],
            total_bytes,
            used_bytes,
            free_bytes,
        });
    }
    out
}

fn parse_proc_stat_ticks(text: &str) -> Option<u64> {
    let rparen = text.rfind(')')?;
    let rest: Vec<&str> = text[rparen + 1..].split_whitespace().collect();
    let utime = rest.get(11)?.parse::<u64>().ok()?;
    let stime = rest.get(12)?.parse::<u64>().ok()?;
    Some(utime.saturating_add(stime))
}

fn read_process_sample(self_dir: &Path) -> ProcessSample {
    let proc_ticks = std::fs::read_to_string(self_dir.join("stat")).ok().and_then(|s| parse_proc_stat_ticks(&s)).unwrap_or(0);
    let mut threads = HashMap::new();
    if let Ok(entries) = std::fs::read_dir(self_dir.join("task")) {
        for entry in entries.flatten() {
            let Ok(tid) = entry.file_name().to_string_lossy().parse::<u32>() else { continue };
            if let Some(ticks) = std::fs::read_to_string(entry.path().join("stat")).ok().and_then(|s| parse_proc_stat_ticks(&s)) {
                threads.insert(tid, ticks);
            }
        }
    }
    ProcessSample { proc_ticks, threads }
}

fn classify_thread_group(comm: &str, is_main: bool) -> &'static str {
    let lower = comm.to_ascii_lowercase();
    if ["render", "vulkan", "serenity", "qsg", "rhi", "mesa", "amdgpu", "radeon", "vk", "presenter"].iter().any(|k| lower.contains(k)) {
        "render"
    } else if ["audio", "endymion", "pipewire", "pw-", "pulse", "alsa", "snd"].iter().any(|k| lower.contains(k)) {
        "audio"
    } else if ["scrape", "hash", "save", "library", "download", "rewind", "tool", "chd", "galaxia", "telemetry", "identify"].iter().any(|k| lower.contains(k)) {
        "jobs"
    } else if ["core", "emulate", "marsrt", "sapphire", "diamond", "venus", "moon", "mercury", "nephrite", "endou", "compiler", "crystal"].iter().any(|k| lower.contains(k)) {
        "core"
    } else if is_main || ["ui", "qml", "qt", "wayland", "pads"].iter().any(|k| lower.contains(k)) {
        "ui"
    } else {
        "other"
    }
}

fn parse_smaps_rollup(text: &str) -> Option<ProcessMemory> {
    let mut rss = None;
    let mut pss = None;
    let mut priv_clean = 0u64;
    let mut priv_dirty = 0u64;
    let mut has_priv = false;
    let mut swap = None;
    for line in text.lines() {
        let Some((key, rest)) = line.split_once(':') else { continue };
        let Some(kb) = rest.split_whitespace().next().and_then(|n| n.parse::<u64>().ok()) else { continue };
        let bytes = kb.saturating_mul(1024);
        match key.trim() {
            "Rss" => rss = Some(bytes),
            "Pss" => pss = Some(bytes),
            "Private_Clean" => {
                priv_clean = bytes;
                has_priv = true;
            }
            "Private_Dirty" => {
                priv_dirty = bytes;
                has_priv = true;
            }
            "Swap" => swap = Some(bytes),
            _ => {}
        }
    }
    rss.map(|rss_bytes| ProcessMemory { rss_bytes, pss_bytes: pss, private_bytes: has_priv.then_some(priv_clean + priv_dirty), swap_bytes: swap })
}

fn parse_proc_status_mem(text: &str) -> Option<ProcessMemory> {
    let mut rss = None;
    let mut swap = None;
    for line in text.lines() {
        let Some((key, rest)) = line.split_once(':') else { continue };
        let Some(kb) = rest.split_whitespace().next().and_then(|n| n.parse::<u64>().ok()) else { continue };
        match key.trim() {
            "VmRSS" => rss = Some(kb.saturating_mul(1024)),
            "VmSwap" => swap = Some(kb.saturating_mul(1024)),
            _ => {}
        }
    }
    rss.map(|rss_bytes| ProcessMemory { rss_bytes, pss_bytes: None, private_bytes: None, swap_bytes: swap })
}

fn parse_size_unit(val: &str) -> Option<u64> {
    let mut parts = val.split_whitespace();
    let num = parts.next()?.parse::<u64>().ok()?;
    let mult = match parts.next() {
        None | Some("B") => 1,
        Some("KiB") => 1024,
        Some("MiB") => 1024 * 1024,
        Some("GiB") => 1024 * 1024 * 1024,
        Some("KB") => 1000,
        Some("MB") => 1_000_000,
        Some("GB") => 1_000_000_000,
        Some(_) => return None,
    };
    Some(num.saturating_mul(mult))
}

fn parse_ns_unit(val: &str) -> Option<u64> {
    let mut parts = val.split_whitespace();
    let num = parts.next()?.parse::<u64>().ok()?;
    match parts.next() {
        None | Some("ns") => Some(num),
        _ => None,
    }
}

struct ParsedFdinfo {
    client_key: String,
    vram_bytes: Option<u64>,
    gtt_bytes: Option<u64>,
    ticks: DrmClientTicks,
}

fn parse_drm_fdinfo(text: &str, fallback_key: &str) -> Option<ParsedFdinfo> {
    let mut driver = None::<String>;
    let mut client_id = None::<String>;
    let mut pdev = None::<String>;
    let mut vram_bytes = None;
    let mut gtt_bytes = None;
    let mut ticks = DrmClientTicks::default();
    let mut is_drm = false;

    for line in text.lines() {
        let Some((k, v)) = line.split_once(':') else { continue };
        let (key, val) = (k.trim(), v.trim());
        match key {
            "drm-driver" => {
                is_drm = true;
                driver = Some(val.to_string());
            }
            "drm-client-id" => {
                is_drm = true;
                client_id = Some(val.to_string());
            }
            "drm-pdev" => {
                is_drm = true;
                pdev = Some(val.to_string());
            }
            "drm-memory-vram" => vram_bytes = parse_size_unit(val),
            "drm-memory-gtt" => gtt_bytes = parse_size_unit(val),
            "drm-engine-gfx" | "drm-engine-render" => ticks.gfx_ns = parse_ns_unit(val),
            "drm-engine-compute" => ticks.compute_ns = parse_ns_unit(val),
            "drm-engine-copy" => ticks.copy_ns = parse_ns_unit(val),
            _ => {}
        }
    }
    if !is_drm {
        return None;
    }
    let client_key = match (driver.as_deref(), client_id.as_deref()) {
        (Some(d), Some(id)) => format!("{d}:{id}"),
        (None, Some(id)) => format!("drm:{id}"),
        _ => format!("{}:{}:{fallback_key}", driver.as_deref().unwrap_or("drm"), pdev.as_deref().unwrap_or("")),
    };
    Some(ParsedFdinfo { client_key, vram_bytes, gtt_bytes, ticks })
}

struct DrmRawSnapshot {
    sample: DrmSample,
    vram_bytes: Option<u64>,
    gtt_bytes: Option<u64>,
    clients: u32,
}

fn read_drm_fdinfo_raw(self_dir: &Path) -> DrmRawSnapshot {
    let Ok(entries) = std::fs::read_dir(self_dir.join("fd")) else {
        return DrmRawSnapshot { sample: DrmSample::default(), vram_bytes: None, gtt_bytes: None, clients: 0 };
    };
    let mut seen = BTreeSet::new();
    let mut sample = DrmSample::default();
    let mut vram_sum = None::<u64>;
    let mut gtt_sum = None::<u64>;
    for entry in entries.flatten() {
        let fd_name = entry.file_name().to_string_lossy().into_owned();
        let Ok(target) = std::fs::read_link(entry.path()) else { continue };
        let target_str = target.to_string_lossy();
        if !target_str.starts_with("/dev/dri/renderD") && !target_str.starts_with("/dev/dri/card") {
            continue;
        }
        let Ok(info_text) = std::fs::read_to_string(self_dir.join("fdinfo").join(&fd_name)) else { continue };
        let Some(parsed) = parse_drm_fdinfo(&info_text, &fd_name) else { continue };
        if !seen.insert(parsed.client_key.clone()) {
            continue;
        }
        if let Some(v) = parsed.vram_bytes {
            vram_sum = Some(vram_sum.unwrap_or(0).saturating_add(v));
        }
        if let Some(g) = parsed.gtt_bytes {
            gtt_sum = Some(gtt_sum.unwrap_or(0).saturating_add(g));
        }
        sample.clients.insert(parsed.client_key, parsed.ticks);
    }
    DrmRawSnapshot { clients: seen.len() as u32, sample, vram_bytes: vram_sum, gtt_bytes: gtt_sum }
}

fn read_drm_fdinfo_sample(self_dir: &Path) -> DrmSample {
    read_drm_fdinfo_raw(self_dir).sample
}

fn engine_pct(prev: Option<&DrmSample>, curr: &DrmSample, wall_ns: u64, pick: impl Fn(&DrmClientTicks) -> Option<u64>) -> Option<f64> {
    let prev = prev?;
    if wall_ns == 0 {
        return None;
    }
    let mut delta_ns = 0u64;
    let mut any = false;
    for (k, c_ticks) in &curr.clients {
        if let (Some(p_val), Some(c_val)) = (prev.clients.get(k).and_then(&pick), pick(c_ticks)) {
            delta_ns = delta_ns.saturating_add(c_val.saturating_sub(p_val));
            any = true;
        }
    }
    any.then(|| ((delta_ns as f64 / wall_ns as f64) * 100.0).max(0.0))
}

#[allow(clippy::too_many_arguments)]
fn build_process_telemetry(
    proc_root: &Path,
    logical_cores: u32,
    host_delta: Option<u64>,
    wall_ns: u64,
    prev_proc: Option<&ProcessSample>,
    curr_proc: &ProcessSample,
    prev_drm: Option<&DrmSample>,
    _curr_drm: &DrmSample,
    storage: StorageFootprint,
) -> ProcessTelemetry {
    let self_dir = proc_root.join("self");
    let pid = std::fs::read_to_string(self_dir.join("stat"))
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<u32>().ok())
        .unwrap_or_else(std::process::id);

    let cpu_pcts = |prev_ticks: Option<u64>, curr_ticks: u64| -> (Option<f64>, Option<f64>) {
        match (prev_ticks, host_delta) {
            (Some(p), Some(hd)) if hd > 0 => {
                let d = curr_ticks.saturating_sub(p);
                let host = ((d as f64 / hd as f64) * 100.0).clamp(0.0, 100.0);
                let one_core = (host * f64::from(logical_cores.max(1))).clamp(0.0, 100.0);
                (Some(host), Some(one_core))
            }
            _ => (None, None),
        }
    };

    let (cpu_total_host_pct, cpu_one_core_pct) = cpu_pcts(prev_proc.map(|p| p.proc_ticks), curr_proc.proc_ticks);

    let mut groups: BTreeMap<String, ThreadGroupUsage> = ["core", "audio", "render", "ui", "jobs", "other"]
        .into_iter()
        .map(|g| (g.to_string(), ThreadGroupUsage::default()))
        .collect();
    let mut threads = Vec::new();
    let mut sorted_tids: Vec<(u32, u64)> = curr_proc.threads.iter().map(|(&t, &ticks)| (t, ticks)).collect();
    sorted_tids.sort_by_key(|(t, _)| *t);

    for (tid, ticks) in sorted_tids {
        let comm = std::fs::read_to_string(self_dir.join(format!("task/{tid}/comm"))).map(|s| s.trim().to_string()).unwrap_or_default();
        let group = classify_thread_group(&comm, tid == pid);
        let (t_host, t_core) = cpu_pcts(prev_proc.and_then(|p| p.threads.get(&tid).copied()), ticks);
        if let Some(entry) = groups.get_mut(group) {
            entry.count += 1;
            if let Some(v) = t_host {
                entry.cpu_total_host_pct = Some(entry.cpu_total_host_pct.unwrap_or(0.0) + v);
            }
            if let Some(v) = t_core {
                entry.cpu_one_core_pct = Some(entry.cpu_one_core_pct.unwrap_or(0.0) + v);
            }
        }
        threads.push(ThreadInfo { tid, comm, group: group.to_string(), cpu_one_core_pct: t_core, cpu_total_host_pct: t_host });
    }

    let max_thread_core = threads.iter().filter_map(|t| t.cpu_one_core_pct).fold(0.0f64, f64::max);
    let single_core_bound = (max_thread_core >= 85.0 || cpu_one_core_pct.unwrap_or(0.0) >= 85.0) && cpu_total_host_pct.unwrap_or(0.0) < 50.0;

    let memory = std::fs::read_to_string(self_dir.join("smaps_rollup"))
        .ok()
        .and_then(|s| parse_smaps_rollup(&s))
        .or_else(|| std::fs::read_to_string(self_dir.join("status")).ok().and_then(|s| parse_proc_status_mem(&s)));

    let drm_raw = read_drm_fdinfo_raw(&self_dir);
    let gpu = ProcessGpuUsage {
        available: drm_raw.clients > 0,
        gfx_pct: engine_pct(prev_drm, &drm_raw.sample, wall_ns, |t| t.gfx_ns),
        compute_pct: engine_pct(prev_drm, &drm_raw.sample, wall_ns, |t| t.compute_ns),
        copy_pct: engine_pct(prev_drm, &drm_raw.sample, wall_ns, |t| t.copy_ns),
        vram_bytes: drm_raw.vram_bytes,
        gtt_bytes: drm_raw.gtt_bytes,
        clients: drm_raw.clients,
    };

    ProcessTelemetry { pid, cpu_total_host_pct, cpu_one_core_pct, single_core_bound, threads, thread_groups: groups, memory, gpu, storage }
}

fn dir_bytes(root: Option<&Path>, skip: Option<&Path>) -> u64 {
    let Some(root) = root else { return 0 };
    if !root.exists() {
        return 0;
    }
    let skip_canonical = skip.and_then(|s| std::fs::canonicalize(s).ok());
    let mut total = 0u64;
    let walker = WalkDir::new(root).follow_links(false).into_iter().filter_entry(|entry| {
        if let (Some(skip_raw), Some(skip_canon)) = (skip, skip_canonical.as_ref()) {
            if entry.path() == skip_raw {
                return false;
            }
            if entry.file_type().is_dir() && std::fs::canonicalize(entry.path()).ok().as_ref() == Some(skip_canon) {
                return false;
            }
        }
        true
    });
    for entry in walker.flatten() {
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                total = total.saturating_add(meta.len());
            }
        }
    }
    total
}

fn compute_storage_footprint(paths: &ResolvedStoragePaths) -> StorageFootprint {
    let saves = dir_bytes(paths.saves.as_deref(), paths.save_states.as_deref());
    let save_states = dir_bytes(paths.save_states.as_deref(), None);
    let packages = dir_bytes(paths.packages.as_deref(), None);
    let media = dir_bytes(paths.media.as_deref(), None);
    let artwork = dir_bytes(paths.artwork.as_deref(), None);
    let screenshots = dir_bytes(paths.screenshots.as_deref(), None);
    let shaders = dir_bytes(paths.shaders.as_deref(), None);
    let themes = dir_bytes(paths.themes.as_deref(), None);
    let cores = dir_bytes(paths.cores.as_deref(), None);
    let firmware = dir_bytes(paths.firmware.as_deref(), None);
    let logs = dir_bytes(paths.logs.as_deref(), None);
    let unpacked = dir_bytes(paths.unpacked.as_deref(), None);
    let compressed = dir_bytes(paths.compressed.as_deref(), None);
    let writable_total_bytes = saves
        .saturating_add(save_states)
        .saturating_add(packages)
        .saturating_add(media)
        .saturating_add(artwork)
        .saturating_add(screenshots)
        .saturating_add(shaders)
        .saturating_add(themes)
        .saturating_add(cores)
        .saturating_add(firmware)
        .saturating_add(logs)
        .saturating_add(unpacked)
        .saturating_add(compressed);
    let games_library_bytes = paths.games.as_deref().map(|g| dir_bytes(Some(g), None));
    StorageFootprint {
        pending: false,
        stale: false,
        categories: StorageCategories { saves, save_states, packages, media, artwork, screenshots, shaders, themes, cores, firmware, logs, unpacked, compressed },
        writable_total_bytes,
        games_library_bytes,
    }
}

fn spawn_storage_refresh(paths: ResolvedStoragePaths) {
    let _ = std::thread::Builder::new().name("galaxia-storage-walk".into()).spawn(move || {
        let footprint = compute_storage_footprint(&paths);
        let mut cache = storage_lock().lock().unwrap_or_else(|p| p.into_inner());
        if cache.key.as_ref() == Some(&paths) {
            cache.footprint = Some(footprint);
            cache.updated_at = Some(Instant::now());
        }
        cache.in_flight = false;
    });
}

fn resolve_storage_footprint(paths: &ResolvedStoragePaths, force_refresh: bool, async_only: bool) -> StorageFootprint {
    let mut cache = storage_lock().lock().unwrap_or_else(|p| p.into_inner());
    let same_key = cache.key.as_ref() == Some(paths);
    if !same_key {
        cache.key = Some(paths.clone());
        cache.footprint = None;
        cache.updated_at = None;
        cache.in_flight = false;
    }
    let fresh = same_key && cache.updated_at.is_some_and(|t| t.elapsed() < STORAGE_CACHE_TTL);
    if !force_refresh && fresh {
        if let Some(fp) = &cache.footprint {
            return fp.clone();
        }
    }
    if async_only || (!force_refresh && cache.footprint.is_some()) {
        if !cache.in_flight {
            cache.in_flight = true;
            spawn_storage_refresh(paths.clone());
        }
        if let Some(mut fp) = cache.footprint.clone() {
            fp.stale = true;
            return fp;
        }
        return StorageFootprint { pending: true, ..StorageFootprint::default() };
    }
    drop(cache);
    let footprint = compute_storage_footprint(paths);
    let mut cache = storage_lock().lock().unwrap_or_else(|p| p.into_inner());
    cache.key = Some(paths.clone());
    cache.footprint = Some(footprint.clone());
    cache.updated_at = Some(Instant::now());
    cache.in_flight = false;
    footprint
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    #[test]
    fn parses_cpuinfo_and_two_sample_proc_stat_deltas() {
        let cpuinfo = "\
processor\t: 0
model name\t: AMD Ryzen 7 7700X 8-Core Processor
cpu MHz\t\t: 4500.000
physical id\t: 0
core id\t\t: 0
cpu cores\t: 2

processor\t: 1
model name\t: AMD Ryzen 7 7700X 8-Core Processor
cpu MHz\t\t: 4600.000
physical id\t: 0
core id\t\t: 0
cpu cores\t: 2

processor\t: 2
model name\t: AMD Ryzen 7 7700X 8-Core Processor
cpu MHz\t\t: 4700.000
physical id\t: 0
core id\t\t: 1
cpu cores\t: 2
";
        let parsed = parse_cpuinfo(cpuinfo);
        assert_eq!(parsed.model.as_deref(), Some("AMD Ryzen 7 7700X 8-Core Processor"));
        assert_eq!(parsed.physical_cores, 2);
        assert_eq!(parsed.logical_cores, 3);
        assert_eq!(parsed.mhz, vec![Some(4500.0), Some(4600.0), Some(4700.0)]);

        let s1 = parse_proc_stat_text("cpu  100 0 50 840 10 0 0 0\ncpu0 50 0 25 420 5 0 0 0\ncpu1 50 0 25 420 5 0 0 0\n").unwrap();
        let s2 = parse_proc_stat_text("cpu  200 0 100 1020 30 0 0 0\ncpu0 130 0 45 430 15 0 0 0\ncpu1 70 0 55 590 15 0 0 0\n").unwrap();
        let (busy, iowait) = cpu_pct_delta(&s1.aggregate, &s2.aggregate).unwrap();
        assert!((busy - (150.0 / 350.0) * 100.0).abs() < 0.01);
        assert!((iowait - (20.0 / 350.0) * 100.0).abs() < 0.01);
        let (c0_busy, _) = cpu_pct_delta(&s1.cores[0].1, &s2.cores[0].1).unwrap();
        assert!((c0_busy - (100.0 / 120.0) * 100.0).abs() < 0.01);
    }

    #[test]
    fn parses_meminfo_and_smaps_with_status_fallback() {
        let mem = "MemTotal: 32000000 kB\nMemFree: 1000000 kB\nMemAvailable: 20000000 kB\nBuffers: 50000 kB\nCached: 5000000 kB\nSwapTotal: 8000000 kB\nSwapFree: 6000000 kB\n";
        let hm = parse_meminfo(mem).unwrap();
        assert_eq!(hm.total_bytes, 32_000_000 * 1024);
        assert_eq!(hm.available_bytes, 20_000_000 * 1024);
        assert_eq!(hm.used_bytes, 12_000_000 * 1024);
        assert_eq!(hm.swap_used_bytes, 2_000_000 * 1024);

        let smaps = "Rss: 2000 kB\nPss: 1500 kB\nPrivate_Clean: 200 kB\nPrivate_Dirty: 800 kB\nSwap: 100 kB\n";
        let pm = parse_smaps_rollup(smaps).unwrap();
        assert_eq!(pm.rss_bytes, 2000 * 1024);
        assert_eq!(pm.pss_bytes, Some(1500 * 1024));
        assert_eq!(pm.private_bytes, Some(1000 * 1024));
        assert_eq!(pm.swap_bytes, Some(100 * 1024));

        let fallback = parse_proc_status_mem("Name: emusen\nVmRSS: 4096 kB\nVmSwap: 0 kB\n").unwrap();
        assert_eq!(fallback, ProcessMemory { rss_bytes: 4096 * 1024, pss_bytes: None, private_bytes: None, swap_bytes: Some(0) });
    }

    #[test]
    fn enumerates_multi_gpu_sysfs_honestly_and_handles_missing_nodes() {
        let tmp = TempDir::new("drm-sysfs");
        let drm = tmp.path();
        assert_eq!(read_drm_gpus(drm, None, None, None), (None, Vec::new()));

        let c0 = drm.join("card0/device");
        std::fs::create_dir_all(&c0).unwrap();
        std::fs::write(c0.join("uevent"), "DRIVER=amdgpu\nPCI_SLOT_NAME=0000:0f:00.0\n").unwrap();
        std::fs::write(c0.join("vendor"), "0x1002\n").unwrap();
        std::fs::write(c0.join("device"), "0x164e\n").unwrap();
        std::fs::write(c0.join("boot_vga"), "0\n").unwrap();
        std::fs::write(c0.join("mem_info_vram_total"), "536870912\n").unwrap();
        std::fs::create_dir_all(drm.join("card0-DP-1")).unwrap();

        let c1 = drm.join("card1/device/hwmon/hwmon2");
        std::fs::create_dir_all(&c1).unwrap();
        let c1_dev = drm.join("card1/device");
        std::fs::write(c1_dev.join("uevent"), "DRIVER=amdgpu\nPCI_SLOT_NAME=0000:03:00.0\n").unwrap();
        std::fs::write(c1_dev.join("vendor"), "0x1002\n").unwrap();
        std::fs::write(c1_dev.join("device"), "0x73bf\n").unwrap();
        std::fs::write(c1_dev.join("boot_vga"), "1\n").unwrap();
        std::fs::write(c1_dev.join("gpu_busy_percent"), "24\n").unwrap();
        std::fs::write(c1_dev.join("mem_info_vram_total"), "17163091968\n").unwrap();
        std::fs::write(c1_dev.join("mem_info_vram_used"), "104857600\n").unwrap();
        std::fs::write(c1.join("temp1_input"), "52500\n").unwrap();

        let (primary, all) = read_drm_gpus(drm, None, None, None);
        assert_eq!(all.len(), 2);
        let p = primary.unwrap();
        assert_eq!(p.card, "card1");
        assert_eq!(p.busy_pct, Some(24.0));
        assert_eq!(p.mem_busy_pct, None);
        assert_eq!(p.temp_c, Some(52.5));
        assert_eq!(all[0].busy_pct, None);
        assert_eq!(all[0].temp_c, None);
    }

    #[test]
    fn parses_mountinfo_deduplicates_mounts_and_parses_fdinfo() {
        let mountinfo = "\
46 1 0:38 / / rw,relatime - btrfs /dev/nvme0n1p3 rw
99 46 0:55 / /mnt/nas\\040roms rw - nfs 10.1.1.5:/roms rw
";
        let targets = vec![
            ("root", PathBuf::from("/")),
            ("data_store", PathBuf::from("/home/red")),
            ("games", PathBuf::from("/mnt/nas roms/snes")),
        ];
        let mounts = collect_mounts(mountinfo, &targets, &|p| match p.to_str() {
            Some("/") => Some((1000, 400, 600)),
            Some("/mnt/nas roms") => Some((5000, 2000, 3000)),
            _ => None,
        });
        assert_eq!(mounts.len(), 2);
        assert_eq!(mounts[0].roles, vec!["root", "data_store"]);
        assert_eq!(mounts[1].mountpoint, "/mnt/nas roms");
        assert_eq!(mounts[1].fs_type, "nfs");

        let fd_a = "drm-driver:\tamdgpu\ndrm-client-id:\t42\ndrm-memory-vram:\t1024 KiB\ndrm-memory-gtt:\t512 KiB\ndrm-engine-gfx:\t10000000 ns\n";
        let parsed = parse_drm_fdinfo(fd_a, "3").unwrap();
        assert_eq!(parsed.client_key, "amdgpu:42");
        assert_eq!(parsed.vram_bytes, Some(1024 * 1024));
        assert_eq!(parsed.gtt_bytes, Some(512 * 1024));
        assert_eq!(parsed.ticks.gfx_ns, Some(10_000_000));
    }

    #[test]
    fn storage_breakdown_separates_writable_from_games_without_double_counting_save_states() {
        let tmp = TempDir::new("storage-walk");
        let ds = tmp.join("home");
        let games = tmp.join("roms");
        std::fs::create_dir_all(ds.join("Saves/Save States")).unwrap();
        std::fs::create_dir_all(ds.join("Media")).unwrap();
        std::fs::create_dir_all(ds.join("Shaders")).unwrap();
        std::fs::create_dir_all(&games).unwrap();
        std::fs::write(ds.join("Saves/zelda.srm"), vec![0u8; 100]).unwrap();
        std::fs::write(ds.join("Saves/Save States/zelda.state"), vec![0u8; 250]).unwrap();
        std::fs::write(ds.join("Media/cover.png"), vec![0u8; 400]).unwrap();
        std::fs::write(ds.join("Shaders/crt.slang"), vec![0u8; 50]).unwrap();
        std::fs::write(games.join("zelda.sfc"), vec![0u8; 1024]).unwrap();

        let paths = TelemetryPaths {
            data_store: Some(ds.to_string_lossy().into_owned()),
            games: Some(games.to_string_lossy().into_owned()),
            refresh_storage: true,
            ..TelemetryPaths::default()
        };
        let resolved = ResolvedStoragePaths::from_input(&paths);
        let fp = resolve_storage_footprint(&resolved, true, false);
        assert_eq!(fp.categories.saves, 100);
        assert_eq!(fp.categories.save_states, 250);
        assert_eq!(fp.categories.media, 400);
        assert_eq!(fp.categories.shaders, 50);
        assert_eq!(fp.writable_total_bytes, 800);
        assert_eq!(fp.games_library_bytes, Some(1024));
        assert!(!fp.pending);
    }

    #[test]
    fn sampler_synthesizes_process_and_thread_cpu_groups_from_synthetic_proc_sys() {
        let tmp = TempDir::new("proc-sys-synth");
        let proc_root = tmp.join("proc");
        let sys_root = tmp.join("sys");
        std::fs::create_dir_all(proc_root.join("self/task/100")).unwrap();
        std::fs::create_dir_all(proc_root.join("self/task/101")).unwrap();
        std::fs::create_dir_all(proc_root.join("self/task/102")).unwrap();
        std::fs::create_dir_all(proc_root.join("self/fd")).unwrap();
        std::fs::create_dir_all(sys_root.join("devices/system/cpu/cpu0/cpufreq")).unwrap();
        std::fs::create_dir_all(sys_root.join("devices/system/cpu/cpu1/cpufreq")).unwrap();
        std::fs::write(sys_root.join("devices/system/cpu/cpu0/cpufreq/scaling_cur_freq"), "5000000\n").unwrap();
        std::fs::write(sys_root.join("devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq"), "5500000\n").unwrap();
        std::fs::write(sys_root.join("devices/system/cpu/cpu1/cpufreq/scaling_cur_freq"), "4800000\n").unwrap();
        std::fs::write(sys_root.join("devices/system/cpu/cpu1/cpufreq/cpuinfo_max_freq"), "5500000\n").unwrap();

        std::fs::write(proc_root.join("cpuinfo"), "processor : 0\nmodel name : Test CPU\nphysical id : 0\ncore id : 0\n\nprocessor : 1\nmodel name : Test CPU\nphysical id : 0\ncore id : 1\n").unwrap();
        std::fs::write(proc_root.join("meminfo"), "MemTotal: 1000 kB\nMemAvailable: 400 kB\nSwapTotal: 200 kB\nSwapFree: 150 kB\n").unwrap();
        std::fs::write(proc_root.join("self/smaps_rollup"), "Rss: 500 kB\nPss: 400 kB\nPrivate_Dirty: 300 kB\nSwap: 0 kB\n").unwrap();
        std::fs::write(proc_root.join("self/task/100/comm"), "emusen-qt\n").unwrap();
        std::fs::write(proc_root.join("self/task/101/comm"), "crystal-core\n").unwrap();
        std::fs::write(proc_root.join("self/task/102/comm"), "endymion-audio\n").unwrap();

        let write_sample = |host_b: u64, host_i: u64, t100: u64, t101: u64, t102: u64| {
            std::fs::write(proc_root.join("stat"), format!("cpu  {host_b} 0 0 {host_i} 0 0 0 0\ncpu0 {host_b} 0 0 {host_i} 0 0 0 0\ncpu1 0 0 0 0 0 0 0 0\n")).unwrap();
            let tot = t100 + t101 + t102;
            std::fs::write(proc_root.join("self/stat"), format!("100 (emusen) R 1 1 1 0 -1 0 0 0 0 0 {tot} 0 0 0 20 0 3 0 0 0 0 0")).unwrap();
            std::fs::write(proc_root.join("self/task/100/stat"), format!("100 (emusen-qt) R 1 1 1 0 -1 0 0 0 0 0 {t100} 0 0 0 20 0 3 0 0 0 0 0")).unwrap();
            std::fs::write(proc_root.join("self/task/101/stat"), format!("101 (crystal-core) R 1 1 1 0 -1 0 0 0 0 0 {t101} 0 0 0 20 0 3 0 0 0 0 0")).unwrap();
            std::fs::write(proc_root.join("self/task/102/stat"), format!("102 (endymion-audio) R 1 1 1 0 -1 0 0 0 0 0 {t102} 0 0 0 20 0 3 0 0 0 0 0")).unwrap();
        };

        let ds = tmp.join("home");
        std::fs::create_dir_all(&ds).unwrap();
        let paths = TelemetryPaths { data_store: Some(ds.to_string_lossy().into_owned()), ..TelemetryPaths::default() };
        let mut local = SamplerState::default();
        write_sample(100, 100, 10, 10, 5);
        let _ = sample_into_state(&mut local, &paths, &proc_root, &sys_root);
        write_sample(200, 200, 12, 100, 7);
        let snap = sample_into_state(&mut local, &paths, &proc_root, &sys_root);

        assert_eq!(snap.host.cpu.logical_cores, 2);
        assert!((snap.host.cpu.avg_mhz.unwrap() - 4900.0).abs() < 0.01);
        assert!((snap.process.cpu_total_host_pct.unwrap() - 47.0).abs() < 0.01);
        assert!((snap.process.cpu_one_core_pct.unwrap() - 94.0).abs() < 0.01);
        assert!(snap.process.single_core_bound);
        let core_group = snap.process.thread_groups.get("core").unwrap();
        assert_eq!(core_group.count, 1);
        assert!((core_group.cpu_one_core_pct.unwrap() - 90.0).abs() < 0.01);
        assert_eq!(snap.process.thread_groups.get("audio").unwrap().count, 1);
        assert_eq!(snap.process.thread_groups.get("ui").unwrap().count, 1);
    }

    #[test]
    fn engine_pct_dedups_drm_clients_and_async_storage_populates_cache() {
        let mut prev = DrmSample::default();
        prev.clients.insert("amdgpu:7".into(), DrmClientTicks { gfx_ns: Some(10_000_000), compute_ns: Some(0), copy_ns: None });
        let mut curr = DrmSample::default();
        curr.clients.insert("amdgpu:7".into(), DrmClientTicks { gfx_ns: Some(35_000_000), compute_ns: Some(5_000_000), copy_ns: None });
        let gfx = engine_pct(Some(&prev), &curr, 100_000_000, |t| t.gfx_ns).unwrap();
        let cmp = engine_pct(Some(&prev), &curr, 100_000_000, |t| t.compute_ns).unwrap();
        let cpy = engine_pct(Some(&prev), &curr, 100_000_000, |t| t.copy_ns);
        assert!((gfx - 25.0).abs() < 0.01);
        assert!((cmp - 5.0).abs() < 0.01);
        assert_eq!(cpy, None);

        let tmp = TempDir::new("async-storage");
        let ds = tmp.join("ds");
        std::fs::create_dir_all(ds.join("Logs")).unwrap();
        std::fs::write(ds.join("Logs/a.log"), vec![0u8; 333]).unwrap();
        let paths = ResolvedStoragePaths::from_input(&TelemetryPaths { data_store: Some(ds.to_string_lossy().into_owned()), async_storage: true, ..TelemetryPaths::default() });
        let first = resolve_storage_footprint(&paths, false, true);
        if first.pending {
            for _ in 0..50 {
                std::thread::sleep(Duration::from_millis(10));
                let next = resolve_storage_footprint(&paths, false, true);
                if !next.pending {
                    assert_eq!(next.categories.logs, 333);
                    return;
                }
            }
            panic!("async storage walk did not finish in time");
        }
        assert_eq!(first.categories.logs, 333);
    }
}
