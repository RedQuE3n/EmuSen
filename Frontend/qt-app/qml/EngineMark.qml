import QtQuick
import QtQuick.Shapes
import EmuSen.Models

// The mark before an engine's name (EmuSen_Interface.md §24.6): a sky-named core's symbol or its maker's glyph in the console's ink, or libretro's square badge with the project's own pixel alien.
Item {
    id: root
    property string mark
    property bool libretro
    property color ink: "#FFFFFF"
    Accessible.role: Accessible.Graphic
    Accessible.name: root.mark
    implicitWidth: 15
    implicitHeight: 15
    Glyph {
        anchors.fill: parent
        visible: !root.libretro
        path: Marks.path(root.mark)
        ink: root.ink
        strokeWidth: 2
    }
    Shape {
        visible: root.libretro
        width: 11
        height: 11
        scale: Math.min(root.width, root.height) / 11
        transformOrigin: Item.TopLeft
        ShapePath {
            fillColor: "#C9C9D0"
            strokeColor: "transparent"
            PathSvg { path: Marks.badge }
        }
        ShapePath {
            fillColor: "#26262B"
            strokeColor: "transparent"
            PathSvg { path: Marks.alien }
        }
    }
}
