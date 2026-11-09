// EmuSen_Kit.md §7.35: the kit's EdgeFade fades what it holds to transparency, so each faded pixel is the row's colour and the
// stripes behind it mixed by the ramp, over light and dark stripes alike, falling smoothly to the stripes' own pixels at the
// edge; the gaps between rows show the stripes, in the first frame a layer is made in as in every one after. The software backend by default; EMUSEN_QT_GPU draws with Vulkan in a
// compositor of the test's own. The grab is written to EMUSEN_QT_COMPARE (else the temporary folder) as fade-<backend>.png.

use emusen_kit_core::tokens::Colour;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::Harness;

const SCENE: &str = "qrc:/qt/qml/EmuSen/Scenes/qml/FadeScene.qml";
const ROW: [f64; 3] = [48.0, 112.0, 224.0];

fn levels(c: Option<Colour>) -> [f64; 3] {
    let c = c.expect("a pixel");
    [f64::from(c.r), f64::from(c.g), f64::from(c.b)]
}

fn off(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| (a[i] - b[i]).abs()).fold(0.0, f64::max)
}

fn mix(row: [f64; 3], behind: [f64; 3], kept: f64) -> [f64; 3] {
    [0, 1, 2].map(|i| row[i] * kept + behind[i] * (1.0 - kept))
}

#[test]
fn qt_kit_7_35_an_edge_fade_fades_to_what_lies_behind() {
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
    scene.wait(300);
    let out = std::env::var_os("EMUSEN_QT_COMPARE").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir);
    assert!(scene.grab(&out.join(format!("fade-{}.png", if gpu { "gpu" } else { "software" })).display().to_string()));

    // The stripes are 5 light and 5 dark a column; the fade holds 40 to 360 across and 40 to 200 down, a row of the accent
    // from 170 to 200 at its foot, and fades its last 40 down and across.
    for x in [101.0, 107.0] {
        let behind = levels(scene.colours_at(&[(x, 250.0)])[0]);
        let gap = levels(scene.colours_at(&[(x, 85.0)])[0]);
        assert!(off(gap, behind) <= 2.0, "the gap between rows shows the stripes at {x}: {gap:?}, the stripes {behind:?}");
        let inside = levels(scene.colours_at(&[(x, 100.0)])[0]);
        assert!(off(inside, ROW) <= 2.0, "a row is whole above the fade at {x}: {inside:?}");
        let ys: Vec<f64> = (172..200).map(f64::from).collect();
        let seen: Vec<[f64; 3]> = scene.colours_at(&ys.iter().map(|&y| (x, y)).collect::<Vec<_>>()).into_iter().map(levels).collect();
        let mut last = f64::MAX;
        for (y, got) in ys.iter().zip(&seen) {
            let kept = (200.0 - (y + 0.5)) / 40.0;
            let want = mix(ROW, behind, kept);
            assert!(off(*got, want) <= 6.0, "at ({x}, {y}) the row kept {kept:.3} over the stripes is {want:?}, read {got:?}");
            let left = off(*got, behind);
            assert!(left <= last + 1.0, "the fade falls toward the stripes without a step back at ({x}, {y}): {left} after {last}");
            last = left;
        }
        for pair in seen.windows(2) {
            assert!(off(pair[0], pair[1]) <= 12.0, "no hard edge in the fade at {x}: {:?} then {:?}", pair[0], pair[1]);
        }
        assert!(off(seen[seen.len() - 1], behind) <= 6.0, "the edge is the stripes' own pixel at {x}: {:?}, the stripes {behind:?}", seen[seen.len() - 1]);
    }
    // Across: the row from 90 to 120 fades over its last 40, light stripe at 351, dark at 357.
    for x in [341.0, 351.0, 357.0] {
        let behind = levels(scene.colours_at(&[(x, 250.0)])[0]);
        let got = levels(scene.colours_at(&[(x, 100.0)])[0]);
        let want = mix(ROW, behind, (360.0 - (x + 0.5)) / 40.0);
        assert!(off(got, want) <= 6.0, "across at {x} the row over the stripes is {want:?}, read {got:?}");
    }
    // The first frame drawn after the fade's layer is made anew, by turning the fade off and on and by a new size, read
    // from one grab with no frame between: the gaps show the stripes and a row its colour within a level or two.
    let first = |scene: &mut emusen_qt_test::Scene<'_>, what: &str| {
        let seen: Vec<[f64; 3]> = scene.colours_at(&[(101.0, 85.0), (107.0, 85.0), (101.0, 100.0), (101.0, 250.0), (107.0, 250.0)]).into_iter().map(levels).collect();
        assert!(off(seen[0], seen[3]) <= 2.0 && off(seen[1], seen[4]) <= 2.0, "{what}: the first frame's gaps are the stripes: {:?} {:?}, the stripes {:?} {:?}", seen[0], seen[1], seen[3], seen[4]);
        assert!(off(seen[2], ROW) <= 2.0, "{what}: the first frame's row is its colour: {:?}", seen[2]);
    };
    assert!(scene.set_property("scene", "fading", "false"));
    assert!(scene.set_property_now("scene", "fading", "true"));
    first(&mut scene, "shown again");
    scene.wait(100);
    assert!(scene.set_property_now("fade", "width", "300"));
    first(&mut scene, "resized");
}
