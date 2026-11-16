import QtQuick
import EmuSen.Kit as Kit

// Two of the kit's glows on white, alike but for the second's offset down, as a tile's resting shadow on the boards (EmuSen_Kit.md §7.119); the sizes are test inputs.
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 800
    height: 400
    color: "#FFFFFF"
    property real offsetY: 10
    Kit.Glow {
        objectName: "plain"
        x: 100
        y: 150
        width: 200
        height: 100
        radius: 12
        color: "#73000000"
        blur: 20
    }
    Kit.Glow {
        objectName: "offset"
        x: 500
        y: 150
        width: 200
        height: 100
        radius: 12
        color: "#73000000"
        blur: 20
        offsetY: scene.offsetY
    }
}
