import QtQuick
import EmuSen.Kit as Kit

// One of the kit's glows at a corona disc's halo, its side and colour test inputs: the disc's 2100 board pixels are 1400 units, drawn 1400 pixels in a 1280 × 800 window and 3780 in a 3840 × 2160 one (EmuSen_Kit.md §7.113).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 1280
    height: 800
    color: "#000000"
    property real side: 0
    property color ink: "#47ECA0BA"
    // The halo's blur and spread at the window's unit, Eclipse's tokens times the side over 1400.
    readonly property real unit: scene.side / 1400
    Kit.Glow {
        objectName: "halo"
        visible: scene.side > 0
        x: -scene.side / 2
        y: 100
        width: scene.side
        height: scene.side
        radius: scene.side / 2
        color: scene.ink
        blur: 113.33 * scene.unit
        spread: 30.67 * scene.unit
    }
}
