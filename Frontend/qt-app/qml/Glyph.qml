import QtQuick
import QtQuick.Shapes

// One of the design's drawn marks, a path in a 24-unit box from the models' Marks, stroked or filled in one ink (EmuSen_Interface.md §24.1, §24.6).
Item {
    id: root
    property string path
    property color ink: "#FFFFFF"
    property real strokeWidth: 1.8
    property bool filled: false
    Accessible.role: Accessible.Graphic
    Shape {
        width: 24
        height: 24
        scale: Math.min(root.width, root.height) / 24
        transformOrigin: Item.TopLeft
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            fillColor: root.filled ? root.ink : "transparent"
            strokeColor: root.filled ? "transparent" : root.ink
            strokeWidth: root.filled ? -1 : root.strokeWidth
            fillRule: ShapePath.OddEvenFill
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            PathSvg { path: root.path }
        }
    }
}
