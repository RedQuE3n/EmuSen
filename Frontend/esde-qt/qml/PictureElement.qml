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
    required property real brightness
    required property bool mirror
    required property bool mirrorVertically
    required property bool mipmap
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
        brightness: root.brightness
        mirror: root.mirror
        mirrorVertically: root.mirrorVertically
        mipmap: root.mipmap
        onOwnHeightChanged: root.esde.measured(root.elementId, picture.ownWidth, picture.ownHeight)
        // A new picture of the same size is reported too, since the scene keys a measurement by what it shows.
        onStatusChanged: if (picture.status === Image.Ready) root.esde.measured(root.elementId, picture.ownWidth, picture.ownHeight)
    }
}
