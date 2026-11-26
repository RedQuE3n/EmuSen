import QtQuick
import EmuSen.Kit as Kit

// Frame immediate PadRouter pump on pendingChanged and held-direction auto-repeat.
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 640
    height: 480
    color: "#000000"
    property bool holdDown: false
    property bool tapUp: false
    property bool tapAccept: false
    property int activatedCount: 0
    onHoldDownChanged: Kit.PadRouter.push(Kit.Pad.Down, scene.holdDown)
    onTapUpChanged: if (scene.tapUp) {
        Kit.PadRouter.push(Kit.Pad.Up, true)
        Kit.PadRouter.push(Kit.Pad.Up, false)
    }
    onTapAcceptChanged: if (scene.tapAccept) {
        Kit.PadRouter.push(Kit.Pad.Accept, true)
        Kit.PadRouter.push(Kit.Pad.Accept, false)
    }
    Kit.Frame {
        id: frame
        objectName: "frame"
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: true
        Column {
            spacing: 12
            Kit.ActionButton {
                id: first
                objectName: "first"
                controller: frame.controller
                width: 200
                height: 48
                focus: true
                KeyNavigation.down: second
                onActivated: scene.activatedCount += 1
            }
            Kit.ActionButton {
                id: second
                objectName: "second"
                controller: frame.controller
                width: 200
                height: 48
                KeyNavigation.up: first
                KeyNavigation.down: third
                onActivated: scene.activatedCount += 1
            }
            Kit.ActionButton {
                id: third
                objectName: "third"
                controller: frame.controller
                width: 200
                height: 48
                KeyNavigation.up: second
            }
        }
    }
    Component.onCompleted: frame.forceActiveFocus()
}
