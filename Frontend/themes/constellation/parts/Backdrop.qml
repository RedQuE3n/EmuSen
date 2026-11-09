pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// Constellation's sky: the kit's night sky and glow by night; by day the Chart's paper gradient with its own stars at 35% in the ink; the field drifts (-6, -3) over 60 s and back while resting (constellation.md §1.4, §1.5).
Item {
    id: root
    property real glowHue: -1
    property real glowX: 0.5
    property real glowY: 0.4
    property real starsOpacity: 1
    // Resting motion: off under reduced motion and on battery.
    property bool resting: true
    readonly property bool light: Tokens.variant === Tokens.Light
    property real drift: 0
    clip: true
    SequentialAnimation on drift {
        running: root.resting && root.visible
        loops: Animation.Infinite
        NumberAnimation { from: 0; to: 1; duration: 60000 }
        NumberAnimation { from: 1; to: 0; duration: 60000 }
    }
    Sky {
        objectName: "nightSky"
        x: -6 * root.drift
        y: -3 * root.drift
        width: root.width + 6
        height: root.height + 3
        visible: !root.light
        glowHue: root.glowHue
        glowX: root.glowX
        glowY: root.glowY
        starsOpacity: root.starsOpacity
    }
    Scene {
        objectName: "chartSky"
        anchors.fill: parent
        visible: root.light
        readonly property bool backdrop: true
        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0; color: "#EEF2F8" }
                GradientStop { position: 0.55; color: "#E6EBF4" }
                GradientStop { position: 1; color: "#DDE5F0" }
            }
        }
        Item {
            x: -6 * root.drift
            y: -3 * root.drift
            width: root.width + 6
            height: root.height + 3
            opacity: 0.35 * root.starsOpacity
            Repeater {
                model: 110
                Rectangle {
                    id: dayStar
                    required property int index
                    readonly property real size: (dayStar.index * 37) % 7 === 0 ? 2 : 1.2
                    x: (dayStar.index * 7919 + 211) % 1440 * (root.width + 6) / 1440
                    y: (dayStar.index * 104729 + 97) % 740
                    width: size
                    height: size
                    radius: size / 2
                    opacity: 0.2 + ((dayStar.index * 53) % 56) / 100
                    color: "#1F2A44"
                }
            }
        }
    }
}
