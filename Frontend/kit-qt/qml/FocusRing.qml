pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// The focus mark around the box it fills: a gap, the ring and the glow under them, sized and coloured by the tokens (EmuSen_Kit.md §7.113).
Item {
    id: root
    enum Placement { Gapped, Tight, Edge }
    // The box's corner radius.
    property real radius
    // Where the ring stands to the box: past a gap of ringOffset, within ringOffset of it, or on its edge (§7.113).
    property int placement: FocusRing.Gapped
    // The ring as the box's own border, for a box that draws it in the Edge placement.
    readonly property real edgeWidth: Kit.Tokens.ringThickness
    readonly property color edgeColor: Kit.Tokens.accent
    // How far past the box the ring's inner edge stands.
    readonly property real inner: root.placement === FocusRing.Gapped ? Kit.Tokens.ringOffset
        : (root.placement === FocusRing.Tight ? Kit.Tokens.ringOffset - Kit.Tokens.ringThickness : -Kit.Tokens.ringThickness)
    readonly property real outer: root.inner + Kit.Tokens.ringThickness
    // How far past the box the glow's canvas reaches: the spread and twice the blur, as RaisedShadow's (§7.55), in whole units.
    readonly property real glowReach: Math.ceil(Kit.Tokens.glowSpread + 2 * Kit.Tokens.glowBlur)
    Loader {
        objectName: "focusGlow"
        active: root.visible && Kit.Tokens.glowColor.a > 0
        x: -root.glowReach
        y: -root.glowReach
        width: root.width + 2 * root.glowReach
        height: root.height + 2 * root.glowReach
        sourceComponent: Canvas {
            id: glow
            objectName: "focusGlowCanvas"
            property int paints: 0
            readonly property real reach: root.glowReach
            readonly property real corner: root.radius
            readonly property real spread: Kit.Tokens.glowSpread
            readonly property real blur: Kit.Tokens.glowBlur
            readonly property color ink: Kit.Tokens.glowColor
            contextType: "2d"
            onReachChanged: requestPaint()
            onCornerChanged: requestPaint()
            onInkChanged: requestPaint()
            onWidthChanged: requestPaint()
            onHeightChanged: requestPaint()
            onPaint: {
                const ctx = getContext(contextType)
                const away = glow.width
                ctx.reset()
                ctx.fillRule = Qt.OddEvenFill
                ctx.beginPath()
                ctx.rect(0, 0, glow.width, glow.height)
                ctx.roundedRect(glow.reach, glow.reach, glow.width - 2 * glow.reach, glow.height - 2 * glow.reach, glow.corner, glow.corner)
                ctx.clip()
                ctx.shadowColor = glow.ink
                ctx.shadowBlur = glow.blur
                ctx.shadowOffsetX = away
                ctx.fillStyle = Qt.rgba(glow.ink.r, glow.ink.g, glow.ink.b, 1)
                ctx.beginPath()
                ctx.roundedRect(glow.reach - glow.spread - away, glow.reach - glow.spread, glow.width - 2 * (glow.reach - glow.spread), glow.height - 2 * (glow.reach - glow.spread), glow.corner + glow.spread, glow.corner + glow.spread)
                ctx.fill()
                glow.paints = glow.paints + 1
            }
        }
    }
    Rectangle {
        objectName: "focusGap"
        visible: root.inner > 0 && Kit.Tokens.ringGapColor.a > 0
        x: -root.inner
        y: -root.inner
        width: root.width + 2 * root.inner
        height: root.height + 2 * root.inner
        radius: root.radius + root.inner
        color: "transparent"
        border.width: root.inner
        border.color: Kit.Tokens.ringGapColor
    }
    Rectangle {
        objectName: "focusBand"
        visible: root.placement !== FocusRing.Edge
        x: -root.outer
        y: -root.outer
        width: root.width + 2 * root.outer
        height: root.height + 2 * root.outer
        radius: root.radius + Kit.Tokens.ringOffset
        color: "transparent"
        border.width: Kit.Tokens.ringThickness
        border.color: Kit.Tokens.accent
    }
}
