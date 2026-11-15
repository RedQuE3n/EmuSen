import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// The kit's one glow on black: a focus ring round a box that draws nothing, a disc's glow, and three controls the focus walks; the places and sizes are test inputs (EmuSen_Kit.md §7.113).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 1920
    height: 1200
    color: "#000000"
    // Whether the lone ring shows.
    property bool ringOn: true
    // Which of the disc's two glows the disc draws: 0 the glow, 1 the halo.
    property int halo: 0
    readonly property color glowColor: Kit.Tokens.glowColor
    readonly property color discColor: scene.halo === 0 ? Kit.Tokens.coronaGlowColor : Kit.Tokens.coronaHaloColor
    Kit.FocusRing {
        objectName: "ring"
        visible: scene.ringOn
        x: 100
        y: 100
        width: 160
        height: 48
        radius: Kit.Tokens.buttonRadius
        placement: Kit.FocusRing.Gapped
    }
    Kit.Glow {
        objectName: "disc"
        x: 760
        y: 400
        width: 400
        height: 400
        radius: 200
        color: scene.discColor
        blur: scene.halo === 0 ? Kit.Tokens.coronaGlowBlur : Kit.Tokens.coronaHaloBlur
        spread: scene.halo === 0 ? Kit.Tokens.coronaGlowSpread : Kit.Tokens.coronaHaloSpread
    }
    Kit.FrameController {
        id: controller
    }
    Button {
        objectName: "button"
        x: 100
        y: 400
        width: 160
        height: 48
    }
    Switch {
        objectName: "switch"
        x: 100
        y: 550
    }
    Kit.ActionButton {
        objectName: "action"
        controller: controller
        x: 100
        y: 700
        width: 160
        height: 48
    }
}
