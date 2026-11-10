import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// The world risen on the horizon at the right: a game's hero in a 300 disc, a console's body at 300, or a place's glass disc with its mark; the corona in the hue; the new one rising 24 as it fades in (EmuSen_Qt.md §3.4.3).
Item {
    id: root
    // What the world shows, and the key whose change raises it anew.
    property string key
    property string picture
    property string world
    property string mark
    property real hue: -1
    property bool place: false
    property real pixelScale: 1
    readonly property bool night: Tokens.variant === Tokens.Dark
    readonly property bool lit: !root.place && root.hue >= 0
    objectName: "world"
    width: 300
    height: 300
    onKeyChanged: rise.restart()
    Shape {
        objectName: "corona"
        readonly property bool backdrop: true
        visible: root.lit
        x: -50
        y: -50
        width: 400
        height: 400
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeWidth: -1
            strokeColor: "transparent"
            fillGradient: RadialGradient {
                centerX: 200
                centerY: 200
                centerRadius: 282.8
                focalX: 200
                focalY: 200
                GradientStop { position: 0; color: Qt.hsla(root.hue / 360, 0.7, 0.6, 0.38) }
                GradientStop { position: 0.37; color: Qt.hsla(root.hue / 360, 0.7, 0.6, 0.38) }
                GradientStop { position: 0.52; color: Qt.hsla(root.hue / 360, 0.7, 0.6, 0) }
            }
            PathAngleArc {
                centerX: 200
                centerY: 200
                radiusX: 200
                radiusY: 200
                startAngle: 0
                sweepAngle: 360
            }
        }
    }
    Item {
        id: shown
        width: 300
        height: 300
        // The disc's shadow, 0 16 48 at 45%, under a game's picture and a place's glass.
        Shape {
            objectName: "worldShadow"
            readonly property bool backdrop: true
            visible: root.world.length === 0
            x: -48
            y: -32
            width: 396
            height: 396
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                strokeWidth: -1
                strokeColor: "transparent"
                fillGradient: RadialGradient {
                    centerX: 198
                    centerY: 198
                    centerRadius: 198
                    focalX: 198
                    focalY: 198
                    GradientStop { position: 0.62; color: Qt.rgba(0, 0, 0, root.night ? 0.45 : 0.25) }
                    GradientStop { position: 1; color: "transparent" }
                }
                PathAngleArc {
                    centerX: 198
                    centerY: 198
                    radiusX: 198
                    radiusY: 198
                    startAngle: 0
                    sweepAngle: 360
                }
            }
        }
        // No picture: the generated card's hue, soft, at 40%.
        Rectangle {
            objectName: "worldCard"
            visible: !root.place && root.world.length === 0 && (root.picture.length === 0 || art.status === Image.Error)
            width: 300
            height: 300
            radius: 150
            opacity: 0.4
            gradient: Gradient {
                GradientStop { position: 0; color: Qt.hsla(Math.max(0, root.hue) / 360, 0.45, 0.32, 1) }
                GradientStop { position: 1; color: Qt.hsla(Math.max(0, root.hue) / 360, 0.5, 0.16, 1) }
            }
        }
        RoundPicture {
            id: art
            objectName: "worldPicture"
            visible: !root.place && root.world.length === 0 && root.picture.length > 0
            anchors.fill: parent
            source: root.place ? "" : root.picture
            radius: 150
            crop: true
            pixelScale: root.pixelScale
        }
        Rectangle {
            objectName: "worldGlass"
            visible: root.place
            width: 300
            height: 300
            radius: 150
            color: root.night ? "#8026262B" : "#99FFFFFF"
        }
        Glyph {
            objectName: "worldMark"
            visible: root.place
            x: 86
            y: 86
            width: 128
            height: 128
            path: root.mark
            ink: Tokens.textStrong
            strokeWidth: 1.4
        }
        Rectangle {
            objectName: "worldEdge"
            visible: root.world.length === 0
            width: 300
            height: 300
            radius: 150
            color: "transparent"
            border.width: 1
            border.color: root.place ? (root.night ? "#2E8C96DC" : "#241F2A44") : "#2EFFFFFF"
        }
        Body {
            id: body
            objectName: "worldBody"
            visible: root.world.length > 0
            world: root.world
            focused: true
            height: 300
            width: 300 * body.tileWidth / Math.max(1, body.tileHeight)
            x: (300 - width) / 2
        }
    }
    ParallelAnimation {
        id: rise
        NumberAnimation {
            target: shown
            property: "y"
            from: 24
            to: 0
            duration: Tokens.enter
            easing.type: Easing.OutQuad
        }
        NumberAnimation {
            target: shown
            property: "opacity"
            from: 0
            to: 1
            duration: Tokens.enterFade
            easing.type: Easing.OutQuad
        }
    }
}
