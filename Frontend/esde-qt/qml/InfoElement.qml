pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Esde

// gamelistinfo: the counts and the filter's mark, each a mark an em high at its own width, on its words' baseline, spaced as ES-DE's captures measure (EmuSen_Qt.md §4.8.7).
ElementBox {
    id: root
    required property url fontSource
    required property real px
    required property color color
    required property color background
    required property int hAlign
    required property int vAlign
    // gamelistinfo has no lineSpacing of its own, so a line is a text's at THEMES.md's default.
    readonly property real lineSpacing: 1.5
    // Reported as the words' extent, for a size the theme left at 0 (EmuSen_Qt.md §4.8.6).
    readonly property size extent: Qt.size(items.width, root.px * root.lineSpacing)
    onExtentChanged: root.esde.measured(root.elementId, root.extent.width, root.extent.height)
    PartsModel {
        id: parts
        scene: root.esde
        revision: root.esde ? root.esde.revision : 0
        element: root.elementId
    }
    Rectangle {
        anchors.fill: parent
        color: root.background
    }
    Row {
        id: items
        x: root.hAlign === TextBlock.Align.End ? root.width - items.width : root.hAlign === TextBlock.Align.Center ? (root.width - items.width) / 2 : 0
        height: root.height
        spacing: root.px * 0.375
        Repeater {
            model: parts
            Row {
                id: item
                required property string mark
                required property string text
                height: root.height
                spacing: root.px * 0.25
                Mark {
                    id: icon
                    y: words.baselineY - root.px * icon.baselineShare
                    width: root.px * icon.aspect
                    height: root.px
                    mark: item.mark
                    color: root.color
                }
                TextBlock {
                    id: words
                    width: contentWidth
                    height: root.height
                    text: item.text
                    fontSource: root.fontSource
                    px: root.px
                    lineSpacing: root.lineSpacing
                    color: root.color
                    vAlign: root.vAlign
                }
            }
        }
    }
}
