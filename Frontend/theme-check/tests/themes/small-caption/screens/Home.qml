import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// Home's places as rows, and words under the capitals' floor by the name the floor's table of exceptions holds for
// Moonrise's caption, inside a wrapper the theme names "moonrise": written in no kit file, so the floor's finding.
ThemeScreen {
    objectName: "Home"
    role: ThemeScreen.Home
    Rows {
        rows: Library.places
        role: "name"
    }
    NormalizedCanvas {
        Item {
            objectName: "moonrise"
            anchors.fill: parent
            Text {
                objectName: "moonriseCaption"
                x: 672
                y: 400
                text: Words.noGames
                color: Tokens.text
                font.family: "Barlow"
                font.pixelSize: 12
            }
        }
    }
}
