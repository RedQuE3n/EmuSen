import QtQuick
import QtQuick.Effects
import EmuSen.Theme

// Meridian's sky: the kit's night sky and glow in the dark variant; by day the pale gradient with no stars and a day moon at (W - 136, 96) (meridian.md §1.4).
Item {
    id: root
    property real glowHue: -1
    property real glowX: 0.5
    property real glowY: 0.4
    property real starsOpacity: 1
    property real moonOpacity: 1
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
        objectName: "daySky"
        anchors.fill: parent
        visible: root.light
        readonly property bool backdrop: true
        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0; color: "#EEF1F6" }
                GradientStop { position: 0.5; color: "#E6EBF3" }
                GradientStop { position: 1; color: "#DFE6F1" }
            }
        }
        RectangularShadow {
            anchors.fill: dayMoon
            radius: 36
            blur: 40
            opacity: root.moonOpacity
            color: "#80FFFFFF"
        }
        Rectangle {
            id: dayMoon
            objectName: "dayMoon"
            x: root.width - 136 - 36
            y: 96 - 36
            width: 72
            height: 72
            radius: 36
            // Paler than the spec's 50%, which on the day sky reads as a disc over the heads at the right.
            opacity: 0.25 * root.moonOpacity
            color: "#FFFFFF"
        }
    }
}
