import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Esde

// helpsystem: the kit's help bar in the theme's face and colours, filled glyphs, each entry a button making the pad's call.
ElementBox {
    id: root
    required property url fontSource
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
    // THEMES.md: fontSize sets the text and the icons; entryRelativeScale above 1 shrinks the icons, below 1 the text.
    // The icon's side over the font's size, measured on ES-DE 3.4.1's captures (EmuSen_Qt.md §4.8.5).
    readonly property real iconShare: 0.9
    readonly property real glyphSide: root.px * root.iconShare * Math.min(1, 1 / Math.max(0.2, root.entryScale))
    readonly property real labelPx: root.px * Math.min(1, root.entryScale)
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
        width: bar.width + root.padLeft + root.padRight
        height: bar.height + root.padTop + root.padBottom
        radius: root.backgroundRadius
        gradient: Gradient {
            orientation: root.backgroundVertical ? Gradient.Vertical : Gradient.Horizontal
            GradientStop { position: 0; color: root.background }
            GradientStop { position: 1; color: root.backgroundEnd }
        }
    }
    // The bar's own extent is the element's size: the scene places it by its origin once measured (EmuSen_Qt.md §4.8.5).
    Kit.HelpBar {
        id: bar
        objectName: "esdeHelp"
        width: implicitWidth
        height: implicitHeight
        onImplicitWidthChanged: root.esde.measured(root.elementId, bar.implicitWidth, bar.implicitHeight)
        onImplicitHeightChanged: root.esde.measured(root.elementId, bar.implicitWidth, bar.implicitHeight)
        model: parts
        fitted: true
        ruled: false
        family: root.family
        glyphStyle: root.filled ? Kit.GlyphStyles.Filled : Kit.GlyphStyles.Outlined
        labelFont: Qt.font({ family: face.status === FontLoader.Ready ? face.font.family : body.font.family, pixelSize: root.labelPx })
        entrySpacing: root.entrySpacing
        glyphGap: root.iconGap
        labelColor: root.color
        iconColor: root.iconColor
        glyphSize: root.glyphSide
        color: Qt.rgba(0, 0, 0, 0)
        onPressed: (button) => root.esde.pressed(button, false, 0)
    }
}
