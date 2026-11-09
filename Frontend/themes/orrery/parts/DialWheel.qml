pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// A small wheel of the dial, 44 across: a 1-unit ring and its teeth, each a 1-unit stroke from radius `inner` to `outer`, the first at `first` degrees clockwise from the top, and an optional hub dot (orrery.md §1.2).
Scene {
    id: wheel
    property int teeth: 8
    property real inner: 17
    property real outer: 21
    property real first: 0
    property real toothWidth: 1
    property bool ring: true
    property bool hub: false
    property color ink: Tokens.muted
    property color hubInk: wheel.ink
    width: 44
    height: 44
    Rectangle {
        visible: wheel.ring
        anchors.fill: parent
        anchors.margins: 0.5
        radius: width / 2
        color: "transparent"
        border.width: 1
        border.color: wheel.ink
    }
    Repeater {
        model: wheel.teeth
        Rectangle {
            id: tooth
            required property int index
            x: 22 - wheel.toothWidth / 2
            y: 22 - wheel.outer
            width: wheel.toothWidth
            height: wheel.outer - wheel.inner
            color: wheel.ink
            transform: Rotation {
                origin.x: wheel.toothWidth / 2
                origin.y: wheel.outer
                angle: wheel.first + 360 * tooth.index / Math.max(1, wheel.teeth)
            }
        }
    }
    Rectangle {
        visible: wheel.hub
        x: 19
        y: 19
        width: 6
        height: 6
        radius: 3
        color: wheel.hubInk
    }
}
