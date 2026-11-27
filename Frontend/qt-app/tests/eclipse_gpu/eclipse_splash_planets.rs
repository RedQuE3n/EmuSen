//! Eclipse's splash with more cores than it has places for planets: at most its sixteen, none stacked, and the true count under the name, a GPU job (EmuSen_Qt.md §3.4.8, R-20261123-25).

// The modules the scene imports register through the program's crate, which a test must link.

use crate::cores_world;
#[allow(unused_imports)]
use crate::eclipse_board;
#[allow(unused_imports)]
use crate::eclipse_home_oracle;
#[allow(unused_imports)]
use crate::start_held;

use eclipse_board::read;
use emusen_client_fixture::World;
use emusen_qt_models::words;
use emusen_qt_test::compositor::Compositor;
use start_held::{started, PLAIN};
use std::collections::BTreeSet;

/// A moment every planet of the sixteen has risen and none has begun to set, in ms: the 16th rises from 11% + 15 × 9/11% of the 11 s loop and the first sets from 49% (themes/eclipse/screens/Splash.qml).
const ALL_RISEN: i64 = 4400;

#[test]
fn qt_3_4_8_past_sixteen_cores_the_splash_draws_sixteen_planets_none_stacked() {
    // A compositor larger than the window, so its decoration leaves the window 1920 by 1200 whole.
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(600);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    // A held clock is refused under a threaded render loop, which installs its own animation driver (EmuSen_Kit.md §7.36).
    std::env::set_var("QSG_RENDER_LOOP", "basic");
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    std::env::set_var("TZ", "UTC");
    std::env::set_var("SDL_AUDIO_DRIVER", "disk");
    let mut harness = emusen_qt_test::Harness::new();
    for cores in [30, 120] {
        let world = World::new(&format!("qt-app-eclipse-splash-planets-{cores}"));
        std::env::set_var("SDL_AUDIO_DISK_OUTPUT_FILE", world.root.join("heard.raw"));
        let (_, media) = eclipse_home_oracle::synthetic(&world, &mut harness);
        cores_world::install(&world, cores);
        let mut scene = started(&mut harness, (1920, 1200), &PLAIN, &media);
        assert_eq!(scene.property("themeSplash", "set"), "true", "{cores}: the start read and the sound ready");
        scene.hold_clock(Some(ALL_RISEN));
        scene.wait(100);
        let tree = scene.tree();
        // Every planet the two layers make, by its place; each place's shown one, on its layer.
        let made: BTreeSet<usize> = tree.nodes.iter().filter_map(|n| n.name.strip_prefix("splashPlanet.")?.parse().ok()).collect();
        assert_eq!(made, (0..16).collect::<BTreeSet<_>>(), "{cores} cores: the planets made");
        let shown: Vec<_> = tree.nodes.iter().filter(|n| n.name.starts_with("splashPlanet.") && n.visible && n.opacity > 0.0).collect();
        assert_eq!(shown.len(), 16, "{cores} cores: one planet shown for each place");
        let places: BTreeSet<(i64, i64)> = shown.iter().map(|n| ((n.x + n.w / 2.0).round() as i64, (n.y + n.h / 2.0).round() as i64)).collect();
        assert_eq!(places.len(), 16, "{cores} cores: no two planets share a place: {places:?}");
        assert_eq!(read(&tree, "splashCounts", "text"), words::splash_counts(6, cores), "{cores} cores: the line gives the true count");
        scene.hold_clock(None);
        drop(scene);
        drop(world);
    }
    harness.assert_quiet();
    drop(kwin);
}
