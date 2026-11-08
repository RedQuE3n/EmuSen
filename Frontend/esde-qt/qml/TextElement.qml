import QtQuick
import EmuSen.Esde

// A text, a date or the clock, drawn by one TextBlock; each new extent reported once for the scene's scroll and origin.
ElementBox {
    id: root
    required property string text
    required property url fontSource
    required property real px
    required property color color
    required property int hAlign
    required property int vAlign
    required property real lineSpacing
    required property bool wrap
    required property bool ellipsis
    required property bool hasBackground
    required property color background
    required property color backgroundEnd
    required property bool backgroundVertical
    required property real backgroundRadius
    required property real padLeft
    required property real padTop
    required property real padRight
    required property real padBottom
    required property bool scrolling
    required property bool scrollVertical
    required property real scrollX
    required property real repeatAt
    required property real scrollY
    required property real scrollOpacity
    TextBlock {
        id: block
        anchors.fill: parent
        text: root.text
        fontSource: root.fontSource
        px: root.px
        color: root.color
        hAlign: root.hAlign
        vAlign: root.vAlign
        lineSpacing: root.lineSpacing
        wrap: root.wrap
        ellipsis: root.ellipsis
        hasBackground: root.hasBackground
        background: root.background
        backgroundEnd: root.backgroundEnd
        backgroundVertical: root.backgroundVertical
        backgroundRadius: root.backgroundRadius
        padLeft: root.padLeft
        padTop: root.padTop
        padRight: root.padRight
        padBottom: root.padBottom
        scrolling: root.scrolling
        scrollVertical: root.scrollVertical
        scrollX: root.scrollX
        repeatAt: root.repeatAt
        scrollY: root.scrollY
        scrollOpacity: root.scrollOpacity
        onMeasured: (width, height) => root.esde.measured(root.elementId, width, height)
    }
    // New words or a new face are reported even at the same extent, since the scene keys a measurement by what it shows.
    onTextChanged: root.esde.measured(root.elementId, block.contentWidth, block.contentHeight)
    onPxChanged: root.esde.measured(root.elementId, block.contentWidth, block.contentHeight)
    onFontSourceChanged: root.esde.measured(root.elementId, block.contentWidth, block.contentHeight)
}
