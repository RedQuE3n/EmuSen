pragma ComponentBehavior: Bound
import QtQuick

// A picture with rounded corners, fitted whole or filling its box, drawn via a single-pass SDF shader on RHI and plain corners on software (EmuSen_Qt.md §7.1, step 6; EmuSen_Kit.md §7.29).
Item {
    id: root
    property alias source: picture.source
    property alias horizontalAlignment: picture.horizontalAlignment
    property alias verticalAlignment: picture.verticalAlignment
    property real radius
    // Canvas units to pixels, so the picture is decoded at the size it is drawn.
    property real pixelScale: 1
    // Whether the picture fills the box, cut at its centre, rather than standing whole in it.
    property bool crop: false
    readonly property alias status: picture.status
    readonly property bool rounded: GraphicsInfo.api !== GraphicsInfo.Software
    readonly property real paintedWidth: root.crop ? root.width : picture.paintedWidth
    readonly property real paintedHeight: root.crop ? root.height : picture.paintedHeight
    readonly property real paintedX: root.crop || picture.horizontalAlignment === Image.AlignLeft ? 0
        : picture.horizontalAlignment === Image.AlignRight ? root.width - root.paintedWidth : (root.width - root.paintedWidth) / 2
    readonly property real paintedY: root.crop || picture.verticalAlignment === Image.AlignTop ? 0
        : picture.verticalAlignment === Image.AlignBottom ? root.height - root.paintedHeight : (root.height - root.paintedHeight) / 2
    // The texture's scale into the drawn box: the box's own for a whole picture, the larger side's for a cut one.
    readonly property real textureScale: root.crop ? Math.max(root.width / Math.max(1, picture.implicitWidth), root.height / Math.max(1, picture.implicitHeight))
        : root.paintedWidth / Math.max(1, picture.implicitWidth)
    Image {
        id: picture
        anchors.fill: parent
        visible: !root.rounded
        clip: root.crop
        fillMode: root.crop ? Image.PreserveAspectCrop : Image.PreserveAspectFit
        asynchronous: true
        sourceSize: Qt.size(Math.ceil(root.width * root.pixelScale * Screen.devicePixelRatio), Math.ceil(root.height * root.pixelScale * Screen.devicePixelRatio))
        smooth: true
        mipmap: true
    }
    ShaderEffect {
        visible: root.rounded && picture.status === Image.Ready
        x: root.paintedX
        y: root.paintedY
        width: root.paintedWidth
        height: root.paintedHeight
        readonly property var source: picture
        readonly property vector2d boxSize: Qt.vector2d(width, height)
        readonly property vector2d texSize: Qt.vector2d(picture.implicitWidth * root.textureScale, picture.implicitHeight * root.textureScale)
        readonly property real radius: root.radius
        fragmentShader: "qrc:/qt/qml/EmuSen/Kit/shaders/round_picture.frag.qsb"
    }
}
