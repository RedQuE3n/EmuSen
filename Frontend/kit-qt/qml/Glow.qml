pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
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
        sourceComponent: Item {
            id: lit
            layer.enabled: true
            layer.effect: MultiEffect {
                maskEnabled: true
                maskInverted: true
                maskSource: hole
            }
            // The box grown by the spread and moved by the offset, its corners grown with it, blurred by a Gaussian of half the
            // blur radius, as CSS draws a box-shadow (CSS Backgrounds 3 §6.1.1, §6.1.2).
            Item {
                objectName: "glowShadow"
                anchors.fill: parent
                Item {
                    id: grown
                    x: root.reach - root.spread + root.offsetX
                    y: root.reach - root.spread + root.offsetY
                    width: root.width + 2 * root.spread
                    height: root.height + 2 * root.spread
                    Rectangle {
                        anchors.fill: parent
                        radius: Math.min(root.radius, Math.min(root.width, root.height) / 2) + root.spread
                        color: root.color
                        antialiasing: true
                    }
                }
                Loader {
                    active: root.blur > 0
                    sourceComponent: Component {
                        Kit.GaussianBlur {
                            source: grown
                            deviation: root.blur / 2
                            pixelScale: root.pixelScale
                        }
                    }
                }
            }
            Item {
                id: hole
                anchors.fill: parent
                visible: false
                layer.enabled: true
                Rectangle {
                    x: root.reach
                    y: root.reach
                    width: root.width
                    height: root.height
                    radius: root.radius
                    // Opaque: the mask reads only its alpha, a constant of the masking itself.
                    color: Qt.rgba(0, 0, 0, 1)
                    antialiasing: true
                }
            }
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
