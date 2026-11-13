//! The tools of jobs (tools/jobs.rs): Library Manager and Compressed Discs over the platform library built from this
//! commit and a scratch library, each job started by A, its progress and result lines read as it runs; a job that asks
//! first, and Stop, over a scripted world; and the screens that open them from Home and the Home card.

use emusen_client_fixture::{library, World};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt_models::screens::{step, step_in, tool_back, Ask, Shown};
use emusen_qt_models::tools::cores::backend::{platform_job, Backend, Scripted};
use emusen_qt_models::tools::jobs::{self, Jobs};
use emusen_qt_models::tools::{Source, Tone, Tool};
use emusen_kit_core::input::UiButton;
use std::sync::Arc;

const SHEET: &str = "FILE \"Disc.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n";

/// A scratch library holding one NES game and one generated disc, named by the settings as the ROM folder.
fn sandbox(name: &str) -> (World, Arc<Platform>) {
    let world = World::new(name);
    let nes = world.dir("Roms/nes");
    let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    image.resize(16 + 16384, 0);
    std::fs::write(nes.join("Alpha.nes"), image).unwrap();
    let psx = world.dir("Roms/psx");
    std::fs::write(psx.join("Disc.cue"), SHEET).unwrap();
    std::fs::write(psx.join("Disc.bin"), vec![0u8; 2352 * 300]).unwrap();
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    (world, platform)
}

fn tool(key: &str, platform: Arc<Platform>) -> Tool<Jobs> {
    Tool::new(Jobs::new(key, Box::new(move |r| platform_job(&platform, r))).unwrap())
}

fn index(t: &Tool<Jobs>, key: &str) -> usize {
    t.shown().iter().position(|i| i.key == key).unwrap()
}

/// Polled as the screen's timer polls, until the job ends; the bar's progress as each poll left it.
fn follow(t: &mut Tool<Jobs>) -> Vec<f64> {
    let mut seen = Vec::new();
    for _ in 0..600 {
        t.poll();
        if !t.source.running() {
            return seen;
        }
        seen.extend(t.bar().progress);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("the job did not end in 6 s");
}

#[test]
fn compressed_discs_lists_a_generated_disc_with_its_state_through_the_platform_library() {
    let (_world, platform) = sandbox("jobs-compressed");
    let mut t = tool("compressed", platform);
    assert_eq!(t.source.title(), "Compressed Discs");
    let names: Vec<String> = t.shown().iter().map(|i| i.cells[0].words.clone()).collect();
    assert_eq!(names, ["List disc games", "Convert every disc", "Check converted discs"]);
    assert!(t.help().iter().all(|e| e.0 != UiButton::Menu), "nothing is marked, so no Apply: {:?}", t.help());
    let ls = index(&t, "ls");
    t.set_focus(ls);
    assert_eq!(t.action().map(|a| a.label), Some("Start".to_owned()));
    assert_eq!(t.activate(ls), None);
    assert!(t.source.running(), "A starts the job at once");
    assert_eq!(t.bar().progress, Some(0.0), "the bar shows a job running before its first event");
    follow(&mut t);
    let run = t.source.run("ls").unwrap();
    assert_eq!((run.total, run.done), (Some(1), 1), "{run:?}");
    assert_eq!(run.lines, [emusen_qt_models::tools::Words::text("psx/Disc.cue: not converted")]);
    let outcome = run.outcome.clone().unwrap();
    assert_eq!(outcome.tone, Tone::Success, "{outcome:?}");
    assert!(outcome.words.starts_with("1 not converted; "), "{outcome:?}");
    assert_eq!(t.bar().line, outcome, "the bar says the outcome once it has ended");
    assert_eq!(t.rows()[ls].cells[1].words, "Done");
    let pane: Vec<String> = t.pane().sentences.iter().map(|s| s.words.clone()).collect();
    assert_eq!(pane[1..], ["psx/Disc.cue: not converted".to_owned(), outcome.words.clone()], "the pane: the latest line, then the outcome");
}

#[test]
fn library_manager_checks_every_dump_and_x_lists_every_line() {
    let (_world, platform) = sandbox("jobs-library");
    let mut t = tool("library", platform);
    assert_eq!(t.source.title(), "Library Manager");
    let check = index(&t, "check");
    t.activate(check);
    let progress = follow(&mut t);
    assert!(progress.iter().all(|p| (0.0..=1.0).contains(p)), "{progress:?}");
    let run = t.source.run("check").unwrap().clone();
    assert_eq!(run.total, Some(2), "every file of the library, a sheet with its track one: {run:?}");
    assert_eq!(run.done, 2);
    assert_eq!(run.lines.iter().map(|l| l.words.as_str()).collect::<Vec<_>>(), ["nes/Alpha.nes: ok", "psx/Disc.cue: ok"]);
    assert!(run.outcome.as_ref().is_some_and(|o| o.words.contains("ok")), "{run:?}");
    t.more();
    let (_, _, rows, _) = t.card().expect("X opens the job's lines");
    assert_eq!(rows.len(), run.lines.len() + 1, "every line, the outcome last");
    assert_eq!(rows.last().unwrap().label, run.outcome.unwrap().words);
}

/// A world whose convert runs until stopped, and ends with what it did when it is.
fn scripted() -> (tempdir::Dir, Tool<Jobs>) {
    let dir = tempdir::Dir::new("jobs-scripted");
    let world = dir.0.join("world.json");
    let events = serde_json::json!({ "jobs": { "convert": {
        "events": [
            { "event": "start", "op": "convert", "discs": 2, "refused": [] },
            { "event": "disc", "index": 1, "of": 2, "source": "psx/A.cue", "state": "working" },
            { "event": "disc", "index": 1, "of": 2, "source": "psx/A.cue", "state": "failed", "words": "chdman refused it" },
            { "event": "disc", "index": 2, "of": 2, "source": "psx/B.cue", "state": "working" }
        ],
        "ends": false,
        "stopped": [
            { "event": "disc", "index": 2, "of": 2, "source": "psx/B.cue", "state": "stopped" },
            { "event": "end", "op": "convert", "counts": { "failed": 1, "stopped": 1 }, "seconds": 1.5 }
        ]
    } } });
    std::fs::write(&world, events.to_string()).unwrap();
    let scripted = Scripted::open(world.to_str().unwrap()).unwrap();
    (dir, Tool::new(Jobs::new("compressed", Box::new(move |r| scripted.start(r))).unwrap()))
}

mod tempdir {
    pub struct Dir(pub std::path::PathBuf);
    impl Dir {
        pub fn new(name: &str) -> Dir {
            let at = std::env::temp_dir().join(format!("emusen-{name}-{}", std::process::id()));
            std::fs::create_dir_all(&at).unwrap();
            Dir(at)
        }
    }
    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

#[test]
fn a_job_that_writes_asks_first_and_a_running_job_is_stopped_by_a() {
    let (_dir, mut t) = scripted();
    let convert = index(&t, "convert");
    t.set_focus(convert);
    assert_eq!(t.action().map(|a| a.label), Some("Start…".to_owned()));
    t.activate(convert);
    assert_eq!(t.question().map(|q| q.title.as_str()), Some("Convert every disc?"));
    t.answer(0);
    assert!(!t.source.running(), "No starts nothing");
    t.activate(convert);
    t.answer(1);
    assert!(t.source.running(), "Yes starts it");
    t.poll();
    assert_eq!(t.bar().progress, Some(0.5), "one disc of two ended");
    assert_eq!(t.bar().line.words, "Converting: 1 of 2");
    assert_eq!(t.rows()[convert].cells[1].words, "1 of 2");
    let pane: Vec<String> = t.pane().sentences.iter().map(|s| s.words.clone()).collect();
    assert!(pane.contains(&"Converting psx/B.cue".to_owned()), "the disc it is on: {pane:?}");
    assert!(pane.contains(&"psx/A.cue: failed, chdman refused it".to_owned()), "{pane:?}");
    assert_eq!(t.action().map(|a| (a.key, a.label)), Some((jobs::STOP.to_owned(), "Stop".to_owned())), "A on the running job stops it");
    t.set_focus(index(&t, "ls"));
    assert_eq!(t.action(), None, "another row offers nothing while a job runs");
    t.activate(convert);
    t.poll();
    assert!(!t.source.running(), "it ends once stopped");
    let run = t.source.run("convert").unwrap();
    assert_eq!(run.outcome.as_ref().map(|o| (o.words.as_str(), o.tone)), Some(("1 failed, 1 stopped; 1.5 s.", Tone::Attention)));
    assert_eq!(run.lines.last().map(|l| l.tone), Some(Tone::Muted), "a stopped disc's line");
}

#[test]
fn home_and_the_home_card_open_the_tools_of_jobs_and_b_comes_back() {
    assert_eq!(jobs::tools(), ["library", "compressed", "packages"]);
    assert_eq!(step(Shown::Home, &Ask::Place(2)), Some((Shown::Tool, Some("packages".into()))));
    assert_eq!(step(Shown::Home, &Ask::Place(3)), Some((Shown::Tool, Some("library".into()))));
    assert_eq!(step(Shown::Home, &Ask::Place(4)), Some((Shown::Cores, None)));
    assert_eq!(step(Shown::Home, &Ask::Place(1)), Some((Shown::Tool, Some("scraper".into()))), "the Scraper on the same tool screen");
    assert_eq!(step(Shown::Tool, &Ask::Back), Some((Shown::Home, None)));
    for themed in [false, true] {
        let library = if themed { Shown::Esde } else { Shown::Home };
        assert_eq!(step_in(themed, Shown::HomeCard, &Ask::Act("compressed".into())), Some((Shown::Tool, Some("compressed".into()))));
        assert_eq!(tool_back(themed, Some(Shown::HomeCard), Shown::Tool, &Ask::Back), Some(library));
    }
}
