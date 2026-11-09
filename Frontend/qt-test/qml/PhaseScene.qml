pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit

// A fixture: a ringed body with moons and a pad glyph at one fractional phase inside a clipping Flickable and uncut, beside the same under no clip and the same body's shapes drawn plainly by Qt, for the phase test (EmuSen_Kit.md §7.34).
Window {
    id: scene
    objectName: "scene"
    width: 400
    height: 340
    color: Kit.Tokens.surface
    component Pair: Item {
        width: 160
        height: 120
        Kit.Body {
            x: 30.37
            y: 20.25
            width: 46.3
            height: 46.3
            hue: 210
            focused: true
            ring: Kit.Rings.Big
            moons: 2
        }
        Kit.PadGlyph {
            x: 100.62
            y: 30.5
            width: 41.7
            height: 41.7
            style: Kit.GlyphStyles.Filled
            button: 0
            ink: Kit.Tokens.accent
        }
    }
    Flickable {
        x: 20
        y: 40
        width: 160
        height: 120
        clip: true
        interactive: false
        contentWidth: 160
        contentHeight: 120
        Pair {}
    }
    Pair {
        x: 220
        y: 40
    }
    // The oracle: kit-core's shapes for the same body, each a plain Shape as Qt draws it, with nothing of the kit's own around them.
    Item {
        id: bare
        x: 30.37
        y: 220.25
        width: 46.3
        height: 46.3
        Kit.BodyShapes {
            id: shapes
            hue: 210
            focused: true
            ring: Kit.Rings.Big
            moons: 2
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
                    xScale: bare.width / shapes.unitWidth
                    yScale: bare.height / shapes.unitHeight
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
}
