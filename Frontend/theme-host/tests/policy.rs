//! The sandbox's rules as plain data (EmuSen_Qt.md §3.5 as built), with no engine: what each kind of URL may reach.

use emusen_theme_host::host::folder_url;
use emusen_theme_host::policy::{module_of, never_resolved, Kind, Policy};

const QT_QML: &str = "/usr/lib64/qt6/qml";

fn policy() -> Policy {
    Policy::new("file:///themes/Dawn", &["/media/covers".to_owned()], QT_QML)
}

fn passes(p: &Policy, url: &str, kind: Kind) -> bool {
    let local = url.strip_prefix("file://").unwrap_or("");
    p.judge(url, local, kind, "").is_empty()
}

#[test]
fn qt_3_5_the_folder_and_the_cover_store_are_reached_and_nothing_else_on_disk() {
    let p = policy();
    assert!(passes(&p, "file:///themes/Dawn/assets/sky.png", Kind::Url));
    assert!(passes(&p, "file:///media/covers/snes/Game.png", Kind::Url));
    assert!(!passes(&p, "file:///media/covers/evil.qml", Kind::Qml), "a cover store holds no QML");
    assert!(!passes(&p, "file:///etc/hostname", Kind::Url));
    assert!(!passes(&p, "file:///themes/Dawn/../Other/x.png", Kind::Url), "a path climbing out of the folder");
    assert!(!passes(&p, "file:///themes/Dawnish/x.png", Kind::Url), "a sibling whose name begins with the folder's");
}

#[test]
fn qt_3_5_no_network_and_no_unknown_scheme() {
    let p = policy();
    for url in ["http://example.invalid/a.png", "https://example.invalid/a.png", "ftp://example.invalid/a", "smb://nas/a.png"] {
        assert!(p.judge(url, "", Kind::Url, "").contains("a theme reaches no network"), "{url}");
    }
    assert!(passes(&p, "data:image/png;base64,AAAA", Kind::Url));
    assert!(passes(&p, "image://provider/x", Kind::Url));
    assert!(passes(&p, "", Kind::Url), "an unset source is passed, not refused");
}

#[test]
fn qt_3_5_qt_s_versioned_probes_follow_the_module_they_probe_for() {
    let p = policy();
    for probe in ["qrc:/qt/qml/EmuSen/Theme.1.0/qmldir", "qrc:/qt/qml/EmuSen.1.0/Theme/qmldir", "qrc:/qt/qml/EmuSen/Theme.1/qmldir", "qrc:/qt/qml/EmuSen.1/Theme/qmldir"] {
        assert!(passes(&p, probe, Kind::Qmldir), "a probe for EmuSen.Theme 1.0: {probe}");
    }
    assert!(passes(&p, "qrc:/qt/qml/EmuSen/Kit.1.0/qmldir", Kind::Qmldir));
    let models = p.judge("qrc:/qt/qml/EmuSen.1.0/Models/qmldir", "", Kind::Qmldir, "");
    assert_eq!(models, "EmuSen.Models is a module no theme may use", "a probe for a module never resolved is still refused");
    let app = p.judge("qrc:/qt/qml/EmuSen/Scenes.1/qmldir", "", Kind::Qmldir, "");
    assert!(app.ends_with("is a resource of the program, not of the theme's modules"), "a module of the program not on the list stays refused and logged: {app}");
    assert!(!passes(&p, "qrc:/qt/qml/EmuSen/App.1/qml/Main.qml", Kind::Qml), "only a qmldir probe is read without its version");
}

#[test]
fn qt_3_5_qt_own_modules_resolve_and_the_named_ones_never() {
    let p = policy();
    assert!(passes(&p, "file:///usr/lib64/qt6/qml/QtQuick/qmldir", Kind::Qmldir));
    assert!(passes(&p, "file:///usr/lib64/qt6/qml/QtQuick/Controls/Basic/Button.qml", Kind::Qml));
    assert!(!passes(&p, "file:///usr/lib64/qt6/qml/QtMultimedia/qmldir", Kind::Qmldir));
    assert!(!passes(&p, "file:///usr/lib64/qt6/qml/QtQuick/LocalStorage/qmldir", Kind::Qmldir));
    assert!(!passes(&p, "file:///usr/lib64/qt6/qml/Qt/labs/folderlistmodel/qmldir", Kind::Qmldir));
    assert!(!passes(&p, "qrc:/qt/qml/EmuSen/Models/qmldir", Kind::Qmldir));
    assert!(passes(&p, "qrc:/qt/qml/EmuSen/Theme/qmldir", Kind::Qmldir));
    assert!(passes(&p, "qrc:/qt/qml/EmuSen/Kit/Frame.qml", Kind::Qml));
    assert!(passes(&p, "qrc:/qt/qml/EmuSen/Button.qml", Kind::Qml), "the EmuSen style's controls");
    assert!(passes(&p, "qrc:/qt-project.org/imports/QtQuick/Controls/Basic/Button.qml", Kind::Qml));
    assert!(!passes(&p, "qrc:/qt/qml/EmuSen/App/qml/Main.qml", Kind::Qml), "the program's own screens");
    assert!(!passes(&p, "qrc:/emusen/esde/picture.frag.qsb", Kind::Url), "another module's resource");
    assert_eq!(module_of("QtQuick/Controls/qmldir").as_deref(), Some("QtQuick.Controls"));
    assert!(never_resolved("Qt.labs.platform") && never_resolved("EmuSen.Models") && !never_resolved("EmuSen.Theme"));
    assert!(!never_resolved("QtQuick") && !never_resolved("QtCoreish"), "matched by whole dotted parts");
}

#[test]
fn qt_3_5_a_theme_file_is_held_to_the_import_allow_list() {
    let p = policy();
    let home = "file:///themes/Dawn/screens/Home.qml";
    assert!(p.reads(home, "/themes/Dawn/screens/Home.qml", Kind::Qml));
    assert!(!p.reads("file:///usr/lib64/qt6/qml/QtQuick/Controls/Basic/Button.qml", "/usr/lib64/qt6/qml/QtQuick/Controls/Basic/Button.qml", Kind::Qml));
    let good = "import QtQuick\nimport QtQuick.Layouts\nimport EmuSen.Theme 1.0\nimport \"../parts\"\nItem {}\n";
    assert_eq!(p.judge(home, "/themes/Dawn/screens/Home.qml", Kind::Qml, good), "");
    let bad = "import QtQuick\nimport QtMultimedia\nItem {}\n";
    let words = p.judge(home, "/themes/Dawn/screens/Home.qml", Kind::Qml, bad);
    assert!(words.starts_with("screens/Home.qml:2 imports QtMultimedia, which a theme may not import"), "{words}");
    let kit = p.judge(home, "/themes/Dawn/screens/Home.qml", Kind::Qml, "import EmuSen.Kit as Kit\nItem {}\n");
    assert!(kit.contains("imports EmuSen.Kit"), "the kit is reached through EmuSen.Theme only: {kit}");
    let script = p.judge("file:///themes/Dawn/parts/a.js", "/themes/Dawn/parts/a.js", Kind::Script, ".import QtQuick.LocalStorage 2.0 as Sql\n");
    assert!(script.contains("parts/a.js:1 imports QtQuick.LocalStorage"), "{script}");
}

#[test]
fn qt_3_5_a_folder_in_the_resources_is_a_folder_too() {
    let p = Policy::new("qrc:/emusen/themes/emusen", &[], QT_QML);
    assert!(p.reads("qrc:/emusen/themes/emusen/screens/Home.qml", "", Kind::Qml));
    assert!(p.judge("qrc:/emusen/themes/emusen/assets/a.png", "", Kind::Url, "").is_empty());
    assert_eq!(folder_url("/a/b/"), "file:///a/b");
    assert_eq!(folder_url("qrc:/emusen/themes/emusen"), "qrc:/emusen/themes/emusen");
}
