pragma ComponentBehavior: Bound
import QtQuick

// A theme's picture in one shader pass: fitted, cropped or tiled, desaturated, tinted, dimmed, rounded and faded (EmuSen_Qt.md §4.8).
Item {
    id: root
    enum Fit {
        Fill,
        Contain,
        Cover,
        Tile
    }
    // The picture's file as a URL; empty draws nothing.
    property url source
    property int fit: Picture.Fit.Contain
    // Where a contained picture sits in the box, and where a cover crop is taken from, as fractions.
    property real alignX: 0.5
    property real alignY: 0.5
    property real cropX: 0.5
    property real cropY: 0.5
    // A tile's size in pixels; an axis at 0 follows the other by the picture's aspect, both at 0 is the picture's own size.
    property real tileWidth: 0
    property real tileHeight: 0
    property bool tileRight: false
    property bool tileBottom: false
    property color tint: Qt.rgba(1, 1, 1, 1)
    property color tintEnd: root.tint
    property bool gradientVertical: false
    property real saturation: 1
    property real dim: 1
    property real radius: 0
    property bool nearest: false
    // The reflection's ramp: the opacity at the top edge, falling to nothing at 1 / fadeFalloff of the height.
    property real fadeFrom: 1
    property real fadeFalloff: 0
    // The decoded size over the drawn size, for a picture a transform will grow.
    property real sourceScale: 1
    // Drawn upside down, as a reflection is; the ramp still starts at the top of what is drawn.
    property bool flipped: false
    readonly property alias status: image.status
    readonly property alias ownWidth: image.implicitWidth
    readonly property alias ownHeight: image.implicitHeight
    // The painted size Qt gives a contained or covered picture, from its own size, since a raster's hidden Image stretches.
    readonly property real paintScale: root.fit === Picture.Fit.Contain ? Math.min(root.width / Math.max(1, image.implicitWidth), root.height / Math.max(1, image.implicitHeight)) : Math.max(root.width / Math.max(1, image.implicitWidth), root.height / Math.max(1, image.implicitHeight))
    readonly property bool fitted: root.fit === Picture.Fit.Contain || root.fit === Picture.Fit.Cover
    readonly property real paintedWidth: root.fitted ? image.implicitWidth * root.paintScale : root.width
    readonly property real paintedHeight: root.fitted ? image.implicitHeight * root.paintScale : root.height
    // A raster is never decoded larger than its own size or the box; Qt's aspect modes would scale it up (EmuSen_Qt.md §4.8.1).
    readonly property bool vector: /\.svgz?$/i.test(root.source.toString())
    readonly property bool tiled: root.fit === Picture.Fit.Tile
    readonly property real tileW: root.tileWidth > 0 ? root.tileWidth : Math.max(1, image.implicitWidth)
    readonly property real tileH: root.tileHeight > 0 ? root.tileHeight : Math.max(1, image.implicitHeight)
    readonly property real boxAspect: root.width / Math.max(1, root.height)
    readonly property real ownAspect: image.implicitWidth / Math.max(1, image.implicitHeight)
    readonly property real cropW: root.fit === Picture.Fit.Cover ? Math.min(1, root.boxAspect / root.ownAspect) : 1
    readonly property real cropH: root.fit === Picture.Fit.Cover ? Math.min(1, root.ownAspect / root.boxAspect) : 1

    Image {
        id: image
        visible: false
        width: root.width
        height: root.height
        source: root.source
        asynchronous: true
        cache: true
        smooth: !root.nearest
        fillMode: !root.vector ? Image.Stretch : root.fit === Picture.Fit.Contain ? Image.PreserveAspectFit : root.fit === Picture.Fit.Cover ? Image.PreserveAspectCrop : Image.Stretch
        // A vector contained or covered, and any picture covered, asks only a width, keeping the aspect the scene measures (EmuSen_Qt.md §4.8.5).
        sourceSize.width: root.nearest ? 0 : root.tiled ? root.tileWidth : Math.ceil((root.fit === Picture.Fit.Cover ? Math.max(root.width, 2 * root.height) : root.width) * root.sourceScale)
        sourceSize.height: root.nearest || root.fit === Picture.Fit.Cover || (root.vector && root.fit === Picture.Fit.Contain) ? 0 : root.tiled ? root.tileHeight : Math.ceil(root.height * root.sourceScale)
    }
    ShaderEffect {
        readonly property var source: image
        readonly property color tint: root.tint
        readonly property color tintEnd: root.tiled ? root.tint : root.tintEnd
        readonly property real vertical: root.gradientVertical ? 1 : 0
        readonly property real saturation: Math.max(0, Math.min(1, root.saturation))
        readonly property real dim: root.dim
        readonly property real radius: root.radius
        readonly property size size: Qt.size(width, height)
        readonly property rect uvRect: Qt.rect((1 - root.cropW) * root.cropX, (1 - root.cropH) * root.cropY + (root.flipped ? root.cropH : 0), root.cropW, root.flipped ? -root.cropH : root.cropH)
        readonly property size tiles: root.tiled ? Qt.size(width / root.tileW, height / root.tileH) : Qt.size(1, 1)
        readonly property point tileOrigin: root.tiled ? Qt.point(root.tileRight ? (width % root.tileW) / root.tileW : 0, root.tileBottom ? (height % root.tileH) / root.tileH : 0) : Qt.point(0, 0)
        readonly property real tiled: root.tiled ? 1 : 0
        readonly property real fadeFrom: root.fadeFrom
        readonly property real fadeFalloff: root.fadeFalloff
        visible: image.status === Image.Ready
        width: root.fit === Picture.Fit.Contain ? root.paintedWidth : root.width
        height: root.fit === Picture.Fit.Contain ? root.paintedHeight : root.height
        x: (root.width - width) * root.alignX
        y: (root.height - height) * root.alignY
        fragmentShader: "qrc:/emusen/esde/picture.frag.qsb"
    }
}
