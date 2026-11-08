pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Esde

// A grid: cells at the scene's scroll, each grown, dimmed and faded by its focus (the selection rising, the item left falling).
ElementBox {
    id: root
    required property int selected
    required property bool leaving
    required property int columns
    required property real itemWidth
    required property real itemHeight
    required property real pitchX
    required property real pitchY
    required property real marginX
    required property real marginY
    required property real contentY
    required property real itemScale
    required property int focusFrom
    required property real fromLevel
    required property real toLevel
    required property real progress
    required property real unfocusedOpacity
    required property real unfocusedSaturation
    required property real unfocusedDim
    required property int fit
    required property real cropX
    required property real cropY
    required property bool nearest
    required property real imageScale
    required property real radius
    required property color tint
    required property color tintEnd
    required property bool gradientVertical
    required property bool hasSelectedTint
    required property color selectedTint
    required property real saturation
    required property bool hasBackground
    required property color background
    required property url backgroundSource
    required property real backgroundScale
    required property real backgroundRadius
    required property bool hasSelector
    required property color selectorColor
    required property url selectorSource
    required property real selectorScale
    required property real selectorRadius
    required property bool selectorOnTop
    required property url fontSource
    required property real px
    required property real lineSpacing
    required property color color
    required property color itemBackground
    required property color selectedColor
    required property color selectedItemBackground
    required property real textRadius
    clip: true
    RowsModel {
        id: rows
        scene: root.esde
        revision: root.esde ? root.esde.revision : 0
        leaving: root.leaving
    }
    GridView {
        objectName: "esdeGrid"
        x: root.marginX
        width: root.width - root.marginX
        height: root.height
        interactive: false
        cacheBuffer: 0
        cellWidth: root.pitchX
        cellHeight: root.pitchY
        model: rows
        contentY: root.contentY - root.marginY
        delegate: Item {
            id: cell
            required property int index
            required property string text
            required property url image
            readonly property real level: cell.index === root.selected ? (root.focusFrom === cell.index ? 1 : root.toLevel + (1 - root.toLevel) * root.progress) : cell.index === root.focusFrom ? root.fromLevel * (1 - root.progress) : 0
            readonly property bool chosen: cell.index === root.selected
            readonly property real dim: root.unfocusedDim + (1 - root.unfocusedDim) * cell.level
            width: root.itemWidth
            height: root.itemHeight
            z: cell.level
            opacity: root.unfocusedOpacity + (1 - root.unfocusedOpacity) * cell.level
            scale: 1 + (root.itemScale - 1) * cell.level
            HoverHandler {
                onHoveredChanged: root.esde.hover(hovered ? cell.index : -1)
            }
            TapHandler {
                onTapped: root.esde.tap(cell.index)
            }
            Rectangle {
                anchors.centerIn: parent
                visible: root.hasBackground
                width: cell.width * root.backgroundScale
                height: cell.height * root.backgroundScale
                radius: root.backgroundRadius
                color: root.background
            }
            Picture {
                anchors.centerIn: parent
                width: cell.width * root.backgroundScale
                height: cell.height * root.backgroundScale
                source: root.backgroundSource
                radius: root.backgroundRadius
            }
            Rectangle {
                anchors.centerIn: parent
                visible: root.hasSelector && cell.chosen
                z: root.selectorOnTop ? 2 : 0
                width: cell.width * root.selectorScale
                height: cell.height * root.selectorScale
                radius: root.selectorRadius
                color: root.selectorColor
            }
            Picture {
                anchors.centerIn: parent
                visible: cell.chosen
                z: root.selectorOnTop ? 2 : 0
                width: cell.width * root.selectorScale
                height: cell.height * root.selectorScale
                source: root.selectorSource
                radius: root.selectorRadius
            }
            Picture {
                anchors.centerIn: parent
                z: 1
                visible: cell.image.toString().length > 0
                width: cell.width * root.imageScale
                height: cell.height * root.imageScale
                source: cell.image
                fit: root.fit
                cropX: root.cropX
                cropY: root.cropY
                nearest: root.nearest
                radius: root.radius
                tint: cell.chosen && root.hasSelectedTint ? root.selectedTint : root.tint
                tintEnd: cell.chosen && root.hasSelectedTint ? root.selectedTint : root.tintEnd
                gradientVertical: root.gradientVertical
                saturation: root.saturation * (root.unfocusedSaturation + (1 - root.unfocusedSaturation) * cell.level)
                dim: cell.dim
                sourceScale: root.itemScale
            }
            TextBlock {
                anchors.fill: parent
                z: 1
                visible: cell.image.toString().length === 0
                text: cell.text
                fontSource: root.fontSource
                px: root.px
                lineSpacing: root.lineSpacing
                hAlign: TextBlock.Align.Center
                wrap: true
                color: cell.chosen ? root.selectedColor : root.color
                hasBackground: true
                background: cell.chosen ? root.selectedItemBackground : root.itemBackground
                backgroundRadius: root.textRadius
            }
        }
    }
}
