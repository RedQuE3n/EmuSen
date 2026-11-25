import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Pad button glyph in Eclipse card board pixels, following active pad family and A/B swap.
Item {
    id: root
    property real unit: 1
    property int button: Kit.Pad.Accept
    property int family: Kit.Families.Xbox
    property bool lead: false
    property bool leads: true
    property bool keyPairs: false
    property int pair: -1
    readonly property bool drawsPair: root.keyPairs && root.button === Kit.Pad.Left
    readonly property bool keyed: root.drawsPair && root.pair === Kit.Pad.Right
    readonly property bool filled: root.keyPairs ? root.button === Kit.Pad.Accept : root.lead && root.leads
    property real side: 34
    property real letterSize: 16
    property real leadLetterSize: 17
    property int leadWeight: Font.Bold
    property int letterWeight: Font.Normal
    property real wordSize: 15
    property real padding: 12
    property real boxSide: 30
    property real boxRadius: 8
    property real boxPadding: 10
    property bool boxMenu: false
    readonly property bool shoulderOrTrigger: root.button === Kit.Pad.PageUp || root.button === Kit.Pad.PageDown || root.button === Kit.Pad.First || root.button === Kit.Pad.Last
    readonly property bool boxed: root.keyed || (root.boxMenu && root.button === Kit.Pad.Menu) || root.shoulderOrTrigger
    property color ring: Kit.Tokens.muted
    property color ink: Kit.Tokens.text
    property color leadFill: Kit.Tokens.accent
    property color leadInk: Kit.Tokens.overAccent
    readonly property real line: 2 * root.unit
    readonly property string letter: mark.letter
    readonly property bool named: root.keyed || root.letter.length > 1 || (root.letter.length === 0 && (root.button === Kit.Pad.Menu || root.button === Kit.Pad.Options))
    readonly property string word: root.keyed ? Words.keyPair : root.letter.length > 1 ? root.letter : mark.name.toUpperCase()
    readonly property real outer: (root.boxed ? root.boxSide : root.side) * root.unit + (root.filled ? 0 : 2 * root.line)
    readonly property real inset: (root.boxed ? root.boxPadding : root.padding) * root.unit
    readonly property real corner: root.boxed ? root.boxRadius * root.unit : (root.named ? (root.side / 2) * root.unit : root.width / 2)
    implicitWidth: root.named ? metrics.advanceWidth * root.unit + 2 * root.inset + 2 * root.line : root.outer
    implicitHeight: root.outer
    width: implicitWidth
    height: implicitHeight
    Rectangle {
        anchors.fill: parent
        radius: root.corner
        color: root.filled ? root.leadFill : "transparent"
        border.width: root.filled ? 0 : root.line
        border.color: root.ring
    }
    Kit.PadGlyph {
        id: mark
        anchors.centerIn: parent
        width: root.side * root.unit * 0.7
        height: width
        visible: !root.named && root.letter.length === 0
        family: root.family
        button: root.button
        ink: root.ink
        swap: Kit.PadRouter.swapAcceptBack
    }
    TextMetrics {
        id: metrics
        font.family: Kit.Tokens.faceText
        font.pixelSize: root.named ? root.wordSize : (root.filled ? root.leadLetterSize : root.letterSize)
        font.weight: root.filled ? root.leadWeight : root.letterWeight
        text: root.named ? root.word : root.letter
    }
    Shape {
        width: root.width / root.unit
        height: root.height / root.unit
        scale: root.unit
        transformOrigin: Item.TopLeft
        visible: root.named || root.letter.length > 0
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            fillColor: root.filled ? root.leadInk : root.ink
            strokeColor: "transparent"
            fillRule: ShapePath.WindingFill
            PathText {
                x: (root.width / root.unit - metrics.tightBoundingRect.width) / 2 - metrics.tightBoundingRect.x
                y: (root.height / root.unit - metrics.tightBoundingRect.height) / 2
                font: metrics.font
                text: metrics.text
            }
        }
    }
}
