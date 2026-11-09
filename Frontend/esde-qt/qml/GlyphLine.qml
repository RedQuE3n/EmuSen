import QtQuick

// One line of words as ES-DE draws it: each glyph at a whole-pixel pen, advanced by its advance rounded, at the size unrounded, the 'S' centred on the line (EmuSen_Qt.md §4.8.7).
Item {
    id: root
    property string text
    property color color: "white"
    property string family
    property real px: 12
    readonly property int base: Math.max(1, Math.round(root.px))
    readonly property real stretch: root.px / root.base
    // Each glyph with its pen, and the pen past the last as the line's width.
    readonly property var placed: {
        let pen = 0;
        const out = [];
        for (const g of Array.from(root.text)) {
            out.push({ glyph: g, pen: pen });
            pen += Math.round(metrics.advanceWidth(g) * root.stretch);
        }
        return { glyphs: out, width: pen };
    }
    readonly property rect reference: metrics.tightBoundingRect("S")
    implicitWidth: root.placed.width
    implicitHeight: root.reference.height * root.stretch
    FontMetrics {
        id: metrics
        font.family: root.family
        font.pixelSize: root.base
        font.kerning: false
        font.hintingPreference: Font.PreferNoHinting
    }
    Repeater {
        model: root.placed.glyphs
        Text {
            required property var modelData
            x: modelData.pen
            y: (-root.reference.y - baselineOffset) * root.stretch
            transformOrigin: Item.TopLeft
            scale: root.stretch
            text: modelData.glyph
            color: root.color
            font: metrics.font
        }
    }
}
