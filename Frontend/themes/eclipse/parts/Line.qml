import QtQuick
import EmuSen.Theme

// Words at a board's own type size, which a font's whole pixel size cannot hold in canvas units: the Text at the board's
// pixels, scaled by the board's measure, its box the canvas's (EmuSen_Qt.md §3.4.8, R-20261115-24).
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    property real unit: 1
    property alias text: words.text
    property alias color: words.color
    // The board's size, weight, letter spacing and line height, in board pixels.
    property real size: 16
    property int weight: Font.Normal
    property real spacing: 0
    property alias lineHeight: words.lineHeight
    property alias lineHeightMode: words.lineHeightMode
    property alias capitalization: words.font.capitalization
    property alias family: words.font.family
    property alias elide: words.elide
    property alias wrapMode: words.wrapMode
    property alias maximumLineCount: words.maximumLineCount
    property alias horizontalAlignment: words.horizontalAlignment
    readonly property alias lineCount: words.lineCount
    readonly property alias truncated: words.truncated
    readonly property Text words: words
    // Placed as a board's CSS places a line: the box its line-height's, the words' baseline where Chrome puts it (EmuSen_Qt.md §3.5.3, the words' baselines).
    property bool boardLine: false
    // The board's line-height in board pixels; 0 is CSS's "normal", the face's own ascent and descent.
    property real boardLineHeight: 0
    // Chrome's rounded ascent and descent, the line box, and the baseline under the box's top with the half-leading floored.
    readonly property real boardAscent: Math.round(metrics.ascent)
    readonly property real boardDescent: Math.round(metrics.descent)
    readonly property real boardBox: root.boardLineHeight > 0 ? root.boardLineHeight : root.boardAscent + root.boardDescent
    readonly property real boardBaseline: root.boardAscent + Math.floor((root.boardBox - root.boardAscent - root.boardDescent) / 2)
    implicitWidth: words.implicitWidth * root.unit
    implicitHeight: (root.boardLine ? root.boardBox : words.implicitHeight) * root.unit
    FontMetrics {
        id: metrics
        font: words.font
    }
    Text {
        id: words
        // The Line's name, so a reader and the tests find the words themselves by it.
        objectName: root.objectName
        // Its own width where the box is its words', which the division by the measure could leave a hair short of and so elide.
        width: root.width === root.implicitWidth ? words.implicitWidth : root.width / root.unit
        y: root.boardLine ? (root.boardBaseline - words.baselineOffset) * root.unit : 0
        height: root.boardLine ? words.implicitHeight : root.height / root.unit
        scale: root.unit
        transformOrigin: Item.TopLeft
        font.family: Tokens.faceText
        font.pixelSize: root.size
        font.weight: root.weight
        font.letterSpacing: root.spacing
    }
}
