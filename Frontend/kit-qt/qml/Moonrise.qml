import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit

// Moonrise: the splash laid out from kit-core's frame, drawn in the draft's order: the halo's glow, the moon's outline, its face and the shadow inside it, the row of worlds (the draft's lit disc for a plain one, a host's world as a Body), the sweep, the line and the words (EmuSen_Kit.md §7.5, §7.10).
Item {
    id: root
    property alias title: splash.title
    property alias caption: splash.caption
    property alias worlds: splash.worlds
    property list<real> hues
    property alias model: splash
    property bool running: true
    readonly property alias finished: splash.finished
    readonly property alias starsShown: sky.starsShown
    FontLoader {
        id: condensed
        source: "qrc:/emusen/fonts/BarlowCondensed-Light.ttf"
    }
    FontLoader {
        id: barlow
        source: "qrc:/emusen/fonts/Barlow-Medium.ttf"
    }
    Kit.MoonriseModel {
        id: splash
        width: root.width
        height: root.height
    }
    Binding {
        target: splash
        property: "hues"
        value: root.hues
    }
    Kit.Sky {
        id: sky
        anchors.fill: parent
        time: splash.draftTime
        seed: splash.seed
        count: splash.starCount
        moonriseSky: true
        starsOpacity: splash.starsOpacity
        animating: false
    }
    Shape {
        id: halo
        readonly property bool backdrop: true
        x: splash.moonX - splash.haloRadius
        y: splash.moonY - splash.haloRadius
        width: 2 * splash.haloRadius
        height: 2 * splash.haloRadius
        visible: splash.haloOpacity > 0
        opacity: splash.haloOpacity
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeWidth: -1
            fillGradient: RadialGradient {
                centerX: splash.haloRadius
                centerY: splash.haloRadius
                centerRadius: splash.haloRadius
                focalX: splash.haloRadius
                focalY: splash.haloRadius
                focalRadius: splash.haloInnerRadius
                GradientStop { position: 0; color: splash.haloInk }
                GradientStop { position: 1; color: splash.haloEdgeInk }
            }
            PathAngleArc {
                centerX: splash.haloRadius
                centerY: splash.haloRadius
                radiusX: splash.haloRadius
                radiusY: splash.haloRadius
                startAngle: 0
                sweepAngle: 360
            }
        }
    }
    Shape {
        id: outline
        objectName: "moonOutline"
        x: splash.moonX - splash.moonRadius
        y: splash.moonY - splash.moonRadius
        width: 2 * splash.moonRadius
        height: 2 * splash.moonRadius
        visible: splash.outlineDrawn > 0
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            fillColor: "transparent"
            strokeColor: splash.outlineInk
            strokeWidth: splash.outlineWidth
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
    Shape {
        id: lit
        x: splash.moonX - splash.moonRadius
        y: splash.moonY - splash.moonRadius
        width: 2 * splash.moonRadius
        height: 2 * splash.moonRadius
        visible: splash.fill > 0
        opacity: splash.fill
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeWidth: -1
            fillGradient: RadialGradient {
                centerX: splash.moonRadius
                centerY: splash.moonRadius
                centerRadius: splash.litRadius
                focalX: splash.litFocusX
                focalY: splash.litFocusY
                focalRadius: splash.litFocusRadius
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
    Shape {
        id: shadow
        x: splash.moonX - splash.moonRadius
        y: splash.moonY - splash.moonRadius
        width: 2 * splash.moonRadius
        height: 2 * splash.moonRadius
        visible: splash.fill > 0
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeWidth: -1
            fillColor: splash.shadowInk
            PathPolyline { path: splash.shadowOutline }
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
            required property bool plain
            required property color lightInk
            required property color rimInk
            required property real focusX
            required property real focusY
            required property real focusRadius
            required property int bodyKind
            required property real hue
            required property real craters
            required property real bands
            required property real cap
            required property int ringKind
            required property int moons
            required property string inks
            opacity: world.planetOpacity
            Rectangle {
                x: world.planetX - world.glowRadius
                y: world.planetY - world.glowRadius
                width: 2 * world.glowRadius
                height: 2 * world.glowRadius
                radius: world.glowRadius
                color: world.glowInk
            }
            Shape {
                visible: world.plain
                x: world.planetX - world.planetRadius
                y: world.planetY - world.planetRadius
                width: 2 * world.planetRadius
                height: 2 * world.planetRadius
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    strokeWidth: -1
                    fillGradient: RadialGradient {
                        centerX: world.planetRadius
                        centerY: world.planetRadius
                        centerRadius: world.planetRadius
                        focalX: world.focusX
                        focalY: world.focusY
                        focalRadius: world.focusRadius
                        GradientStop { position: 0; color: world.lightInk }
                        GradientStop { position: 1; color: world.rimInk }
                    }
                    PathAngleArc {
                        centerX: world.planetRadius
                        centerY: world.planetRadius
                        radiusX: world.planetRadius
                        radiusY: world.planetRadius
                        startAngle: 0
                        sweepAngle: 360
                    }
                }
            }
            Kit.Body {
                visible: !world.plain
                x: world.boxX
                y: world.boxY
                width: world.boxWidth
                height: world.boxHeight
                shape: world.bodyKind
                hue: world.hue
                focused: true
                craters: world.craters
                bands: world.bands
                cap: world.cap
                ring: world.ringKind
                moons: world.moons
                inks: world.inks
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
        z: 1
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
    Shape {
        id: line
        readonly property bool backdrop: true
        z: 1
        visible: splash.lineVisible
        x: splash.lineFromX
        y: splash.lineY - splash.lineWidth / 2
        width: splash.lineToX - splash.lineFromX
        height: splash.lineWidth
        opacity: splash.lineOpacity
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeColor: splash.lineInk
            strokeWidth: splash.lineWidth
            capStyle: ShapePath.FlatCap
            fillColor: "transparent"
            startX: 0
            startY: splash.lineWidth / 2
            PathLine {
                x: splash.lineToX - splash.lineFromX
                y: splash.lineWidth / 2
            }
        }
    }
    Text {
        id: titleWords
        objectName: "moonriseTitle"
        visible: splash.wordsVisible
        text: splash.title
        color: splash.titleInk
        opacity: splash.wordsOpacity
        font.family: condensed.font.family
        font.weight: Font.Light
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
        font.family: barlow.font.family
        font.weight: Font.Medium
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
