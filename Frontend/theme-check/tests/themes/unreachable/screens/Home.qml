import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// Home's places as rows, and the shelf's first game as a tile beside them that no walk leads to.
ThemeScreen {
    objectName: "Home"
    role: ThemeScreen.Home
    Rows {
        id: rows
        rows: Library.places
        role: "name"
    }
    NormalizedCanvas {
        Walkable {
            objectName: "stranded"
            controller: null
            x: 800
            y: 64
            width: 240
            height: 56
            Text {
                anchors.centerIn: parent
                text: Words.recent
                color: Tokens.text
                font.family: "Barlow"
                font.pixelSize: 26
            }
        }
    }
}
