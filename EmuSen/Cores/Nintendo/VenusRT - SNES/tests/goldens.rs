//! VenusRT's goldens, recorded from VenusRT itself and graded frame by frame through the core ABI, with the open
//! replacements in place of firmware (VenusRT_Native.md §68.2); moved from WiseMan's VenusGoldenTests on 2026-11-06.

mod common;

use std::path::{Path, PathBuf};

use common::golden::{self, Run};
use common::{grader, programs};
use emusen_goldens::Trace;

/// The synthetic programs by frames: the display on LoROM and HiROM, its effects, the sound processor, each coprocessor, rendering skipped, and two resets.
fn frame_runs() -> Vec<Run> {
    let mut runs = vec![
        Run::frames("programs/display", programs::display(), "pads", 600),
        Run::frames("programs/display-hirom", programs::hi_rom(), "pads", 600),
        Run::frames("programs/effects", programs::effects(), "pads", 600),
        Run::frames("programs/sound", programs::sound(), "pads", 600),
        Run::frames("programs/sa1", programs::sa1(), "pads", 600),
        Run::frames("programs/gsu", programs::super_fx(), "none", 600),
        Run::frames("programs/dsp1", programs::nec_dsp(), "pads", 600),
        Run::frames("programs/obc1", programs::obc1(), "pads", 600),
    ];
    let mut skipped = Run::frames("programs/effects-skipped", programs::effects(), "pads", 300);
    skipped.skip_rendering = true;
    runs.push(skipped);
    let mut resets = Run::frames("programs/sound-resets", programs::sound(), "pads", 300);
    resets.resets = vec![100, 200];
    runs.push(resets);
    runs
}

/// Random bytes as code on each board, eight seeds, 300 frames.
fn random_runs(board: &str) -> Vec<Run> {
    (0..8).map(|seed| Run::frames(&format!("random/{board}-{seed}"), programs::random(board, seed), "pads", 300)).collect()
}

/// Every corpus ROM of the manifest to the frames the grader reads it at.
fn corpus_runs() -> Vec<Run> {
    let Some(root) = grader::corpus_root() else { return Vec::new() };
    grader::unique(&root)
        .into_iter()
        .map(|rel| {
            let path = root.join(&rel);
            let rom = std::fs::read(&path).unwrap_or_default();
            let frames = *grader::frames(&path.to_string_lossy()).iter().max().unwrap();
            Run { name: format!("corpus/{}", rel.replace('\\', "/")), rom, input: "none", skip_rendering: false, frames, corpus: Some(path), resets: Vec::new() }
        })
        .collect()
}

/// The commercial games by the first sixteen digits of their SHA-256 and their size; absent, the case passes unrun.
const GAMES: [(&str, u64, &str); 6] = [
    ("D70C9C7716AD12C6", 524800, "LoROM"),
    ("740646F3535BFB36", 4194304, "SA-1"),
    ("DEABC7D04688D00F", 2097664, "GSU-2"),
    ("6495D32E95774B1C", 524800, "DSP-1B"),
    ("A031B5AB828CDAD8", 1049088, "ST010"),
    ("A81FD8F3BC36A063", 2097664, "OBC1"),
];

/// A folder of SNES images, read in place; the games are found there by size and hash.
const GAMES_VARIABLE: &str = "EMUSEN_VENUSRT_ROMS";

fn game_runs() -> Vec<Run> {
    let Some(root) = std::env::var_os(GAMES_VARIABLE).map(PathBuf::from).filter(|p| p.is_dir()) else { return Vec::new() };
    let mut files = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("sfc") || x.eq_ignore_ascii_case("smc")) {
                out.push(p);
            }
        }
    }
    walk(&root, &mut files);
    GAMES
        .iter()
        .filter_map(|(sha, size, _)| {
            files
                .iter()
                .filter(|f| f.metadata().is_ok_and(|m| m.len() == *size))
                .filter_map(|f| std::fs::read(f).ok())
                .find(|b| emusen_core_host::sha256::sha256_hex(b).to_ascii_uppercase().starts_with(sha))
                .map(|rom| Run::frames(&format!("games/{sha}"), rom, "bench", 3600))
        })
        .collect()
}

fn traces(under: &str) -> Vec<String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "trace") {
                out.push(p);
            }
        }
    }
    let root = golden::directory();
    let mut found = Vec::new();
    walk(&root.join(under), &mut found);
    found.iter().map(|p| p.strip_prefix(&root).unwrap().with_extension("").to_string_lossy().replace('\\', "/")).collect()
}

#[test]
fn each_synthetic_program_runs_as_its_golden_recorded() {
    for r in frame_runs() {
        println!("{}", golden::check(&r));
    }
}

#[test]
fn random_programs_run_as_their_goldens_recorded() {
    for board in programs::BOARDS {
        for r in random_runs(board) {
            println!("{}", golden::check(&r));
        }
    }
}

#[test]
fn the_corpus_runs_as_its_goldens_recorded() {
    let runs = corpus_runs();
    if runs.is_empty() {
        println!("{} unset, not run", grader::CORPUS_VARIABLE);
        return;
    }
    let threads = std::env::var("EMUSEN_VENUSRT_THREADS").ok().and_then(|t| t.parse().ok()).unwrap_or_else(|| (std::thread::available_parallelism().map_or(4, |n| n.get()) / 4).max(1));
    let next = std::sync::atomic::AtomicUsize::new(0);
    let failures = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(r) = runs.get(i) else { break };
                if let Err(e) = std::panic::catch_unwind(|| golden::check(r)) {
                    let words = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    failures.lock().unwrap().push(format!("{}: {words}", r.name));
                }
            });
        }
    });
    let mut failures = failures.into_inner().unwrap();
    let named: Vec<&String> = runs.iter().map(|r| &r.name).collect();
    failures.extend(traces("corpus").into_iter().filter(|t| !named.contains(&t)).map(|t| format!("{t}: recorded, not in this corpus")));
    failures.sort();
    assert!(failures.is_empty(), "{} of {} corpus runs differ from their goldens:\n{}", failures.len(), runs.len(), failures.join("\n"));
}

#[test]
fn the_games_run_as_their_goldens_recorded() {
    let runs = game_runs();
    if runs.is_empty() {
        println!("{GAMES_VARIABLE} unset or holding none of the games, not run");
    }
    for r in runs {
        println!("{}", golden::check(&r));
    }
}

/// Every ROM C# Venus passed in the frozen baseline passes on VenusRT, and none falls below the C# core's verdict (VenusRT_Native.md §68.3).
#[test]
fn no_corpus_verdict_is_worse_than_the_csharp_baseline() {
    let text = std::fs::read_to_string(golden::directory().join("VenusRtBaseline.tsv")).unwrap();
    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty() && !l.starts_with('#')).collect();
    let header: Vec<&str> = lines[0].split('\t').collect();
    let column = |n: &str| header.iter().position(|h| *h == n).unwrap();
    let (rom, venus) = (column("rom"), column("venus"));
    let rows: Vec<Vec<&str>> = lines[1..].iter().map(|l| l.split('\t').collect()).collect();
    assert_eq!(295, rows.len());
    let rank = |v: &str| match v {
        "Passed" => 3,
        "Done" | "Visual" | "Incomplete" => 1,
        "Failed" => 0,
        _ => -1,
    };
    let verdict = |r: &str| {
        let path = golden::trace_path(&format!("corpus/{r}"));
        assert!(path.exists(), "{r} is in the baseline and has no golden");
        Trace::parse(&std::fs::read_to_string(path).unwrap()).transcript["verdict"].split(' ').next().unwrap().to_string()
    };
    let worse: Vec<String> = rows.iter().filter(|c| rank(&verdict(c[rom])) < rank(c[venus])).map(|c| format!("{}: C# {}, VenusRT {}", c[rom], c[venus], verdict(c[rom]))).collect();
    assert!(worse.is_empty(), "{}", worse.join("\n"));
    let csharp = rows.iter().filter(|c| c[venus] == "Passed").count();
    let ours = rows.iter().filter(|c| verdict(c[rom]) == "Passed").count();
    assert!(ours >= csharp, "VenusRT passes {ours}, C# Venus passed {csharp}");
}

/// Every trace in the tree is a run the catalogue names, so none is left ungraded.
#[test]
fn every_synthetic_trace_belongs_to_a_named_run() {
    let mut named: Vec<String> = frame_runs().into_iter().chain(programs::BOARDS.iter().flat_map(|b| random_runs(b))).map(|r| r.name).collect();
    let mut present: Vec<String> = traces("programs").into_iter().chain(traces("random")).collect();
    named.sort();
    present.sort();
    assert_eq!(named, present);
    let games: Vec<String> = GAMES.iter().map(|g| format!("games/{}", g.0)).collect();
    assert!(traces("games").iter().all(|t| games.contains(t)));
}

/// One hash changed in frame 300 of a committed trace, in each column, fails naming the frame and the part.
#[test]
fn a_deliberately_broken_trace_is_caught() {
    let r = frame_runs().into_iter().find(|r| r.name == "programs/sound").unwrap();
    let now = golden::run(&r, None, |_, _| {});
    for (part, column) in [("state", 1), ("picture", 2), ("sound", 3)] {
        let mut broken = Trace::parse(&std::fs::read_to_string(golden::trace_path(&r.name)).unwrap());
        let cell = &mut broken.rows[299][column];
        let last = if cell.ends_with('0') { '1' } else { '0' };
        cell.pop();
        cell.push(last);
        let difference = Trace::parse(&broken.text()).first_difference(&now).unwrap();
        assert_eq!(format!("frame 300: the {part} differs"), difference.split(" (").next().unwrap());
    }
}

/// The same for a corpus chain: one digit of a chained row changed is caught at its row.
#[test]
fn a_deliberately_broken_corpus_chain_is_caught() {
    let Some(r) = corpus_runs().into_iter().find(|r| r.name.ends_with("/CPUADC.sfc")) else { return };
    let mut broken = Trace::parse(&std::fs::read_to_string(golden::trace_path(&r.name)).unwrap());
    let cell = &mut broken.rows[10][1];
    let last = if cell.ends_with('0') { '1' } else { '0' };
    cell.pop();
    cell.push(last);
    let at = broken.rows[10][0].clone();
    let difference = Trace::parse(&broken.text()).first_difference(&golden::run(&r, None, |_, _| {})).unwrap();
    assert_eq!(format!("frame {at}: the chain differs"), difference.split(" (").next().unwrap());
}
