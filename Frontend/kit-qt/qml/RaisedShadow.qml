import QtQuick
import QtQuick.Effects
import EmuSen.Kit as Kit

// The one shadow a raised shape of this item's box and corner radius casts, from the shadow tokens, on every backend (EmuSen_Interface.md §5.4, EmuSen_Kit.md §7.55).
Item {
    id: root
    // The raised shape's corner radius.
    property real radius
    // The software backend draws no MultiEffect; there Qt's own Canvas casts the shadow (EmuSen_Kit.md §7.55).
    readonly property bool software: GraphicsInfo.api === GraphicsInfo.Software
    // How far past the shape the software shadow is painted: twice the blur and the larger offset, so nothing of it is cut.
    readonly property real reach: 2 * Kit.Tokens.shadowBlur + Math.max(Math.abs(Kit.Tokens.shadowOffsetX), Math.abs(Kit.Tokens.shadowOffsetY))
    Rectangle {
        id: caster
        anchors.fill: parent
        radius: root.radius
        color: Kit.Tokens.shadowColor
        visible: false
    }
    MultiEffect {
        objectName: "raisedShadow"
        visible: !root.software
        source: caster
        anchors.fill: caster
        shadowEnabled: true
        shadowColor: Kit.Tokens.shadowColor
        shadowOpacity: Kit.Tokens.shadowOpacity
        shadowHorizontalOffset: Kit.Tokens.shadowOffsetX
        shadowVerticalOffset: Kit.Tokens.shadowOffsetY
        blurMax: Kit.Tokens.shadowBlur
        shadowBlur: 1
    }
    Canvas {
        objectName: "raisedShadowPainted"
        visible: root.software
        x: -root.reach
        y: -root.reach
        width: root.width + 2 * root.reach
        height: root.height + 2 * root.reach
        contextType: "2d"
        property real corner: root.radius
        onCornerChanged: requestPaint()
        onWidthChanged: requestPaint()
        onHeightChanged: requestPaint()
        onPaint: {
            const ink = Kit.Tokens.shadowColor
            const ctx = getContext(contextType)
            ctx.reset()
            ctx.shadowColor = Qt.rgba(ink.r, ink.g, ink.b, Kit.Tokens.shadowOpacity)
            ctx.shadowBlur = Kit.Tokens.shadowBlur
            ctx.shadowOffsetX = Kit.Tokens.shadowOffsetX
            ctx.shadowOffsetY = Kit.Tokens.shadowOffsetY
            ctx.fillStyle = ink
            ctx.beginPath()
            ctx.roundedRect(root.reach, root.reach, root.width, root.height, root.radius, root.radius)
            ctx.fill()
        }
    }
}
