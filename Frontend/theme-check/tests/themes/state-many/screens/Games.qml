import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// A fixture of theme-audit's states: Home's places as rows, and the section's count set 130 from the right edge, which a long count runs past.
ThemeScreen {
    objectName: "Games"
    role: ThemeScreen.Games
    Rows {
        rows: Library.places
        role: "name"
    }
    NormalizedCanvas {
        Text {
            objectName: "count"
            x: parent.width - 130
            y: 64
            text: Words.games(Browse.games.count)
            color: Tokens.text
            font.family: "Barlow"
            font.pixelSize: 26
        }
    }
}
