//! The questions before a game opens, over the platform library built from this commit and the test core, with no
//! network: C#'s `CoreApprovalTests` as Rust (a player's core is asked about once and approved through the runtime;
//! declined, nothing is recorded; a shipped core is never asked about), the firmware question's words, and the
//! LaunchModel through QML (EmuSen_Qt.md §7.7).

use emusen_client_fixture::cores::sha256;
use emusen_client_fixture::{library, test_core, World};
use emusen_crystal_client::shapes::{FirmwareNeed, Replacement};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_qt_models::launch::{approval_of, approval_question, firmware_answer, firmware_question, FirmwareAnswer};
use emusen_qt_test::Harness;
use std::path::{Path, PathBuf};

fn runtime(cores: &[&Path], player: &[&Path]) -> Runtime {
    let names = |d: &[&Path]| Some(d.iter().map(|p| p.to_str().unwrap().to_owned()).collect());
    Runtime::new(&Api::load(library()).unwrap(), &Config { development: false, cores: names(cores), player_cores: names(player) }).unwrap()
}

/// The test core in the player's folder, read and waiting for approval: approved once so the runtime writes its
/// description beside it, then the approval revoked, as a core the player added and never approved stands.
fn waiting_core(world: &World) -> (PathBuf, PathBuf, String) {
    let player = world.dir("Cores");
    let core = world.player_core(&player);
    let sha = sha256(&core);
    let rt = runtime(&[], &[&player]);
    rt.core_approve(&sha).unwrap();
    rt.core_revoke(&sha).unwrap();
    (player, core, sha)
}

#[test]
fn a_player_s_core_is_asked_about_once_and_approved_through_the_runtime() {
    let world = World::new("launch-approve");
    let (player, core, sha) = waiting_core(&world);
    let game = world.game("game.tst", &[1, 2, 3, 4]);
    let rt = runtime(&[], &[&player]);
    let listed = rt.cores().unwrap();
    assert!(listed.iter().any(|c| c.trust.as_deref() == Some("needs_approval")), "{listed:?}");

    let found = approval_of(&rt, game.to_str().unwrap()).expect("the core the game opens on waits");
    let q = approval_question(&found);
    assert_eq!(q.choices, ["Not Now", "Approve"]);
    assert_eq!(q.back, Some(0));
    assert!(q.caption.replace([' ', '\n'], "").contains(&sha), "{}", q.caption);
    assert!(q.sentences.iter().any(|s| s.contains("full permissions")), "{:?}", q.sentences);
    assert!(q.sentences[0].contains(core.file_name().unwrap().to_str().unwrap()), "{:?}", q.sentences);

    rt.core_approve(&found.asking.sha256).unwrap();
    assert!(approval_of(&rt, game.to_str().unwrap()).is_none(), "asked once");
    assert!(rt.firmware_needed(game.to_str().unwrap(), None).unwrap().is_empty(), "the test core needs no firmware, so no question");
}

#[test]
fn declined_nothing_is_recorded_and_a_shipped_core_is_never_asked_about() {
    let world = World::new("launch-decline");
    let (player, _, sha) = waiting_core(&world);
    let game = world.game("game.tst", &[1, 2, 3, 4]);
    let rt = runtime(&[], &[&player]);
    assert!(approval_of(&rt, game.to_str().unwrap()).is_some());
    // Not Now records nothing: a runtime made afresh still finds the core waiting.
    let again = runtime(&[], &[&player]);
    assert!(again.cores().unwrap().iter().any(|c| c.sha256.as_deref() == Some(sha.as_str()) && c.trust.as_deref() == Some("needs_approval")));

    // The installation's own test core, vouched for by its manifest, runs the .tst game with no question.
    let shipped = world.dir("Installation");
    std::fs::copy(test_core(), shipped.join(test_core().file_name().unwrap())).unwrap();
    std::fs::write(shipped.join("cores.manifest"), format!("{}  {}\n", sha256(test_core()), test_core().file_name().unwrap().to_string_lossy())).unwrap();
    let installed = runtime(&[&shipped], &[]);
    assert!(approval_of(&installed, game.to_str().unwrap()).is_none(), "{:?}", installed.cores().unwrap());
}

#[test]
fn the_firmware_question_names_the_file_and_the_folder_and_plays_without_it_first() {
    let need = FirmwareNeed { core: "Test".into(), chip: Some("DSP-1".into()), file: "dsp1.rom".into(), size: 8192, required: true, ..FirmwareNeed::default() };
    let q = firmware_question(&need, "/home/player/EmuSen/Firmware");
    assert_eq!(q.title, "DSP-1 firmware needed");
    assert_eq!(q.sentences[0], "This game asks for dsp1.rom, 8,192 bytes: a dump of your own, which EmuSen never downloads.");
    assert_eq!(q.sentences[1], "Put it in /home/player/EmuSen/Firmware, then choose Check Again. Without it, DSP-1 is not emulated.");
    assert_eq!(q.choices, ["Play Without It", "Check Again"]);
    assert_eq!(q.back, None, "B goes back without starting the game");
    assert_eq!([firmware_answer(0), firmware_answer(1), firmware_answer(-1)], [FirmwareAnswer::PlayWithout, FirmwareAnswer::CheckAgain, FirmwareAnswer::Back]);
    let replaced = FirmwareNeed { replacement: Some(Replacement { effect: Some("the open BIOS boots every disc".into()), ..Replacement::default() }), ..need };
    assert!(firmware_question(&replaced, "x").sentences[1].ends_with("Without it, EmuSen's open replacement runs in its place: the open BIOS boots every disc."));
}

#[test]
fn the_launch_model_asks_then_names_the_game_once_answered() {
    let world = World::new("launch-model");
    let (player, _, _) = waiting_core(&world);
    let game = world.game("game.tst", &[1, 2, 3, 4]);
    // The model's runtime takes the player's folder from Galaxia beside the cores folder it is named: the scratch tree's.
    let galaxia = emusen_platform_client::Platform::load(library()).unwrap().directory(emusen_platform_client::galaxia::consts::DIR_CORES).unwrap().unwrap();
    std::fs::create_dir_all(&galaxia).unwrap();
    for entry in std::fs::read_dir(&player).unwrap().flatten() {
        std::fs::copy(entry.path(), galaxia.join(entry.file_name())).unwrap();
    }
    let empty = world.dir("NoCores");
    let mut harness = Harness::new();
    let scene_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/LaunchScene.qml");
    let mut scene = harness.load(&format!("file://{scene_file}"), 640, 480);
    assert!(scene.set_property("scene", "cores", empty.to_str().unwrap()));
    assert!(scene.set_property("scene", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("scene", "game", game.to_str().unwrap()));
    let wait = |scene: &mut emusen_qt_test::Scene<'_>, property: &str, value: &str| {
        for _ in 0..100 {
            if scene.property("scene", property) == value {
                return;
            }
            scene.wait(50);
        }
        panic!("{property} is {} not {value}", scene.property("scene", property));
    };
    wait(&mut scene, "asking", "true");
    assert!(scene.property("scene", "title").starts_with("Approve "), "{}", scene.property("scene", "title"));
    assert_eq!(scene.property("scene", "readied"), "");
    assert!(scene.set_property("scene", "answer", "1"));
    wait(&mut scene, "readied", game.to_str().unwrap());
    assert!(scene.property("scene", "notice").starts_with("Approved "), "{}", scene.property("scene", "notice"));
    // Asked once: the same game again opens with no question.
    assert!(scene.set_property("scene", "readied", ""));
    assert!(scene.set_property("scene", "game", ""));
    assert!(scene.set_property("scene", "game", game.to_str().unwrap()));
    wait(&mut scene, "readied", game.to_str().unwrap());
    assert_eq!(scene.property("scene", "asking"), "false");
}
