import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.App

// Program-drawn Eclipse core planet sphere with radial gradient and glow (EclipseStartConsole:46).
Item {
    id: root
    required property real unit
    required property int planet
    property real side: 22
    property real glowBlur: 10
    property real glowSpread: 2
    property real glowAlpha: 0.55
    property real pixelScale: 1
    readonly property int k: Math.max(0, root.planet) % 8
    readonly property color light: k === 0 ? "#F4F1EC" : k === 1 ? "#FFEBB5" : k === 2 ? "#FFC2A8" : k === 3 ? "#D6E6FF" : k === 4 ? "#C8F0DA" : k === 5 ? "#D9F5C4" : k === 6 ? "#FFFFFF" : "#CFE0FF"
    readonly property color middle: k === 0 ? "#C9CCD8" : k === 1 ? "#E8B34B" : k === 2 ? "#D4553F" : k === 3 ? "#7FA6D9" : k === 4 ? "#4FA87A" : k === 5 ? "#6FAE5A" : k === 6 ? "#BFE3F5" : "#3F6FE0"
    readonly property color shade: k === 0 ? "#5E6272" : k === 1 ? "#7A5416" : k === 2 ? "#6B1E16" : k === 3 ? "#2C4670" : k === 4 ? "#1C4A36" : k === 5 ? "#274A1E" : k === 6 ? "#4A7890" : "#162C6B"
    width: root.side * root.unit
    height: width
    Kit.Glow {
        objectName: "panePlanetGlow"
        visible: root.planet >= 0
        anchors.fill: parent
        radius: width / 2
        color: Qt.rgba(root.middle.r, root.middle.g, root.middle.b, root.glowAlpha)
        blur: root.glowBlur * root.unit
        spread: root.glowSpread * root.unit
        pixelScale: root.pixelScale
    }
    Shape {
        visible: root.planet >= 0
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeWidth: -1
            fillGradient: RadialGradient {
                centerX: 0.32 * root.width
                centerY: 0.30 * root.width
                centerRadius: Math.hypot(0.68, 0.70) * root.width
                focalX: 0.32 * root.width
                focalY: 0.30 * root.width
                focalRadius: 0
                GradientStop { position: 0; color: root.light }
                GradientStop { position: 0.52; color: root.middle }
                GradientStop { position: 1; color: root.shade }
            }
            PathAngleArc {
                centerX: root.width / 2
                centerY: root.width / 2
                radiusX: root.width / 2
                radiusY: root.width / 2
                startAngle: 0
                sweepAngle: 360
            }
        }
    }
    Kit.PlanetMark {
        visible: root.planet === -1
        anchors.fill: parent
        mark: Kit.PlanetMarks.libretro
        libretro: true
        ink: Kit.Tokens.muted
        cut: Kit.Tokens.surface
    }
    Rectangle {
        visible: root.planet === -2
        anchors.fill: parent
        radius: width / 2
        color: "transparent"
        border.width: 2 * root.unit
        border.color: ThemeInk.pillBorder
    }
}
