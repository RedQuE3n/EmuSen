pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// A chip naming the band's first action, "A Resume" or "A Open": the first entry of a help kind, its glyph at 26 and its words at 18 (meridian.md §2).
Row {
    id: chip
    property int kind: Help.Place
    property bool onArt
    required property Faces faces
    spacing: 8
    height: 26
    Repeater {
        model: Help.entries(chip.kind)
        Row {
            id: first
            required property int index
            required property int button
            required property string label
            visible: first.index === 0
            spacing: 8
            PadGlyph {
                objectName: "chipGlyph"
                width: 26
                height: 26
                family: Device.family
                button: first.button
                ink: chip.onArt ? Tokens.overArt : Tokens.textStrong
            }
            Text {
                objectName: "chipLabel"
                anchors.verticalCenter: parent.verticalCenter
                text: first.label
                color: chip.onArt ? Tokens.overArt : Tokens.textStrong
                font.family: chip.faces.regular
                font.pixelSize: 19
                lineHeight: 22.8
                lineHeightMode: Text.FixedHeight
            }
        }
    }
}
