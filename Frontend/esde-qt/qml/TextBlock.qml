pragma ComponentBehavior: Bound
import QtQuick

// A theme's text: its face, a fixed line height, a background grown by its padding, and the scene's scroll (EmuSen_Qt.md §4.8).
Item {
    id: root
    enum Align {
        Start,
        Center,
        End
    }
    property string text
    // The theme's font file as a URL; empty, or a file that does not load, sets the kit's body face.
    property url fontSource
    property real px: 16
    property color color
    property int hAlign: TextBlock.Align.Start
    property int vAlign: TextBlock.Align.Center
    // A line's height over px; 0 is the face's own, a single row's.
    property real lineSpacing: 1.5
    property bool wrap: false
    property bool ellipsis: false
    property bool hasBackground: false
    property color background
    property color backgroundEnd: root.background
    property bool backgroundVertical: false
    property real backgroundRadius: 0
    property real padLeft: 0
    property real padTop: 0
    property real padRight: 0
    property real padBottom: 0
    // The scene's scroll: leftward by scrollX with a copy at repeatAt (0 for none), or upward by scrollY at scrollOpacity.
    property bool scrolling: false
    property bool scrollVertical: false
    property real scrollX: 0
    property real repeatAt: 0
    property real scrollY: 0
    property real scrollOpacity: 1
    // A fixed line's pitch, and the lines' height at it (one for no words): Qt's contentHeight adds a descent to a fixed height.
    readonly property real pitch: root.lineSpacing > 0 ? root.px * root.lineSpacing : 0
    readonly property real laidHeight: root.pitch > 0 ? Math.max(1, label.lineCount) * root.pitch : label.contentHeight
    // Each fixed line holds the face's own line centred, as ES-DE draws it (EmuSen_Qt.md §4.8.6).
    readonly property real halfLeading: root.pitch > 0 ? (root.pitch - metrics.height) / 2 : 0
    readonly property real alignedY: root.pitch <= 0 || root.scrollVertical || root.vAlign === TextBlock.Align.Start ? 0 : Math.max(0, root.vAlign === TextBlock.Align.End ? root.height - root.laidHeight : (root.height - root.laidHeight) / 2)
    readonly property alias contentWidth: label.contentWidth
    readonly property alias contentHeight: label.contentHeight
    readonly property string family: label.font.family
    readonly property alias fontStatus: face.status
    // The words' laid-out extent, once per change of it, for the scene's measured().
    signal measured(real width, real height)

    FontMetrics {
        id: metrics
        font: label.font
    }
    FontLoader {
        id: face
        source: root.fontSource
    }
    FontLoader {
        id: body
        source: "qrc:/emusen/fonts/Barlow-Regular.ttf"
    }
    Rectangle {
        visible: root.hasBackground
        x: -root.padLeft
        y: -root.padTop
        width: root.width + root.padLeft + root.padRight
        height: root.height + root.padTop + root.padBottom
        radius: root.backgroundRadius
        gradient: Gradient {
            orientation: root.backgroundVertical ? Gradient.Vertical : Gradient.Horizontal
            GradientStop { position: 0; color: root.background }
            GradientStop { position: 1; color: root.backgroundEnd }
        }
    }
    Item {
        anchors.fill: parent
        clip: root.scrolling
        opacity: root.scrollOpacity
        Text {
            id: label
            x: -root.scrollX
            y: root.alignedY + root.halfLeading - root.scrollY
            width: root.width
            height: root.scrollVertical ? undefined : root.height
            text: root.text
            color: root.color
            font.family: face.status === FontLoader.Ready ? face.font.family : body.font.family
            font.styleName: face.status === FontLoader.Ready ? face.font.styleName : body.font.styleName
            font.pixelSize: Math.max(1, Math.round(root.px))
            lineHeightMode: root.lineSpacing > 0 ? Text.FixedHeight : Text.ProportionalHeight
            lineHeight: root.lineSpacing > 0 ? root.px * root.lineSpacing : 1
            wrapMode: root.wrap ? Text.WordWrap : Text.NoWrap
            elide: root.ellipsis && !root.scrolling ? Text.ElideRight : Text.ElideNone
            horizontalAlignment: root.scrolling && !root.scrollVertical ? Text.AlignLeft : root.hAlign === TextBlock.Align.End ? Text.AlignRight : root.hAlign === TextBlock.Align.Center ? Text.AlignHCenter : Text.AlignLeft
            verticalAlignment: root.pitch > 0 || root.vAlign === TextBlock.Align.Start ? Text.AlignTop : root.vAlign === TextBlock.Align.End ? Text.AlignBottom : Text.AlignVCenter
            onContentSizeChanged: root.measured(label.contentWidth, root.laidHeight)
        }
        Text {
            visible: root.repeatAt > 0
            x: label.x + root.repeatAt
            y: label.y
            width: label.width
            height: label.height
            text: label.text
            color: label.color
            font: label.font
            lineHeightMode: label.lineHeightMode
            lineHeight: label.lineHeight
            verticalAlignment: label.verticalAlignment
        }
    }
}
