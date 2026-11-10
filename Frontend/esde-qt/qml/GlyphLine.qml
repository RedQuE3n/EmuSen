pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Esde

// One line of words as ES-DE draws a help label: laid out by its text rules (EmuSen_Qt.md §4.9), the 'S' centred on the line (§4.8.7).
Item {
    id: root
    property string text
    property color color
    // The character the line is centred by, the scene's (EsdeModel.reference).
    property string reference
    // The theme's font file, empty for the kit's body face, and the family Qt loaded from it.
    property url fontSource
    property string family
    property real px: 12
    // ES-DE's own face, which it draws at a whole size (EmuSen_Qt.md §4.9.1).
    property bool ownFace
    readonly property real stretch: laid.size > 0 ? laid.size / metrics.font.pixelSize : 1
    readonly property rect referenceBox: metrics.font.pixelSize > 0 ? metrics.tightBoundingRect(root.reference) : Qt.rect(0, 0, 0, 0)
    implicitWidth: laid.contentWidth
    implicitHeight: root.referenceBox.height * root.stretch
    TextLayout {
        id: laid
        font: root.fontSource
        text: root.text
        px: root.px
        ownFace: root.ownFace
    }
    FontMetrics {
        id: metrics
        font.family: root.family
        font.pixelSize: Math.max(1, Math.round(laid.size))
        font.kerning: false
        font.hintingPreference: Font.PreferNoHinting
    }
    GlyphText {
        // The reference's top on the line's top: its baseline below by the reference's ascent.
        y: -root.referenceBox.y * root.stretch - Math.round(laid.baseline)
        laid: laid
        family: root.family
        color: root.color
    }
}
