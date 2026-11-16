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
    // A disc given its own glows, in a corner the first disc's halo does not reach; its box, glow and colour are test inputs.
    Kit.CoronaDisc {
        objectName: "ownDisc"
        x: 60
        y: 60
        width: 40
        height: 40
        glows: [
            Kit.CoronaGlow {
                blur: 2
                spread: 4
                color: "#FF00FF00"
            }
        ]
    }
    // A disc whose one glow is deleted while the disc lives, as a dropped engine deletes a list's entries before their delegates.
    Kit.CoronaDisc {
        objectName: "bareDisc"
        x: 1800
        y: 60
        width: 40
        height: 40
        glows: [
            Kit.CoronaGlow {
                objectName: "goneGlow"
                blur: 2
                spread: 4
                color: "#FF00FF00"
            }
        ]
    }
}
