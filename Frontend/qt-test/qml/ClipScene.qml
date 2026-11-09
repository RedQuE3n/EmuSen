pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// A fixture: rows of a body, a pad glyph and words in a clipping Flickable whose foot cuts the third row in half, for the clip test (EmuSen_Kit.md §7.34).
Window {
    id: scene
    objectName: "scene"
    width: 400
    height: 300
    color: Kit.Tokens.surface
    Flickable {
        objectName: "clipper"
        x: 40
        y: 40
        width: 320
        height: 160
        clip: true
        interactive: false
        contentWidth: 320
        contentHeight: 256
        Column {
            Repeater {
                model: 4
                Item {
                    id: row
                    required property int index
                    width: 320
                    height: 64
                    Kit.Body {
                        objectName: "body." + row.index
                        x: 8
                        y: 4
                        width: 56
                        height: 56
                        hue: 30
                        focused: true
                        ring: row.index === 0 ? Kit.Rings.Big : Kit.Rings.NoRing
                        moons: row.index === 0 ? 2 : 0
                    }
                    Kit.PadGlyph {
                        objectName: "glyph." + row.index
                        x: 96
                        y: 8
                        width: 48
                        height: 48
                        style: Kit.GlyphStyles.Filled
                        button: 0
                        ink: Kit.Tokens.accent
                    }
                    Text {
                        objectName: "words." + row.index
                        x: 176
                        y: 4
                        text: "World"
                        color: Kit.Tokens.accent
                        font.pixelSize: 48
                    }
                }
            }
        }
    }
}
