pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// The side-column fixture with its makers past the first cut off: no press reaches them or their consoles.
ThemeScreen {
    objectName: "Consoles"
    role: ThemeScreen.Consoles
    NormalizedCanvas {
        id: canvas
        FocusScope {
            id: root
            anchors.fill: parent
            focus: true
            property int chosen: 0
            property int made: 0
            // Set false by the broken copy of this fixture, whose makers past the first no press reaches.
            readonly property bool joined: false
            GroupGridLayout {
                id: layout
                model: Library.consoles
                columns: 4
            }
            Connections {
                target: Library.consoles
                function onModelReset() { layout.read() }
            }
            Frame {
                id: frame
                anchors.fill: parent
                focus: true
                Item {
                    x: -frame.safeX
                    y: -frame.safeY
                    width: root.width
                    height: root.height
                    Repeater {
                        id: makers
                        model: Library.consoles
                        onItemAdded: root.made = root.made + 1
                        Walkable {
                            id: maker
                            required property string maker
                            required property int group
                            required property int groupIndex
                            readonly property int after: layout.firsts[maker.group + 1] ?? -1
                            controller: frame.controller
                            visible: maker.groupIndex === 0
                            Accessible.name: maker.maker
                            x: 64
                            y: 120 + maker.group * 40
                            width: 160
                            height: 32
                            KeyNavigation.down: root.joined && root.made > 0 && maker.after >= 0 ? makers.itemAt(maker.after) : null
                            KeyNavigation.right: root.made > 0 ? consoles.itemAt(layout.firsts[maker.group] ?? 0) : null
                            onActiveFocusChanged: if (maker.activeFocus) root.chosen = maker.group
                            Text {
                                anchors.fill: parent
                                text: maker.maker
                                color: Tokens.text
                                font.pixelSize: 20
                            }
                        }
                    }
                    Repeater {
                        id: consoles
                        model: Library.consoles
                        onItemAdded: root.made = root.made + 1
                        Walkable {
                            id: tile
                            required property int index
                            required property string name
                            required property int group
                            required property int groupIndex
                            readonly property int leftIndex: layout.left[tile.index] ?? -1
                            readonly property int rightIndex: layout.right[tile.index] ?? -1
                            readonly property int upIndex: layout.up[tile.index] ?? -1
                            readonly property int downIndex: layout.down[tile.index] ?? -1
                            controller: frame.controller
                            visible: tile.group === root.chosen
                            focus: tile.group === 0 && tile.groupIndex === 0
                            Accessible.name: tile.name
                            x: 260 + (layout.columnOf[tile.index] ?? 0) * 170
                            y: 120 + (layout.rowOf[tile.index] ?? 0) * 60
                            width: 150
                            height: 48
                            KeyNavigation.left: root.made > 0 ? (tile.leftIndex >= 0 ? consoles.itemAt(tile.leftIndex) : makers.itemAt(layout.firsts[tile.group] ?? 0)) : null
                            KeyNavigation.right: root.made > 0 && tile.rightIndex >= 0 ? consoles.itemAt(tile.rightIndex) : null
                            KeyNavigation.up: root.made > 0 && tile.upIndex >= 0 ? consoles.itemAt(tile.upIndex) : null
                            KeyNavigation.down: root.made > 0 && tile.downIndex >= 0 ? consoles.itemAt(tile.downIndex) : null
                            onActiveFocusChanged: if (tile.activeFocus) root.chosen = tile.group
                            Text {
                                anchors.fill: parent
                                text: tile.name
                                color: Tokens.text
                                font.pixelSize: 20
                            }
                        }
                    }
                }
            }
        }
    }
}
