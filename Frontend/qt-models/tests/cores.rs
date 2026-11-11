//! Cores as a tool's source, over the world Mistress's recordings script (tests/cores/world.json, from
//! WindowFitAuditTests.CoresWindow.cs): the rows, their status words and tones, the facts, the pane's sentences, A's
//! action in the help bar, the marks and the plan, a job's progress pulled, and the approval asked and recorded
//! (EmuSen_Interface.md §15.2–15.6; EmuSen_Qt.md §7.7).

use emusen_crystal_client::shapes::{CoreRow, FirmwareFile, SettingRow, SystemRow};
use emusen_qt_models::tools::cores::backend::{Backend, Job, Scripted};
use emusen_qt_models::tools::cores::{Cores, Host};
use emusen_qt_models::tools::{Source, Tone, Tool, Words};
use serde_json::Value;
use std::sync::{Arc, Mutex};

const WORLD: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cores/world.json");
const NO_CATALOGUE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cores/no-catalogue.json");

/// The scripted world, with the runtime's approvals and revocations recorded.
struct Recorded {
    world: Scripted,
    approved: Arc<Mutex<Vec<String>>>,
    started: Arc<Mutex<Vec<Value>>>,
}

impl Backend for Recorded {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        self.world.systems()
    }
    fn cores(&self) -> Result<Vec<CoreRow>, String> {
        self.world.cores()
    }
    fn setting(&self, console: &str, key: &str) -> Option<String> {
        self.world.setting(console, key)
    }
    fn set_setting(&self, console: &str, key: &str, value: &str) -> Result<(), String> {
        self.world.set_setting(console, key, value)
    }
    fn schema(&self, console: &str, engine: &str) -> Vec<SettingRow> {
        self.world.schema(console, engine)
    }
    fn firmware_folder(&self) -> Option<String> {
        self.world.firmware_folder()
    }
    fn approve(&self, sha256: &str) -> Result<(), String> {
        self.approved.lock().unwrap().push(sha256.to_owned());
        Ok(())
    }
    fn revoke(&self, sha256: &str) -> Result<(), String> {
        self.world.revoke(sha256)
    }
    fn firmware(&self, system: &str) -> Option<Vec<FirmwareFile>> {
        self.world.firmware(system)
    }
    fn rescan(&self) {}
    fn start(&self, request: &Value) -> Result<Box<dyn Job>, String> {
        self.started.lock().unwrap().push(request.clone());
        self.world.start(request)
    }
    fn space(&self) -> Option<(u64, u64)> {
        self.world.space()
    }
    fn size(&self, library: &str) -> Option<u64> {
        self.world.size(library)
    }
}

fn opened(world: &str) -> (Tool<Cores>, Arc<Mutex<Vec<String>>>) {
    let (tool, approved, _) = opened_recording(world);
    (tool, approved)
}

fn opened_recording(world: &str) -> (Tool<Cores>, Arc<Mutex<Vec<String>>>, Arc<Mutex<Vec<Value>>>) {
    let approved = Arc::new(Mutex::new(Vec::new()));
    let started = Arc::new(Mutex::new(Vec::new()));
    let backend = Recorded { world: Scripted::open(world).unwrap(), approved: approved.clone(), started: started.clone() };
    let mut tool = Tool::new(Cores::new(Box::new(backend), Host::default()));
    tool.refresh();
    for _ in 0..8 {
        tool.poll();
    }
    assert!(!tool.source.running(), "the listing and the catalogue's fetch end");
    (tool, approved, started)
}

fn cells(tool: &Tool<Cores>) -> Vec<Vec<String>> {
    tool.rows().iter().map(|i| i.cells.iter().map(|c| c.words.clone()).collect()).collect()
}

fn words(said: &[Words]) -> Vec<&str> {
    said.iter().map(|w| w.words.as_str()).collect()
}

fn help(tool: &Tool<Cores>) -> Vec<String> {
    tool.help().into_iter().map(|e| e.1).collect()
}

fn focus_on(tool: &mut Tool<Cores>, name: &str) {
    let at = tool.shown().iter().position(|i| i.cells[0].words == name).unwrap_or_else(|| panic!("{name} is shown"));
    tool.set_focus(at);
}

#[test]
fn the_listing_joins_the_runtime_s_cores_and_the_tool_s_and_says_each_status() {
    let (mut tool, _) = opened(WORLD);
    let facts: Vec<(String, Tone)> = tool.source.facts().into_iter().map(|f| (f.words, f.tone)).collect();
    assert_eq!(
        facts,
        [
            ("9 cores".to_owned(), Tone::Muted),
            ("7 installed".to_owned(), Tone::Muted),
            ("⚠ 1 to approve".to_owned(), Tone::Attention),
            ("↑ 1 update".to_owned(), Tone::Muted),
            ("catalogue read 31 Oct 2026".to_owned(), Tone::Muted),
        ]
    );
    let sections: Vec<String> = tool.source.sections().into_iter().map(|s| s.1).collect();
    assert_eq!(sections, ["All", "Super Nintendo", "PlayStation", "PlayStation 2"], "by maker, then by year");
    let all: Vec<String> = cells(&tool).into_iter().map(|c| c[0].clone()).collect();
    assert_eq!(all, ["VenusRT", "Snes9x", "Diamond", "Beetle PSX", "Only GL", "PCEE2", "Sapphire", "Extra PS2 Core", "LRPS2"]);
    tool.choose_section("PlayStation 2");
    let rows = tool.rows();
    let shown: Vec<(&str, &str, &str, Tone)> = rows.iter().map(|i| (i.cells[0].words.as_str(), i.cells[1].words.as_str(), i.cells[2].words.as_str(), i.cells[2].tone)).collect();
    assert_eq!(
        shown,
        [
            ("PCEE2", "PlayStation 2", "● Default", Tone::Text),
            ("Sapphire", "PlayStation 2", "Built in", Tone::Muted),
            ("Extra PS2 Core", "PlayStation 2", "⚠ To approve", Tone::Attention),
            ("LRPS2", "PlayStation 2", "Available", Tone::Muted),
        ]
    );
    let bar = tool.bar();
    assert_eq!((bar.line.words.as_str(), bar.meter_words.as_str()), ("Nothing marked", "/Cores 51 MB · 410 GB free"));
    assert!((bar.meter - (1.0 - 410.0 / 1024.0)).abs() < 1e-9);
    assert!(bar.progress.is_none());
}

#[test]
fn the_pane_says_the_core_in_sentences_and_a_names_the_row_s_action() {
    let (mut tool, _) = opened(WORLD);
    tool.choose_section("PlayStation 2");
    focus_on(&mut tool, "LRPS2");
    let pane = tool.pane();
    assert_eq!(pane.title, "LRPS2");
    assert_eq!(pane.caption, "From libretro's buildbot · PlayStation 2 · listed there as “Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)”");
    assert_eq!(
        words(&pane.sentences),
        [
            "Runs PlayStation 2 games through EmuSen's libretro host, in a sandbox of its own.",
            "By PCSX2 Team, libretro, under the GPLv3. Nightly build of 31 October 2026.",
            "Needs a PlayStation 2 BIOS of your own, which EmuSen never downloads.",
            "Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it.",
            "Not tested by EmuSen.",
        ]
    );
    assert_eq!(help(&tool), ["Install…", "Back", "Console", "More", "List", "Apply…"]);

    tool.choose_section("All");
    focus_on(&mut tool, "Extra PS2 Core");
    let pane = tool.pane();
    assert_eq!(pane.caption, "Added by you · PlayStation 2");
    assert_eq!(
        words(&pane.sentences),
        ["Runs PlayStation 2 games, from a file of your own.", "By Someone Else, under the MIT.", "⚠ Waiting for your approval. Nothing downloaded runs until you approve it."]
    );
    assert_eq!(pane.sentences[2].tone, Tone::Attention);
    assert_eq!(help(&tool)[0], "Approve…");

    // A project core that is not its console's engine is made one by A; the default has no A with no options to open.
    focus_on(&mut tool, "Sapphire");
    assert_eq!(help(&tool)[0], "Make Default");
    assert!(words(&tool.pane().sentences).contains(&"Not the PlayStation 2's engine: PCEE2 is. A makes Sapphire the engine instead."));
    tool.activate(tool.focus().unwrap());
    assert_eq!(tool.source.cores().iter().find(|c| c.name == "Sapphire").map(|c| c.is_default), Some(true));
    assert_eq!(tool.source.cores().iter().find(|c| c.name == "PCEE2").map(|c| c.is_default), Some(false));
    focus_on(&mut tool, "VenusRT");
    assert_eq!(help(&tool), ["Options", "Back", "Console", "More", "List", "Apply…"], "the default's A is its options");
}

#[test]
fn marks_make_the_plan_and_its_install_runs_with_its_steps_ticked() {
    let (mut tool, _) = opened(WORLD);
    tool.choose_section("PlayStation 2");
    focus_on(&mut tool, "LRPS2");
    // X's page marks the row; its status says the mark in Accent.
    tool.more();
    let (title, breadcrumb, rows, _) = tool.card().unwrap();
    assert_eq!((title.as_str(), breadcrumb.as_str()), ("LRPS2", "Cores"));
    let labels: Vec<&str> = rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(labels, ["Mark to install", "Details"]);
    tool.card_choose(0);
    assert!(!tool.card_open());
    let lrps2 = tool.rows().into_iter().find(|i| i.cells[0].words == "LRPS2").unwrap();
    assert_eq!((lrps2.cells[2].words.as_str(), lrps2.cells[2].tone), ("install", Tone::Accent));
    tool.choose_section("PlayStation");
    focus_on(&mut tool, "Beetle PSX");
    tool.more();
    assert_eq!(tool.card().unwrap().2[0].label, "Mark to update");
    tool.card_choose(0);
    assert_eq!(tool.bar().line.words, "2 marked: install 1, update 1, about 8 MB to download");

    tool.apply();
    let plan = tool.question().unwrap().clone();
    assert_eq!(plan.title, "Apply 2 changes?");
    assert_eq!(
        plan.sentences,
        [
            "Download and install 1 core from buildbot.libretro.com: LRPS2.",
            "Update Beetle PSX; the build installed now is kept for Roll Back.",
            "Each core is checked before it is kept, and nothing downloaded runs until you approve it.",
        ]
    );
    assert_eq!((plan.choices.as_slice(), plan.back), (["No".to_owned(), "Yes".to_owned()].as_slice(), Some(0)));

    // Yes: the install runs and does not end; its row and the pane say where it is.
    tool.answer(1);
    tool.poll();
    assert!(tool.source.running());
    tool.choose_section("PlayStation 2");
    focus_on(&mut tool, "LRPS2");
    let lrps2 = tool.rows().into_iter().find(|i| i.cells[0].words == "LRPS2").unwrap();
    assert_eq!(lrps2.cells[2].words, "Installing…");
    let bar = tool.bar();
    assert_eq!(bar.line.words, "Installing LRPS2: 18 MB of 41 MB");
    assert!(bar.progress.is_some_and(|p| p > 0.0 && p < 0.5), "{bar:?}");
    assert_eq!(
        words(&tool.pane().sentences),
        [
            "Downloading the nightly build of 31 October 2026 from buildbot.libretro.com, 41 MB.",
            "✓ Secure connection to buildbot.libretro.com",
            "↓ Download · 18 MB of 41 MB",
            "· Checksum, one file of the right name, its size",
            "· Built for this machine",
            "· SHA-256 recorded",
            "Then it waits for your approval: nothing downloaded runs until you approve it.",
        ]
    );
    assert_eq!(help(&tool), ["Back", "Console", "List"], "nothing acts while a job runs");
    assert!(!tool.can_apply());
}

#[test]
fn the_approval_is_asked_in_the_question_form_and_recorded_through_the_runtime() {
    let (mut tool, approved) = opened(WORLD);
    focus_on(&mut tool, "Extra PS2 Core");
    tool.activate(tool.focus().unwrap());
    let q = tool.question().unwrap().clone();
    assert_eq!(q.title, "Approve Extra PS2 Core?");
    assert_eq!(q.choices, ["Not Now", "Approve"]);
    assert_eq!(q.back, Some(0), "B answers Not Now");
    assert_eq!(
        q.sentences[0],
        "Extra PS2 Core, in libextraps2.so, did not ship with EmuSen and has not been approved. By Someone Else, under the MIT; it runs PlayStation 2 games."
    );
    assert!(q.sentences[1].contains("full permissions"));
    assert_eq!(q.caption, format!("SHA-256\n{0} {0} {0} {0}\n{0} {0} {0} {0}", "44444444"));
    tool.answer(0);
    assert!(approved.lock().unwrap().is_empty(), "Not Now records nothing");
    tool.activate(tool.focus().unwrap());
    tool.answer(1);
    assert_eq!(*approved.lock().unwrap(), ["4".repeat(64)]);
    assert_eq!(tool.source.notice(), Some(format!("Approved Extra PS2 Core (SHA-256 {}).", "4".repeat(64)).as_str()));
}

#[test]
fn with_no_catalogue_the_runtime_s_cores_are_listed_and_the_fetch_s_refusal_said() {
    let (tool, _) = opened(NO_CATALOGUE);
    let facts: Vec<String> = tool.source.facts().into_iter().map(|f| f.words).collect();
    assert_eq!(facts, ["4 cores", "4 installed", "⚠ 1 to approve", "catalogue could not be read: buildbot.libretro.com could not be reached"]);
    let all: Vec<String> = cells(&tool).into_iter().map(|c| c[0].clone()).collect();
    assert_eq!(all, ["VenusRT", "Diamond", "Sapphire", "Extra PS2 Core"]);
    let bar = tool.bar();
    assert_eq!((bar.line.words.as_str(), bar.meter_words.as_str()), ("Buildbot.libretro.com could not be reached.", "/Cores 0 B · 410 GB free"));
}

#[test]
fn the_list_card_filters_and_sorts_and_the_details_page_holds_the_raw_facts() {
    let (mut tool, _) = opened(WORLD);
    tool.list();
    let labels: Vec<String> = tool.card().unwrap().2.into_iter().map(|r| format!("{} {}", r.label, r.value)).collect();
    assert_eq!(labels, ["Search ", "Filter All", "Sort Console", "Check for Updates ", "Update All… ", "Verify "]);
    tool.card_step(1, 4);
    assert_eq!(tool.card().unwrap().2[1].value, "To approve");
    let names: Vec<String> = cells(&tool).into_iter().map(|c| c[0].clone()).collect();
    assert_eq!(names, ["Extra PS2 Core"]);
    tool.card_step(1, -4);
    tool.card_step(2, 1);
    let names: Vec<String> = cells(&tool).into_iter().map(|c| c[0].clone()).collect();
    assert_eq!(names.first().map(String::as_str), Some("Beetle PSX"), "by name: {names:?}");
    tool.card_back();
    tool.set_search("snes9x");
    let names: Vec<String> = cells(&tool).into_iter().map(|c| c[0].clone()).collect();
    assert_eq!(names, ["Snes9x"]);
    tool.set_search("");

    focus_on(&mut tool, "LRPS2");
    tool.more();
    let details = tool.card().unwrap().2.len() - 1;
    tool.card_choose(details);
    let (title, breadcrumb, rows, _) = tool.card().unwrap();
    assert_eq!((title.as_str(), breadcrumb.as_str()), ("LRPS2 · details", "LRPS2"));
    let facts: Vec<String> = rows.iter().map(|r| format!("{}: {}", r.label, r.pane.lines[0].0)).collect();
    assert!(facts.contains(&"From: https://buildbot.libretro.com/nightly/linux/x86_64/latest/pcsx2_libretro.so.zip, index CRC-32 3a224a58".to_owned()), "{facts:?}");
    assert!(facts.contains(&"Firmware: PlayStation 2 BIOS at pcsx2/bios/ps2.bin, required".to_owned()), "{facts:?}");
    tool.card_back();
    tool.card_back();
    assert!(!tool.card_open());
}

#[test]
fn the_list_card_s_search_docks_the_keyboard_and_its_words_narrow_the_list() {
    let (mut tool, _) = opened(WORLD);
    tool.list();
    tool.card_choose(0);
    assert!(tool.searching() && !tool.card_open(), "Search closes the card and docks the keyboard");
    tool.set_search("gl");
    let names: Vec<String> = cells(&tool).into_iter().map(|c| c[0].clone()).collect();
    assert_eq!(names, ["Only GL"]);
    tool.end_search();
    assert!(!tool.searching());
    assert_eq!(tool.search(), "gl", "the words are kept when the keyboard closes");
}

#[test]
fn the_search_s_shoulders_move_the_focus_between_the_keys_and_the_rows_as_the_games_search_s_do() {
    use emusen_qt_models::screens::{search_shoulder, TO_GRID};
    // R-20261111-31: one rule for both searches; from the keys toGrid, from the results either shoulder's action.
    assert_eq!(search_shoulder(true, TO_GRID), Some(false));
    assert_eq!((search_shoulder(false, "previous"), search_shoulder(false, "next")), (Some(true), Some(true)));
    assert_eq!((search_shoulder(true, "next"), search_shoulder(false, TO_GRID)), (None, None));
    let (mut tool, _) = opened(WORLD);
    assert!(!tool.shoulder(TO_GRID), "no search, no move");
    tool.list();
    tool.card_choose(0);
    assert!(tool.search_keys(), "the search opens on the keys");
    assert!(!tool.action_lines().contains("PageUp"), "the keys' shoulders are the keyboard's help's");
    assert!(tool.shoulder(TO_GRID) && !tool.search_keys() && tool.searching(), "RB: the rows, the keyboard still docked");
    assert!(tool.action_lines().contains("previous PageUp\nnext PageDown\n"), "{}", tool.action_lines());
    let section = tool.section_key();
    assert!(tool.shoulder("previous") && tool.search_keys(), "LB: the keys again");
    assert_eq!(tool.section_key(), section, "the section kept");
    tool.focus_keys(false);
    assert!(!tool.search_keys(), "the pointer on a row");
    tool.end_search();
    assert!(!tool.search_keys() && !tool.action_lines().contains("PageUp"));
    let keys: Vec<(String, String)> = emusen_qt_models::help::entries(emusen_qt_models::help::Screen::ToolSearch)
        .iter()
        .chain(emusen_qt_models::help::unnamed(emusen_qt_models::help::Screen::ToolSearch))
        .filter_map(|(b, _, k)| k.map(|k| (format!("{b:?}"), k.to_owned())))
        .filter(|(b, _)| b.starts_with("Page"))
        .collect();
    assert_eq!(keys, [("PageUp".to_owned(), TO_GRID.to_owned()), ("PageDown".to_owned(), TO_GRID.to_owned())]);
}

#[test]
fn options_are_the_engine_s_settings_stepped_and_stored_for_its_console() {
    let (mut tool, _) = opened(WORLD);
    focus_on(&mut tool, "PCEE2");
    assert_eq!(help(&tool)[0], "Options");
    tool.activate(tool.focus().unwrap());
    let (title, breadcrumb, rows, _) = tool.card().unwrap();
    assert_eq!((title.as_str(), breadcrumb.as_str()), ("PCEE2 · Options", "Cores"));
    let shown: Vec<(String, String, bool)> = rows.iter().map(|r| (r.label.clone(), r.value.clone(), r.steps)).collect();
    assert_eq!(
        shown,
        [
            ("Renderer".to_owned(), "Vulkan".to_owned(), true),
            ("Widescreen patches".to_owned(), "Off".to_owned(), true),
            ("Upscale".to_owned(), "1".to_owned(), true),
        ]
    );
    assert_eq!(rows[0].pane.lines[0].0, "How the picture is drawn.");
    tool.card_step(2, 1);
    assert_eq!(tool.card().unwrap().2[2].value, "2", "a count steps within its range");
    tool.card_choose(1);
    assert_eq!(tool.card().unwrap().2[1].value, "On", "A turns a switch");
    assert!(tool.card_open(), "a stepped option keeps the page");
    // A choice that hides another: Software has no upscale.
    tool.card_step(0, 1);
    let labels: Vec<String> = tool.card().unwrap().2.into_iter().map(|r| format!("{} {}", r.label, r.value)).collect();
    assert_eq!(labels, ["Renderer Software", "Widescreen patches On"]);
    tool.card_back();
    tool.more();
    let page: Vec<String> = tool.card().unwrap().2.into_iter().map(|r| r.label).collect();
    assert!(page.contains(&"Options".to_owned()), "{page:?}");
}

#[test]
fn firmware_lists_the_console_s_files_found_or_not_and_where_they_go() {
    let (mut tool, _) = opened(WORLD);
    tool.choose_section("PlayStation");
    focus_on(&mut tool, "Diamond");
    let said = words(&tool.pane().sentences).iter().map(|w| w.to_string()).collect::<Vec<_>>();
    assert!(said.contains(&"Can use a PlayStation BIOS of your own, which EmuSen never downloads; it runs without one.".to_owned()), "{said:?}");
    assert!(said.contains(&"✓ You have one.".to_owned()), "{said:?}");
    tool.more();
    let page: Vec<String> = tool.card().unwrap().2.into_iter().map(|r| r.label).collect();
    assert_eq!(page, ["Options", "Firmware", "Details"]);
    tool.card_choose(1);
    let (title, _, rows, _) = tool.card().unwrap();
    assert_eq!(title, "Diamond · Firmware");
    // One row a need, worded as Settings' Firmware page words it (firmware.rs), short on a card's row, the copy in the pane.
    assert_eq!((rows[0].label.as_str(), rows[0].value.as_str()), ("PlayStation BIOS", "✓ Found"));
    assert_eq!(rows[0].pane.lines[0].0, "Your own: SCPH5501");
    let open = rows.last().unwrap();
    assert_eq!((open.key.as_str(), open.label.as_str()), ("folder:/home/player/EmuSen/Firmware", "Open Folder"), "the program's one Open Folder");
    assert_eq!(rows[0].pane.lines[1], ("scph5501.bin · Optional".to_owned(), "Diamond runs without it, on its open replacement.".to_owned()));
    assert_eq!(rows[0].pane.lines[2].0, "Put it in /home/player/EmuSen/Firmware.");
    // No firmware answer for the PlayStation 2, so LRPS2's page offers none, as C#'s did with no firmware pane.
    tool.card_back();
    tool.card_back();
    tool.choose_section("PlayStation 2");
    focus_on(&mut tool, "LRPS2");
    tool.more();
    let page: Vec<String> = tool.card().unwrap().2.into_iter().map(|r| r.label).collect();
    assert_eq!(page, ["Mark to install", "Details"]);
}

#[test]
fn roll_back_asks_then_puts_the_kept_build_back_through_the_core_tool() {
    let (mut tool, _, started) = opened_recording(WORLD);
    focus_on(&mut tool, "PCEE2");
    assert!(words(&tool.pane().sentences).contains(&"The build of 2026-10-24 is kept for Roll Back."));
    tool.more();
    let rows = tool.card().unwrap().2;
    let at = rows.iter().position(|r| r.label == "Roll Back…").expect("Roll Back… where a build is kept");
    tool.card_choose(at);
    let q = tool.question().unwrap().clone();
    assert_eq!(q.title, "Roll Back PCEE2?");
    assert_eq!(q.sentences[0], "Put back the build of 24 October 2026, kept when this one was installed in its place.");
    assert_eq!((q.choices.as_slice(), q.back), (["No".to_owned(), "Yes".to_owned()].as_slice(), Some(0)));
    tool.answer(0);
    assert!(!started.lock().unwrap().iter().any(|r| r["op"] == "core-rollback"), "No runs nothing");
    tool.more();
    tool.card_choose(at);
    tool.answer(1);
    assert!(started.lock().unwrap().iter().any(|r| r["op"] == "core-rollback" && r["cores"][0] == "pcee2"));
    for _ in 0..8 {
        tool.poll();
    }
    assert_eq!(tool.source.notice(), Some("Rolled back PCEE2 to the build of 24 October 2026."));
    let pcee2 = tool.rows().into_iter().find(|i| i.cells[0].words == "PCEE2").unwrap();
    assert_eq!(pcee2.cells[2].words, "Rolled back");
}

#[test]
fn update_all_asks_the_plan_for_every_core_with_an_update_then_runs_the_core_tool_s_update() {
    // The recorded world gives Beetle PSX an update; this one gives Snes9x one too.
    let mut world: Value = serde_json::from_str(&std::fs::read_to_string(WORLD).unwrap()).unwrap();
    for e in world["jobs"]["core-ls"]["events"].as_array_mut().unwrap() {
        if e["stem"] == "snes9x" {
            e["state"] = "update".into();
        }
    }
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("cores-two-updates.json");
    std::fs::write(&path, world.to_string()).unwrap();
    let (mut tool, _, started) = opened_recording(path.to_str().unwrap());
    tool.list();
    let labels: Vec<String> = tool.card().unwrap().2.into_iter().map(|r| r.label).collect();
    assert_eq!(labels, ["Search", "Filter", "Sort", "Check for Updates", "Update All…", "Verify"]);
    tool.card_choose(4);
    let q = tool.question().unwrap().clone();
    assert_eq!(q.title, "Update 2 cores?");
    assert_eq!(
        q.sentences,
        [
            "Update Snes9x, Beetle PSX; the build installed now is kept for Roll Back.",
            "Each core is checked before it is kept, and nothing downloaded runs until you approve it.",
        ]
    );
    assert_eq!((q.choices.as_slice(), q.back), (["No".to_owned(), "Yes".to_owned()].as_slice(), Some(0)));
    tool.answer(0);
    assert!(!started.lock().unwrap().iter().any(|r| r["op"] == "core-update"), "No runs nothing");

    tool.list();
    tool.card_choose(4);
    tool.answer(1);
    let updates: Vec<Value> = started.lock().unwrap().iter().filter(|r| r["op"] == "core-update").cloned().collect();
    assert_eq!(updates, [serde_json::json!({ "op": "core-update", "cores": ["snes9x", "mednafen_psx"] })], "one job naming both");
    let bar = tool.bar();
    assert_eq!(bar.line.words, "Updating Snes9x, Beetle PSX…");
    assert_eq!(bar.progress, Some(0.0));
    let beetle = tool.rows().into_iter().find(|i| i.cells[0].words == "Beetle PSX").unwrap();
    assert_eq!(beetle.cells[2].words, "Updating…");
    assert_eq!(help(&tool), ["Back", "Console", "List"], "nothing acts while it runs");
    tool.poll();
    assert_eq!(tool.source.notice(), Some("Installed Beetle PSX. Nothing downloaded runs until you approve it."));
    for _ in 0..8 {
        tool.poll();
    }
    assert!(!tool.source.running());
}

#[test]
fn update_all_names_one_core_by_its_name_and_is_not_offered_with_none() {
    let (mut tool, _) = opened(WORLD);
    tool.list();
    let at = tool.card().unwrap().2.iter().position(|r| r.label == "Update All…").expect("offered with an update");
    tool.card_choose(at);
    let q = tool.question().unwrap().clone();
    assert_eq!(q.title, "Update Beetle PSX?");
    assert_eq!(q.sentences[0], "Update Beetle PSX; the build installed now is kept for Roll Back.");
    tool.answer(0);

    let (mut tool, _) = opened(NO_CATALOGUE);
    tool.list();
    let labels: Vec<String> = tool.card().unwrap().2.into_iter().map(|r| r.label).collect();
    assert_eq!(labels, ["Search", "Filter", "Sort", "Check for Updates", "Verify"]);
}
