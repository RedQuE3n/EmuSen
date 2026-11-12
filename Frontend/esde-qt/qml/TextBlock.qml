pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Esde

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
    // ES-DE's own face (the theme named none), which it draws at its size rounded up to a whole pixel (EmuSen_Qt.md §4.9.1).
    property bool ownFace
    property real px: 16
    property color color
    property int hAlign: TextBlock.Align.Start
    property int vAlign: TextBlock.Align.Center
    // A line's height over px; 0 is the face's own, a single row's.
    property real lineSpacing: 1.5
    property bool wrap: false
    property bool ellipsis: false
    // A vertical container keeps every line, so what overflows its box is measured before it first scrolls.
    property bool everyLine: false
    // A vertical container shows whole rows at its start and end: its window is cut to them (EmuSen_Qt.md §4.12.3).
    property bool snapRows: false
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
    // The words laid out by ES-DE's rules, with the lines held as a whole in the box (EmuSen_Qt.md §4.9).
    readonly property real laidHeight: laid.contentHeight
    readonly property real alignedY: root.scrollVertical || root.vAlign === TextBlock.Align.Start ? 0 : Math.max(0, root.vAlign === TextBlock.Align.End ? root.height - root.laidHeight : (root.height - root.laidHeight) / 2)
    readonly property real contentWidth: laid.contentWidth
    readonly property real contentHeight: laid.contentHeight
    readonly property int wholeRows: root.snapRows && laid.pitch > 0 ? Math.floor((root.height + 1e-6) / laid.pitch) : 0
    readonly property real windowHeight: root.wholeRows > 0 ? Math.round(root.wholeRows * laid.pitch) : root.height
    // The scroll travels the measured height less the box, so the window's shortfall is measured with the words.
    readonly property real measuredHeight: laid.contentHeight + root.height - root.windowHeight
    readonly property string family: face.status === FontLoader.Ready ? face.font.family : body.font.family
    readonly property alias fontStatus: face.status
    // The first line's baseline, in the block's own coordinates.
    readonly property real baselineY: root.alignedY + Math.round(laid.baseline)
    // The words' laid-out extent, once per change of it, for the scene's measured().
    signal measured(real width, real height)
    onMeasuredHeightChanged: root.measured(laid.contentWidth, root.measuredHeight)

    TextLayout {
        id: laid
        font: face.status === FontLoader.Ready ? root.fontSource : ""
        text: root.text
        px: root.px
        ownFace: root.ownFace
        // A scrolling line is laid out whole; a vertical scroll keeps every line.
        boxWidth: root.scrolling && !root.scrollVertical ? 0 : root.width
        boxHeight: root.scrolling || root.everyLine ? 0 : root.height
        lineSpacing: root.lineSpacing
        wrap: root.wrap
        ellipsis: root.ellipsis && !root.scrolling
        align: root.scrolling && !root.scrollVertical ? TextBlock.Align.Start : root.hAlign
        onContentWidthChanged: root.measured(laid.contentWidth, root.measuredHeight)
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
        width: root.width
        height: root.windowHeight
        clip: root.scrolling || root.everyLine
        opacity: root.scrollOpacity
        GlyphText {
            id: words
            x: -root.scrollX
            y: root.alignedY - root.scrollY
            laid: laid
            family: root.family
            styleName: face.status === FontLoader.Ready ? face.font.styleName : body.font.styleName
            color: root.color
        }
        GlyphText {
            visible: root.repeatAt > 0
            x: words.x + root.repeatAt
            y: words.y
            laid: laid
            family: words.family
            styleName: words.styleName
            color: words.color
        }
    }
}
