//! A console started with nothing in it (EmuSen_Qt.md §7.9), over the platform library built from this commit and the
//! client fixture's console core, a PlayStation that starts empty: the runtime lists the console as one that starts
//! empty, its session opens with no game, and the library offers it on Start Console with its engine and its firmware line.

use emusen_client_fixture::cores::sha256;
use emusen_client_fixture::{console_core, library, World};
use emusen_crystal_client::{Api, Config, Open, Runtime};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::gamelists::Gamelists;
use emusen_qt_models::library::{firmware_line, refresh_with, LibraryView};

#[test]
fn the_console_core_starts_its_console_with_nothing_in_it() {
    let world = World::new("start-console");
    let player = world.dir("Cores");
    let core = world.player_library(&player, console_core());
    let config = Config { development: false, cores: Some(Vec::new()), player_cores: Some(vec![player.to_str().unwrap().into()]) };
    let rt = Runtime::new(&Api::load(library()).unwrap(), &config).unwrap();
    rt.core_approve(&sha256(&core)).unwrap();
    let systems = rt.systems().unwrap();
    let psx = systems.iter().find(|s| s.starts_empty).unwrap_or_else(|| panic!("a console starts empty: {systems:?}"));
    let session = rt.open(&Open { console: Some(psx.console.clone()), ..Open::default() }).expect("the console starts empty");
    // Its drive empty, the session is read: no disc in it (EmuSen_Qt.md §7.9).
    let info = rt.session_info(session).expect("the console's session read");
    assert_eq!((info.console.as_str(), info.discs.index, info.game), (psx.console.as_str(), None, None));
    rt.close(session).unwrap();

    // The library offers it on Start Console, by its name with its engine and the firmware line the runtime's status
    // gives, and on its ES-DE shelf.
    let platform = Platform::load(library()).unwrap();
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &platform.model_new(MODEL_APP_SETTINGS).unwrap()).unwrap();
    let source = ClientSource::open(library().to_str().unwrap(), &config).unwrap();
    let mut view = LibraryView::empty();
    view.apply(refresh_with(&source, &Gamelists::default(), false));
    let startable = view.startable();
    assert_eq!(startable.len(), 1, "{startable:?}");
    let s = &startable[0];
    assert_eq!((s.console.as_str(), s.system.as_str()), (psx.console.as_str(), psx.id.as_str()));
    assert!(s.engine.starts_with("V1ConsoleCore"), "{}", s.engine);
    let status = rt.firmware_status(&psx.id).unwrap().map(|f| f.firmware).unwrap_or_default();
    assert!(!status.is_empty(), "the console core names a firmware file");
    assert_eq!(s.firmware, firmware_line(Ok(status)));
    assert_eq!(s.firmware, "✗ Not found: add it in Settings → Firmware");
    assert_eq!(view.startable_on_shelf(&psx.esde_system).as_ref(), Some(s));
    assert_eq!(view.startable_on_shelf("nes"), None);
}

#[test]
fn a_firmware_line_says_found_missing_or_that_it_could_not_be_said() {
    use emusen_crystal_client::shapes::FirmwareFile;
    let file = |status: &str| FirmwareFile { status: status.into(), ..FirmwareFile::default() };
    assert_eq!(firmware_line(Ok(vec![file("missing"), file("present")])), "✓ Firmware found.");
    assert_eq!(firmware_line(Ok(vec![file("missing")])), "✗ Not found: add it in Settings → Firmware");
    assert_eq!(firmware_line(Ok(vec![file("not_needed")])), "");
    assert_eq!(firmware_line(Err("the table is locked".into())), "⚠ EmuSen's runtime could not say whether you have it: the table is locked");
}
