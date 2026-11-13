pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Esde

// A carousel of any type: the scene's slots, each scaled about its growth point and turned about the wheel, farthest first; every carousel clips.
ElementBox {
    id: root
    required property int selected
    required property bool leaving
    required property int fit
    required property real alignX
    required property real alignY
    required property real cropX
    required property real cropY
    required property bool nearest
    required property real radius
    required property real brightness
    required property bool hasBackground
    required property color background
    required property color backgroundEnd
    required property bool backgroundVertical
    required property url fontSource
    required property bool ownFace
    required property real px
    required property real lineSpacing
    required property color color
    required property color itemBackground
    required property color selectedColor
    required property color selectedItemBackground
    required property real textRadius
    required property bool oneLine
    required property bool scrolling
    required property real scrollX
    required property real repeatAt
    clip: true
    Rectangle {
        anchors.fill: parent
        visible: root.hasBackground
        gradient: Gradient {
            orientation: root.backgroundVertical ? Gradient.Vertical : Gradient.Horizontal
            GradientStop { position: 0; color: root.background }
            GradientStop { position: 1; color: root.backgroundEnd }
        }
    }
    SlotsModel {
        id: slots
        scene: root.esde
        revision: root.esde ? root.esde.revision : 0
        leaving: root.leaving
    }
    Repeater {
        model: slots
        Item {
            id: slot
            required property int row
            required property real boxX
            required property real boxY
            required property real boxWidth
            required property real boxHeight
            required property real grow
            required property real growX
            required property real growY
            required property real turn
            required property real turnX
            required property real turnY
            required property real alpha
            required property real dim
            required property real saturation
            required property color tint
            required property color tintEnd
            required property bool gradientVertical
            required property bool reflected
            required property real reflectionOpacity
            required property real reflectionFalloff
            required property string text
            required property url image
            required property bool chosen
            readonly property bool pictured: slot.image.toString().length > 0
            readonly property color ink: slot.chosen ? root.selectedColor : root.color
            readonly property color behind: slot.chosen ? root.selectedItemBackground : root.itemBackground
            // A name newly selected is measured again, since the scene scrolls the selected one only.
            onChosenChanged: if (slot.chosen) root.esde.measured(root.elementId, label.contentWidth, label.measuredHeight)
            anchors.fill: parent
            opacity: slot.alpha
            transform: [
                Scale {
                    origin.x: slot.growX
                    origin.y: slot.growY
                    xScale: slot.grow
                    yScale: slot.grow
                },
                Rotation {
                    origin.x: slot.turnX
                    origin.y: slot.turnY
                    angle: slot.turn
                }
            ]
            Picture {
                id: picture
                objectName: "slot-" + slot.row
                x: slot.boxX
                y: slot.boxY
                width: slot.boxWidth
                height: slot.boxHeight
                visible: slot.pictured
                source: slot.image
                fit: root.fit
                alignX: root.alignX
                alignY: root.alignY
                tint: slot.tint
                tintEnd: slot.tintEnd
                gradientVertical: slot.gradientVertical
                cropX: root.cropX
                cropY: root.cropY
                nearest: root.nearest
                radius: root.radius
                brightness: root.brightness
                saturation: slot.saturation
                dim: slot.dim
                sourceScale: slot.grow
                HoverHandler {
                    onHoveredChanged: root.esde.hoverAt(parent, hovered ? slot.row : -1)
                }
                TapHandler {
                    onTapped: root.esde.tap(slot.row)
                }
            }
            Picture {
                x: slot.boxX
                y: slot.boxY + (slot.boxHeight - picture.paintedHeight) * root.alignY + picture.paintedHeight
                width: slot.boxWidth
                height: slot.boxHeight
                visible: slot.pictured && slot.reflected
                source: slot.image
                fit: root.fit
                alignX: root.alignX
                alignY: 0
                flipped: true
                tint: slot.tint
                tintEnd: slot.tintEnd
                gradientVertical: slot.gradientVertical
                cropX: root.cropX
                cropY: root.cropY
                nearest: root.nearest
                radius: root.radius
                brightness: root.brightness
                saturation: slot.saturation
                dim: slot.dim
                sourceScale: slot.grow
                fadeFrom: slot.reflectionOpacity
                fadeFalloff: slot.reflectionFalloff
            }
            TextBlock {
                id: label
                objectName: "slot-" + slot.row
                x: slot.boxX
                y: slot.boxY
                width: slot.boxWidth
                height: slot.boxHeight
                visible: !slot.pictured
                text: slot.text
                fontSource: root.fontSource
                ownFace: root.ownFace
                px: root.px
                lineSpacing: root.lineSpacing
                wrap: true
                oneLine: root.oneLine
                color: Qt.rgba(slot.ink.r * slot.dim, slot.ink.g * slot.dim, slot.ink.b * slot.dim, slot.ink.a)
                hAlign: TextBlock.Align.Center
                hasBackground: slot.behind.a > 0
                background: slot.behind
                backgroundRadius: root.textRadius
                scrolling: slot.chosen && root.scrolling
                scrollX: root.scrollX
                repeatAt: root.repeatAt
                onMeasured: (width, height) => root.esde.measured(slot.chosen ? root.elementId : "", width, height)
                HoverHandler {
                    onHoveredChanged: root.esde.hoverAt(parent, hovered ? slot.row : -1)
                }
                TapHandler {
                    onTapped: root.esde.tap(slot.row)
                }
            }
        }
    }
}
