pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Esde

// systemstatus: the radios and the battery, a mark or the theme's file at the icons' height, the percentage beside.
ElementBox {
    id: root
    required property real iconHeight
    required property url fontSource
    required property real textScale
    required property color color
    required property real entrySpacing
    required property bool hasBackground
    required property color background
    required property real backgroundRadius
    required property real padLeft
    required property real padTop
    required property real padRight
    required property real padBottom
    PartsModel {
        id: parts
        scene: root.esde
        revision: root.esde ? root.esde.revision : 0
        element: root.elementId
    }
    Rectangle {
        x: -root.padLeft
        y: -root.padTop
        width: root.width + root.padLeft + root.padRight
        height: root.height + root.padTop + root.padBottom
        visible: root.hasBackground
        color: root.background
        radius: root.backgroundRadius
    }
    // The icons' row is the element's size: the scene places it by its origin once measured (EmuSen_Qt.md §4.8.5).
    onIconHeightChanged: root.esde.measured(root.elementId, items.width, root.iconHeight)
    Row {
        id: items
        x: root.width - items.width
        height: root.iconHeight
        onWidthChanged: root.esde.measured(root.elementId, items.width, root.iconHeight)
        spacing: root.entrySpacing
        Repeater {
            model: parts
            Row {
                id: item
                required property url source
                required property string mark
                required property string text
                height: root.iconHeight
                Icon {
                    anchors.verticalCenter: parent.verticalCenter
                    width: root.iconHeight
                    height: root.iconHeight
                    source: item.source
                    mark: item.mark
                    tint: root.color
                }
                TextBlock {
                    width: contentWidth
                    height: root.iconHeight
                    visible: item.text.length > 0
                    text: item.text
                    fontSource: root.fontSource
                    px: root.iconHeight * root.textScale
                    color: root.color
                }
            }
        }
    }
}
