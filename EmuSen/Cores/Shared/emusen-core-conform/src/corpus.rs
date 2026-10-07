//! The corpus form (EmuSen_CoreAPI.md §28): the cases whose answer does not depend on the image run once, on one
//! image, and the cases that do run for every image, several images at a time on the machine's processors.

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use emusen_native::core::jsonw::Json;

use crate::api::Lib;
use crate::{Case, Options, Report, cases_json, core_c11, core_cases, image_cases};

/// The ids of the cases run once for the core, and of those run for every image; C4 is in both, by its parts.
pub const CORE_CASES: [&str; 11] = ["C1", "C2", "C3", "C4", "C5", "C9", "C10", "C11", "C12", "C13", "C15"];
pub const IMAGE_CASES: [&str; 5] = ["C4", "C6", "C7", "C8", "C14"];

/// How long one image's cases may take before the image is reported as hung and its thread left.
pub const IMAGE_DEADLINE_SECONDS: u64 = 600;

/// The memory one image's cases are allowed for, when the images run at once are fitted to the memory there is.
pub const JOB_MEMORY: u64 = 256 << 20;

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
fn memory_allowed() -> Option<u64> {
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

/// One image's cases on `lib`, the image read here so that a corpus is never held in memory whole.
fn one_image(lib: &Lib, library: &Path, base: &Options, name: &str, file: &Path, heading: &Report) -> (ImageResult, BTreeSet<i32>) {
    let started = Instant::now();
    let mut report = Report { library: heading.library.clone(), image: Some(name.to_owned()), abi: heading.abi.clone(), id: heading.id.clone(), version: heading.version.clone(), frames: heading.frames, cases: Vec::new() };
    let statuses = match std::fs::read(file) {
        Ok(image) => {
            let (cases, statuses) = image_cases(lib, library, &Options { image, ..base.clone() });
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
    Started(usize, Instant),
    Done(usize, ImageResult, BTreeSet<i32>),
}

/// Runs the core's cases once on `reference` (its image is `opts.image`) and the image cases on every image, `jobs`
/// images at a time; `progress` hears of each image as it ends. An image still running at `deadline` is reported hung.
pub fn run(library: &Path, opts: &Options, reference: &str, images: &[(String, PathBuf)], jobs: usize, deadline: Duration, progress: &mut dyn FnMut(&ImageResult)) -> Corpus {
    let started = Instant::now();
    let jobs = jobs.max(1);
    let core_thread = {
        let (library, opts) = (library.to_owned(), opts.clone());
        std::thread::spawn(move || core_cases(&library, &opts))
    };
    // Leaked on purpose, as C4's is: a hung image's thread keeps the library, which a host never unloads (§6.16).
    let shared: Option<&'static Lib> = Lib::open(library).ok().map(|l| &*Box::leak(Box::new(l)));
    let heading = Arc::new(crate::heading(library, shared, opts.frames));
    let mut results: Vec<Option<ImageResult>> = images.iter().map(|_| None).collect();
    let mut statuses = BTreeSet::new();
    if let Some(lib) = shared {
        let queue = Arc::new(Mutex::new((0..images.len()).collect::<VecDeque<usize>>()));
        let images: Arc<Vec<(String, PathBuf)>> = Arc::new(images.to_vec());
        let base = Arc::new(Options { image: Vec::new(), ..opts.clone() });
        let (tx, rx) = mpsc::channel();
        let worker = || {
            let (queue, images, base, heading, tx, library) = (queue.clone(), images.clone(), base.clone(), heading.clone(), tx.clone(), library.to_owned());
            std::thread::spawn(move || {
                loop {
                    let Some(i) = queue.lock().ok().and_then(|mut q| q.pop_front()) else { break };
                    if tx.send(Message::Started(i, Instant::now())).is_err() {
                        break;
                    }
                    let (result, statuses) = one_image(lib, &library, &base, &images[i].0, &images[i].1, &heading);
                    if tx.send(Message::Done(i, result, statuses)).is_err() {
                        break;
                    }
                }
            });
        };
        for _ in 0..jobs.min(images.len()) {
            worker();
        }
        let mut running: HashMap<usize, Instant> = HashMap::new();
        let mut left = images.len();
        while left > 0 {
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(Message::Started(i, at)) => {
                    running.insert(i, at);
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
            let late: Vec<usize> = running.iter().filter(|(_, at)| at.elapsed() >= deadline).map(|(&i, _)| i).collect();
            for i in late {
                running.remove(&i);
                let seconds = deadline.as_secs();
                let mut report = Report { image: Some(images[i].0.clone()), cases: Vec::new(), ..crate::heading(library, Some(lib), opts.frames) };
                report.cases.push(Case { id: "C6", name: "the frame", passed: false, evidence: vec![format!("the image's cases did not finish in {seconds} s; its thread was left running")], images: Vec::new() });
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
    Corpus { core, reference: reference.to_owned(), images: results.into_iter().flatten().collect(), jobs: jobs.min(images.len()).max(1), seconds: started.elapsed().as_secs_f64() }
}

#[cfg(test)]
mod tests {
    use super::uncommented;

    #[test]
    fn a_lists_comment_begins_at_the_lines_start_or_after_two_spaces() {
        assert_eq!(uncommented("# a heading"), "");
        assert_eq!(uncommented("Game (U) [!].bin   # why"), "Game (U) [!].bin");
        assert_eq!(uncommented("Test Program #2 (PD).bin"), "Test Program #2 (PD).bin");
        assert_eq!(uncommented("  Game.bin\t# why"), "Game.bin");
        assert_eq!(uncommented("   "), "");
    }
}
