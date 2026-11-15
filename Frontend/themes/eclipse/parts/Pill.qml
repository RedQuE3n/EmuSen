import QtQuick
import EmuSen.Theme

// A pill button: a pad button's glyph and its label, filled in the accent or outlined in the line colour, round at both ends; every measure in board pixels (Main:61-71, EclipseLibrary:51, EclipseEmpty:46; EmuSen_Qt.md §3.4.8).
Walkable {
    id: root
    // A board pixel in canvas units, Board.unit.
    property real unit: 1
    required property int button
    required property string label
    property bool filled: false
    // Whether a filled pill draws the boards' halo and glow (Main:61); Consoles' Open draws neither (EclipseConsoles:90).
    property bool glows: true
    property real tall: 72
    property real glyphSide: 44
    property real labelSize: 24
    property real lead: 14
    property real trail: 30
    property real gap: 14
    // The entry's action is the host's to take, so a press is the button's own press.
    property bool live: true
    // The pill's place in its row, which names its parts for a reader and the tests.
    property int place
    // The pill's one colour: its fill when filled, else its outline.
    readonly property color paint: root.filled ? Tokens.accent : Tokens.border
    // Round at both ends, as every pill on the boards is.
    readonly property real radius: root.height / 2
    readonly property Item glyph: glyphItem
    readonly property Item words: wordsItem
    width: wordsItem.x + wordsItem.implicitWidth + root.trail * root.unit
    height: root.tall * root.unit
    opacity: root.live ? 1 : 0.38
    Accessible.name: root.label
    Rectangle {
        anchors.fill: parent
        radius: root.radius
        color: root.filled ? root.paint : "transparent"
        border.color: root.filled ? "transparent" : root.paint
        border.width: root.filled ? 0 : Tokens.hoverBorder
    }
    // The filled pill's halo and glow (Main:61, EclipseEmpty:46): rgba(246, 231, 216, 0.18) 6 px out, and 46 px of blur 6 px out
    // in rgba(240, 180, 196, 0.45); constants of the design, as the boards draw them.
    Glow {
        objectName: "buttonHalo." + root.place
        anchors.fill: parent
        visible: root.filled && root.glows
        radius: root.radius
        color: "#2EF6E7D8"
        spread: 6 * root.unit
    }
    Glow {
        objectName: "buttonGlow." + root.place
        anchors.fill: parent
        visible: root.filled && root.glows
        radius: root.radius
        color: "#73F0B4C4"
        blur: 46 * root.unit
        spread: 6 * root.unit
    }
    FocusRing {
        anchors.fill: parent
        radius: root.radius
        visible: root.activeFocus
    }
    PadGlyph {
        id: glyphItem
        objectName: "buttonGlyph." + root.place
        x: root.lead * root.unit
        y: (root.height - height) / 2
        width: root.glyphSide * root.unit
        height: width
        family: Device.family
        button: root.button
        ink: root.filled ? Tokens.overAccent : Tokens.muted
    }
    Text {
        id: wordsItem
        objectName: "buttonLabel." + root.place
        x: glyphItem.x + glyphItem.width + root.gap * root.unit
        y: (root.height - height) / 2
        text: root.label
        color: root.filled ? Tokens.overAccent : Tokens.text
        font.pixelSize: Type.size(root.labelSize * root.unit, Tokens.textScale)
        font.weight: root.filled ? Font.DemiBold : Font.Normal
    }
}
