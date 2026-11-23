pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// Controllers' picture: the help bar's four face buttons as they are drawn now, A filled, 64 px each 18 apart in a panel 34 px deep (G:181-183; EmuSen_Qt.md §3.4.11).
Rectangle {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    readonly property real face: 64
    readonly property real pad: 34
    objectName: "asideFaces"
    width: parent ? parent.width : 0
    height: (root.face + 2 * root.pad) * root.unit
    radius: 22 * root.unit
    color: own.facesGround
    // Its 1 px line inside its edge (G:183).
    border.width: root.unit
    border.color: own.facesLine
    SettingsInk {
        id: own
    }
    // Centred by its own sums, as centring anchors round to whole pixels.
    Row {
        id: faces
        x: (root.width - faces.width) / 2
        y: (root.height - faces.height) / 2
        spacing: 18 * root.unit
        Repeater {
            model: [PadGlyph.Accept, PadGlyph.Back, PadGlyph.Screensaver, PadGlyph.Search]
            delegate: Glyph {
                required property int index
                required property int modelData
                objectName: "asideFace." + index
                unit: root.unit
                button: modelData
                keyPairs: true
                // A filled face is 64 across; a ringed one 64 with its 2 px line, as the board's border-box draws it.
                side: filled ? root.face : root.face - 4
                letterSize: 26
                leadLetterSize: 26
                letterWeight: Font.Bold
            }
        }
    }
}
