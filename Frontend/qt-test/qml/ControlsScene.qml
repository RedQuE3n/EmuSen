import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// A fixture: the styled controls, one to a row the pad walks, in a big-screen frame, for the palette sweep and a grab (EmuSen_Kit.md §7.9).
// A TextField and a ComboBox take Up and Down themselves, so they share the last row and the field is reached by Right.
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    Kit.Frame {
        id: frame
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: true
        help: ListModel {
            ListElement { button: 4; label: "Choose" }
            ListElement { button: 5; label: "Back" }
        }
        Column {
            spacing: Kit.Tokens.base
            width: Kit.Tokens.cardWidth
            Kit.WholeRow {
                id: row0
                width: parent.width
                controller: frame.controller
                Button { objectName: "play"; text: "Play"; focus: true; KeyNavigation.down: row1 }
            }
            Kit.WholeRow {
                id: row1
                width: parent.width
                controller: frame.controller
                Button { objectName: "primary"; text: "Primary"; highlighted: true; focus: true; KeyNavigation.up: row0; KeyNavigation.down: row2 }
            }
            Kit.WholeRow {
                id: row2
                width: parent.width
                controller: frame.controller
                CheckBox { objectName: "check"; text: "Keep saves"; checked: true; focus: true; KeyNavigation.up: row1; KeyNavigation.down: row3 }
            }
            Kit.WholeRow {
                id: row3
                width: parent.width
                controller: frame.controller
                RadioButton { objectName: "radio"; text: "Sharp"; checked: true; focus: true; KeyNavigation.up: row2; KeyNavigation.down: row4 }
            }
            Kit.WholeRow {
                id: row4
                width: parent.width
                controller: frame.controller
                Switch { objectName: "switch"; text: "Rewind"; checked: true; focus: true; KeyNavigation.up: row3; KeyNavigation.down: row5 }
            }
            Kit.WholeRow {
                id: row5
                width: parent.width
                controller: frame.controller
                Slider { objectName: "slider"; Accessible.name: "Volume"; from: 0; to: 100; value: 40; width: Kit.Tokens.controlHeight * 6; focus: true; KeyNavigation.up: row4; KeyNavigation.down: row6 }
            }
            Kit.WholeRow {
                id: row6
                width: parent.width
                controller: frame.controller
                TabBar {
                    objectName: "tabs"
                    focus: true
                    KeyNavigation.up: row5
                    KeyNavigation.down: row7
                    TabButton { text: "Games" }
                    TabButton { text: "Settings" }
                }
            }
            Kit.WholeRow {
                id: row7
                width: parent.width
                controller: frame.controller
                Row {
                    spacing: Kit.Tokens.paneGap
                    ComboBox { id: combo; objectName: "combo"; model: ["Auto", "NTSC", "PAL"]; focus: true; KeyNavigation.right: field }
                    TextField { id: field; objectName: "field"; placeholderText: "Search"; KeyNavigation.left: combo }
                }
            }
            ProgressBar { objectName: "progress"; value: 0.6; width: parent.width }
            Label { objectName: "label"; text: "A label in the tokens" }
        }
    }
}
