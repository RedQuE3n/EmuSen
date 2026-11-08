import QtQuick
import EmuSen.Theme 1.0

ListView {
    orientation: ListView.Horizontal
    spacing: Tokens.tileGap
    delegate: Text {
        text: "CONTINUE PLAYING"
        color: Tokens.text
        font.family: "Barlow"
        font.pixelSize: Tokens.fontTitle
    }
}
