import QtQuick
import EmuSen.Kit as Kit

// One core's planet mark, its name set by the test (EmuSen_Kit.md §7.115).
Window {
    objectName: "scene"
    visible: true
    width: 200
    height: 200
    color: "#000000"
    Kit.PlanetMark {
        objectName: "mark"
        x: 40
        y: 40
        width: 96
        height: 96
    }
}
