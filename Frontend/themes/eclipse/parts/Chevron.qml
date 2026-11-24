import QtQuick
import QtQuick.Shapes

// A row's chevron as the Settings boards draw it: a 24-unit path stroked 2.4 with round ends, in a box of 22 board pixels (G:34, 38, 44).
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    property bool back: false
    property color ink: "transparent"
    // The fill, which the tests read.
    readonly property color paint: root.ink
    // The box in board pixels and the path's own box, the boards' viewBox: constants of the design.
    property real side: 22
    readonly property real box: 24
    width: root.side * root.unit
    height: width
    Shape {
        width: root.box
        height: root.box
        scale: root.width / root.box
        transformOrigin: Item.TopLeft
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            fillColor: "transparent"
            strokeColor: root.ink
            strokeWidth: 2.4
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            // The boards' "M9 5l7 7-7 7", or "M15 5l-7 7 7 7" pointing back.
            PathMove { x: root.back ? 15 : 9; y: 5 }
            PathLine { x: root.back ? 8 : 16; y: 12 }
            PathLine { x: root.back ? 15 : 9; y: 19 }
        }
    }
}
