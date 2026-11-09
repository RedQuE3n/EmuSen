import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// Orrery's sky: the kit's night sky and glow in the dark variant; by day the Atlas, a paper gradient with no stars and the glow as a warm wash (orrery.md §1.4).
Item {
    id: root
    property real glowHue: -1
    property real glowX: 0.5
    property real glowY: 0.4
    property real starsOpacity: 1
    readonly property bool light: Tokens.variant === Tokens.Light
    Sky {
        objectName: "nightSky"
        anchors.fill: parent
        visible: !root.light
        glowHue: root.glowHue
        glowX: root.glowX
        glowY: root.glowY
        starsOpacity: root.starsOpacity
    }
    Scene {
        objectName: "atlas"
        anchors.fill: parent
        visible: root.light
        readonly property bool backdrop: true
        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0; color: "#F7F1E6" }
                GradientStop { position: 0.5; color: "#F3EADA" }
                GradientStop { position: 1; color: "#EFE4D2" }
            }
        }
        Shape {
            objectName: "dayGlow"
            visible: root.glowHue >= 0
            anchors.fill: parent
            ShapePath {
                strokeColor: "transparent"
                fillGradient: RadialGradient {
                    centerX: root.width * root.glowX
                    centerY: root.height * root.glowY
                    centerRadius: 378
                    focalX: root.width * root.glowX
                    focalY: root.height * root.glowY
                    GradientStop { position: 0; color: Qt.hsla(Math.max(0, root.glowHue) / 360, 0.7, 0.5, 0.2) }
                    GradientStop { position: 0.45; color: Qt.hsla(Math.max(0, root.glowHue) / 360, 0.7, 0.4, 0.07) }
                    GradientStop { position: 0.75; color: Qt.hsla(Math.max(0, root.glowHue) / 360, 0.7, 0.4, 0) }
                }
                PathRectangle {
                    width: root.width
                    height: root.height
                }
            }
        }
    }
}
