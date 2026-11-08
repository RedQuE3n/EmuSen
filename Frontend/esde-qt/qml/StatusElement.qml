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
    PartsModel {
        id: parts
        scene: root.esde
        revision: root.esde ? root.esde.revision : 0
        element: root.elementId
    }
    Rectangle {
        anchors.fill: parent
        visible: root.hasBackground
        color: root.background
        radius: root.backgroundRadius
    }
    Row {
        id: items
        x: root.width - items.width
        height: root.height
        spacing: root.entrySpacing
        Repeater {
            model: parts
            Row {
                id: item
                required property url source
                required property string mark
                required property string text
                height: root.height
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
                    height: root.height
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
