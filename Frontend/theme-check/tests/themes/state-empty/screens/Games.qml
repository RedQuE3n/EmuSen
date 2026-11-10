import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// A fixture of theme-audit's states: the section's games as rows, and with none, the empty words in a box too narrow for them.
ThemeScreen {
    objectName: "Games"
    role: ThemeScreen.Games
    Rectangle {
        anchors.fill: parent
        color: Tokens.surface
    }
    Rows {
        visible: Browse.games.count > 0
        rows: Browse.games
        role: "title"
    }
    NormalizedCanvas {
        Text {
            objectName: "noGames"
            visible: Browse.games.count === 0
            x: 64
            y: 64
            width: 60
            elide: Text.ElideRight
            text: Words.noGames
            color: Tokens.text
            font.family: "Barlow"
            font.pixelSize: 26
        }
    }
}
