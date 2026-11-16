import QtQuick
import EmuSen.Kit as Kit

// Two of the kit's glows on black at the disc's halo's measures: the first recoloured by the test, the second painted afresh in the colour the first ends in; the values are test inputs (EmuSen_Kit.md §7.113).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 1400
    height: 700
    color: "#000000"
    property color ink: "#47ECA0BA"
    property real side: 300
    Kit.Glow {
        objectName: "recoloured"
        x: 200
        y: 200
        width: scene.side
        height: scene.side
        radius: scene.side / 2
        color: scene.ink
        blur: 113
        spread: 30
    }
    Kit.Glow {
        objectName: "fresh"
        x: 900
        y: 200
        width: 300
        height: 300
        radius: 150
        color: "#476A4CF0"
        blur: 113
        spread: 30
    }
}
