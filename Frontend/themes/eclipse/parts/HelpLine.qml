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
    // Left and Right drawn as one key, as the Settings boards draw them (G:24).
    property bool keyPairs: false
    // Start drawn in the shoulders' rounded box rather than a capsule (EclipseSettingsCards:77).
    property bool boxMenu: false
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
        // The main action's glyph filled in the corona, its letter cut out of it, as the board draws A (Main:101).
        leadStyle: PadGlyph.Filled
        leadColor: Tokens.accent
        labelFont.family: Tokens.faceText
        // The words at the board's size, which a font's whole pixel size cannot hold in canvas units (parts/Line.qml).
        label: Component {
            Line {
                objectName: "helpWords-" + text
                unit: root.unit
                size: root.size
            }
        }
        entrySpacing: root.spacing * root.unit
        glyphGap: root.glyphGap * root.unit
        glyphSize: root.glyphSide * root.unit
        // The canvas's unit is no pixel, so the entries are centred exactly, as the board centres them (Main:93).
        snapped: false
        // The boards' glyphs: letters in the face in a 2 px ring, A filled, Menu and the shoulders named in a capsule (Main:94-97).
        glyph: Component {
            Glyph {
                unit: root.unit
                side: root.glyphSide
                ink: root.ink
                keyPairs: root.keyPairs
                boxMenu: root.boxMenu
            }
        }
    }
    Item {
        id: sideSlot
        x: root.width - width
        y: (root.height - height) / 2
        width: childrenRect.width
        height: childrenRect.height
    }
}
