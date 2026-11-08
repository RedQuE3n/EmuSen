pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// A fixture: the glyph of every press in every family, a row a family, on the help bar's surface, for the glyph test and a human look (EmuSen_Kit.md §7.12).
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
            model: 4
            Row {
                id: familyRow
                required property int index
                spacing: Kit.Tokens.base * 3
                Repeater {
                    model: 16
                    Kit.PadGlyph {
                        required property int index
                        objectName: "glyph"
                        family: familyRow.index
                        button: index
                    }
                }
            }
        }
    }
}
