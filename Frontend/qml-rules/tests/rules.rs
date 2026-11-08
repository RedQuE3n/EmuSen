// EmuSen_Qt.md §1.3: what a QML file may hold, and the spike's three deliberate breaks as the first cases (§10.3).

use emusen_qml_rules::{check, Options, Rule};

fn rules(text: &str) -> Vec<Rule> {
    check("t.qml", text, Options::default()).into_iter().map(|f| f.rule).collect()
}

#[test]
fn qt_1_3_layout_bindings_handlers_and_names_pass() {
    let text = r##"
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit
// a comment with "words" and function() in it
Item {
    id: root
    objectName: "frame"
    width: Kit.Tokens.cardWidth * 2 + 4
    color: root.card ? Kit.Tokens.scrim : "transparent"
    border.color: "#4C9AE8"
    font.family: "Barlow"
    Keys.onPressed: (event) => event.accepted = root.controller.key(event.key, event.modifiers)
    Component.onCompleted: root.forceActiveFocus()
    Connections {
        target: Kit.PadRouter
        function onPressed(button) { root.controller.offer(button) }
    }
    NumberAnimation { property: "opacity"; duration: Kit.Tokens.quick }
    text: model[control.textRole]
}
"##;
    assert_eq!(rules(text), vec![]);
}

#[test]
fn qt_10_3_the_spikes_three_breaks_are_caught() {
    assert_eq!(rules("Text { text: tile.name.toUpperCase() + \" games\" }"), vec![Rule::Words]);
    assert_eq!(rules("Shortcut { sequence: \"Q\"; onActivated: Qt.quit() }"), vec![]);
    assert_eq!(rules("Timer { onTriggered: Qt.callLater(root.go) }"), vec![Rule::CallLater]);
    assert_eq!(rules("Item { function go() { root.x = 1 } }"), vec![Rule::Function]);
}

#[test]
fn qt_1_3_logic_literals_rows_and_imports_are_findings() {
    assert_eq!(rules("Item { property var rows: [1, 2, 3] }"), vec![Rule::Literal]);
    assert_eq!(rules("Item { property var o: { a: 1 } }"), vec![Rule::Literal]);
    assert_eq!(rules("ListModel { ListElement { name: \"NES\" } }"), vec![Rule::RowsInQml, Rule::Words]);
    assert_eq!(rules("Component.onCompleted: root.load()"), vec![Rule::Completed]);
    assert_eq!(rules("import QtQml.Models"), vec![Rule::Import]);
    assert_eq!(rules("import QtWebEngine"), vec![Rule::Import]);
    assert_eq!(rules("import QtQuick.Controls.impl"), vec![]);
    assert_eq!(rules("import EmuSen.Theme 1.0"), vec![]);
    assert_eq!(
        rules("import EmuSen.Kit"),
        vec![Rule::Import],
        "the kit is always imported as Kit, so Qt's Frame and the kit's never shadow each other"
    );
    assert_eq!(rules("import EmuSen.Kit as Kit"), vec![]);
    assert_eq!(rules("Connections { function onPressed(b) { if (b) go() } }"), vec![Rule::Function], "a handler with logic is a function");
    assert_eq!(rules("Text { text: \"Choose a console\" }"), vec![Rule::Words]);
    assert_eq!(
        check("t.qml", "ListModel { ListElement { name: \"Choose\" } }", Options { fixture: true }),
        vec![],
        "a fixture may hold words and rows"
    );
    assert_eq!(
        check("t.qml", "Item { choices: [\"Stay\", \"Leave\"] }", Options { fixture: true }),
        vec![],
        "and literals that stand in for a model"
    );
}

#[test]
fn qt_1_3_a_finding_reads_as_file_line_rule_words() {
    let finding = &check("Main.qml", "\n\nText { text: \"Hi\" }", Options::default())[0];
    assert_eq!(finding.to_string(), "Main.qml:3 words \"Hi\"");
}
