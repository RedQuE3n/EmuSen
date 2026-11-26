pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit

// Rounded cover picture or procedural Eclipse radial swatch with optional drop shadow.
Item {
    id: root
    required property real unit
    property real pixelScale: 1
    property real radius: 12
    property string cover: ""
    property var swatch: null
    property int slot: 0
    readonly property color fallbackBg: slot === 2 ? "#2C1A57" : slot === 1 ? "#5B1C1A" : "#12806F"
    readonly property color fallbackDot: slot === 2 ? "#E3CCFF" : slot === 1 ? "#FFD1B0" : "#B8FFE9"
    readonly property color fallbackWave: slot === 2 ? "#6C3FB8" : slot === 1 ? "#B5432E" : "#6A4CF0"
    property color bgColor: swatch?.bg ?? fallbackBg
    property color dotColor: swatch?.dot ?? fallbackDot
    property color waveColor: swatch?.wave ?? fallbackWave
    property color shadowColor: "transparent"
    property real shadowBlur: 0
    property real shadowOffsetY: 0

    component GradientShape: Shape {
        id: gradShape
        required property color fillStop
        required property real cx
        required property real cy
        required property real reach
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer
        readonly property real cr: Math.min(root.radius * root.unit, gradShape.width / 2, gradShape.height / 2)
        ShapePath {
            strokeColor: "transparent"
            fillGradient: RadialGradient {
                centerX: gradShape.cx
                centerY: gradShape.cy
                focalX: gradShape.cx
                focalY: gradShape.cy
                centerRadius: Math.max(1, gradShape.reach)
                GradientStop { position: 0.0; color: gradShape.fillStop }
                GradientStop { position: 0.96; color: gradShape.fillStop }
                GradientStop { position: 1.0; color: "transparent" }
            }
            startX: gradShape.cr
            startY: 0
            PathLine { x: gradShape.width - gradShape.cr; y: 0 }
            PathArc { x: gradShape.width; y: gradShape.cr; radiusX: gradShape.cr; radiusY: gradShape.cr }
            PathLine { x: gradShape.width; y: gradShape.height - gradShape.cr }
            PathArc { x: gradShape.width - gradShape.cr; y: gradShape.height; radiusX: gradShape.cr; radiusY: gradShape.cr }
            PathLine { x: gradShape.cr; y: gradShape.height }
            PathArc { x: 0; y: gradShape.height - gradShape.cr; radiusX: gradShape.cr; radiusY: gradShape.cr }
            PathLine { x: 0; y: gradShape.cr }
            PathArc { x: gradShape.cr; y: 0; radiusX: gradShape.cr; radiusY: gradShape.cr }
        }
    }

    Kit.Glow {
        anchors.fill: parent
        visible: root.shadowColor.a > 0
        radius: root.radius * root.unit
        color: root.shadowColor
        blur: root.shadowBlur * root.unit
        offsetY: root.shadowOffsetY * root.unit
        pixelScale: root.pixelScale
    }

    Rectangle {
        anchors.fill: parent
        visible: root.cover.length === 0
        radius: root.radius * root.unit
        color: root.bgColor
    }

    GradientShape {
        visible: root.cover.length === 0
        fillStop: root.waveColor
        cx: root.width * 0.20
        cy: root.height * 1.10
        reach: Math.hypot(root.width * 0.80, root.height * 1.10) * 0.58
    }

    GradientShape {
        visible: root.cover.length === 0
        fillStop: root.dotColor
        cx: root.width * 0.78
        cy: root.height * 0.22
        reach: Math.hypot(root.width * 0.78, root.height * 0.78) * 0.22
    }

    Kit.RoundPicture {
        anchors.fill: parent
        visible: root.cover.length > 0
        radius: root.radius * root.unit
        pixelScale: root.pixelScale
        crop: true
        source: root.cover
    }
}
