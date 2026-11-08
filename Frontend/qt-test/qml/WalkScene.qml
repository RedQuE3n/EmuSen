pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// A fixture: rows in a list with a field and a combo box among them and no declared path, so Up and Down must pass both (EmuSen_Kit.md §7.24).
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    property int caret: 2
    property int value: 1
    property bool open: false
    Kit.Frame {
        id: frame
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: true
        ListView {
            id: rows
            objectName: "rows"
            width: Kit.Tokens.cardWidth
            height: Kit.Tokens.rowHeight * 4 + Kit.Tokens.base * 3
            spacing: Kit.Tokens.base
            focus: true
            keyNavigationEnabled: true
            highlightMoveDuration: Kit.Tokens.instant
            highlightResizeDuration: Kit.Tokens.instant
            model: 4
            delegate: Kit.WholeRow {
                id: row
                required property int index
                objectName: "row." + row.index
                width: Kit.Tokens.cardWidth
                controller: frame.controller
                focus: ListView.isCurrentItem
                onActiveFocusChanged: rows.currentIndex = row.activeFocus ? row.index : rows.currentIndex
                Loader {
                    focus: true
                    sourceComponent: row.index === 1 ? field : (row.index === 2 ? combo : plain)
                }
            }
        }
        Component {
            id: plain
            Button { text: "Play"; focus: true }
        }
        Component {
            id: field
            TextField { objectName: "field"; text: "gran"; cursorPosition: 2; focus: true; width: Kit.Tokens.controlHeight * 6; onCursorPositionChanged: scene.caret = cursorPosition }
        }
        Component {
            id: combo
            ComboBox { objectName: "combo"; model: ["Auto", "NTSC", "PAL"]; currentIndex: 1; focus: true; onCurrentIndexChanged: scene.value = currentIndex; popup.onOpened: scene.open = true }
        }
    }
}
