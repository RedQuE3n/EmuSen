import QtQuick
import QtQuick.Shapes

// A row's pen icon as Edit Details draws it: a 24-unit path stroked 2 with round ends, in a 20 px box (EclipseGameEdit:51).
Item {
    id: root
    required property real unit
    property color ink: "transparent"
    readonly property color paint: root.ink
    property real side: 20
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
            strokeWidth: 2
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            PathMove { x: 4; y: 20 }
            PathLine { x: 8; y: 19 }
            PathLine { x: 19; y: 8 }
            PathLine { x: 16; y: 5 }
            PathLine { x: 5; y: 16 }
            PathLine { x: 4; y: 20 }
        }
        ShapePath {
            fillColor: "transparent"
            strokeColor: root.ink
            strokeWidth: 2
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            PathMove { x: 14; y: 7 }
            PathLine { x: 17; y: 10 }
        }
    }
}
