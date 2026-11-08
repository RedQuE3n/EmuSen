pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit

// The night sky: kit-core's gradient, its stars as antialiased discs, as the draft's canvas draws them, and the one soft glow, a backdrop clipped to its box (EmuSen_Kit.md §7.5, §7.10).
Item {
    id: root
    property alias seed: field.seed
    property alias count: field.count
    property alias moonriseSky: field.moonriseSky
    property alias starsOpacity: field.starsOpacity
    property alias glowHue: field.glowHue
    property alias glowX: field.glowX
    property alias glowY: field.glowY
    property bool animating: true
    readonly property alias starsShown: stars.count
    readonly property alias glowSettled: field.glowSettled
    property alias time: field.time
    Kit.StarField {
        id: field
        width: root.width
        height: root.height
    }
    Item {
        readonly property bool backdrop: true
        anchors.fill: parent
        clip: true
        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: field.stop0At; color: field.stop0Ink }
                GradientStop { position: field.stop1At; color: field.stop1Ink }
                GradientStop { position: field.stop2At; color: field.stop2Ink }
            }
        }
        Shape {
            id: glow
            visible: field.glowVisible
            x: field.glowCenterX - field.glowRadiusX
            y: field.glowCenterY - field.glowRadiusX
            width: 2 * field.glowRadiusX
            height: 2 * field.glowRadiusX
            preferredRendererType: Shape.CurveRenderer
            transform: Scale {
                origin.y: field.glowRadiusX
                yScale: field.glowRadiusX > 0 ? field.glowRadiusY / field.glowRadiusX : 1
            }
            ShapePath {
                strokeWidth: -1
                fillGradient: RadialGradient {
                    centerX: field.glowRadiusX
                    centerY: field.glowRadiusX
                    centerRadius: field.glowRadiusX
                    focalX: field.glowRadiusX
                    focalY: field.glowRadiusX
                    GradientStop { position: field.glowStop0At; color: field.glowStop0Ink }
                    GradientStop { position: field.glowStop1At; color: field.glowStop1Ink }
                    GradientStop { position: field.glowStop2At; color: field.glowStop2Ink }
                    GradientStop { position: field.glowStop3At; color: field.glowStop3Ink }
                }
                PathAngleArc {
                    centerX: field.glowRadiusX
                    centerY: field.glowRadiusX
                    radiusX: field.glowRadiusX
                    radiusY: field.glowRadiusX
                    startAngle: 0
                    sweepAngle: 360
                }
            }
        }
        Repeater {
            id: stars
            model: field
            Shape {
                id: star
                required property real starX
                required property real starY
                required property real starRadius
                required property real starOpacity
                x: star.starX - star.starRadius
                y: star.starY - star.starRadius
                width: 2 * star.starRadius
                height: 2 * star.starRadius
                opacity: star.starOpacity
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    strokeWidth: -1
                    fillColor: field.starInk
                    PathAngleArc {
                        centerX: star.starRadius
                        centerY: star.starRadius
                        radiusX: star.starRadius
                        radiusY: star.starRadius
                        startAngle: 0
                        sweepAngle: 360
                    }
                }
            }
        }
    }
    // The clock runs only while the glow eases, so a still screen is not drawn again (EmuSen_Kit.md §7.5, EmuSen_Qt.md §9.3.2).
    FrameAnimation {
        running: root.animating && root.visible && !field.glowSettled
        onTriggered: field.tick(field.time + frameTime)
    }
}
