//! The corpus form (EmuSen_CoreAPI.md §28): the cases whose answer does not depend on the image run once, on one
//! image, and the cases that do run for every image, several images at a time on the machine's processors.

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use emusen_native::core::jsonw::Json;

use crate::api::Lib;
use crate::progress;
use crate::{Case, Options, Report, cases_json, core_c11, core_cases, image_cases};

/// The ids of the cases run once for the core, and of those run for every image; C4 is in both, by its parts.
pub const CORE_CASES: [&str; 13] = ["C1", "C2", "C3", "C4", "C5", "C9", "C10", "C11", "C12", "C13", "C15", "C22", "C23"];
pub const IMAGE_CASES: [&str; 12] = ["C4", "C6", "C7", "C8", "C14", "C16", "C17", "C18", "C19", "C20", "C21", "C24"];

/// How long one image's cases may run no frame before the image is reported as hung and its thread left (§28.10).
pub const IMAGE_DEADLINE_SECONDS: u64 = 600;

/// The memory one image's cases are allowed for beside the image itself, when the images run at once are fitted to the
/// memory there is (EmuSen_CoreAPI_Extensions.md §7).
pub const JOB_MEMORY: u64 = 256 << 20;

/// The machines an image's cases hold at once: C8's three (`b`, `d` and `e`), each with its copy of the image.
pub const MACHINES_AT_ONCE: u64 = 3;

/// The copies of an image its cases still hold at once where the image is lent (§28.20): C17's own, which it
/// overwrites, the core's copy of it, and the core's copy of the disc inserted beside it until the insert returns.
pub const LENT_COPIES: u64 = 3;

/// The copies of the reference image the core's cases hold at once where the image is lent: C12's host of 1.0, which
/// cannot lend, with C7's two machines.
pub const CORE_LENT_COPIES: u64 = 2;

/// What one image's cases are estimated to take: the base, the kit's own copy of the image and each machine's, or
/// `factor` copies in all where the author names one for a core that expands its image. An image lent is mapped, and
/// only the copies the cases must make count (`LENT_COPIES`).
pub fn estimate(len: u64, factor: Option<u64>, lends: bool) -> u64 {
    JOB_MEMORY + factor.unwrap_or(if lends { LENT_COPIES } else { MACHINES_AT_ONCE + 1 }) * len
}

/// Whether the library at `path` claims `BORROW`, so that the kit maps its images and lends them to every case.
pub fn lends(path: &Path) -> bool {
    Lib::open(path).is_ok_and(|lib| lib.claims(emusen_native::core::sys::caps::BORROW))
}

/// The images admitted at once by their estimates: an image waits until its estimate fits beside the running ones, and
/// one larger than the whole allowance runs alone. The largest sum admitted is kept for the summary.
pub struct Budget {
    allowed: u64,
    state: Mutex<(u64, usize, u64)>,
    freed: Condvar,
}

impl Budget {
    pub fn new(allowed: u64) -> Budget {
        Budget { allowed, state: Mutex::new((0, 0, 0)), freed: Condvar::new() }
    }

    /// Waits for room for `estimate`; whether the image runs alone because it is larger than the allowance.
    pub fn admit(&self, estimate: u64) -> bool {
        let mut st = self.state.lock().unwrap();
        while st.1 > 0 && st.0 + estimate > self.allowed {
            st = self.freed.wait(st).unwrap();
        }
        st.0 += estimate;
        st.1 += 1;
        st.2 = st.2.max(st.0);
        estimate > self.allowed
    }

    pub fn release(&self, estimate: u64) {
        let mut st = self.state.lock().unwrap();
        st.0 -= estimate;
        st.1 -= 1;
        self.freed.notify_all();
    }

    /// The largest sum of estimates admitted at once.
    pub fn largest(&self) -> u64 {
        self.state.lock().unwrap().2
    }
}

/// This process's peak resident memory, `VmHWM`, where the system reports it.
pub fn peak_resident() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status").ok()?.lines().find_map(|l| l.strip_prefix("VmHWM:")?.trim().strip_suffix("kB")?.trim().parse::<u64>().ok()).map(|kb| kb * 1024)
}

/// One image's cases, and how long they took.
pub struct ImageResult {
    pub name: String,
    pub report: Report,
    pub seconds: f64,
}

impl ImageResult {
    pub fn failed(&self) -> Vec<&'static str> {
        self.report.cases.iter().filter(|c| !c.passed).map(|c| c.id).collect()
    }
}

/// A corpus run: the core's cases once, on `reference`, and each image's.
pub struct Corpus {
    pub core: Report,
    pub reference: String,
    pub images: Vec<ImageResult>,
    pub jobs: usize,
    pub seconds: f64,
    /// The allowance the images were admitted against, the largest sum of estimates admitted at once, and the
    /// process's peak resident memory, so that the estimate is checked by every run.
    pub allowed: u64,
    pub largest_admitted: u64,
    pub peak_resident: Option<u64>,
}

impl Corpus {
    pub fn passed(&self) -> bool {
        self.core.passed() && !self.images.is_empty() && self.images.iter().all(|i| i.report.passed())
    }

    /// The summary: the core's cases in full, and each image with its verdict, the cases it failed and its report's file.
    pub fn json(&self, reports: &[String]) -> String {
        let mut j = Json::new();
        j.begin_object().field_str("kit", concat!("emusen-core-conform ", env!("CARGO_PKG_VERSION"))).field_str("library", &self.core.library);
        j.field_str("abi", &self.core.abi).field_str("id", &self.core.id).field_str("version", &self.core.version).field_uint("frames", self.core.frames);
        j.field_uint("jobs", self.jobs as u64).field_uint("milliseconds", (self.seconds * 1000.0) as u64).field_bool("passed", self.passed());
        j.key("memory").begin_object().field_uint("allowed", self.allowed).field_uint("largest_admitted", self.largest_admitted);
        if let Some(peak) = self.peak_resident {
            j.field_uint("peak_resident", peak);
        }
        j.end_object();
        j.key("core").begin_object().field_str("image", &self.reference).field_bool("passed", self.core.passed()).key("cases");
        cases_json(&mut j, &self.core.cases);
        j.end_object();
        let passing = self.images.iter().filter(|i| i.report.passed()).count();
        j.key("counts").begin_object().field_uint("images", self.images.len() as u64).field_uint("passed", passing as u64).field_uint("failed", (self.images.len() - passing) as u64).end_object();
        j.key("images").begin_array();
        for (n, i) in self.images.iter().enumerate() {
            j.begin_object().field_str("image", &i.name).field_bool("passed", i.report.passed()).field_strs("failed", &i.failed());
            j.field_uint("milliseconds", (i.seconds * 1000.0) as u64);
            if let Some(file) = reports.get(n) {
                j.field_str("report", file);
            }
            j.end_object();
        }
        j.end_array().end_object();
        j.finish()
    }
}

/// A list's line without its comment: a `#` begins one at the line's start or after two spaces, so that a name may hold one.
fn uncommented(line: &str) -> &str {
    let end = if line.starts_with('#') { Some(0) } else { line.find("  #").or(line.find("\t#")) };
    line[..end.unwrap_or(line.len())].trim()
}

/// The images of a corpus: every file of a directory, by name, or the names a list file gives, one a line, each found
/// in `from` or beside the list.
pub fn image_list(images: &Path, from: Option<&Path>) -> Result<Vec<(String, PathBuf)>, String> {
    let name_of = |p: &Path| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    if images.is_dir() {
        let mut found: Vec<(String, PathBuf)> = std::fs::read_dir(images)
            .map_err(|e| format!("{}: {e}", images.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_file())
            .map(|p| (name_of(&p), p))
            .collect();
        found.sort();
        return Ok(found);
    }
    let text = std::fs::read_to_string(images).map_err(|e| format!("{}: {e}", images.display()))?;
    let base = from.or(images.parent()).unwrap_or(Path::new("."));
    let (mut found, mut missing) = (Vec::new(), Vec::new());
    for line in text.lines().map(uncommented).filter(|l| !l.is_empty()) {
        let path = base.join(line);
        match path.is_file() {
            true => found.push((line.to_owned(), path)),
            false => missing.push(line.to_owned()),
        }
    }
    match missing.is_empty() {
        true => Ok(found),
        false => Err(format!("{} of the list's images are not in {}: {}", missing.len(), base.display(), missing.join(", "))),
    }
}

/// The memory this process may use: the tightest `memory.max` of its control groups where there is one, or what the
/// machine has available.
pub fn memory_allowed() -> Option<u64> {
    let group = std::fs::read_to_string("/proc/self/cgroup").ok().and_then(|t| t.lines().find_map(|l| l.strip_prefix("0::").map(str::to_owned)));
    let mut tightest: Option<u64> = None;
    if let Some(group) = group {
        let mut dir = PathBuf::from(format!("/sys/fs/cgroup{group}"));
        while dir.starts_with("/sys/fs/cgroup") {
            if let Some(max) = std::fs::read_to_string(dir.join("memory.max")).ok().and_then(|t| t.trim().parse::<u64>().ok()) {
                tightest = Some(tightest.map_or(max, |t| t.min(max)));
            }
            if !dir.pop() {
                break;
            }
        }
    }
    let available = std::fs::read_to_string("/proc/meminfo").ok().and_then(|t| t.lines().find_map(|l| l.strip_prefix("MemAvailable:")?.trim().strip_suffix("kB")?.trim().parse::<u64>().ok())).map(|kb| kb * 1024);
    match (tightest, available) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

/// The images run at once when the author names no number: one for each processor, and no more than the memory
/// allowed holds at `JOB_MEMORY` each, the core's own cases counted as one.
pub fn default_jobs() -> usize {
    let processors = std::thread::available_parallelism().map_or(1, |n| n.get());
    let by_memory = memory_allowed().map_or(usize::MAX, |m| ((m / JOB_MEMORY) as usize).saturating_sub(1));
    processors.min(by_memory).max(1)
}

/// One image's cases on a `Lib` of their own, the image and its companions read here so a corpus is never held whole.
fn one_image(library: &Path, base: &Options, name: &str, file: &Path, heading: &Report) -> (ImageResult, BTreeSet<i32>) {
    let started = Instant::now();
    let mut report = Report { library: heading.library.clone(), image: Some(name.to_owned()), abi: heading.abi.clone(), id: heading.id.clone(), version: heading.version.clone(), frames: heading.frames, cases: Vec::new() };
    // The host a case plays is the Lib's, so images run at once never share one (EmuSen_CoreAPI.md §28.16).
    let lib = match Lib::open(library) {
        Ok(lib) => lib,
        Err(why) => {
            report.cases.push(Case { id: "C1", name: "loading", passed: false, evidence: vec![why], images: Vec::new() });
            return (ImageResult { name: name.to_owned(), report, seconds: started.elapsed().as_secs_f64() }, BTreeSet::new());
        }
    };
    let statuses = match crate::media::open(file, lib.claims(emusen_native::core::sys::caps::BORROW)) {
        Ok((image, companions)) => {
            let mut files = base.files.clone();
            files.extend(companions);
            let (cases, statuses) = image_cases(&lib, library, &Options { image, files, on_disk: crate::media::on_disk(file).ok(), ..base.clone() });
            report.cases = cases;
            statuses
        }
        Err(e) => {
            report.cases.push(Case { id: "C4", name: "create and refusal", passed: false, evidence: vec![format!("the image could not be read: {e}")], images: Vec::new() });
            BTreeSet::new()
        }
    };
    (ImageResult { name: name.to_owned(), report, seconds: started.elapsed().as_secs_f64() }, statuses)
}

enum Message {
    Started(usize, Arc<AtomicU64>),
    Done(usize, ImageResult, BTreeSet<i32>),
}

/// Runs the core's cases once on `reference` (its image is `opts.image`) and the image cases on every image, at most
/// `jobs` images at a time and as many as `memory` admits by their estimates (`factor` copies of each, else
/// `estimate`'s); `progress` hears of each image as it ends. An image whose cases run no frame for `deadline` is
/// reported hung.
#[allow(clippy::too_many_arguments)]
pub fn run(library: &Path, opts: &Options, reference: &str, images: &[(String, PathBuf)], jobs: usize, memory: u64, factor: Option<u64>, deadline: Duration, progress: &mut dyn FnMut(&ImageResult)) -> Corpus {
    run_with(library, opts, opts, reference, images, jobs, memory, factor, deadline, progress)
}

/// `run`, the core's cases on `opts` (the reference image with its companions) and each image's on `base` with its own.
#[allow(clippy::too_many_arguments)]
pub fn run_with(library: &Path, opts: &Options, base: &Options, reference: &str, images: &[(String, PathBuf)], jobs: usize, memory: u64, factor: Option<u64>, deadline: Duration, progress: &mut dyn FnMut(&ImageResult)) -> Corpus {
    let started = Instant::now();
    let jobs = jobs.max(1);
    // Where images are lent, the core's cases are admitted by their own estimate, the copies C12's host of 1.0 makes;
    // elsewhere they are counted as one job, as before.
    let lent = lends(library);
    let budget = Arc::new(Budget::new(if lent { memory } else { memory.saturating_sub(JOB_MEMORY) }));
    let core_thread = {
        let (library, opts, budget) = (library.to_owned(), opts.clone(), budget.clone());
        std::thread::spawn(move || {
            let need = if lent { JOB_MEMORY + CORE_LENT_COPIES * (opts.image.len() as u64) } else { 0 };
            if lent {
                budget.admit(need);
            }
            let cases = core_cases(&library, &opts);
            if lent {
                budget.release(need);
            }
            cases
        })
    };
    // Leaked on purpose, as C4's is: it keeps the library loaded under each image's own Lib, as a host does (§6.16).
    let shared: Option<&'static Lib> = Lib::open(library).ok().map(|l| &*Box::leak(Box::new(l)));
    let heading = Arc::new(crate::heading(library, shared, opts.frames));
    let mut results: Vec<Option<ImageResult>> = images.iter().map(|_| None).collect();
    let mut statuses = BTreeSet::new();
    if let Some(lib) = shared {
        let queue = Arc::new(Mutex::new((0..images.len()).collect::<VecDeque<usize>>()));
        let images: Arc<Vec<(String, PathBuf)>> = Arc::new(images.to_vec());
        let base = Arc::new(Options { image: Default::default(), ..base.clone() });
        let (tx, rx) = mpsc::channel();
        let worker = || {
            let (queue, images, base, heading, tx, library, budget) = (queue.clone(), images.clone(), base.clone(), heading.clone(), tx.clone(), library.to_owned(), budget.clone());
            std::thread::spawn(move || {
                loop {
                    let Some(i) = queue.lock().ok().and_then(|mut q| q.pop_front()) else { break };
                    let need = estimate(crate::media::size(&images[i].1), factor, lent);
                    if budget.admit(need) {
                        eprintln!("{}: its estimate, {} MiB, is more than the {} MiB allowed, so it runs alone", images[i].0, need >> 20, budget.allowed >> 20);
                    }
                    let stamp = Arc::new(AtomicU64::new(progress::now()));
                    progress::watch(Some(stamp.clone()));
                    if tx.send(Message::Started(i, stamp)).is_err() {
                        budget.release(need);
                        break;
                    }
                    let (result, statuses) = one_image(&library, &base, &images[i].0, &images[i].1, &heading);
                    progress::watch(None);
                    budget.release(need);
                    if tx.send(Message::Done(i, result, statuses)).is_err() {
                        break;
                    }
                }
            });
        };
        for _ in 0..jobs.min(images.len()) {
            worker();
        }
        let mut running: HashMap<usize, Arc<AtomicU64>> = HashMap::new();
        let mut left = images.len();
        while left > 0 {
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(Message::Started(i, stamp)) => {
                    running.insert(i, stamp);
                }
                Ok(Message::Done(i, result, returned)) => {
                    running.remove(&i);
                    if results[i].is_none() {
                        statuses.extend(returned);
                        progress(&result);
                        results[i] = Some(result);
                        left -= 1;
                    }
                }
                Err(_) => {}
            }
            let late: Vec<usize> = running.iter().filter(|(_, stamp)| progress::idle(stamp) >= deadline.as_millis() as u64).map(|(&i, _)| i).collect();
            for i in late {
                running.remove(&i);
                let seconds = deadline.as_secs();
                let mut report = Report { image: Some(images[i].0.clone()), cases: Vec::new(), ..crate::heading(library, Some(lib), opts.frames) };
                report.cases.push(Case { id: "C6", name: "the frame", passed: false, evidence: vec![format!("the image's cases ran no frame for {seconds} s; its thread was left running")], images: Vec::new() });
                let result = ImageResult { name: images[i].0.clone(), report, seconds: seconds as f64 };
                progress(&result);
                results[i] = Some(result);
                left -= 1;
                // The hung image keeps its thread; another takes its place.
                worker();
            }
        }
    }
    let (mut core, core_statuses) = core_thread.join().unwrap_or_else(|_| {
        let mut report = crate::heading(library, None, opts.frames);
        report.cases.push(Case { id: "C1", name: "loading", passed: false, evidence: vec!["the core's cases panicked".to_owned()], images: Vec::new() });
        (report, BTreeSet::new())
    });
    statuses.extend(core_statuses);
    if core.cases.len() > 1 {
        let c11 = core_c11(library, opts, statuses);
        let at = core.cases.iter().position(|c| c.id == "C12").unwrap_or(core.cases.len());
        core.cases.insert(at, c11);
    }
    Corpus {
        core,
        reference: reference.to_owned(),
        images: results.into_iter().flatten().collect(),
        jobs: jobs.min(images.len()).max(1),
        seconds: started.elapsed().as_secs_f64(),
        allowed: budget.allowed,
        largest_admitted: budget.largest(),
        peak_resident: peak_resident(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Budget, estimate, uncommented};

    #[test]
    fn the_budget_admits_two_images_of_half_the_allowance_and_runs_a_larger_one_alone() {
        assert_eq!(estimate(700 << 20, None, false), (256 << 20) + 4 * (700 << 20));
        assert_eq!(estimate(700 << 20, None, true), (256 << 20) + 3 * (700 << 20));
        assert_eq!(estimate(10, Some(9), false), (256 << 20) + 90);
        assert_eq!(estimate(10, Some(9), true), (256 << 20) + 90);
        let b = std::sync::Arc::new(Budget::new(1000));
        assert!(!b.admit(500) && !b.admit(500));
        let waiting = {
            let b = b.clone();
            std::thread::spawn(move || b.admit(500))
        };
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(!waiting.is_finished(), "a third half waits for room");
        b.release(500);
        assert!(!waiting.join().unwrap());
        b.release(500);
        b.release(500);
        assert!(b.admit(5000), "an image larger than the allowance runs alone");
        b.release(5000);
        assert_eq!(b.largest(), 5000);
    }

    #[test]
    fn a_lists_comment_begins_at_the_lines_start_or_after_two_spaces() {
        assert_eq!(uncommented("# a heading"), "");
        assert_eq!(uncommented("Game (U) [!].bin   # why"), "Game (U) [!].bin");
        assert_eq!(uncommented("Test Program #2 (PD).bin"), "Test Program #2 (PD).bin");
        assert_eq!(uncommented("  Game.bin\t# why"), "Game.bin");
        assert_eq!(uncommented("   "), "");
    }
}
