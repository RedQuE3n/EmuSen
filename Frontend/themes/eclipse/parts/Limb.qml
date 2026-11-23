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
        // SVG's "M 0 h/2 A w/2 h/2 0 1 0 w h/2 A w/2 h/2 0 1 0 0 h/2 Z": two large half arcs, the sweep flag 0 counter-clockwise.
        startX: 0
        startY: root.height / 2
        PathArc {
            x: root.width
            y: root.height / 2
            radiusX: root.width / 2
            radiusY: root.height / 2
            useLargeArc: true
            direction: PathArc.Counterclockwise
        }
        PathArc {
            x: 0
            y: root.height / 2
            radiusX: root.width / 2
            radiusY: root.height / 2
            useLargeArc: true
            direction: PathArc.Counterclockwise
        }
    }
}
