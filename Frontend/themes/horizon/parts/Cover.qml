import QtQuick
import QtQuick.Effects
import EmuSen.Theme

// A cover on the land: the kit's CoverTile whole at its shape, its shadow (0 10 24 at 45%) and a 1-unit white 14% edge; the ring and the lift are the strip item's (EmuSen_Qt.md §3.4.3).
Item {
    id: root
    property alias source: tile.source
    property alias title: tile.title
    property alias hue: tile.hue
    property alias favourite: tile.favourite
    property alias pixelScale: tile.pixelScale
    RectangularShadow {
        x: tile.paintedX
        y: tile.paintedY + 10
        width: tile.paintedWidth
        height: tile.paintedHeight
        radius: Tokens.tileRadius
        blur: 24
        color: Tokens.variant === Tokens.Dark ? "#73000000" : "#40000000"
    }
    CoverTile {
        id: tile
        anchors.fill: parent
        mark: Marks.heart
        lit: false
        lifts: false
    }
    Rectangle {
        x: tile.paintedX
        y: tile.paintedY
        width: tile.paintedWidth
        height: tile.paintedHeight
        radius: Tokens.tileRadius
        color: "transparent"
        border.width: 1
        border.color: "#24FFFFFF"
    }
}
