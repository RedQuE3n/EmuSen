pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects

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
    // RectangularShadow's blur for a CSS blur: 1.2, a constant of Qt's RectangularShadow page, the closest of those measured (§7.113).
    readonly property real cardBlur: 1.2
    // The software backend renders no shader effect; there Qt's Context2D paints the glow (§7.113).
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
            RectangularShadow {
                objectName: "glowShadow"
                x: root.reach
                y: root.reach
                width: root.width
                height: root.height
                radius: Math.min(root.radius, Math.min(root.width, root.height) / 2)
                color: root.color
                spread: root.spread
                offset: Qt.vector2d(root.offsetX, root.offsetY)
                blur: root.blur * root.cardBlur
                cached: false
                antialiasing: true
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
        sourceComponent: Canvas {
            id: paint
            objectName: "glowCanvas"
            property int paints: 0
            readonly property real reach: root.reach
            readonly property real corner: Math.min(root.radius, Math.min(root.width, root.height) / 2)
            readonly property real spread: root.spread
            readonly property real blur: root.blur
            readonly property color ink: root.color
            readonly property real offsetX: root.offsetX
            readonly property real offsetY: root.offsetY
            contextType: "2d"
            onReachChanged: requestPaint()
            onCornerChanged: requestPaint()
            onSpreadChanged: requestPaint()
            onBlurChanged: requestPaint()
            onInkChanged: requestPaint()
            onOffsetXChanged: requestPaint()
            onOffsetYChanged: requestPaint()
            onWidthChanged: requestPaint()
            onHeightChanged: requestPaint()
            onPaint: {
                // The grown shape is drawn a canvas's width away and only its shadow, offset back, lands, clipped to outside the box.
                const ctx = getContext(contextType)
                const away = paint.width
                const grown = paint.spread
                // The shape's corners grow by the spread, less where they are smaller than it: CSS Backgrounds 3 §6.1.1.
                const share = grown > 0 && paint.corner < grown ? 1 + Math.pow(paint.corner / grown - 1, 3) : 1
                const rounded = paint.corner + grown * share
                ctx.reset()
                ctx.fillRule = Qt.OddEvenFill
                ctx.beginPath()
                ctx.rect(0, 0, paint.width, paint.height)
                ctx.roundedRect(paint.reach, paint.reach, paint.width - 2 * paint.reach, paint.height - 2 * paint.reach, paint.corner, paint.corner)
                ctx.clip()
                ctx.shadowColor = paint.ink
                ctx.shadowBlur = paint.blur
                ctx.shadowOffsetX = away + paint.offsetX
                ctx.shadowOffsetY = paint.offsetY
                ctx.fillStyle = Qt.rgba(paint.ink.r, paint.ink.g, paint.ink.b, 1)
                ctx.beginPath()
                ctx.roundedRect(paint.reach - grown - away, paint.reach - grown, paint.width - 2 * (paint.reach - grown), paint.height - 2 * (paint.reach - grown), rounded, rounded)
                ctx.fill()
                paint.paints = paint.paints + 1
            }
        }
    }
}
