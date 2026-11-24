pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// A console or core mark: shaded planet with glow (planet >= 0), libretro 20 px checkerboard (-1), or 2 px hollow ring (-2) (EclipseSettingsConsoles:47, EclipseSettingsConsole:68).
Item {
    id: root
    required property real unit
    required property int planet
    property real side: 48
    property real corner: 9
    property real glowBlur: 18
    property real glowSpread: 3
    property real glowAlpha: 0x88 / 255
    property real pixelScale: 1
    width: root.side * root.unit
    height: width
    SettingsInk {
        id: own
    }
    PlanetInk {
        id: planetInk
        place: Math.max(0, root.planet)
    }
    Glow {
        visible: root.planet >= 0
        anchors.fill: parent
        radius: width / 2
        color: Qt.rgba(planetInk.middle.r, planetInk.middle.g, planetInk.middle.b, root.glowAlpha)
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
                GradientStop { position: 0; color: planetInk.light }
                GradientStop { position: 0.52; color: planetInk.middle }
                GradientStop { position: 1; color: planetInk.shade }
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
    // Libretro 20 px checkerboard clipped to its rounded square (EclipseSettingsConsoles:47, EclipseSettingsConsole:68).
    Image {
        visible: root.planet === -1 && root.side <= 64
        anchors.fill: parent
        source: "../assets/checkerboard-48.svg"
        sourceSize: Qt.size(192, 192)
        smooth: true
    }
    Image {
        visible: root.planet === -1 && root.side > 64
        anchors.fill: parent
        source: "../assets/checkerboard-92.svg"
        sourceSize: Qt.size(368, 368)
        smooth: true
    }
    Rectangle {
        visible: root.planet === -2
        anchors.fill: parent
        radius: width / 2
        color: "transparent"
        border.width: 2 * root.unit
        border.color: own.ringMuted
    }
}
