// EmuSen_Interface.md §18.6, EmuSen_Kit.md §7.23: a notice at the top right inside the safe area, held 2 s then gone; a second
// replaces the first; over gameplay only the game's own show and the rest wait; the screen still once it has gone.

use emusen_qt_test::Harness;

const KEY_F2: i32 = 0x0100_0031;
const KEY_F3: i32 = 0x0100_0032;
const KEY_F4: i32 = 0x0100_0033;

#[test]
fn qt_18_6_one_notice_at_a_time_at_the_top_right_held_two_seconds() {
    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/NoticeScene.qml", 1280, 800);
    assert_eq!(scene.property("notices", "shown"), "false");
    scene.key(KEY_F2, 0);
    scene.wait(400);
    assert_eq!(scene.property("notices", "shown"), "true");
    let tree = scene.tree();
    let card = &tree.nodes[tree.find_named("notice").expect("the notice") as usize];
    assert!((card.x + card.w - 1216.0).abs() <= 1.0 && (card.y - 40.0).abs() <= 1.0, "top right inside the safe area: {} {}", card.x + card.w, card.y);
    assert!(card.visible);
    let dir = std::env::var("EMUSEN_QT_COMPARE").unwrap_or_default();
    if !dir.is_empty() {
        scene.grab(&format!("{dir}/Notice-1280x800.png"));
    }
    scene.wait(1000);
    scene.key(KEY_F3, 0);
    scene.wait(1300);
    assert_eq!(scene.property("notices", "words"), "Core approval needed", "the second replaced the first");
    assert_eq!(scene.property("notices", "shown"), "true", "its hold started again");
    scene.wait(1200);
    assert_eq!(scene.property("notices", "shown"), "false", "gone after 2 s");

    // Over gameplay a notice of the game's own shows; any other waits until gameplay stops.
    scene.key(KEY_F4, 0);
    scene.key(KEY_F3, 0);
    scene.wait(100);
    assert_eq!(scene.property("notices", "shown"), "false", "waiting over gameplay");
    scene.key(KEY_F2, 0);
    scene.wait(100);
    assert_eq!(scene.property("notices", "words"), "State saved to Slot 3");
    scene.wait(2300);
    scene.key(KEY_F4, 0);
    scene.wait(100);
    assert_eq!(scene.property("notices", "words"), "Core approval needed", "shown once gameplay stopped");
    scene.wait(2600);
    assert_eq!(scene.property("notices", "shown"), "false");
    scene.assert_still("the notice gone");
    scene.assert_quiet();
}
