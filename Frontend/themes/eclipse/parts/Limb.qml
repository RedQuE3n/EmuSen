import QtQuick
import QtQuick.Shapes

// An ellipse filling its box, the Settings boards' limb and its rim: SVG's two half arcs, as border-radius 50% draws a box that is no square (G:91; EmuSen_Qt.md §3.4.11).
Shape {
    id: root
    property color fill: "transparent"
    // The fill, which the tests read.
    readonly property color paint: root.fill
    preferredRendererType: Shape.CurveRenderer
    ShapePath {
        fillColor: root.fill
        strokeColor: "transparent"
        PathSvg {
            path: "M 0 " + root.height / 2 + " A " + root.width / 2 + " " + root.height / 2 + " 0 1 0 " + root.width + " " + root.height / 2
                + " A " + root.width / 2 + " " + root.height / 2 + " 0 1 0 0 " + root.height / 2 + " Z"
        }
    }
}
