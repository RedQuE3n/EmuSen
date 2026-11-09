import QtQuick
import EmuSen.Theme

// A Pointer, one of the library's three shelves as a bright star of the triangle that closes the first row: All games' name above its star in Label 24, the others' below (constellation.md §3).
ChartNode {
    id: pointer
    required property int index
    required property string key
    required property string name
    required property string line
    required property int count
    nodeIndex: pointer.consoleCount + pointer.index
    isPointer: true
    readonly property real hue: -1
    objectName: "pointer." + pointer.index
    Accessible.name: pointer.name
    Accessible.description: pointer.line
    x: pointer.cx - 90
    y: pointer.cy - (pointer.index === 0 ? 43 : 16)
    width: 180
    height: 66
    Star {
        visible: pointer.inView
        x: (pointer.width - width) / 2
        y: pointer.index === 0 ? 27 : 0
        lit: pointer.activeFocus
    }
    Text {
        objectName: "pointerName." + pointer.index
        y: pointer.index === 0 ? 0 : 36
        width: pointer.width
        horizontalAlignment: Text.AlignHCenter
        text: pointer.name
        elide: Text.ElideRight
        color: Tokens.textStrong
        font.family: pointer.faces?.condensed ?? ""
        font.weight: Font.Medium
        font.pixelSize: 24
        font.letterSpacing: 1.5
        font.capitalization: Font.AllUppercase
        lineHeight: 28.8
        lineHeightMode: Text.FixedHeight
    }
}
