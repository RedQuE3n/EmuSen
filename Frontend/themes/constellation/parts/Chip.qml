pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// The caption bar's chip, the first entry of a help kind: its glyph at 26 and its word in Barlow 18, white over the art (constellation.md §2).
Row {
    id: chip
    property int kind: Help.Continue
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
                ink: Tokens.overArt
            }
            Text {
                objectName: "chipLabel"
                anchors.verticalCenter: parent.verticalCenter
                text: first.label
                color: Tokens.overArt
                font.family: chip.faces?.regular ?? ""
                font.pixelSize: 19
                lineHeight: 22.8
                lineHeightMode: Text.FixedHeight
            }
        }
    }
}
