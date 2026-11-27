//! One harness test (EmuSen_Qt.md §8.1): the QML side sees LibraryModel's roles by name and its properties, and a
//! refresh over the real clients, run off the GUI thread, reaches it through qt_thread().queue.

use emusen_client_fixture::{library, World};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt_test::Harness;

#[test]
fn qml_reads_the_library_model_s_rows_through_its_roles() {
    assert_eq!(emusen_qt_models::library::PLACES.len(), 3, "the module is linked into this test");
    let mut harness = Harness::new();
    let scene_file = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/LibraryScene.qml");
    let mut scene = harness.load(&format!("file://{scene_file}"), 640, 480);
    assert_eq!(scene.property("scene", "rows"), "3", "the places, before any source is named");
    assert_eq!(scene.property("scene", "firstName"), "All games");
    assert_eq!(scene.property("scene", "secondKey"), "favourites");
    assert_eq!(scene.property("scene", "thirdKind"), "place");
    assert_eq!(scene.property("scene", "thirdCount"), "0");
    assert_eq!(scene.property("scene", "thirdShown"), "Recently played", "the delegate shows the name role");
    assert_eq!(scene.property("scene", "selectedKey"), "all");
    assert_eq!(scene.property("scene", "selectedIndex"), "0");
    // The Home card's facts follow the lists the binding is given, not what the model held when it was first read.
    let given = r#"[{"console":"psx","name":"PlayStation","engine":"V1ConsoleCore"}]"#;
    assert!(scene.set_property("scene", "startableGiven", given));
    assert!(scene.set_property("scene", "toolsGiven", r#"{"cores":{"brief":"2 engines"}}"#));
    let facts: serde_json::Value = serde_json::from_str(&scene.property("scene", "homeFacts")).unwrap();
    assert_eq!((facts["startable"][0]["name"].as_str(), facts["tools"]["cores"]["brief"].as_str()), (Some("PlayStation"), Some("2 engines")), "{facts}");

    // The real clients named through the model's properties, on a scratch tree holding one synthetic NES game.
    let world = World::new("models-qml");
    let roms = world.dir("Roms/NES");
    let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    image.resize(16 + 16384, 0);
    std::fs::write(roms.join("Synthetic.nes"), image).unwrap();
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    assert!(scene.set_property("scene", "platformLibrary", library().to_str().unwrap()));
    let mut waited = 0;
    while scene.property("scene", "rows") != "4" && waited < 5000 {
        scene.wait(50);
        waited += 50;
    }
    assert_eq!(scene.property("scene", "fourthKey"), "nes", "the NES's row arrived from the background refresh");
    assert_eq!(scene.property("scene", "loading"), "false");
    assert_eq!(scene.property("scene", "selectedKey"), "all", "the selection kept by its key");

    // Equal-length `kind` changes on `EclipseListModel` must emit `modelReset`, update `data(ROLE_KIND)`, and re-select `DelegateChooser`.
    let eclipse_qml = world.root.join("EclipseListTest.qml");
    std::fs::write(
        &eclipse_qml,
        r#"import QtQuick
import QtQml.Models
import EmuSen.Models

Window {
    id: win
    objectName: "listScene"
    width: 320
    height: 160
    property alias rows: listModel.rows
    property int resets: 0
    property int row0Kind: -1
    property int rowCountAtReset: -1
    property int validParentRowCount: -1
    property int targetFocus: 1
    property alias currentIdx: listView.currentIndex
    EclipseListModel {
        id: listModel
        objectName: "listModel"
        onModelReset: {
            win.resets += 1
            win.rowCountAtReset = listModel.count
            win.validParentRowCount = listModel.rowCount(listModel.index(0, 0))
            win.row0Kind = listModel.data(listModel.index(0, 0), 1)
        }
        onDataChanged: win.row0Kind = listModel.data(listModel.index(0, 0), 1)
    }
    ListView {
        id: listView
        anchors.fill: parent
        model: listModel
        readonly property int targetFocusIndex: win.targetFocus >= 0 && win.targetFocus < listModel.count ? win.targetFocus : -1
        Binding {
            target: listView
            property: "currentIndex"
            value: listView.targetFocusIndex
        }
        onCountChanged: if (listView.currentIndex !== listView.targetFocusIndex) listView.currentIndex = listView.targetFocusIndex
        onCurrentIndexChanged: if (listView.currentIndex !== listView.targetFocusIndex) listView.currentIndex = listView.targetFocusIndex
        delegate: DelegateChooser {
            role: "kind"
            DelegateChoice {
                roleValue: 0
                Text { objectName: "kind0"; text: model.item.name }
            }
            DelegateChoice {
                roleValue: 1
                Text { objectName: "kind1"; text: model.item.name }
            }
        }
    }
}
"#,
    )
    .unwrap();
    let mut list_scene = harness.load(&format!("file://{}", eclipse_qml.display()), 320, 160);
    assert!(list_scene.set_property_now("listScene", "rows", r#"[{"kind":0,"name":"alpha"},{"kind":1,"name":"beta"}]"#));
    list_scene.wait(50);
    assert_eq!(list_scene.property("listScene", "resets"), "1");
    assert_eq!(list_scene.property("listScene", "rowCountAtReset"), "2");
    assert_eq!(list_scene.property("listScene", "validParentRowCount"), "0");
    assert_eq!(list_scene.property("listScene", "currentIdx"), "1");
    assert_eq!(list_scene.property("listScene", "row0Kind"), "0");
    assert!(list_scene.tree().find_named("kind0").is_some());

    assert!(list_scene.set_property_now("listScene", "rows", r#"[{"kind":0,"name":"alpha2"},{"kind":1,"name":"beta"}]"#));
    list_scene.wait(50);
    assert_eq!(list_scene.property("listScene", "resets"), "1", "same-kind equal-length edit updates without reset");
    let tree = list_scene.tree();
    let node0 = tree.find_named("kind0").expect("kind0 stays active") as usize;
    assert_eq!(tree.nodes[node0].text.as_deref(), Some("alpha2"));

    assert!(list_scene.set_property_now("listScene", "rows", r#"[{"kind":1,"name":"alpha2"},{"kind":1,"name":"beta"}]"#));
    list_scene.wait(50);
    assert_eq!(list_scene.property("listScene", "resets"), "2", "kind change at equal length triggers modelReset");
    assert_eq!(list_scene.property("listScene", "row0Kind"), "1", "ROLE_KIND updated after reset");
    assert_eq!(list_scene.property("listScene", "currentIdx"), "1", "equal-index focus restored across modelReset");
    let tree = list_scene.tree();
    assert!(tree.find_named("kind0").is_none(), "old DelegateChoice removed");
    let node1 = tree.find_named("kind1").expect("DelegateChooser switched to kind1") as usize;
    assert_eq!(tree.nodes[node1].text.as_deref(), Some("alpha2"));

    assert!(list_scene.set_property_now("listScene", "rows", r#"[{"kind":1,"name":"one"},{"kind":1,"name":"two"},{"kind":1,"name":"three"}]"#));
    list_scene.wait(50);
    assert_eq!(list_scene.property("listScene", "resets"), "3");
    assert_eq!(list_scene.property("listScene", "rowCountAtReset"), "3", "count updated before modelReset");
    assert_eq!(list_scene.property("listScene", "validParentRowCount"), "0");
}

#[test]
fn the_scene_keeps_the_qml_rules() {
    let scenes = emusen_qt_test::lint::frontend().join("qt-models").join("tests");
    let lines = emusen_qt_test::lint::rules(&[(&scenes, true)]);
    assert!(lines.is_empty(), "{}", lines.join("\n"));
}
