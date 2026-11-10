pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// A fixture: a grid whose cells are walkable delegates, and a strip that turns to its current item, whose walkables stand inside each delegate, as the themes' galleries hold them (EmuSen_Kit.md §7.39).
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
        Column {
            x: 40
            y: 40
            spacing: 40
            GridView {
                id: grid
                objectName: "grid"
                width: 5 * 160
                height: 2 * 160
                cellWidth: 160
                cellHeight: 160
                focus: true
                keyNavigationWraps: false
                model: 10
                delegate: Kit.Walkable {
                    id: cell
                    required property int index
                    objectName: "cell." + cell.index
                    Accessible.name: "Cell " + cell.index
                    controller: frame.controller
                    width: 140
                    height: 140
                    focus: cell.GridView.isCurrentItem
                    Rectangle {
                        anchors.fill: parent
                        color: cell.activeFocus ? Kit.Tokens.accent : Kit.Tokens.surface
                    }
                }
            }
            ListView {
                id: strip
                objectName: "strip"
                width: 700
                height: 140
                orientation: ListView.Horizontal
                spacing: 10
                highlightRangeMode: ListView.ApplyRange
                preferredHighlightBegin: 280
                preferredHighlightEnd: 420
                highlightMoveDuration: 300
                highlight: Item {}
                keyNavigationWraps: false
                model: 30
                delegate: FocusScope {
                    id: slot
                    required property int index
                    width: 140
                    height: 140
                    focus: slot.ListView.isCurrentItem
                    Kit.Walkable {
                        id: item
                        objectName: "item." + slot.index
                        Accessible.name: "Item " + slot.index
                        controller: frame.controller
                        anchors.fill: parent
                        focus: true
                        Rectangle {
                            anchors.fill: parent
                            color: item.activeFocus ? Kit.Tokens.accent : Kit.Tokens.surface
                        }
                    }
                }
            }
        }
    }
}
