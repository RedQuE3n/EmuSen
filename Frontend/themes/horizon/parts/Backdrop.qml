import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// The sky and the land: the kit's night sky with its stars above the line only, or Dawn by day; the glow in a hue at the world's centre; the stars' drift, the one resting motion (EmuSen_Qt.md §3.4.3).
Item {
    id: root
    // The glow's hue, -1 for none, and its centre in canvas units.
    property real hue: -1
    property real glowX: root.width - 214
    property real glowY: 290
    // Where the stars end: the horizon line, or the foot of the screen on the splash.
    property real starsEnd: 440
    property real starsOpacity: 1
    readonly property bool night: Tokens.variant === Tokens.Dark
    // Resting motion stops under reduced motion and on battery (decided 2026-11-08).
    readonly property bool resting: !Tokens.reducedMotion && !(Device.battery >= 0 && !Device.charging)
    property real drift: 0
    readonly property color dawnHigh: "#F7E9E4"
    readonly property color dawnMiddle: "#EEE7EE"
    readonly property color dawnLow: "#DCE6F5"
    Item {
        objectName: "nightSky"
        readonly property bool backdrop: true
        visible: root.night
        x: -6 - 6 * root.drift
        y: -6 - 2 * root.drift
        width: root.width + 12
        height: root.height + 12
        Sky {
            anchors.fill: parent
            count: 0
            starsOpacity: 0
            glowHue: -1
        }
        Item {
            width: parent.width
            height: root.starsEnd + 6 + 2 * root.drift
            clip: true
            layer.enabled: GraphicsInfo.api === GraphicsInfo.Software
            Sky {
                objectName: "stars"
                width: root.width + 12
                height: root.height + 12
                glowHue: -1
                starsOpacity: root.starsOpacity
            }
        }
    }
    Rectangle {
        objectName: "dawnSky"
        readonly property bool backdrop: true
        visible: !root.night
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0; color: root.dawnHigh }
            GradientStop { position: 0.55; color: root.dawnMiddle }
            GradientStop { position: 1; color: root.dawnLow }
        }
    }
    // The glow as the kit's sky draws it (756 x 450, 0.2 at the centre), drawn here so Dawn has it too.
    Shape {
        id: glow
        objectName: "glow"
        readonly property bool backdrop: true
        // The last hue shown stays while the glow fades out.
        property real shownHue: 0
        Binding on shownHue {
            when: root.hue >= 0
            value: root.hue
            restoreMode: Binding.RestoreNone
        }
        x: root.glowX - 756
        y: root.glowY - 756
        width: 1512
        height: 1512
        opacity: root.hue < 0 ? 0 : 1
        preferredRendererType: Shape.CurveRenderer
        transform: Scale {
            origin.y: 756
            yScale: 450 / 756
        }
        Behavior on shownHue {
            NumberAnimation {
                duration: Tokens.pan
                easing.type: Easing.OutCubic
            }
        }
        Behavior on opacity {
            NumberAnimation {
                duration: Tokens.pan
                easing.type: Easing.OutCubic
            }
        }
        ShapePath {
            strokeWidth: -1
            strokeColor: "transparent"
            fillGradient: RadialGradient {
                centerX: 756
                centerY: 756
                centerRadius: 756
                focalX: 756
                focalY: 756
                GradientStop { position: 0; color: Qt.hsla(glow.shownHue / 360, 0.7, 0.5, 0.2) }
                GradientStop { position: 0.45; color: Qt.hsla(glow.shownHue / 360, 0.7, 0.4, 0.07) }
                GradientStop { position: 0.75; color: Qt.hsla(glow.shownHue / 360, 0.7, 0.4, 0) }
            }
            PathAngleArc {
                centerX: 756
                centerY: 756
                radiusX: 756
                radiusY: 756
                startAngle: 0
                sweepAngle: 360
            }
        }
    }
    // The drift: (-6, -2) units and back over 60 s each way, linear.
    SequentialAnimation on drift {
        running: root.resting && root.night && root.visible
        loops: Animation.Infinite
        NumberAnimation {
            from: 0
            to: 1
            duration: 60000
        }
        NumberAnimation {
            from: 1
            to: 0
            duration: 60000
        }
    }
}
