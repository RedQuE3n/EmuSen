pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// Shared Eclipse footer help bar using Board glyph/line metrics across Search, Collections, and Tools.
Kit.HelpBar {
    id: root
    required property real unit
    required property string entriesJson
    required property int padFamily

    fitted: true
    ruled: false
    family: root.padFamily
    color: "transparent"
    labelColor: ThemeInk.body
    iconColor: Kit.Tokens.muted
    leadStyle: Kit.GlyphStyles.Filled
    leadColor: Kit.Tokens.accent
    entrySpacing: 34 * root.unit
    glyphGap: 10 * root.unit
    glyphSize: 34 * root.unit
    snapped: false

    model: RowsModel {
        rows: root.entriesJson
    }
    label: Component {
        CardLine {
            objectName: "helpWords-" + text
            unit: root.unit
            boardLine: true
            size: 21
        }
    }
    glyph: Component {
        CardGlyph {
            unit: root.unit
            side: 34
            boxSide: 30
            boxRadius: 8
            boxPadding: 10
            wordSize: 15
            letterSize: 16
            leadLetterSize: 17
            leadWeight: Font.Bold
            padding: 10
            boxMenu: false
            ink: ThemeInk.body
            ring: Kit.Tokens.muted
        }
    }
}
