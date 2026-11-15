import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// Four focusable kinds of the kit, each ringed by FocusRing, and each placement beside the ring it replaced, drawn as the kit drew it before 2026-11-14 (EmuSen_Kit.md §7.113).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 1280
    height: 800
    color: "#000000"
    readonly property real ringOffset: Kit.Tokens.ringOffset
    readonly property real ringThickness: Kit.Tokens.ringThickness
    readonly property real glowBlur: Kit.Tokens.glowBlur
    readonly property real glowSpread: Kit.Tokens.glowSpread
    readonly property color accent: Kit.Tokens.accent
    readonly property color gapColor: Kit.Tokens.ringGapColor
    readonly property color glowColor: Kit.Tokens.glowColor
    // The box every test control is given, in whole pixels so a sample lands inside one band.
    readonly property size box: Qt.size(160, 48)
    Kit.FrameController {
        id: controller
    }
    Button {
        objectName: "button"
        x: 100
        y: 100
        width: scene.box.width
        height: scene.box.height
    }
    Kit.ActionButton {
        objectName: "action"
        controller: controller
        x: 400
        y: 100
        width: scene.box.width
        height: scene.box.height
    }
    TextField {
        objectName: "field"
        x: 700
        y: 100
        width: scene.box.width
        height: scene.box.height
    }
    Kit.CoverTile {
        objectName: "cover"
        x: 1000
        y: 200
        width: scene.box.width
        height: scene.box.width
        hue: 200
        lifts: false
        lit: false
    }
    // Six more sites a FocusRing marks, placed out of the measured rows: their placements are read, not their pixels.
    Switch {
        objectName: "switch"
        x: 100
        y: 250
    }
    CheckBox {
        objectName: "check"
        x: 250
        y: 250
    }
    RadioButton {
        objectName: "radio"
        x: 400
        y: 250
    }
    ComboBox {
        objectName: "combo"
        x: 550
        y: 250
        model: 2
    }
    Slider {
        objectName: "slider"
        x: 750
        y: 250
    }
    TabButton {
        objectName: "tab"
        x: 700
        y: 330
    }
    // Before and after, each pair 300 pixels apart on one row: the Tight ring (ActionButton's and PageFrame's), the Gapped
    // ring (CoverTile's, its literals 7, 14 and 4), and the Edge ring (the style's controls' own border).
    Item {
        objectName: "tightBefore"
        x: 100
        y: 400
        width: scene.box.width
        height: scene.box.height
        Rectangle {
            anchors.fill: parent
            anchors.margins: -Kit.Tokens.ringOffset
            radius: Kit.Tokens.rowRadius + Kit.Tokens.ringOffset
            color: "transparent"
            border.width: Kit.Tokens.ringThickness
            border.color: Kit.Tokens.accent
        }
    }
    Kit.FocusRing {
        objectName: "tightAfter"
        x: 400
        y: 400
        width: scene.box.width
        height: scene.box.height
        radius: Kit.Tokens.rowRadius
        placement: Kit.FocusRing.Tight
    }
    Item {
        objectName: "gappedBefore"
        x: 100
        y: 520
        width: scene.box.width
        height: scene.box.height
        Rectangle {
            x: -7
            y: -7
            width: parent.width + 14
            height: parent.height + 14
            radius: Kit.Tokens.tileRadius + 4
            color: "transparent"
            border.color: Kit.Tokens.accent
            border.width: Kit.Tokens.ringThickness
        }
    }
    Kit.FocusRing {
        objectName: "gappedAfter"
        x: 400
        y: 520
        width: scene.box.width
        height: scene.box.height
        radius: Kit.Tokens.tileRadius
        placement: Kit.FocusRing.Gapped
    }
    Rectangle {
        objectName: "edgeBefore"
        x: 100
        y: 640
        width: scene.box.width
        height: scene.box.height
        radius: Kit.Tokens.rowRadius
        color: Kit.Tokens.surfaceRaised
        border.width: Kit.Tokens.ringThickness
        border.color: Kit.Tokens.accent
    }
    Rectangle {
        objectName: "edgeAfter"
        x: 400
        y: 640
        width: scene.box.width
        height: scene.box.height
        radius: Kit.Tokens.rowRadius
        color: Kit.Tokens.surfaceRaised
        border.width: edge.edgeWidth
        border.color: edge.edgeColor
        Kit.FocusRing {
            id: edge
            anchors.fill: parent
            radius: parent.radius
            placement: Kit.FocusRing.Edge
        }
    }
}
