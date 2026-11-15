//! Covers for a synthetic library, drawn by Qt Quick and grabbed to files, as Mistress's own-look fixture draws them; shared by
//! the program's tests and theme-audit (EmuSen_Kit.md §7.9).

use std::path::{Path, PathBuf};

use cxx_qt::QObject;

use crate::Harness;

/// The flat colour `draw` fills the cover at `index` with.
pub fn fill(index: usize) -> (u8, u8, u8) {
    // Green is the covers folder's: 60 + 6 * 23 % 160, the name's length in SyntheticLibrary.Colour.
    ((40 + 17 * index % 180) as u8, 198, (90 + 31 * index % 150) as u8)
}

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
        let (r, g, b) = fill(*index);
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

/// The three colours `draw_art` gives the cover at `index`, top band first: the flat fill, then two its channels turn into, so
/// every cover holds several colours as a game's art does.
pub fn art(index: usize) -> [(u8, u8, u8); 3] {
    let (r, _, b) = fill(index);
    [fill(index), (b, 90, 230), (230, r, 140)]
}

/// Covers as a game's art stands in a look's pictures: no words and no rim, three bands of `art`'s colours, grabbed as `draw`
/// grabs its own, so a blur or a crop of them is judged against colours a test knows.
pub fn draw_art(harness: &mut Harness, scratch: &Path, covers: &[(PathBuf, (u32, u32), usize)], prepare: impl FnOnce(*mut QObject)) {
    let qml = scratch.join("art.qml");
    std::fs::write(
        &qml,
        "import QtQuick\nWindow {\n  objectName: \"art\"\n  Column { anchors.fill: parent\n\
         Rectangle { objectName: \"band0\"; width: parent.width; height: parent.height / 3 }\n\
         Rectangle { objectName: \"band1\"; width: parent.width; height: parent.height / 3 }\n\
         Rectangle { objectName: \"band2\"; width: parent.width; height: parent.height - 2 * Math.floor(parent.height / 3) } } }\n",
    )
    .unwrap();
    let mut scene = harness.load_prepared(&format!("file://{}", qml.display()), 64, 64, prepare);
    for (path, (w, h), index) in covers {
        scene.resize(*w as i32, *h as i32);
        for (band, (r, g, b)) in art(*index).into_iter().enumerate() {
            assert!(scene.set_property(&format!("band{band}"), "color", &format!("#{r:02x}{g:02x}{b:02x}")));
        }
        scene.wait(50);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        assert!(scene.grab(&path.display().to_string()), "art {}", path.display());
    }
    assert!(scene.set_property("art", "visible", "false"));
    scene.wait(50);
}
