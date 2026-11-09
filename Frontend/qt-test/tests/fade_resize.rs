// EmuSen_Kit.md §7.35: an EdgeFade whose edge stops fading inside a window's resize, hidden or shown, is drawn on with no
// fault: the edge's binding falls while Qt is still telling the fade's own listeners of its new size, and the layer it
// stops must not be deleted under them. The software backend by default; EMUSEN_QT_GPU draws with Vulkan in a compositor of
// the test's own.

use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::Harness;

const SCENE: &str = "qrc:/qt/qml/EmuSen/Scenes/qml/FadeResizeScene.qml";

extern "C" {
    fn mallopt(param: i32, value: i32) -> i32;
}

// glibc's M_PERTURB: freed memory filled with a byte, so a call into a deleted listener faults on every run, not only some.
const M_PERTURB: i32 = -6;

#[test]
fn qt_kit_7_35_an_edge_fade_survives_its_edge_turning_off_in_a_resize() {
    assert_eq!(unsafe { mallopt(M_PERTURB, 0xA5) }, 1);
    let gpu = std::env::var_os("EMUSEN_QT_GPU").is_some();
    let _kwin = gpu.then(|| {
        let kwin = Compositor::start(1280, 800);
        kwin.watch(120);
        std::env::set_var("QT_QUICK_BACKEND", "rhi");
        std::env::set_var("QSG_RHI_BACKEND", "vulkan");
        std::env::set_var("QT_VK_PHYSICAL_DEVICE_INDEX", std::env::var("EMUSEN_GPU_INDEX").unwrap_or_else(|_| "0".into()));
        kwin
    });
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 400, 300);
    scene.wait(100);
    // Hidden, as Meridian's Recent band was, then shown: each time the window grows past what the row holds and back.
    for shown in [false, true] {
        if shown {
            assert!(scene.set_property("band", "visible", "true"));
            assert!(scene.set_property("band", "opacity", "1"));
        }
        for _ in 0..2 {
            assert_eq!(scene.property("row", "fadeRight"), "true");
            scene.resize(3840, 2160);
            scene.wait(50);
            assert_eq!(scene.property("row", "fadeRight"), "false", "the row fits a window 3840 wide");
            scene.resize(400, 300);
            scene.wait(50);
        }
    }
    scene.assert_quiet();
}
