pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Esde

// systemstatus: the radios and the battery, a mark or the theme's file at the icons' height, the percentage beside.
ElementBox {
    id: root
    required property real iconHeight
    required property url fontSource
    required property bool ownFace
    required property real textScale
    required property color color
    required property real entrySpacing
    required property bool hasBackground
    required property color background
    required property color backgroundEnd
    required property bool backgroundVertical
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
        visible: root.hasBackground && shown.count > 0
        radius: root.backgroundRadius
        gradient: Gradient {
            orientation: root.backgroundVertical ? Gradient.Vertical : Gradient.Horizontal
            GradientStop { position: 0; color: root.background }
            GradientStop { position: 1; color: root.backgroundEnd }
        }
    }
    // The icons' row is the element's size, 0 × 0 with none: the scene places it by its origin once measured (EmuSen_Qt.md §4.8.5).
    readonly property size extent: Qt.size(items.width, shown.count > 0 ? root.iconHeight : 0)
    onExtentChanged: root.esde.measured(root.elementId, root.extent.width, root.extent.height)
    Row {
        id: items
        x: root.width - items.width
        height: root.iconHeight
        spacing: root.entrySpacing
        Repeater {
            id: shown
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
                    ownFace: root.ownFace
                    px: root.iconHeight * root.textScale
                    color: root.color
                }
            }
        }
    }
}
