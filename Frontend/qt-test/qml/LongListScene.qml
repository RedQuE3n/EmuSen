pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// A fixture: a plain ListView of 9,000 walkable rows, walked by Up and Down, as a theme's list of a large console is (EmuSen_Kit.md §7.11).
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    Kit.NormalizedCanvas {
        id: canvas
        anchors.fill: parent
        Kit.Frame {
            id: frame
            anchors.fill: parent
            form: Kit.Forms.BigScreen
            focus: true
            ListView {
                id: list
                objectName: "rows"
                width: 480
                height: 600
                spacing: 8
                clip: true
                focus: true
                keyNavigationEnabled: true
                highlightMoveDuration: Kit.Tokens.instant
                model: 9000
                delegate: Kit.Walkable {
                    id: row
                    required property int index
                    objectName: "row." + row.index
                    controller: frame.controller
                    Accessible.name: "Row " + row.index
                    width: 480
                    height: 56
                    focus: ListView.isCurrentItem
                    onActiveFocusChanged: list.currentIndex = row.activeFocus ? row.index : list.currentIndex
                    Rectangle {
                        anchors.fill: parent
                        color: row.activeFocus ? Kit.Tokens.surfaceRaised : Kit.Tokens.surface
                    }
                }
            }
        }
    }
}
