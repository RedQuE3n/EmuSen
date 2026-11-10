pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Esde

// A text list: the rows at the scene's scroll, the selector bar where the scene puts it, the selected name's marquee; nothing animates here.
ElementBox {
    id: root
    required property int selected
    required property bool leaving
    required property real rowHeight
    required property real bandHeight
    required property real contentY
    required property bool hasSelector
    required property real selectorX
    required property real selectorY
    required property real selectorW
    required property real selectorH
    required property color selectorColor
    required property bool hasSelectedBackground
    required property color selectedBackground
    required property real selectedBackgroundRadius
    required property real selectedBackgroundLeft
    required property real selectedBackgroundRight
    required property color primaryColor
    required property color secondaryColor
    required property color selectedColor
    required property color selectedSecondaryColor
    required property url fontSource
    required property bool ownFace
    required property real px
    required property real lineSpacing
    required property real margin
    required property int hAlign
    required property real markerWidth
    required property bool scrolling
    required property real scrollX
    required property real repeatAt
    clip: true
    RowsModel {
        id: rows
        scene: root.esde
        revision: root.esde ? root.esde.revision : 0
        leaving: root.leaving
    }
    Rectangle {
        visible: root.hasSelector
        x: root.selectorX
        y: root.selectorY
        width: root.selectorW
        height: root.selectorH
        color: root.selectorColor
    }
    ListView {
        objectName: "esdeTextList"
        anchors.fill: parent
        interactive: false
        cacheBuffer: 0
        model: rows
        contentY: root.contentY
        delegate: Item {
            id: line
            required property int index
            required property string text
            required property string marker
            required property bool folder
            readonly property bool chosen: line.index === root.selected
            // The band sits at the row's top, as ES-DE's selector does; ES-DE prefixes the marker to the name, so a row without one starts at the margin (EmuSen_Qt.md §4.8.5).
            readonly property real room: line.marker.length > 0 ? root.markerWidth : 0
            objectName: "row-" + line.index
            width: root.width
            height: root.rowHeight
            HoverHandler {
                onHoveredChanged: root.esde.hover(hovered ? line.index : -1)
            }
            TapHandler {
                onTapped: root.esde.tap(line.index)
            }
            Rectangle {
                visible: line.chosen && root.hasSelectedBackground
                x: root.margin - root.selectedBackgroundLeft
                y: 0
                width: line.room + Math.min(name.contentWidth, name.width) + root.selectedBackgroundLeft + root.selectedBackgroundRight
                height: root.bandHeight
                radius: root.selectedBackgroundRadius
                color: root.selectedBackground
            }
            Mark {
                x: root.margin
                y: (root.bandHeight - root.px) / 2
                width: root.px
                height: root.px
                mark: line.marker
                color: line.chosen ? root.selectedColor : root.primaryColor
            }
            TextBlock {
                id: name
                x: root.margin + line.room
                y: 0
                width: root.width - 2 * root.margin - line.room
                height: root.bandHeight
                text: line.text
                fontSource: root.fontSource
                ownFace: root.ownFace
                px: root.px
                lineSpacing: 0
                hAlign: root.hAlign
                ellipsis: !line.chosen
                color: line.chosen ? (line.folder ? root.selectedSecondaryColor : root.selectedColor) : (line.folder ? root.secondaryColor : root.primaryColor)
                scrolling: line.chosen && root.scrolling
                scrollX: root.scrollX
                repeatAt: root.repeatAt
                onMeasured: (width, height) => root.esde.measured(line.chosen ? root.elementId : "", width, height)
            }
        }
    }
}
