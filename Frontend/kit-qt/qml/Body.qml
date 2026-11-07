pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit

// A world's body drawn by the scene graph from kit-core's shapes: one Shape per row, scaled out of the unit box (EmuSen_Kit.md §7.5).
Item {
    id: root
    property alias shape: shapes.shape
    property alias hue: shapes.hue
    property alias focused: shapes.focused
    property alias craters: shapes.craters
    property alias bands: shapes.bands
    property alias cap: shapes.cap
    property alias ring: shapes.ring
    property alias moons: shapes.moons
    implicitWidth: shapes.unitWidth
    implicitHeight: shapes.unitHeight
    BodyShapes {
        id: shapes
    }
    Repeater {
        model: shapes
        Shape {
            id: drawn
            required property var outline
            required property color fillInk
            required property bool hasGradient
            required property real gradientX
            required property real gradientY
            required property real gradientRadius
            required property real stop0At
            required property color stop0Ink
            required property real stop1At
            required property color stop1Ink
            required property real stop2At
            required property color stop2Ink
            required property real stop3At
            required property color stop3Ink
            required property color strokeInk
            required property real strokeSize
            required property real inkOpacity
            width: shapes.unitWidth
            height: shapes.unitHeight
            opacity: drawn.inkOpacity
            preferredRendererType: Shape.CurveRenderer
            transform: Scale {
                xScale: root.width / shapes.unitWidth
                yScale: root.height / shapes.unitHeight
            }
            RadialGradient {
                id: radial
                centerX: drawn.gradientX
                centerY: drawn.gradientY
                centerRadius: drawn.gradientRadius
                focalX: drawn.gradientX
                focalY: drawn.gradientY
                GradientStop { position: drawn.stop0At; color: drawn.stop0Ink }
                GradientStop { position: drawn.stop1At; color: drawn.stop1Ink }
                GradientStop { position: drawn.stop2At; color: drawn.stop2Ink }
                GradientStop { position: drawn.stop3At; color: drawn.stop3Ink }
            }
            ShapePath {
                fillColor: drawn.fillInk
                fillGradient: drawn.hasGradient ? radial : null
                strokeColor: drawn.strokeInk
                strokeWidth: drawn.strokeSize > 0 ? drawn.strokeSize : -1
                capStyle: ShapePath.RoundCap
                joinStyle: ShapePath.RoundJoin
                PathPolyline { path: drawn.outline }
            }
        }
    }
}
