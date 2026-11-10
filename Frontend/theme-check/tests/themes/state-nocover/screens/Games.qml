import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// A fixture of theme-audit's states: the section's games as rows whose title is drawn only beside a cover, so a game with none is a tile no reader can name.
ThemeScreen {
    objectName: "Games"
    role: ThemeScreen.Games
    Rows {
        rows: Browse.games
        role: "title"
    }
}
