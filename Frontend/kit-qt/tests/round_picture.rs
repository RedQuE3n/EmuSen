// RoundPicture loads image sources asynchronously and renders cleanly on the software backend.

use std::path::PathBuf;

use emusen_qt_test::covers::{art, draw_art};
use emusen_qt_test::Harness;

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/RoundPictureScene.qml");

struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn round_picture_loads_asynchronously_on_software_backend() {
    let dir = TempDir(std::env::temp_dir().join(format!("kit-qt-round-picture-{}", std::process::id())));
    std::fs::create_dir_all(&dir.0).unwrap();
    let image = dir.0.join("sample.png");
    let mut harness = Harness::new();
    draw_art(&mut harness, &dir.0, &[(image.clone(), (120, 120), 0)], |_| {});

    let mut scene = harness.load(SCENE, 240, 240);
    assert!(scene.set_property("picture", "source", &format!("file://{}", image.display())));

    let mut waited = 0;
    while scene.property("picture", "status") != "Ready" && waited < 2000 {
        scene.wait(20);
        waited += 20;
    }
    assert_eq!(scene.property("picture", "status"), "Ready", "image decoded asynchronously within 2 s");
    scene.wait(30);

    let pixel = scene.colours_at(&[(120.0, 45.0)])[0].expect("decoded pixel");
    let expected = art(0)[0];
    assert!(
        (i16::from(pixel.r) - i16::from(expected.0)).abs() <= 4
            && (i16::from(pixel.g) - i16::from(expected.1)).abs() <= 4
            && (i16::from(pixel.b) - i16::from(expected.2)).abs() <= 4,
        "got {pixel:?}, expected {expected:?}"
    );
    scene.assert_quiet();
}
