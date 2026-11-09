//! Covers for a synthetic library, drawn by Qt Quick and grabbed to files, as Mistress's own-look fixture draws them; shared by
//! the program's tests and theme-audit (EmuSen_Kit.md §7.9).

use std::path::{Path, PathBuf};

use cxx_qt::QObject;

use crate::Harness;

/// The covers as Mistress's own-look fixture draws them, each a flat colour with a white rim and its words, grabbed from
/// one Qt Quick window resized to each box's shape (SyntheticLibrary.Label's drawing). `prepare` is handed the window
/// before it shows (on Vulkan, the program's device adopted), and the window is hidden once drawn: a second window's
/// swapchain released only when the engine was dropped faulted in Qt's release.
pub fn draw(harness: &mut Harness, scratch: &Path, covers: &[(PathBuf, (u32, u32), usize, String)], prepare: impl FnOnce(*mut QObject)) {
    let qml = scratch.join("cover.qml");
    std::fs::write(
        &qml,
        "import QtQuick\nWindow {\n  objectName: \"covers\"\n  Rectangle { objectName: \"fill\"; anchors.fill: parent\n\
         Rectangle { x: 3; y: 3; width: parent.width - 6; height: parent.height - 6; color: \"transparent\"; border.color: \"white\"; border.width: 6 }\n\
         Text { objectName: \"words\"; anchors.centerIn: parent; width: parent.width - 20; horizontalAlignment: Text.AlignHCenter; color: \"white\"; font.pixelSize: Math.max(12, Math.floor(parent.height / 12)) } } }\n",
    )
    .unwrap();
    let mut scene = harness.load_prepared(&format!("file://{}", qml.display()), 64, 64, prepare);
    for (path, (w, h), index, words) in covers {
        // Green is the covers folder's: 60 + 6 * 23 % 160, the name's length in SyntheticLibrary.Colour.
        let (r, g, b) = (40 + 17 * index % 180, 198, 90 + 31 * index % 150);
        scene.resize(*w as i32, *h as i32);
        assert!(scene.set_property("fill", "color", &format!("#{r:02x}{g:02x}{b:02x}")));
        assert!(scene.set_property("words", "text", words));
        scene.wait(50);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        assert!(scene.grab(&path.display().to_string()), "cover {}", path.display());
    }
    // Hidden while its scene graph still holds its instance, so its swapchain is released before the window goes.
    assert!(scene.set_property("covers", "visible", "false"));
    scene.wait(50);
}
