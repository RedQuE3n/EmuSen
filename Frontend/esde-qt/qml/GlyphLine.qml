pragma ComponentBehavior: Bound
import QtQuick

// One line of words as ES-DE draws it: each glyph at a whole-pixel pen, advanced by its advance rounded, at the size unrounded, the 'S' centred on the line (EmuSen_Qt.md §4.8.7).
Item {
    id: root
    property string text
    property color color
    // The character the line is centred by, the scene's (EsdeModel.reference).
    property string reference
    property string family
    property real px: 12
    readonly property int base: Math.max(1, Math.round(root.px))
    readonly property real stretch: root.px / root.base
    // Each glyph with its pen, and the pen past the last as the line's width.
    // Read through the metrics' own font, so a new size or face lays the line out again (advanceWidth is a call, not a property).
    readonly property var placed: {
        const scale = root.px / metrics.font.pixelSize;
        let pen = 0;
        const out = [];
        for (const g of Array.from(root.text)) {
            out.push({ glyph: g, pen: pen });
            pen += Math.round(metrics.advanceWidth(g) * scale);
        }
        return { glyphs: out, width: pen };
    }
    readonly property rect referenceBox: {
        const size = metrics.font.pixelSize;
        return size > 0 ? metrics.tightBoundingRect(root.reference) : Qt.rect(0, 0, 0, 0);
    }
    implicitWidth: root.placed.width
    implicitHeight: root.referenceBox.height * root.stretch
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
            id: glyph
            required property var modelData
            x: glyph.modelData.pen
            y: (-root.referenceBox.y - glyph.baselineOffset) * root.stretch
            transformOrigin: Item.TopLeft
            scale: root.stretch
            text: glyph.modelData.glyph
            color: root.color
            font: metrics.font
        }
    }
}
