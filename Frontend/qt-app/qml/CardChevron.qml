import QtQuick
import QtQuick.Shapes

// Row chevron in a 24-unit viewBox stroked 2.4 with round caps (EclipseListCard:39-40).
Item {
    id: root
    required property real unit
    property bool back: false
    property color ink: "transparent"
    readonly property color paint: root.ink
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
            PathMove { x: root.back ? 15 : 9; y: 5 }
            PathLine { x: root.back ? 8 : 16; y: 12 }
            PathLine { x: root.back ? 15 : 9; y: 19 }
        }
    }
}
