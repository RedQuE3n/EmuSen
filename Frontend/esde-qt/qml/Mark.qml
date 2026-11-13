pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Esde

// One of the scene's built-in marks, drawn by the scene graph from its parts in pixels, as the kit's Body draws kit-core's shapes (EmuSen_Qt.md §4.8).
Item {
    id: root
    property alias mark: shapes.mark
    property alias color: shapes.color
    readonly property alias aspect: shapes.aspect
    readonly property alias baselineShare: shapes.baseline
    MarkShapes {
        id: shapes
        width: root.width
        height: root.height
    }
    Repeater {
        model: shapes
        Shape {
            id: drawn
            required property string outline
            required property color fillInk
            required property color strokeInk
            required property real strokeSize
            anchors.fill: parent
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                fillColor: drawn.fillInk
                fillRule: ShapePath.OddEvenFill
                strokeColor: drawn.strokeInk
                strokeWidth: drawn.strokeSize > 0 ? drawn.strokeSize : -1
                capStyle: ShapePath.RoundCap
                joinStyle: ShapePath.RoundJoin
                PathSvg { path: drawn.outline }
            }
        }
    }
}
