import QtQuick
import QtQuick.Shapes
import EmuSen.Kit

// Moonrise: the splash laid out from kit-core's frame, the moon, the row of worlds, the sweep, the line and the words (EmuSen_Kit.md §7.5).
Item {
    id: root
    property alias title: splash.title
    property alias caption: splash.caption
    property alias model: splash
    property bool running: true
    readonly property alias finished: splash.finished
    MoonriseModel {
        id: splash
        width: root.width
        height: root.height
    }
    Sky {
        anchors.fill: parent
        seed: splash.seed
        count: splash.starCount
        moonriseSky: true
        starsOpacity: splash.starsOpacity
        animating: root.running
    }
    Rectangle {
        id: halo
        x: splash.moonX - splash.haloRadius
        y: splash.moonY - splash.haloRadius
        width: 2 * splash.haloRadius
        height: 2 * splash.haloRadius
        radius: splash.haloRadius
        color: splash.haloInk
        opacity: splash.haloOpacity
    }
    Shape {
        id: lit
        x: splash.moonX - splash.moonRadius
        y: splash.moonY - splash.moonRadius
        width: 2 * splash.moonRadius
        height: 2 * splash.moonRadius
        opacity: splash.fill
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeWidth: -1
            fillGradient: RadialGradient {
                centerX: splash.moonRadius
                centerY: splash.moonRadius
                centerRadius: splash.moonRadius
                focalX: splash.moonRadius * 0.7
                focalY: splash.moonRadius * 0.7
                GradientStop { position: 0; color: splash.litInnerInk }
                GradientStop { position: 1; color: splash.litOuterInk }
            }
            PathAngleArc {
                centerX: splash.moonRadius
                centerY: splash.moonRadius
                radiusX: splash.moonRadius
                radiusY: splash.moonRadius
                startAngle: 0
                sweepAngle: 360
            }
        }
    }
    Rectangle {
        id: shadow
        x: splash.shadowX - splash.shadowRadius
        y: splash.shadowY - splash.shadowRadius
        width: 2 * splash.shadowRadius
        height: 2 * splash.shadowRadius
        radius: splash.shadowRadius
        color: splash.shadowInk
        opacity: splash.fill
    }
    Shape {
        id: outline
        x: splash.moonX - splash.moonRadius
        y: splash.moonY - splash.moonRadius
        width: 2 * splash.moonRadius
        height: 2 * splash.moonRadius
        visible: splash.outlineDrawn > 0
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            fillColor: "transparent"
            strokeColor: splash.outlineInk
            strokeWidth: 2 * splash.unit
            capStyle: ShapePath.RoundCap
            PathAngleArc {
                centerX: splash.moonRadius
                centerY: splash.moonRadius
                radiusX: splash.moonRadius
                radiusY: splash.moonRadius
                startAngle: -90
                sweepAngle: 360 * splash.outlineDrawn
            }
        }
    }
    Repeater {
        model: splash
        Item {
            id: world
            required property real planetX
            required property real planetY
            required property real planetRadius
            required property real planetOpacity
            required property real glowRadius
            required property color glowInk
            required property real flareRadius
            required property real flareOpacity
            required property color flareInk
            required property real boxX
            required property real boxY
            required property real boxWidth
            required property real boxHeight
            required property int bodyKind
            required property real hue
            required property real craters
            required property real bands
            required property real cap
            required property int ringKind
            required property int moons
            opacity: world.planetOpacity
            Rectangle {
                x: world.planetX - world.glowRadius
                y: world.planetY - world.glowRadius
                width: 2 * world.glowRadius
                height: 2 * world.glowRadius
                radius: world.glowRadius
                color: world.glowInk
            }
            Body {
                x: world.boxX
                y: world.boxY
                width: world.boxWidth
                height: world.boxHeight
                shape: world.bodyKind
                hue: world.hue
                craters: world.craters
                bands: world.bands
                cap: world.cap
                ring: world.ringKind
                moons: world.moons
            }
            Rectangle {
                x: world.planetX - world.flareRadius
                y: world.planetY - world.flareRadius
                width: 2 * world.flareRadius
                height: 2 * world.flareRadius
                radius: world.flareRadius
                color: world.flareInk
                opacity: world.flareOpacity
            }
        }
    }
    Rectangle {
        id: sweep
        visible: splash.sweepVisible
        x: splash.sweepX
        y: splash.sweepY
        width: splash.sweepWidth
        height: splash.sweepHeight
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 0.5; color: splash.sweepInk }
            GradientStop { position: 1; color: "transparent" }
        }
    }
    Rectangle {
        id: line
        visible: splash.lineVisible
        x: splash.lineFromX
        y: splash.lineY
        width: splash.lineToX - splash.lineFromX
        height: 1
        color: splash.lineInk
        opacity: splash.lineOpacity
    }
    Text {
        id: titleWords
        objectName: "moonriseTitle"
        visible: splash.wordsVisible
        text: splash.title
        color: splash.titleInk
        opacity: splash.wordsOpacity
        font.pixelSize: splash.titleSize
        font.letterSpacing: splash.titleSpacing
        anchors.horizontalCenter: parent.horizontalCenter
        y: splash.titleBaselineY - titleWords.baselineOffset
    }
    Text {
        id: captionWords
        objectName: "moonriseCaption"
        visible: splash.wordsVisible
        text: splash.caption
        color: splash.captionInk
        opacity: splash.wordsOpacity
        font.pixelSize: splash.captionSize
        font.letterSpacing: splash.captionSpacing
        anchors.horizontalCenter: parent.horizontalCenter
        y: splash.captionBaselineY - captionWords.baselineOffset
    }
    FrameAnimation {
        running: root.running && !splash.finished
        onTriggered: splash.tick(elapsedTime)
    }
}
