import QtQuick
import EmuSen.Esde

// An element's box as the scene placed it: position, size, turn and opacity, each a role (EmuSen_Qt.md §4.8.4).
Item {
    id: root
    required property string elementId
    required property string name
    required property real boxX
    required property real boxY
    required property real boxWidth
    required property real boxHeight
    required property real turn
    required property real originX
    required property real originY
    required property real alpha
    property EsdeModel esde: null
    objectName: "element-" + root.name
    x: root.boxX
    y: root.boxY
    width: root.boxWidth
    height: root.boxHeight
    opacity: root.alpha
    transform: Rotation {
        origin.x: root.originX * root.width
        origin.y: root.originY * root.height
        angle: root.turn
    }
}
