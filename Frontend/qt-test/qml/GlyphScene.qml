pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// A fixture: the glyph of every press in every family, a row a family, outlined and then filled, on the help bar's surface, for the glyph test and a human look (EmuSen_Kit.md §7.12, §7.28).
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    color: Kit.Tokens.surface
    Column {
        x: Kit.Tokens.base * 8
        y: Kit.Tokens.base * 8
        spacing: Kit.Tokens.base * 4
        Repeater {
            model: 8
            Row {
                id: familyRow
                required property int index
                readonly property bool filled: familyRow.index >= 4
                spacing: Kit.Tokens.base * 3
                Repeater {
                    model: 16
                    Kit.PadGlyph {
                        required property int index
                        objectName: familyRow.filled ? "glyphFilled" : "glyph"
                        family: familyRow.index % 4
                        style: familyRow.filled ? Kit.GlyphStyles.Filled : Kit.GlyphStyles.Outlined
                        button: index
                    }
                }
            }
        }
    }
}
