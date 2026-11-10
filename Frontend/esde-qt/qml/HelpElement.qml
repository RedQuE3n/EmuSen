pragma ComponentBehavior: Bound
import QtQuick
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
    // The element's height over the font's size, the side of ES-DE's icon box, measured on its captures (EmuSen_Qt.md §4.8.7).
    readonly property real heightShare: 0.9
    readonly property real lineHeight: root.px * root.heightShare
    readonly property real glyphSide: root.lineHeight * Math.min(1, 1 / Math.max(0.2, root.entryScale))
    // The element's width: the entries at their whole-pixel places, the last one whole, since nothing follows it.
    readonly property size extent: Qt.size(bar.implicitWidth + root.glyphSide + root.iconGap - root.labelAt, root.lineHeight)
    readonly property real labelPx: root.px * Math.min(1, root.entryScale)
    // ES-DE sets a label and the next entry at whole pixels, the fraction dropped (EmuSen_Qt.md §4.12).
    readonly property real labelAt: Math.floor(root.glyphSide + root.iconGap)
    readonly property real entryAt: Math.floor(root.glyphSide + root.iconGap + root.entrySpacing)
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
    Kit.HelpBar {
        id: bar
        objectName: "esdeHelp"
        // The entries start at the whole pixel at or before the element's place (EmuSen_Qt.md §4.12).
        x: Math.floor(root.boxX) - root.boxX
        y: (root.lineHeight - bar.height) / 2
        width: implicitWidth
        height: implicitHeight
        model: parts
        fitted: true
        ruled: false
        family: root.family
        glyphStyle: root.filled ? Kit.GlyphStyles.Filled : Kit.GlyphStyles.Outlined
        labelFont: Qt.font({ family: root.faceName, pixelSize: Math.round(root.labelPx) })
        label: Component {
            GlyphLine {
                fontSource: face.status === FontLoader.Ready ? root.fontSource : ""
                ownFace: root.ownFace
                family: root.faceName
                reference: root.esde ? root.esde.reference : ""
                px: root.labelPx
            }
        }
        pairGlyph: true
        entrySpacing: root.entryAt - root.labelAt
        glyphGap: root.labelAt - root.glyphSide
        labelColor: root.color
        iconColor: root.iconColor
        glyphSize: root.glyphSide
        color: Qt.rgba(0, 0, 0, 0)
        onPressed: (button) => root.esde.pressed(button, false, 0)
    }
}
