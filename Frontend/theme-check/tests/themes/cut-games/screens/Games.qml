import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// The shelf's count placed in window pixels as though every screen were 800 high: under the foot at 1280 x 720.
ThemeScreen {
    id: screen
    objectName: "Games"
    role: ThemeScreen.Games
    Rows {
        rows: Library.recent
        role: "title"
    }
    Text {
        objectName: "count"
        x: screen.width / 20
        y: 730
        text: Words.games(Library.recent.count)
        color: Tokens.text
        font.family: "Barlow"
        font.pixelSize: screen.height / 30
    }
}
