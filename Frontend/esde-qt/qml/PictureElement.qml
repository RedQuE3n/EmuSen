import QtQuick
import EmuSen.Esde

// An image, a video's still or an animation's picture, drawn by one Picture; its own size reported for the scene to fit.
ElementBox {
    id: root
    required property url source
    required property int fit
    required property real cropX
    required property real cropY
    required property real tileWidth
    required property real tileHeight
    required property bool tileRight
    required property bool tileBottom
    required property color tint
    required property color tintEnd
    required property bool gradientVertical
    required property real saturation
    required property real radius
    required property bool nearest
    Picture {
        id: picture
        anchors.fill: parent
        source: root.source
        fit: root.fit
        cropX: root.cropX
        cropY: root.cropY
        tileWidth: root.tileWidth
        tileHeight: root.tileHeight
        tileRight: root.tileRight
        tileBottom: root.tileBottom
        tint: root.tint
        tintEnd: root.tintEnd
        gradientVertical: root.gradientVertical
        saturation: root.saturation
        radius: root.radius
        nearest: root.nearest
        onOwnHeightChanged: root.esde.measured(root.elementId, picture.ownWidth, picture.ownHeight)
    }
}
