pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit
import EmuSen.Esde

// helpsystem: the kit's help bar in the theme's face and colours, filled glyphs, each entry a button making the pad's call.
ElementBox {
    id: root
    required property url fontSource
    required property bool ownFace
    required property real px
    required property color color
    required property color iconColor
    required property real entrySpacing
    required property real iconGap
    required property real entryScale
    required property bool filled
    required property bool hasBackground
    required property color background
    required property color backgroundEnd
    required property bool backgroundVertical
    required property real backgroundRadius
    required property real padLeft
    required property real padTop
    required property real padRight
    required property real padBottom
    property int family: Kit.Families.Generic
    readonly property string faceName: face.status === FontLoader.Ready ? face.font.family : body.font.family
    // THEMES.md: fontSize sets the text and the icons; entryRelativeScale above 1 shrinks the icons, below 1 the text.
    // The element's height over the font's size, measured on its captures (EmuSen_Qt.md §4.8.7).
    readonly property real heightShare: 0.9
    readonly property real lineHeight: root.px * root.heightShare
    required property real barWidth
    readonly property real labelPx: root.px * Math.min(1, root.entryScale)
    // The bar starts at the nearest pixel to its place and ends at its last label (EmuSen_Qt.md §4.12.1).
    readonly property real barStart: Math.round(root.boxX)
    readonly property size extent: Qt.size(root.barWidth, root.lineHeight)
    PartsModel {
        id: parts
        scene: root.esde
        revision: root.esde ? root.esde.revision : 0
        element: root.elementId
    }
    FontLoader {
        id: face
        source: root.fontSource
    }
    FontLoader {
        id: body
        source: "qrc:/emusen/fonts/Barlow-Regular.ttf"
    }
    // The background grows outward by its paddings and leaves pos and origin alone, as THEMES.md has it.
    Rectangle {
        visible: root.hasBackground
        x: -root.padLeft
        y: -root.padTop
        width: root.extent.width + root.padLeft + root.padRight
        height: root.lineHeight + root.padTop + root.padBottom
        radius: root.backgroundRadius
        gradient: Gradient {
            orientation: root.backgroundVertical ? Gradient.Vertical : Gradient.Horizontal
            GradientStop { position: 0; color: root.background }
            GradientStop { position: 1; color: root.backgroundEnd }
        }
    }
    // The bar's width and the line's height are the element's size, the bar centred on the line: the scene places it by its origin once measured (EmuSen_Qt.md §4.8.5).
    onExtentChanged: root.esde.measured(root.elementId, root.extent.width, root.extent.height)
    Item {
        id: bar
        objectName: "esdeHelp"
        width: root.barWidth
        height: root.lineHeight
        Repeater {
            model: parts
            AbstractButton {
                id: entry
                required property int index
                required property int button
                required property int pair
                required property string label
                required property var model
                // A theme's own picture for the button, drawn in the glyph's place.
                readonly property url picture: entry.model.icon ?? ""
                required property real start
                required property real labelStart
                // The glyph is drawn the size of its box, the share of the entry before its gap (EmuSen_Qt.md §4.12.1).
                readonly property real side: entry.labelStart - entry.start - root.iconGap
                // Each part at the nearest whole pixel of the window, its place carried from the bar's start (EmuSen_Qt.md §4.12.1).
                objectName: "help-" + entry.label
                x: Math.round(root.barStart + entry.start) - root.boxX
                width: words.x + words.width
                height: root.lineHeight
                focusPolicy: Qt.NoFocus
                background: null
                onClicked: root.esde.pressed(entry.button, false, 0)
                // The glyph centred in its box, the box and the label each at the nearest pixel (EmuSen_Qt.md §4.12.1).
                Image {
                    anchors.centerIn: glyph
                    visible: entry.picture.toString().length > 0
                    width: entry.side
                    height: entry.side
                    sourceSize: Qt.size(entry.side, entry.side)
                    fillMode: Image.PreserveAspectFit
                    source: entry.picture
                }
                Kit.PadGlyph {
                    id: glyph
                    x: (Math.round(root.barStart + entry.labelStart) - root.boxX - entry.x - root.iconGap - width) / 2
                    anchors.verticalCenter: parent.verticalCenter
                    visible: entry.picture.toString().length === 0
                    width: entry.side
                    height: entry.side
                    family: root.family
                    style: root.filled ? Kit.GlyphStyles.Filled : Kit.GlyphStyles.Outlined
                    ink: root.iconColor
                    button: entry.button
                    pair: entry.pair
                    swap: Kit.PadRouter.swapAcceptBack
                }
                // A pair the kit has no one glyph for is drawn as two, the second beside the first.
                Kit.PadGlyph {
                    x: glyph.x + glyph.width
                    anchors.verticalCenter: parent.verticalCenter
                    visible: entry.pair >= 0 && !glyph.paired
                    width: entry.side
                    height: entry.side
                    family: root.family
                    style: glyph.style
                    ink: root.iconColor
                    button: Math.max(0, entry.pair)
                    swap: Kit.PadRouter.swapAcceptBack
                }
                GlyphLine {
                    id: words
                    x: Math.round(root.barStart + entry.labelStart) - root.boxX - entry.x
                    anchors.verticalCenter: parent.verticalCenter
                    text: entry.label
                    color: root.color
                    fontSource: face.status === FontLoader.Ready ? root.fontSource : ""
                    ownFace: root.ownFace
                    family: root.faceName
                    reference: root.esde ? root.esde.reference : ""
                    px: root.labelPx
                }
            }
        }
    }
}
