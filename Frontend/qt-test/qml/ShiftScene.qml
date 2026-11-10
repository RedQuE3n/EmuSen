pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// A fixture: a row of walkables that slide as the focused one grows, and marks made again for each focus, as a theme's orbits turn and its covers follow the focus (EmuSen_Kit.md §7.11).
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    property int focused: 0
    Kit.Frame {
        id: frame
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: true
        Row {
            x: 40
            y: 40
            spacing: 20
            Repeater {
                id: tiles
                model: 6
                Kit.Walkable {
                    id: tile
                    required property int index
                    objectName: "tile." + tile.index
                    Accessible.name: "Tile " + tile.index
                    controller: frame.controller
                    width: tile.activeFocus ? 200 : 120
                    height: 120
                    focus: tile.index === 0
                    KeyNavigation.right: tiles.itemAt(tile.index + 1)
                    KeyNavigation.left: tiles.itemAt(tile.index - 1)
                    onActiveFocusChanged: if (tile.activeFocus) scene.focused = tile.index
                    Behavior on width {
                        NumberAnimation { duration: 300 }
                    }
                    Rectangle {
                        anchors.fill: parent
                        color: tile.activeFocus ? Kit.Tokens.accent : Kit.Tokens.surface
                    }
                }
            }
        }
        Row {
            x: 40
            y: 200
            spacing: 10
            Repeater {
                model: scene.focused + 1
                Rectangle {
                    width: 40
                    height: 40
                    color: Kit.Tokens.surface
                }
            }
        }
    }
}
