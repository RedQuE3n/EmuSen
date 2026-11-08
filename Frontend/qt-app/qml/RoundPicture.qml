pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes

// A picture with rounded corners, fitted whole or filling its box, drawn as a Shape filled with the Image's own texture; plain corners on the software backend, which has no texture fills (EmuSen_Qt.md §7.1, step 6).
Item {
    id: root
    property alias source: picture.source
    property alias horizontalAlignment: picture.horizontalAlignment
    property alias verticalAlignment: picture.verticalAlignment
    property real radius
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
        asynchronous: false
        smooth: true
        mipmap: true
    }
    Shape {
        visible: root.rounded && picture.status === Image.Ready
        x: root.paintedX
        y: root.paintedY
        width: root.paintedWidth
        height: root.paintedHeight
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeColor: "transparent"
            fillColor: "transparent"
            fillItem: root.rounded ? picture : null
            fillTransform: PlanarTransform.fromAffineMatrix(root.textureScale, 0, 0, root.textureScale,
                (root.paintedWidth - picture.implicitWidth * root.textureScale) / 2, (root.paintedHeight - picture.implicitHeight * root.textureScale) / 2)
            PathRectangle {
                width: root.paintedWidth
                height: root.paintedHeight
                radius: root.radius
            }
        }
    }
}
