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
    // ES-DE's row: 1.25 × the rendered S at the font's size, unrounded (EmuSen_Qt.md §4.12.4).
    required property real rowHeight
    // ES-DE's disc reaches 58 of its icon's 64 units; the kit's glyph is sized so its own disc does (EmuSen_Qt.md §4.12.1).
    readonly property real discEdge: 58 / 64
    // The grid's width, its trailing spacing in; the scene places the strip from it and gives the box the strip seen.
    required property real barWidth
    required property real labelPx
    readonly property real barStart: root.boxX
    readonly property size extent: Qt.size(root.barWidth, root.rowHeight)
    PartsModel {
        id: parts
        scene: root.esde
        revision: root.esde ? root.esde.revision : 0
        element: root.elementId
    }
    Kit.PadGlyphMarks {
        id: kitMarks
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
        width: root.boxWidth + root.padLeft + root.padRight
        height: root.rowHeight + root.padTop + root.padBottom
        radius: root.backgroundRadius
        gradient: Gradient {
            orientation: root.backgroundVertical ? Gradient.Vertical : Gradient.Horizontal
            GradientStop { position: 0; color: root.background }
            GradientStop { position: 1; color: root.backgroundEnd }
        }
    }
    // The grid's width and the row's height are its measure: the scene places the strip by ES-DE's rule from them (EmuSen_Qt.md §4.12.4).
    onExtentChanged: root.esde.measured(root.elementId, root.extent.width, root.extent.height)
    Item {
        id: bar
        objectName: "esdeHelp"
        width: root.boxWidth
        height: root.rowHeight
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
                required property real iconStart
                required property real iconWidth
                // entryLayout textFirst: the label, its gap, then the icon (HelpComponent::updateGrid).
                readonly property bool textFirst: entry.iconStart > entry.labelStart
                // The icon box's left within the entry: 0 before its label, past the label and gap after it.
                readonly property real iconLeft: entry.textFirst ? Math.round(root.barStart + entry.iconStart) - root.boxX - entry.x : 0
                // The icon's box, the entry before its gap; the kit's glyph drawn so its disc's edge is ES-DE's (EmuSen_Qt.md §4.12.1).
                readonly property real box: entry.textFirst ? entry.iconWidth : entry.labelStart - entry.start - root.iconGap
                readonly property real side: entry.box * root.discEdge / kitMarks.discShare
                // Each part at the nearest whole pixel of the window, its place carried from the bar's start (EmuSen_Qt.md §4.12.1).
                objectName: "help-" + entry.label
                x: Math.round(root.barStart + entry.start) - root.boxX
                width: entry.textFirst ? entry.iconLeft + entry.box : words.x + words.width
                height: root.rowHeight
                focusPolicy: Qt.NoFocus
                background: null
                onClicked: root.esde.pressed(entry.button, false, 0)
                // The glyph centred in its box, the box and the label each at the nearest pixel (EmuSen_Qt.md §4.12.1).
                Image {
                    objectName: "helpPicture-" + entry.label
                    anchors.centerIn: glyph
                    visible: entry.picture.toString().length > 0
                    width: entry.box
                    height: entry.box
                    sourceSize: Qt.size(entry.box, entry.box)
                    fillMode: Image.PreserveAspectFit
                    source: entry.picture
                }
                Kit.PadGlyph {
                    id: glyph
                    objectName: "helpGlyph-" + entry.label
                    x: entry.textFirst ? entry.iconLeft + (entry.box - width) / 2 : (Math.round(root.barStart + entry.labelStart) - root.boxX - entry.x - root.iconGap - width) / 2
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
