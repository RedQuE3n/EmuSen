pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// The glow or shadow of the rounded box this item fills, outside the box only, as a CSS outer box-shadow draws it (EmuSen_Kit.md §7.113, §7.119).
Item {
    id: root
    // The box's corner radius.
    property real radius
    // The glow's colour, its alpha the glow's strongest.
    property color color: "transparent"
    // The CSS blur radius: the glow falls off as a Gaussian of a standard deviation half of it.
    property real blur
    // How far past the box the glow's shape is grown before it is blurred.
    property real spread
    // How far the shadow's shape is moved from the box, as a box-shadow's offsets; the glow of 0, 0 stands round it (§7.119).
    property real offsetX: 0
    property real offsetY: 0
    // How far past the box the glow is drawn: the spread, twice the blur and the offset, in whole units.
    readonly property real reach: Math.ceil(root.spread + 2 * root.blur + Math.max(Math.abs(root.offsetX), Math.abs(root.offsetY)))
    // The card's blur as a share of the given blur: 1 since the glow is drawn as a Gaussian (EmuSen_Kit.md §7.119).
    readonly property real cardBlur: 1
    // Scene pixels to one of this item's units, so the card's blur runs at the pixels it draws.
    property real pixelScale: 1
    // The software backend renders no shader effect; there the kit's GlowPaint draws the glow (§7.113).
    readonly property bool software: GraphicsInfo.api === GraphicsInfo.Software
    // A glow of no blur and no spread lies under its box alone, where it is not drawn: nothing is made.
    readonly property bool glows: root.visible && root.color.a > 0 && root.width > 0 && root.height > 0 && (root.blur > 0 || root.spread > 0 || root.offsetX !== 0 || root.offsetY !== 0)
    Loader {
        objectName: "glowCast"
        active: root.glows && !root.software
        x: -root.reach
        y: -root.reach
        width: root.width + 2 * root.reach
        height: root.height + 2 * root.reach
        sourceComponent: ShaderEffect {
            objectName: "glowShadow"
            readonly property color color: root.color
            readonly property vector2d boxSize: Qt.vector2d(root.width, root.height)
            readonly property vector2d offset: Qt.vector2d(root.offsetX, root.offsetY)
            readonly property real radius: root.radius
            readonly property real spread: root.spread
            readonly property real blur: root.blur
            readonly property real reach: root.reach
            readonly property real pixelScale: root.pixelScale
            fragmentShader: "qrc:/qt/qml/EmuSen/Kit/shaders/glow.frag.qsb"
        }
    }
    Loader {
        objectName: "glowPainted"
        active: root.glows && root.software
        x: -root.reach
        y: -root.reach
        width: root.width + 2 * root.reach
        height: root.height + 2 * root.reach
        // Painted in native code, its blur made once a shape and tinted once a colour (EmuSen_Kit.md §7.113).
        sourceComponent: Kit.GlowPaint {
            objectName: "glowCanvas"
            color: root.color
            corner: Math.min(root.radius, Math.min(root.width, root.height) / 2)
            blur: root.blur
            spread: root.spread
            reach: root.reach
            offsetX: root.offsetX
            offsetY: root.offsetY
        }
    }
}
