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
    implicitWidth: words.implicitWidth * root.unit
    implicitHeight: words.implicitHeight * root.unit
    Text {
        id: words
        // The Line's name, so a reader and the tests find the words themselves by it.
        objectName: root.objectName
        // Its own width where the box is its words', which the division by the measure could leave a hair short of and so elide.
        width: root.width === root.implicitWidth ? words.implicitWidth : root.width / root.unit
        height: root.height / root.unit
        scale: root.unit
        transformOrigin: Item.TopLeft
        font.family: Tokens.faceText
        font.pixelSize: root.size
        font.weight: root.weight
        font.letterSpacing: root.spacing
    }
}
