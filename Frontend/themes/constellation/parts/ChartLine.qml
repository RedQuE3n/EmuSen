pragma ComponentBehavior: Bound
import QtQuick

// One line of the chart, 1.5 units in one ink from (x1, y1) to (x2, y2); dashed 4 on 6 off for a walk edge no chart line draws (constellation.md §1.2).
Item {
    id: line
    property real x1
    property real y1
    property real x2
    property real y2
    property color ink: "#FFFFFF"
    property bool dashed: false
    readonly property real length: Math.sqrt((line.x2 - line.x1) * (line.x2 - line.x1) + (line.y2 - line.y1) * (line.y2 - line.y1))
    x: line.x1
    y: line.y1 - 0.75
    width: line.length
    height: 1.5
    transformOrigin: Item.Left
    rotation: Math.atan2(line.y2 - line.y1, line.x2 - line.x1) * 180 / Math.PI
    Behavior on opacity { NumberAnimation { duration: 120; easing.type: Easing.OutQuad } }
    Rectangle {
        visible: !line.dashed
        anchors.fill: parent
        radius: 0.75
        antialiasing: true
        color: line.ink
    }
    Repeater {
        model: line.dashed ? Math.ceil(line.length / 10) : 0
        Rectangle {
            required property int index
            x: index * 10
            width: Math.min(4, line.length - index * 10)
            height: 1.5
            antialiasing: true
            color: line.ink
        }
    }
}
