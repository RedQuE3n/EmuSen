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
    // All games' name stands above its star, or beside it where the chart's top would cut it (the compact tier's first row).
    readonly property bool above: pointer.index === 0 && pointer.cy - 43 >= 0
    readonly property bool beside: pointer.index === 0 && !pointer.above
    x: pointer.beside ? pointer.cx - 16 : pointer.cx - 90
    y: pointer.cy - (pointer.above ? 43 : 16)
    width: 180
    height: pointer.beside ? 32 : 66
    Star {
        visible: pointer.inView
        x: pointer.beside ? 0 : (pointer.width - width) / 2
        y: pointer.above ? 27 : 0
        lit: pointer.activeFocus
    }
    Text {
        objectName: "pointerName." + pointer.index
        x: pointer.beside ? 40 : 0
        y: pointer.above ? 0 : (pointer.beside ? 1.6 : 36)
        width: pointer.beside ? pointer.width - 40 : pointer.width
        horizontalAlignment: pointer.beside ? Text.AlignLeft : Text.AlignHCenter
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
