import QtQuick
import EmuSen.Kit as Kit

// A fixture for the harness's own tests: a big-screen frame, a menu panel of eight rows that scrolls, and a help bar (EmuSen_Kit.md §7.9).
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    property int activated: -1
    property int form: Kit.Forms.BigScreen
    Kit.Frame {
        id: frame
        anchors.fill: parent
        form: scene.form
        focus: true
        help: ListModel {
            ListElement { button: 4; label: "Open" }
            ListElement { button: 5; label: "Back" }
        }
        Kit.MenuPanel {
            id: panel
            objectName: "panel"
            controller: frame.controller
            width: Kit.Tokens.cardWidth
            height: Kit.Tokens.rowHeight * 5 + Kit.Tokens.base * 4
            onActivated: (index) => scene.activated = index
            model: ListModel {
                ListElement { label: "Display"; value: "4:3" }
                ListElement { label: "Shader"; value: "CRT" }
                ListElement { label: "Sound"; value: "On" }
                ListElement { label: "Pads"; value: "2" }
                ListElement { label: "Saves"; value: "Auto" }
                ListElement { label: "Rewind"; value: "Off" }
                ListElement { label: "Speed"; value: "100%" }
                ListElement { label: "Region"; value: "Auto" }
            }
        }
    }
}
