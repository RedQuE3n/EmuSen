import QtQuick
import EmuSen.Kit as Kit

// Board-pixel exact text scaled into canvas units with Chrome baseline alignment (R-20261115-24).
Item {
    id: root
    property real unit: 1
    property alias text: words.text
    property alias color: words.color
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
    property bool boardLine: false
    property real boardLineHeight: 0
    readonly property real boardAscent: Math.round(metrics.ascent)
    readonly property real boardDescent: Math.round(metrics.descent)
    readonly property real boardBox: root.boardLineHeight > 0 ? root.boardLineHeight : root.boardAscent + root.boardDescent
    readonly property real boardBaseline: root.boardAscent + Math.floor((root.boardBox - root.boardAscent - root.boardDescent) / 2)
    implicitWidth: words.implicitWidth * root.unit
    implicitHeight: (root.boardLine ? root.boardBox * Math.max(1, words.lineCount) : words.implicitHeight) * root.unit
    FontMetrics {
        id: metrics
        font: words.font
    }
    Text {
        id: words
        objectName: root.objectName
        width: root.width === root.implicitWidth ? words.implicitWidth : root.width / root.unit
        y: root.boardLine ? (root.boardBaseline - words.baselineOffset) * root.unit : 0
        height: root.boardLine ? words.implicitHeight : root.height / root.unit
        scale: root.unit
        transformOrigin: Item.TopLeft
        font.family: Kit.Tokens.faceText
        font.pixelSize: root.size
        font.weight: root.weight
        font.letterSpacing: root.spacing
    }
}
