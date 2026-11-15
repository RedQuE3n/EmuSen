pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
import EmuSen.Kit as Kit

// The eclipse's dark disc in its box, its rim and its glows round it, drawn past the box as a box-shadow is; every measure a token or a property (EmuSen_Kit.md §7.116).
Item {
    id: root
    // The disc's fill.
    property color fill: Kit.Tokens.surface
    // The rim's colour: a game's corona where one is given, the look's otherwise.
    property color corona: Kit.Tokens.coronaColor
    // The rim's width outside the disc.
    property real rim: Kit.Tokens.coronaRim
    // The glows under the rim, the first on top, as a box-shadow lists them.
    property list<Kit.CoronaGlow> glows: [
        Kit.CoronaGlow {
            blur: Kit.Tokens.coronaGlowBlur
            spread: Kit.Tokens.coronaGlowSpread
            color: Kit.Tokens.coronaGlowColor
        },
        Kit.CoronaGlow {
            blur: Kit.Tokens.coronaHaloBlur
            spread: Kit.Tokens.coronaHaloSpread
            color: Kit.Tokens.coronaHaloColor
        }
    ]
    readonly property real diameter: Math.min(root.width, root.height)
    // The software backend draws no MultiEffect; there Qt's own Canvas casts each glow, as RaisedShadow's shadow (EmuSen_Kit.md §7.55).
    readonly property bool software: GraphicsInfo.api === GraphicsInfo.Software
    Repeater {
        model: root.glows
        Item {
            id: glow
            required property int index
            required property Kit.CoronaGlow modelData
            objectName: "coronaGlow." + glow.index
            readonly property real blur: glow.modelData.blur
            readonly property real spread: glow.modelData.spread
            readonly property color colour: glow.modelData.color
            // How far past the grown circle the painted glow reaches: twice the blur, so nothing of it is cut.
            readonly property real reach: glow.spread + 2 * glow.blur
            anchors.centerIn: parent
            width: root.diameter
            height: root.diameter
            // Under the rim and the disc, each glow under those before it.
            z: -1 - glow.index
            Rectangle {
                id: caster
                anchors.fill: parent
                radius: width / 2
                color: glow.colour
                visible: false
            }
            MultiEffect {
                objectName: "coronaGlowEffect." + glow.index
                visible: !root.software
                source: caster
                anchors.fill: caster
                shadowEnabled: true
                shadowColor: glow.colour
                shadowOpacity: 1
                shadowScale: root.diameter > 0 ? (root.diameter / 2 + glow.spread) / (root.diameter / 2) : 1
                blurMax: glow.blur
                shadowBlur: 1
            }
            Canvas {
                objectName: "coronaGlowPainted." + glow.index
                visible: root.software
                x: -glow.reach
                y: -glow.reach
                width: glow.width + 2 * glow.reach
                height: glow.height + 2 * glow.reach
                contextType: "2d"
                property real grown: root.diameter / 2 + glow.spread
                property real softness: glow.blur
                property color ink: glow.colour
                onGrownChanged: requestPaint()
                onSoftnessChanged: requestPaint()
                onInkChanged: requestPaint()
                onWidthChanged: requestPaint()
                onPaint: {
                    // The grown circle is drawn off the canvas and only its shadow, offset back, lands on it: the glow without the circle.
                    const away = width + 2 * grown
                    const ctx = getContext(contextType)
                    ctx.reset()
                    ctx.shadowColor = ink
                    ctx.shadowBlur = softness
                    ctx.shadowOffsetX = away
                    ctx.shadowOffsetY = 0
                    // Opaque, so the glow's alpha is its colour's own.
                    ctx.fillStyle = Qt.rgba(0, 0, 0, 1)
                    ctx.beginPath()
                    ctx.arc(width / 2 - away, height / 2, grown, 0, 2 * Math.PI)
                    ctx.fill()
                }
            }
        }
    }
    Rectangle {
        objectName: "coronaRim"
        anchors.centerIn: parent
        width: root.diameter + 2 * root.rim
        height: root.diameter + 2 * root.rim
        radius: width / 2
        color: root.corona
        visible: root.rim > 0
    }
    Rectangle {
        objectName: "coronaDisc"
        anchors.centerIn: parent
        width: root.diameter
        height: root.diameter
        radius: width / 2
        color: root.fill
    }
}
