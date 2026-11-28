//! Eclipse Schema 2 Desktop Face Vulkan RHI render & screenshot verification across all 4 Desktop screens.

use crate::common::{approved_console_core, draw_covers, sandbox, wait_for};

use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::QObject;
use cxx_qt_lib::QString;
use emusen_client_fixture::library;
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::{Harness, Scene};
use emusen_theme_host::host::qobject::ThemeHost;
use std::path::PathBuf;

fn shots_dir(fallback: &std::path::Path) -> PathBuf {
    std::env::var_os("EMUSEN_ECLIPSE_DESKTOP_SHOTS")
        .map(PathBuf::from)
        .unwrap_or_else(|| fallback.join("shots"))
}

fn click_named(scene: &mut Scene<'_>, name: &str) {
    let tree = scene.tree();
    let idx = tree
        .nodes
        .iter()
        .rposition(|n| n.name == name && n.visible && n.w > 0.0 && n.h > 0.0)
        .unwrap_or_else(|| panic!("{name} is not visible"));
    let n = &tree.nodes[idx];
    scene.click(n.x + n.w / 2.0, n.y + n.h / 2.0);
}

fn assert_non_black(scene: &mut Scene<'_>, label: &str, points: &[(f64, f64)]) {
    let colours = scene.colours_at(points);
    for (pt, c) in points.iter().zip(colours.iter()) {
        let px = c.unwrap_or_else(|| panic!("{label}: missing pixel at {pt:?}"));
        let lum = u32::from(px.r) + u32::from(px.g) + u32::from(px.b);
        assert!(lum > 0, "{label}: pixel at {pt:?} is pure black ({px:?})");
    }
}

fn save_shot(scene: &mut Scene<'_>, dir: &PathBuf, name: &str) {
    scene.wait(220);
    let path = dir.join(name);
    assert!(scene.grab(path.to_str().unwrap()), "grab failed for {}", path.display());
}

#[test]
fn qt_eclipse_desktop_face_renders_all_screens_on_vulkan() {
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");

    let world = sandbox(
        "qt-app-eclipse-desktop-gpu",
        &[
            ("SNES", "Aurora Drift (Synthetic).sfc"),
            ("SNES", "Chrono Harbor (Synthetic).sfc"),
            ("SNES", "Super Starlight (Synthetic).sfc"),
            ("SNES", "Nebula Quest (Synthetic).sfc"),
            ("GBA", "Astral Advance (Synthetic).gba"),
            ("GBA", "Eclipse Tactics (Synthetic).gba"),
            ("NES", "Orbit Runner (Synthetic).nes"),
            ("GB", "Laurel Crown (Synthetic).gb"),
        ],
    );
    let shots = shots_dir(&world.root);
    std::fs::create_dir_all(&shots).expect("create screenshot output dir");
    let cores = world.dir("Cores");
    approved_console_core(&cores);
    let media = world.dir("Media");
    let mut harness = Harness::new();
    let covers = vec![
        (media.join("snes/covers/Aurora Drift (Synthetic).png"), (300, 420), 0, "Aurora Drift".into()),
        (media.join("snes/covers/Chrono Harbor (Synthetic).png"), (300, 420), 1, "Chrono Harbor".into()),
        (media.join("snes/covers/Super Starlight (Synthetic).png"), (300, 420), 2, "Super Starlight".into()),
        (media.join("snes/covers/Nebula Quest (Synthetic).png"), (300, 420), 3, "Nebula Quest".into()),
        (media.join("gba/covers/Astral Advance (Synthetic).png"), (300, 300), 4, "Astral Advance".into()),
        (media.join("gba/covers/Eclipse Tactics (Synthetic).png"), (300, 300), 5, "Eclipse Tactics".into()),
        (media.join("nes/covers/Orbit Runner (Synthetic).png"), (300, 420), 6, "Orbit Runner".into()),
        (media.join("gb/covers/Laurel Crown (Synthetic).png"), (300, 300), 7, "Laurel Crown".into()),
    ];
    draw_covers(&mut harness, &world.root, &covers, |_| {});

    let mut scene = harness.load(MAIN_URL, 1440, 900);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "library", "loading", "false", 15000);
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    assert!(scene.set_property("program", "bigScreen", "false"));
    wait_for(&mut scene, "eclipseDesktopLibrary", "visible", "true", 3000);

    // 1. Desktop Library (Constellation rail, grid, inspector, top ribbon, status bar).
    assert_non_black(&mut scene, "Library", &[(120.0, 240.0), (560.0, 320.0), (1260.0, 360.0), (200.0, 26.0)]);
    save_shot(&mut scene, &shots, "01-eclipse-desktop-library.png");

    // 2. Desktop Table + multi-select floating batch bar.
    click_named(&mut scene, "eclipseDesktopViewTable");
    wait_for(&mut scene, "eclipseDesktopTable", "visible", "true", 2000);
    click_named(&mut scene, "eclipseDesktopTableSelectAll");
    wait_for(&mut scene, "eclipseDesktopTableBatchBar", "visible", "true", 2000);
    assert_non_black(&mut scene, "Table", &[(480.0, 240.0), (720.0, 840.0)]);
    save_shot(&mut scene, &shots, "02-eclipse-desktop-table-batch.png");

    // 3. Desktop Playing: closed auto-hiding strip & translucent slide-over drawer over pure #000000 viewport frame.
    let theme_host_obj = scene.object("host.theme");
    let host_ref = unsafe { <ThemeHost as Upcast<QObject>>::from_base_ptr(theme_host_obj as *const QObject) } as *mut ThemeHost;
    let theme_engine = unsafe { host_ref.as_ref() }.expect("host").engine();
    let mut actions = emusen_theme_qt::host::actions(unsafe { Pin::new_unchecked(&mut *theme_engine) }).expect("Actions");
    let screens_qobj = scene.object("screens") as *const QObject;
    let screens_obj = unsafe { <emusen_qt_models::screens::qobject::Screens as Upcast<QObject>>::from_base_ptr(screens_qobj) }
        as *mut emusen_qt_models::screens::qobject::Screens;
    unsafe { Pin::new_unchecked(&mut *screens_obj) }.play();
    wait_for(&mut scene, "eclipseDesktopPlaying", "visible", "true", 2000);
    assert!(scene.set_property("desktopModel", "drawerOpen", "false"));
    scene.wait(220);
    let center_px = scene.colours_at(&[(720.0, 480.0)])[0].expect("center pixel");
    assert_eq!((center_px.r, center_px.g, center_px.b), (0, 0, 0), "Playing viewport center stays pure #000000");
    save_shot(&mut scene, &shots, "03-eclipse-desktop-playing-closed.png");

    assert!(actions.as_mut().workspace_set_drawer(true));
    wait_for(&mut scene, "eclipsePlayingDrawer", "visible", "true", 1500);
    assert_non_black(&mut scene, "PlayingDrawer", &[(1240.0, 320.0)]);
    save_shot(&mut scene, &shots, "03b-eclipse-desktop-playing-drawer.png");
    assert!(actions.as_mut().workspace_set_drawer(false));
    click_named(&mut scene, "eclipsePlayingBackLibrary");
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    wait_for(&mut scene, "eclipseDesktopLibrary", "visible", "true", 2000);
    assert_eq!(scene.property("desktopModel", "view"), "0", "< Library switches back to Library view");
    assert_eq!(scene.property("desktopModel", "gameShown"), "false", "< Library exits Playing view");

    // 4. Desktop Settings + Themes gallery.
    click_named(&mut scene, "eclipseDesktopNavSettings");
    wait_for(&mut scene, "eclipseDesktopSettings", "visible", "true", 2000);
    assert_non_black(&mut scene, "Settings", &[(140.0, 240.0), (540.0, 280.0), (1240.0, 280.0)]);
    save_shot(&mut scene, &shots, "04-eclipse-desktop-settings.png");

    assert!(actions.as_mut().open_desktop_door(&QString::from("themes")));
    wait_for(&mut scene, "eclipseDesktopThemesRows", "visible", "true", 2000);
    assert_non_black(&mut scene, "Themes", &[(540.0, 280.0)]);
    save_shot(&mut scene, &shots, "05-eclipse-desktop-themes.png");
    scene.assert_quiet();
}
