import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// Cover picture or procedural Eclipse radial planetary horizon plate.
Item {
    id: root
    required property real px
    property real radius: 14
    property string source: ""
    property int slot: 0
    property int planet: -1
    property real pixelScale: 1

    readonly property int hueSeed: Math.abs(root.slot * 47 + Math.max(0, root.planet) * 73) % 360
    readonly property real hueNorm: root.hueSeed / 360
    readonly property color bgColor: Qt.hsla(root.hueNorm, 0.42, 0.18, 1)
    readonly property color waveColor: Qt.hsla(((root.hueSeed + 34) % 360) / 360, 0.56, 0.36, 1)
    readonly property color discColor: Qt.hsla(((root.hueSeed + 18) % 360) / 360, 0.72, 0.80, 1)
    readonly property real corner: Math.min(root.radius * root.px, width / 2, height / 2)

    Rectangle {
        anchors.fill: parent
        visible: root.source.length === 0
        radius: root.corner
        color: root.bgColor
    }

    Shape {
        anchors.fill: parent
        visible: root.source.length === 0
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeColor: "transparent"
            fillGradient: RadialGradient {
                centerX: root.width * 0.22
                centerY: root.height * 1.05
                focalX: root.width * 0.22
                focalY: root.height * 1.05
                centerRadius: Math.max(1, Math.hypot(root.width * 0.78, root.height * 1.05) * 0.62)
                GradientStop { position: 0.0; color: root.waveColor }
                GradientStop { position: 0.96; color: root.waveColor }
                GradientStop { position: 1.0; color: "transparent" }
            }
            startX: root.corner; startY: 0
            PathLine { x: root.width - root.corner; y: 0 }
            PathArc { x: root.width; y: root.corner; radiusX: root.corner; radiusY: root.corner }
            PathLine { x: root.width; y: root.height - root.corner }
            PathArc { x: root.width - root.corner; y: root.height; radiusX: root.corner; radiusY: root.corner }
            PathLine { x: root.corner; y: root.height }
            PathArc { x: 0; y: root.height - root.corner; radiusX: root.corner; radiusY: root.corner }
            PathLine { x: 0; y: root.corner }
            PathArc { x: root.corner; y: 0; radiusX: root.corner; radiusY: root.corner }
        }
    }

    Shape {
        anchors.fill: parent
        visible: root.source.length === 0
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeColor: "transparent"
            fillGradient: RadialGradient {
                centerX: root.width * 0.78
                centerY: root.height * 0.24
                focalX: root.width * 0.78
                focalY: root.height * 0.24
                centerRadius: Math.max(1, Math.min(root.width, root.height) * 0.28)
                GradientStop { position: 0.0; color: root.discColor }
                GradientStop { position: 0.96; color: root.discColor }
                GradientStop { position: 1.0; color: "transparent" }
            }
            startX: root.corner; startY: 0
            PathLine { x: root.width - root.corner; y: 0 }
            PathArc { x: root.width; y: root.corner; radiusX: root.corner; radiusY: root.corner }
            PathLine { x: root.width; y: root.height - root.corner }
            PathArc { x: root.width - root.corner; y: root.height; radiusX: root.corner; radiusY: root.corner }
            PathLine { x: root.corner; y: root.height }
            PathArc { x: 0; y: root.height - root.corner; radiusX: root.corner; radiusY: root.corner }
            PathLine { x: 0; y: root.corner }
            PathArc { x: root.corner; y: 0; radiusX: root.corner; radiusY: root.corner }
        }
    }

    RoundPicture {
        anchors.fill: parent
        visible: root.source.length > 0
        source: root.source
        radius: root.corner
        crop: true
        pixelScale: root.pixelScale
    }
}
