import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// The glow's parts for a look on the card: three controls drawn twice their size, or a corona disc of the question board's 780 px at three quarters, on the ground the boards draw under them, `surface` (EmuSen_Kit.md §7.113, §7.116).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 1280
    height: 800
    color: Kit.Tokens.surface
    // Which part shows: the controls, or the disc.
    property bool disc: false
    Kit.FrameController {
        id: controller
    }
    Item {
        visible: !scene.disc
        anchors.centerIn: parent
        width: 160
        height: 300
        scale: 2
        Button {
            objectName: "button"
            y: 0
            width: 160
            height: 48
        }
        Switch {
            objectName: "switch"
            y: 120
        }
        Kit.ActionButton {
            objectName: "action"
            controller: controller
            y: 240
            width: 160
            height: 48
        }
    }
    Item {
        visible: scene.disc
        anchors.centerIn: parent
        // EclipseDialog.dc.html:32's 780 px disc on a 1200-pixel board (canvas.json), in design units: test inputs.
        width: 780 * Kit.CanvasUnits.designHeight / 1200
        height: width
        scale: 0.75
        Kit.CoronaDisc {
            objectName: "corona"
            anchors.fill: parent
        }
    }
}
