import QtQuick
import EmuSen.Kit as Kit

// One corona disc on black in a box wider than it is high, drawn three times the design's size so its rim spans pixels; the box and the scale are test inputs (EmuSen_Kit.md §7.116).
Window {
    objectName: "scene"
    visible: true
    width: 1900
    height: 1900
    color: "#000000"
    Item {
        anchors.centerIn: parent
        width: 120
        height: 80
        scale: 3
        Kit.CoronaDisc {
            objectName: "disc"
            anchors.fill: parent
        }
    }
}
