pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// Controllers' picture: the help bar's four face buttons as they are drawn now, A filled, 64 px each 18 apart in a panel 34 px deep (G:181-183; EmuSen_Qt.md §3.4.11).
Rectangle {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    // A face's side and the panel's depth around it, board pixels (G:182-183).
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
    // A face as the help bar draws its button now: filled where it leads, else ringed, 64 px with its 2 px line as the board's border-box draws it, its letter bold at 26 px.
    component Face: Glyph {
        keyPairs: true
        side: filled ? 64 : 64 - 4
        letterSize: 26
        leadLetterSize: 26
        letterWeight: Font.Bold
    }
    // Centred by its own sums, as centring anchors round to whole pixels.
    Row {
        id: faces
        x: (root.width - faces.width) / 2
        y: (root.height - faces.height) / 2
        spacing: 18 * root.unit
        Face {
            objectName: "asideFace.0"
            unit: root.unit
            button: PadGlyph.Accept
        }
        Face {
            objectName: "asideFace.1"
            unit: root.unit
            button: PadGlyph.Back
        }
        Face {
            objectName: "asideFace.2"
            unit: root.unit
            button: PadGlyph.Screensaver
        }
        Face {
            objectName: "asideFace.3"
            unit: root.unit
            button: PadGlyph.Search
        }
    }
}
