import QtQuick
import EmuSen.Theme

// The help line's Sections entry on its glass at the line's right, Help.sections in the kit's HelpBar (Main:101-105, EclipseTools:66; EmuSen_Qt.md §3.4.8).
Rectangle {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    required property var controller
    // The words' colour, the help line's.
    property color wordsColor: "transparent"
    // The glass's height and side padding, the words' size, and the glyphs' gap and side, in board pixels (Main:101-104).
    property real tall: 46
    property real pad: 18
    property real size: 21
    property real glyphGap: 10
    property real glyphSide: 30
    objectName: "helpSections"
    width: bar.width + 2 * root.pad * root.unit
    height: root.tall * root.unit
    radius: height / 2
    HelpBar {
        id: bar
        objectName: "helpSectionsBar"
        x: root.pad * root.unit
        y: (root.height - height) / 2
        model: Help.sections
        controller: root.controller
        family: Device.family
        fitted: true
        ruled: false
        color: "transparent"
        labelColor: root.wordsColor
        iconColor: Tokens.muted
        labelFont.pixelSize: Type.size(root.size * root.unit, Tokens.textScale)
        glyphGap: root.glyphGap * root.unit
        glyphSize: root.glyphSide * root.unit
    }
}
