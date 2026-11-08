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
    required property real backgroundRadius
    required property real padLeft
    required property real padTop
    property int family: Kit.Families.Generic
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
        labelFont: Qt.font({ family: face.status === FontLoader.Ready ? face.font.family : body.font.family, pixelSize: root.px })
        entrySpacing: root.entrySpacing
        glyphGap: root.iconGap
        labelColor: root.color
        iconColor: root.iconColor
        entryScale: root.entryScale
        color: root.hasBackground ? root.background : Qt.rgba(0, 0, 0, 0)
        radius: root.backgroundRadius
        horizontalPadding: root.padLeft
        verticalPadding: root.padTop
        onPressed: (button) => root.esde.pressed(button, false, 0)
    }
}
