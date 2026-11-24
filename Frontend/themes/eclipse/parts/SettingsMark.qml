pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// EmuSen's About eclipse mark: a 200 px dark disc with its 2 px rim and two glows clipped inside a 424 by 300 rounded card (EclipseSettingsEmuSen:67-69; EmuSen_Qt.md §3.4.11).
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    // Scene pixels to a canvas unit, so the glows blur at the pixels drawn.
    property real pixelScale: 1
    readonly property real radius: 22 * root.unit
    readonly property bool rounded: GraphicsInfo.api !== GraphicsInfo.Software
    objectName: "asideMark"
    width: parent ? parent.width : 0
    height: 300 * root.unit
    Ink {
        id: ink
    }
    SettingsInk {
        id: own
    }
    // The disc and its glows drawn inside the card, clipped to its 22 px corner via ShaderEffectSource and Shape on RHI, plain clipped on software.
    Item {
        id: art
        anchors.fill: parent
        clip: !root.rounded
        Rectangle {
            objectName: "asideMarkGround"
            anchors.fill: parent
            radius: root.radius
            color: ink.emptyGround
        }
        Glow {
            objectName: "asideMarkHalo"
            anchors.fill: disc
            radius: disc.radius
            color: own.markHalo
            blur: 140 * root.unit
            spread: 40 * root.unit
            pixelScale: root.pixelScale
        }
        Glow {
            objectName: "asideMarkGlow"
            anchors.fill: disc
            radius: disc.radius
            color: own.markGlow
            blur: 60 * root.unit
            spread: 14 * root.unit
            pixelScale: root.pixelScale
        }
        Glow {
            objectName: "asideMarkRim"
            anchors.fill: disc
            radius: disc.radius
            color: Tokens.accent
            spread: 2 * root.unit
            pixelScale: root.pixelScale
        }
        Rectangle {
            id: disc
            objectName: "asideMarkDisc"
            x: 112 * root.unit
            y: 50 * root.unit
            width: 200 * root.unit
            height: width
            radius: width / 2
            color: ink.emptyGround
        }
    }
    ShaderEffectSource {
        id: clipped
        sourceItem: root.visible && root.rounded ? art : null
        hideSource: root.rounded
        live: true
        visible: false
        textureSize: Qt.size(Math.ceil(root.width * root.pixelScale), Math.ceil(root.height * root.pixelScale))
    }
    Shape {
        objectName: "asideMarkFrame"
        anchors.fill: parent
        visible: root.rounded
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeColor: "transparent"
            fillColor: "transparent"
            fillItem: clipped
            fillTransform: PlanarTransform.fromScale(root.width / Math.max(1, clipped.textureSize.width), root.height / Math.max(1, clipped.textureSize.height))
            PathRectangle {
                width: root.width
                height: root.height
                radius: root.radius
            }
        }
    }
    // Its 1 px inset line over the clipped glows (EclipseSettingsEmuSen:67).
    Rectangle {
        objectName: "asideMarkLine"
        anchors.fill: parent
        radius: root.radius
        color: "transparent"
        border.width: root.unit
        border.color: own.facesLine
    }
}
