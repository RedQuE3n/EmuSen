pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// A fixture: a list laid on an orbit by the kit's OrbitSlot, as a theme's screen holds one, in an item the test deletes as a window deletes a placed screen (EmuSen_Kit.md §7.40).
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
        Item {
            objectName: "screen"
            anchors.fill: parent
            focus: true
            ListView {
                id: orbit
                objectName: "orbit"
                property real turning: Math.max(0, orbit.currentIndex)
                x: 0
                y: 330
                width: 1280
                height: 412
                focus: true
                orientation: ListView.Horizontal
                highlightRangeMode: ListView.StrictlyEnforceRange
                preferredHighlightBegin: orbit.width / 2 - 100
                preferredHighlightEnd: orbit.width / 2 + 100
                highlightMoveDuration: 0
                model: 8
                delegate: Kit.OrbitSlot {
                    id: cover
                    objectName: "cover." + cover.index
                    Accessible.name: "Cover " + cover.index
                    controller: frame.controller
                    view: orbit
                    // The list by its id is null while the screen is torn down, as a theme's is.
                    turning: orbit?.turning ?? 0
                    lit: cover.ListView.isCurrentItem && (orbit?.activeFocus ?? false)
                    focus: cover.ListView.isCurrentItem
                    rx: 620
                    ry: 540
                    cy: 0
                    step: 18
                    coverHeight: 170
                    scaleStep: 0.1
                    minScale: 0.7
                    coverShape: 0.75
                    Rectangle {
                        anchors.fill: parent
                        color: "#808080"
                    }
                }
            }
        }
    }
}
