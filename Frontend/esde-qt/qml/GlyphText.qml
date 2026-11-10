pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Esde

// A theme's words as ES-DE lays them out (EmuSen_Qt.md §4.9): each glyph a Text at the whole size, at its whole-pixel pen, on its line's baseline.
Item {
    id: root
    required property TextLayout laid
    property string family
    property string styleName
    property color color
    readonly property int base: Math.max(1, Math.round(root.laid.size))
    readonly property real stretch: root.laid.size > 0 ? root.laid.size / root.base : 1
    Repeater {
        model: root.laid
        Text {
            id: piece
            required property string glyph
            required property int pen
            required property int lineTop
            x: piece.pen
            // Drawn at the whole size nearest and scaled to the layout's, its baseline on the line's.
            y: piece.lineTop + Math.round(root.laid.baseline) - piece.baselineOffset * root.stretch
            transformOrigin: Item.TopLeft
            scale: root.stretch
            text: piece.glyph
            textFormat: Text.PlainText
            color: root.color
            font.family: root.family
            font.styleName: root.styleName
            font.pixelSize: root.base
            font.kerning: false
            font.hintingPreference: Font.PreferNoHinting
        }
    }
}
