import QtQuick
import EmuSen.Theme

// The help line on a screen's foot: the kit's HelpBar laid out as the boards draw it, and whatever a screen sets at its right (Main:99-108, EclipseConsoles:99-105, EclipseLibrary:82-92; EmuSen_Qt.md §3.4.8).
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    required property var controller
    property alias model: bar.model
    property color ink: "transparent"
    property real size: 21
    property real spacing: 34
    property real glyphGap: 10
    property real glyphSide: 34
    // The line's centre over the board's foot, in board pixels from the top.
    property real middle: 1137
    property real inset: 96
    default property alias side: sideSlot.data
    readonly property Item bar: bar
    x: root.inset * root.unit
    width: parent.width - 2 * x
    height: Math.max(bar.height, sideSlot.childrenRect.height)
    y: root.middle * root.unit - height / 2
    HelpBar {
        id: bar
        objectName: "help"
        y: (root.height - height) / 2
        controller: root.controller
        family: Device.family
        fitted: true
        ruled: false
        color: "transparent"
        labelColor: root.ink
        iconColor: Tokens.muted
        labelFont.pixelSize: Type.size(root.size * root.unit, Tokens.textScale)
        labelFont.family: Tokens.faceText
        entrySpacing: root.spacing * root.unit
        glyphGap: root.glyphGap * root.unit
        glyphSize: root.glyphSide * root.unit
    }
    Item {
        id: sideSlot
        x: root.width - width
        y: (root.height - height) / 2
        width: childrenRect.width
        height: childrenRect.height
    }
}
