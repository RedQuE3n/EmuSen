import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// A pad button's glyph as the boards draw it (Main:63-69, 94-97): a lettered button's letter in the face inside a 2 px ring or
// on a filled disc, Menu and View named in a 2 px capsule, the shoulders and triggers named in a 2 px box (EclipseConsoles:103,
// Main:100), any other the kit's mark in
// the ring. The letter and the name are the kit's for the pad's family; every measure in board pixels (EmuSen_Qt.md §3.4.8).
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    property real unit: 1
    property int button
    property int family: Device.family
    // The screen's main action, drawn filled.
    property bool lead: false
    // The side inside the ring, the letter's and the name's size, and the capsule's padding, in board pixels.
    property real side: 34
    property real letterSize: 16
    property real leadLetterSize: 17
    property int leadWeight: Font.Bold
    property real wordSize: 15
    property real padding: 12
    // The shoulders' and triggers' box: its height inside the line, its corner and its padding (EclipseConsoles:103).
    property real boxSide: 30
    property real boxRadius: 8
    property real boxPadding: 10
    readonly property bool boxed: ([PadGlyph.PageUp, PadGlyph.PageDown, PadGlyph.First, PadGlyph.Last].indexOf(root.button) >= 0)
    property color ring: Tokens.muted
    property color ink: Tokens.text
    property color leadFill: Tokens.accent
    property color leadInk: Tokens.overAccent
    readonly property real line: 2 * root.unit
    // The kit's glyph for the button, which names it and draws a mark with no letter.
    readonly property string letter: mark.letter
    readonly property bool named: root.letter.length > 1 || (root.letter.length === 0 && (root.button === PadGlyph.Menu || root.button === PadGlyph.Options))
    readonly property string word: root.letter.length > 1 ? root.letter : mark.name.toUpperCase()
    readonly property real outer: (root.boxed ? root.boxSide : root.side) * root.unit + (root.lead ? 0 : 2 * root.line)
    // The name's padding either side and the shape's corner as drawn, in canvas units.
    readonly property real inset: (root.boxed ? root.boxPadding : root.padding) * root.unit
    readonly property real corner: root.boxed ? root.boxRadius * root.unit : (root.named ? (root.side / 2) * root.unit : root.width / 2)
    implicitWidth: root.named ? metrics.advanceWidth * root.unit + 2 * root.inset + 2 * root.line : root.outer
    implicitHeight: root.outer
    width: implicitWidth
    height: implicitHeight
    Rectangle {
        anchors.fill: parent
        radius: root.corner
        color: root.lead ? root.leadFill : "transparent"
        border.width: root.lead ? 0 : root.line
        border.color: root.ring
    }
    PadGlyph {
        id: mark
        anchors.centerIn: parent
        width: root.side * root.unit * 0.7
        height: width
        visible: !root.named && root.letter.length === 0
        family: root.family
        button: root.button
        ink: root.ink
    }
    TextMetrics {
        id: metrics
        font.family: Tokens.faceText
        // The board's size in board pixels; the shape below is scaled by the board's measure, as a font's whole pixel size cannot hold it in canvas units.
        font.pixelSize: root.named ? root.wordSize : (root.lead ? root.leadLetterSize : root.letterSize)
        font.weight: root.lead ? root.leadWeight : Font.Normal
        text: root.named ? root.word : root.letter
    }
    // The letter or the name drawn as the kit draws a glyph's letter, a path of the face's outline centred on the glyph.
    Shape {
        width: root.width / root.unit
        height: root.height / root.unit
        scale: root.unit
        transformOrigin: Item.TopLeft
        visible: root.named || root.letter.length > 0
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            fillColor: root.lead ? root.leadInk : root.ink
            strokeColor: "transparent"
            fillRule: ShapePath.WindingFill
            PathText {
                // Centred on its ink, as the kit's PadGlyph centres a letter; a capital's ink is the line's middle within a 25th of the size.
                x: (root.width / root.unit - metrics.tightBoundingRect.width) / 2 - metrics.tightBoundingRect.x
                y: (root.height / root.unit - metrics.tightBoundingRect.height) / 2
                font: metrics.font
                text: metrics.text
            }
        }
    }
}
